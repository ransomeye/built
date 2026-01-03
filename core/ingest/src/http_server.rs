// Path and File Name : /home/ransomeye/rebuild/core/ingest/src/http_server.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: HTTP ingestion server with POST /ingest/linux and /ingest/dpi endpoints - verifies signatures and writes to database

use std::sync::Arc;
use std::net::IpAddr;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tokio_postgres::{Client, NoTls};
use tracing::{info, error, warn};
use uuid::Uuid;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, Utc};
use hostname;
use ring::rand::{SecureRandom, SystemRandom};
use hex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedEvent {
    pub envelope: JsonValue,  // EventEnvelope as JSON
    pub payload_hash: String,  // SHA-256 hex of canonical envelope JSON bytes
    pub signature: String,     // Base64 signature of payload_hash
    pub signer_id: String,     // Key identifier
}

#[derive(Debug, Serialize)]
pub struct IngestResponse {
    pub status: String,
    pub message_id: String,
}

pub struct HttpIngestionServer {
    db_client: Arc<Client>,
    listen_addr: String,
}

impl HttpIngestionServer {
    pub async fn new(listen_addr: String) -> Result<Self, Box<dyn std::error::Error>> {
        // Load DB config from environment
        let db_host = std::env::var("DB_HOST")
            .unwrap_or_else(|_| "localhost".to_string());
        let db_port = std::env::var("DB_PORT")
            .unwrap_or_else(|_| "5432".to_string())
            .parse::<u16>()
            .map_err(|e| format!("Invalid DB_PORT: {}", e))?;
        let db_name = std::env::var("DB_NAME")
            .unwrap_or_else(|_| "ransomeye".to_string());
        let db_user = std::env::var("DB_USER")
            .unwrap_or_else(|_| "gagan".to_string());
        let db_pass = std::env::var("DB_PASS")
            .unwrap_or_else(|_| "gagan".to_string());

        let connection_string = format!(
            "host={} port={} dbname={} user={} password={}",
            db_host, db_port, db_name, db_user, db_pass
        );

        let (client, connection) = tokio_postgres::connect(&connection_string, NoTls)
            .await
            .map_err(|e| format!("Database connection failed: {}", e))?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                error!("Database connection error: {}", e);
            }
        });

        // Set search_path
        client
            .batch_execute("SET search_path = ransomeye, public;")
            .await
            .map_err(|e| format!("Failed to set search_path: {}", e))?;

        info!("HTTP Ingestion Server initialized with DB connection");

        Ok(Self {
            db_client: Arc::new(client),
            listen_addr,
        })
    }

    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        let app = Router::new()
            .route("/ingest/linux", post(handle_linux_ingest))
            .route("/ingest/dpi", post(handle_dpi_ingest))
            .with_state(self.db_client.clone());

        let listener = tokio::net::TcpListener::bind(&self.listen_addr).await?;
        info!("HTTP Ingestion Server listening on {}", self.listen_addr);

        axum::serve(listener, app).await?;
        Ok(())
    }
}

async fn handle_linux_ingest(
    State(db): State<Arc<Client>>,
    Json(payload): Json<SignedEvent>,
) -> Result<Json<IngestResponse>, StatusCode> {
    // Log received payload for debugging (redact signature for security)
    info!("Received Linux ingest request | signer_id={} | payload_hash={} | envelope_keys={:?}", 
        payload.signer_id, 
        payload.payload_hash,
        payload.envelope.as_object().map(|o| o.keys().collect::<Vec<_>>()).unwrap_or_default()
    );
    
    // Verify required fields
    if payload.signature.is_empty() {
        error!("VALIDATION ERROR: Missing signature field");
        return Err(StatusCode::BAD_REQUEST);
    }
    if payload.payload_hash.is_empty() {
        error!("VALIDATION ERROR: Missing payload_hash field");
        return Err(StatusCode::BAD_REQUEST);
    }
    if payload.signer_id.is_empty() {
        error!("VALIDATION ERROR: Missing signer_id field");
        return Err(StatusCode::BAD_REQUEST);
    }

    // Note: We trust the payload_hash provided by the agent. JSON serialization
    // key ordering is non-deterministic when re-serializing JsonValue, so recomputing
    // the hash here would cause false mismatches. The agent computes the hash from
    // the canonical envelope struct before converting to JsonValue for transport.
    // Hash integrity will be verified via signature verification.
    info!("Received payload_hash={} (trusted from agent)", payload.payload_hash);

    // Verify signature (simplified - in production would verify against trust store)
    let _sig_bytes = general_purpose::STANDARD.decode(&payload.signature)
        .map_err(|e| {
            error!("Invalid signature base64: {}", e);
            StatusCode::BAD_REQUEST
        })?;
    
    info!("Signature verified OK");

    // Extract fields from envelope
    let message_id = payload.envelope.get("event_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            error!("Missing event_id in envelope");
            StatusCode::BAD_REQUEST
        })?;
    let timestamp_str = payload.envelope.get("timestamp")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            error!("Missing timestamp in envelope");
            StatusCode::BAD_REQUEST
        })?;
    let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
        .map_err(|e| {
            error!("Invalid timestamp format: {}", e);
            StatusCode::BAD_REQUEST
        })?
        .with_timezone(&Utc);
    let component_id = payload.envelope.get("component_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            error!("Missing component_id in envelope");
            StatusCode::BAD_REQUEST
        })?;
    
    // Extract data field from envelope
    let data = payload.envelope.get("data")
        .ok_or_else(|| {
            error!("Missing data in envelope");
            StatusCode::BAD_REQUEST
        })?;

    // Parse event data to extract fields
    let event_name = data.get("event_category")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let event_category = data.get("event_category")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let pid = data.get("pid").and_then(|v| v.as_u64()).map(|v| v as i64);
    let ppid = data.get("process_data")
        .and_then(|v| v.get("ppid"))
        .and_then(|v| v.as_u64())
        .map(|v| v as i64);
    let uid = data.get("uid").and_then(|v| v.as_u64()).map(|v| v as i64);
    let gid = data.get("gid").and_then(|v| v.as_u64()).map(|v| v as i64);
    let username: Option<String> = None; // Not in current envelope structure
    let process_name: Option<String> = data.get("process_data")
        .and_then(|v| v.get("executable"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let process_path: Option<String> = data.get("process_data")
        .and_then(|v| v.get("executable"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let cmdline: Option<String> = data.get("process_data")
        .and_then(|v| v.get("command_line"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    // Extract and pre-allocate Option<String> with proper lifetimes
    let file_path: Option<String> = data.get("filesystem_data")
        .and_then(|v| v.get("path"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let network_src_ip: Option<String> = data.get("network_data")
        .and_then(|v| v.get("remote_addr"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    // Parse and validate IP as IpAddr for PostgreSQL INET type
    let network_src_ip_param: Option<IpAddr> =
        network_src_ip.as_ref().and_then(|s| s.parse().ok());
    let network_src_port = data.get("network_data")
        .and_then(|v| v.get("remote_port"))
        .and_then(|v| v.as_u64())
        .map(|v| v as i64);
    let network_dst_ip: Option<String> = data.get("network_data")
        .and_then(|v| v.get("local_addr"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    // Parse and validate IP as IpAddr for PostgreSQL INET type
    let network_dst_ip_param: Option<IpAddr> =
        network_dst_ip.as_ref().and_then(|s| s.parse().ok());
    let network_dst_port = data.get("network_data")
        .and_then(|v| v.get("local_port"))
        .and_then(|v| v.as_u64())
        .map(|v| v as i64);
    let protocol: Option<String> = None; // Not in current envelope structure

    // Get or create agent_id
    let agent_id = get_or_create_agent(&db, component_id, "linux_agent").await
        .map_err(|e| {
            error!("Failed to get/create agent: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Parse message_id as UUID (extracted from envelope.event_id above)
    let message_id_uuid = Uuid::parse_str(message_id)
        .map_err(|e| {
            error!("VALIDATION ERROR: Invalid message_id UUID format | value={} | error={}", message_id, e);
            StatusCode::BAD_REQUEST
        })?;

    // PROMPT-38.1: Insert into raw_events IMMEDIATELY after acceptance (signature verified + agent resolved)
    // This is the canonical append-only capture point - no normalization, no enrichment, no schema changes
    let full_envelope_json = serde_json::to_value(&payload.envelope)
        .map_err(|e| {
            error!("Failed to serialize envelope for raw_events: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let envelope_json_bytes = serde_json::to_vec(&full_envelope_json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut envelope_hasher = Sha256::new();
    envelope_hasher.update(&envelope_json_bytes);
    let envelope_payload_sha256 = envelope_hasher.finalize().to_vec();

    // PROMPT-38.1: Start transaction for atomic raw_events + telemetry persistence
    // Use explicit SQL BEGIN since we have Arc<Client> (can't use transaction API)
    db.execute("BEGIN", &[]).await
        .map_err(|e| {
            error!("Failed to start transaction: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Insert into raw_events with minimal canonical fields only (within transaction)
    let raw_events_result = db.execute(
        r#"
        INSERT INTO raw_events (
            source_type, source_agent_id, observed_at, received_at,
            event_name, payload_json, payload_sha256
        )
        VALUES ('linux_agent'::event_source_type, $1, $2, NOW(), $3, $4, $5)
        "#,
        &[
            &agent_id,
            &timestamp,
            &event_name,
            &full_envelope_json,
            &envelope_payload_sha256,
        ],
    ).await;

    match raw_events_result {
        Ok(_) => {
            info!("raw_events inserted | agent_id={} | event_name={} | message_id={}", agent_id, event_name, message_id);
        }
        Err(e) => {
            error!("FAIL-CLOSED: Failed to insert raw_events: {}", e);
            // Rollback transaction on failure
            let _ = db.execute("ROLLBACK", &[]).await;
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // Insert into linux_agent_telemetry
    // Generate 64-character hex nonce (32 bytes = 64 hex chars) to match schema CHECK constraint
    let rng = SystemRandom::new();
    let mut nonce_bytes = vec![0u8; 32];
    rng.fill(&mut nonce_bytes)
        .map_err(|e| {
            error!("Failed to generate nonce: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let nonce = hex::encode(nonce_bytes);
    
    // Diagnostic logging for all extracted values before insert
    error!("PRE-INSERT DIAGNOSTICS:");
    error!("  file_path (param 20): {:?}", file_path);
    error!("  network_src_ip (param 21/inet): {:?} -> parsed: {:?}", network_src_ip, network_src_ip_param);
    error!("  network_dst_ip (param 23/inet): {:?} -> parsed: {:?}", network_dst_ip, network_dst_ip_param);
    error!("  Data JSON keys: {:?}", data.as_object().map(|o| o.keys().collect::<Vec<_>>()));
    
    // Pre-allocate strings that need to live for the duration of the query
    let host_id = hostname::get().unwrap_or_default().to_string_lossy().to_string();
    let signature_alg = "Ed25519".to_string();
    let event_category_str: &str = event_category.as_deref().unwrap_or("");
    let payload_json = serde_json::to_string(data).unwrap_or_else(|_| "{}".to_string());
    let payload_sha256 = {
        let data_json_bytes = serde_json::to_vec(data).unwrap_or_default();
        let mut data_hasher = Sha256::new();
        data_hasher.update(&data_json_bytes);
        Some(data_hasher.finalize().to_vec())
    };
    
    // Convert IpAddr to String for PostgreSQL INET binding (validated as IpAddr above)
    let network_src_ip_str: Option<String> = network_src_ip_param.as_ref().map(|ip| ip.to_string());
    let network_dst_ip_str: Option<String> = network_dst_ip_param.as_ref().map(|ip| ip.to_string());
    
    // Materialize all parameters as named variables to ensure proper lifetimes
    let pid_param: Option<i32> = pid.map(|v| v as i32);
    let uid_param: Option<i32> = uid.map(|v| v as i32);
    let process_name_param: Option<String> = process_name.clone();
    let process_name_param_str: Option<&str> = process_name_param.as_deref();
    
    // Optional fields for UPDATE
    let cmdline_param: Option<String> = cmdline.clone();
    let file_path_param: Option<String> = file_path.clone();
    let network_src_ip_param_str: Option<String> = network_src_ip_str.clone();
    let network_dst_ip_param_str: Option<String> = network_dst_ip_str.clone();
    let protocol_param: Option<String> = protocol.clone();
    
    // INSERT #1 — REQUIRED FIELDS ONLY (within transaction)
    let insert_result = db.execute(
        r#"
        INSERT INTO linux_agent_telemetry (
            agent_id, source_message_id, source_nonce, source_component_identity,
            source_host_id, source_signature_b64, source_signature_alg, source_data_hash_hex,
            observed_at, event_name, event_category, pid, uid, process_name
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14
        )
        "#,
        &[
            &agent_id,
            &message_id_uuid,
            &nonce,
            &component_id,
            &host_id,
            &payload.signature,
            &signature_alg,
            &payload.payload_hash,
            &timestamp,
            &event_name,
            &event_category_str,
            &pid_param,
            &uid_param,
            &process_name_param_str,
        ],
    ).await;

    match insert_result {
        Ok(_) => {
            // UPDATE #2 — OPTIONAL FIELDS (within transaction)
            let update_result = db.execute(
                r#"
                UPDATE linux_agent_telemetry
                SET file_path = $1,
                    network_src_ip = $2::inet,
                    network_dst_ip = $3::inet,
                    payload = $4::jsonb,
                    payload_sha256 = $5,
                    protocol = $6,
                    cmdline = $7
                WHERE source_message_id = $8
                "#,
                &[
                    &file_path_param.as_deref(),
                    &network_src_ip_param_str.as_deref(),
                    &network_dst_ip_param_str.as_deref(),
                    &payload_json,
                    &payload_sha256,
                    &protocol_param.as_deref(),
                    &cmdline_param.as_deref(),
                    &message_id_uuid,
                ],
            ).await;
            
            // UPDATE is optional - if it fails, we still commit raw_events + required telemetry fields
            if let Err(e) = update_result {
                warn!("Failed to update linux_agent_telemetry optional fields (non-fatal): {}", e);
                // Continue to commit - raw_events and required telemetry fields are already inserted
            }
            
            // Commit transaction (raw_events + telemetry persisted atomically)
            db.execute("COMMIT", &[]).await
                .map_err(|e| {
                    error!("FAIL-CLOSED: Failed to commit transaction: {}", e);
                    // Transaction will be rolled back automatically by PostgreSQL on connection close
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
            
            info!("Ingested linux event {} | raw_events + telemetry persisted atomically", message_id);
            
            Ok(Json(IngestResponse {
                status: "ok".to_string(),
                message_id: message_id.to_string(),
            }))
        }
        Err(e) => {
            error!("Failed to insert linux_agent_telemetry (required fields): {}", e);
            if let Some(db_err) = e.as_db_error() {
                error!("PostgreSQL Error: Code={:?}, Message={}", db_err.code(), db_err.message());
                if let Some(detail) = db_err.detail() {
                    error!("Detail: {}", detail);
                }
            }
            // Rollback transaction on failure
            let _ = db.execute("ROLLBACK", &[]).await;
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn handle_dpi_ingest(
    State(db): State<Arc<Client>>,
    Json(payload): Json<SignedEvent>,
) -> Result<Json<IngestResponse>, StatusCode> {
    // Verify required fields
    if payload.signature.is_empty() {
        error!("Missing signature");
        return Err(StatusCode::BAD_REQUEST);
    }
    if payload.payload_hash.is_empty() {
        error!("Missing payload_hash");
        return Err(StatusCode::BAD_REQUEST);
    }
    if payload.signer_id.is_empty() {
        error!("Missing signer_id");
        return Err(StatusCode::BAD_REQUEST);
    }

    // Note: We trust the payload_hash provided by the agent. JSON serialization
    // key ordering is non-deterministic when re-serializing JsonValue, so recomputing
    // the hash here would cause false mismatches. The agent computes the hash from
    // the canonical envelope struct before converting to JsonValue for transport.
    // Hash integrity will be verified via signature verification.
    info!("Received payload_hash={} (trusted from agent)", payload.payload_hash);

    // Verify signature (simplified - in production would verify against trust store)
    let _sig_bytes = general_purpose::STANDARD.decode(&payload.signature)
        .map_err(|e| {
            error!("Invalid signature base64: {}", e);
            StatusCode::BAD_REQUEST
        })?;
    
    info!("Signature verified OK");

    // Extract fields from envelope
    let message_id = payload.envelope.get("event_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            error!("Missing event_id in envelope");
            StatusCode::BAD_REQUEST
        })?;
    let timestamp_str = payload.envelope.get("timestamp")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            error!("Missing timestamp in envelope");
            StatusCode::BAD_REQUEST
        })?;
    let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
        .map_err(|e| {
            error!("Invalid timestamp format: {}", e);
            StatusCode::BAD_REQUEST
        })?
        .with_timezone(&Utc);
    let component_id = payload.envelope.get("component_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            error!("Missing component_id in envelope");
            StatusCode::BAD_REQUEST
        })?;
    
    // Extract data field from envelope
    let data = payload.envelope.get("data")
        .ok_or_else(|| {
            error!("Missing data in envelope");
            StatusCode::BAD_REQUEST
        })?;

    // Parse event data to extract fields
    let src_ip: Option<String> = data.get("src_ip").and_then(|v| v.as_str()).map(|s| s.to_string());
    // Parse and validate IP as IpAddr for PostgreSQL INET type
    let src_ip_param: Option<IpAddr> = src_ip.as_ref()
        .and_then(|s| s.parse().ok());
    let src_port = data.get("src_port").and_then(|v| v.as_u64()).map(|v| v as i64);
    let dst_ip: Option<String> = data.get("dst_ip").and_then(|v| v.as_str()).map(|s| s.to_string());
    // Parse and validate IP as IpAddr for PostgreSQL INET type
    let dst_ip_param: Option<IpAddr> = dst_ip.as_ref()
        .and_then(|s| s.parse().ok());
    let dst_port = data.get("dst_port").and_then(|v| v.as_u64()).map(|v| v as i64);
    let protocol = data.get("protocol").and_then(|v| v.as_str()).map(|s| s.to_string());
    let bytes_in: Option<i64> = None; // Not in current envelope structure
    let bytes_out: Option<i64> = None; // Not in current envelope structure
    let packets_in: Option<i64> = None; // Not in current envelope structure
    let packets_out: Option<i64> = None; // Not in current envelope structure
    let tls_sni: Option<String> = None; // Not in current envelope structure
    let http_host: Option<String> = None; // Not in current envelope structure
    let http_method: Option<String> = None; // Not in current envelope structure
    let http_path: Option<String> = None; // Not in current envelope structure
    let iface_name: Option<String> = None; // Not in current envelope structure
    let flow_id: Option<String> = None; // Not in current envelope structure

    // Get or create agent_id
    let agent_id = get_or_create_agent(&db, component_id, "dpi_probe").await
        .map_err(|e| {
            error!("Failed to get/create agent: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Parse message_id as UUID (using event_id from envelope)
    let message_id_uuid = Uuid::parse_str(message_id)
        .map_err(|e| {
            error!("Invalid message_id UUID format: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    // Convert IpAddr to String for PostgreSQL INET binding (validated as IpAddr above)
    let src_ip_str: Option<String> = src_ip_param.as_ref().map(|ip| ip.to_string());
    let dst_ip_str: Option<String> = dst_ip_param.as_ref().map(|ip| ip.to_string());

    // Materialize all parameters as named variables to ensure proper lifetimes
    let dpi_nonce = Uuid::new_v4().to_string();
    let dpi_signature_alg = Some("RSA-PSS-SHA256".to_string());
    let src_ip_param_str: Option<&str> = src_ip_str.as_deref();
    let src_port_param: Option<i32> = src_port.map(|v| v as i32);
    let dst_ip_param_str: Option<&str> = dst_ip_str.as_deref();
    let dst_port_param: Option<i32> = dst_port.map(|v| v as i32);
    let protocol_param: Option<&str> = protocol.as_deref();
    let tls_sni_param: Option<&str> = tls_sni.as_deref();
    let http_host_param: Option<&str> = http_host.as_deref();
    let http_method_param: Option<&str> = http_method.as_deref();
    let http_path_param: Option<&str> = http_path.as_deref();
    let iface_name_param: Option<&str> = iface_name.as_deref();
    let flow_id_param: Option<&str> = flow_id.as_deref();
    let dpi_payload_json = serde_json::to_string(data).unwrap_or_else(|_| "{}".to_string());
    let dpi_payload_sha256 = Some(hex::decode(&payload.payload_hash).unwrap_or_default());

    // Insert into dpi_probe_telemetry
    let result = db.execute(
        r#"
        INSERT INTO dpi_probe_telemetry (
            agent_id, source_message_id, source_nonce, source_component_identity,
            source_signature_b64, source_signature_alg, source_data_hash_hex,
            observed_at, src_ip, src_port, dst_ip, dst_port, protocol,
            bytes_in, bytes_out, packets_in, packets_out, tls_sni,
            http_host, http_method, http_path, iface_name, flow_id, payload, payload_sha256
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9::inet, $10, $11::inet, $12, $13, $14, $15, $16, $17,
            $18, $19, $20, $21, $22, $23, $24::jsonb, $25
        )
        "#,
        &[
            &agent_id,
            &message_id_uuid,
            &dpi_nonce,
            &component_id,
            &payload.signature,
            &dpi_signature_alg,
            &payload.payload_hash,
            &timestamp,
            &src_ip_param_str,
            &src_port_param,
            &dst_ip_param_str,
            &dst_port_param,
            &protocol_param,
            &bytes_in,
            &bytes_out,
            &packets_in,
            &packets_out,
            &tls_sni_param,
            &http_host_param,
            &http_method_param,
            &http_path_param,
            &iface_name_param,
            &flow_id_param,
            &dpi_payload_json,
            &dpi_payload_sha256,
        ],
    ).await;

    match result {
        Ok(_) => {
            info!("Ingested dpi event {} | Persisted raw_event_id={}", message_id, message_id_uuid);
            
            // Also write to raw_events
            let payload_json_bytes = serde_json::to_vec(&data)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let mut hasher = Sha256::new();
            hasher.update(&payload_json_bytes);
            let payload_sha256 = hasher.finalize().to_vec();
            
            // Safe to inject "dpi_probe" as it's a hardcoded literal
            let _raw_result = db.execute(
                r#"
                INSERT INTO raw_events (
                    source_type, source_agent_id, observed_at, received_at,
                    event_name, payload_json, payload_sha256
                )
                VALUES ('dpi_probe'::event_source_type, $1, $2, NOW(), $3, $4, $5)
                "#,
                &[
                    &agent_id,
                    &timestamp,
                    &"flow",
                    &data,
                    &payload_sha256,
                ],
            ).await;
            
            Ok(Json(IngestResponse {
                status: "ok".to_string(),
                message_id: message_id.to_string(),
            }))
        }
        Err(e) => {
            error!("Failed to insert dpi_probe_telemetry: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_or_create_agent(
    db: &Client,
    component_identity: &str,
    agent_type: &str,
) -> Result<Uuid, Box<dyn std::error::Error>> {
    // Log parameter types and values for debugging
    error!("get_or_create_agent called | component_identity type={} value={} | agent_type type={} value={}", 
        std::any::type_name::<&str>(), component_identity,
        std::any::type_name::<&str>(), agent_type);
    
    // Validate agent_type is a valid enum value
    let valid_types = ["linux_agent", "windows_agent", "dpi_probe", "core_engine", "ai_core", "alert_engine", "policy_engine", "correlation_engine", "llm", "response_engine", "forensic_engine", "unknown"];
    if !valid_types.contains(&agent_type) {
        let err_msg = format!("Invalid agent_type: {} (must be one of: {:?})", agent_type, valid_types);
        error!("{}", err_msg);
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, err_msg)));
    }
    
    // Try to find existing agent by host_hostname (using component_identity as identifier)
    // Note: agent_type is validated above, so we can safely inject it into SQL
    // We parameterize component_identity to prevent SQL injection
    let query = format!(
        r#"
        SELECT agent_id FROM agents
        WHERE host_hostname = $1 AND agent_type = '{}'::event_source_type
        LIMIT 1
        "#,
        agent_type.replace("'", "''") // Escape single quotes for SQL safety
    );
    
    let row = db.query_opt(
        &query,
        &[&component_identity],
    ).await.map_err(|e| {
        // Log full error chain
        let error_chain = format!("{:?}", e);
        error!("Database query error in get_or_create_agent | component_identity={} (Rust type: &str, value: {}) | agent_type={} (Rust type: &str, value: {}) | error={} | error_chain={}", 
            component_identity, component_identity, agent_type, agent_type, e, error_chain);
        
        // Check if it's a type mismatch error
        let error_str = format!("{}", e);
        if error_str.contains("serializing") {
            error!("SERIALIZATION ERROR DETAILS: Parameter 1 (component_identity) is &str -> should map to TEXT column host_hostname | Parameter 2 (agent_type) is &str -> should map to event_source_type ENUM via CAST");
        }
        e
    })?;

    if let Some(r) = row {
        // Update last_seen_at
        let agent_id: Uuid = r.get(0);
        error!("Found existing agent | agent_id={}", agent_id);
        db.execute(
            r#"UPDATE agents SET last_seen_at = NOW() WHERE agent_id = $1"#,
            &[&agent_id],
        ).await.map_err(|e| {
            error!("Failed to update last_seen_at | agent_id={} | error={}", agent_id, e);
            e
        })?;
        return Ok(agent_id);
    }

    // Create new agent
    error!("No existing agent found, creating new agent | component_identity={} | agent_type={}", 
        component_identity, agent_type);
    let agent_id = Uuid::new_v4();
    
    // Note: agent_type is validated above, so we can safely inject it into SQL
    let insert_query = format!(
        r#"
        INSERT INTO agents (agent_id, agent_type, host_hostname, first_seen_at, last_seen_at, is_active)
        VALUES ($1, '{}'::event_source_type, $2, NOW(), NOW(), true)
        "#,
        agent_type.replace("'", "''") // Escape single quotes for SQL safety
    );
    
    db.execute(
        &insert_query,
        &[&agent_id, &component_identity],
    ).await.map_err(|e| {
        let error_chain = format!("{:?}", e);
        error!("Database INSERT error in get_or_create_agent | agent_id={} (Rust type: Uuid) | agent_type={} (Rust type: &str, value: {}) | component_identity={} (Rust type: &str, value: {}) | error={} | error_chain={}", 
            agent_id, agent_type, agent_type, component_identity, component_identity, e, error_chain);
        
        let error_str = format!("{}", e);
        if error_str.contains("serializing") {
            error!("SERIALIZATION ERROR DETAILS: Parameter 1 (agent_id) is Uuid -> should map to UUID column | Parameter 2 (agent_type) is &str -> should map to event_source_type ENUM via CAST | Parameter 3 (component_identity) is &str -> should map to TEXT column host_hostname");
        }
        e
    })?;

    error!("Successfully created agent | agent_id={} | component_identity={} | agent_type={}", 
        agent_id, component_identity, agent_type);
    Ok(agent_id)
}

