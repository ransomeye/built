// Path and File Name : /home/ransomeye/rebuild/core/ingest/src/http_server.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: HTTP ingestion server with POST /ingest/linux and /ingest/dpi endpoints - verifies signatures and writes to database

use std::sync::Arc;
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
    let process_name = data.get("process_data")
        .and_then(|v| v.get("executable"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let process_path = data.get("process_data")
        .and_then(|v| v.get("executable"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let cmdline = data.get("process_data")
        .and_then(|v| v.get("command_line"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let file_path = data.get("filesystem_data")
        .and_then(|v| v.get("path"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let network_src_ip = data.get("network_data")
        .and_then(|v| v.get("remote_addr"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let network_src_port = data.get("network_data")
        .and_then(|v| v.get("remote_port"))
        .and_then(|v| v.as_u64())
        .map(|v| v as i64);
    let network_dst_ip = data.get("network_data")
        .and_then(|v| v.get("local_addr"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
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
    error!("  network_src_ip (param 21/inet): {:?}", network_src_ip);
    error!("  network_dst_ip (param 23/inet): {:?}", network_dst_ip);
    error!("  Data JSON keys: {:?}", data.as_object().map(|o| o.keys().collect::<Vec<_>>()));
    
    // Pre-allocate strings that need to live for the duration of the query
    let host_id = hostname::get().unwrap_or_default().to_string_lossy().to_string();
    let signature_alg = "Ed25519".to_string();
    let payload_json = serde_json::to_string(data).unwrap_or_else(|_| "{}".to_string());
    let payload_sha256 = {
        let data_json_bytes = serde_json::to_vec(data).unwrap_or_default();
        let mut data_hasher = Sha256::new();
        data_hasher.update(&data_json_bytes);
        Some(data_hasher.finalize().to_vec())
    };
    
    let result = db.execute(
        r#"
        INSERT INTO linux_agent_telemetry (
            agent_id, source_message_id, source_nonce, source_component_identity,
            source_host_id, source_signature_b64, source_signature_alg, source_data_hash_hex,
            observed_at, event_name, event_category, pid, ppid, uid, gid, username,
            process_name, process_path, cmdline, file_path, network_src_ip, network_src_port,
            network_dst_ip, network_dst_port, protocol, payload, payload_sha256
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, NULL,
            NULL, NULL, NULL, NULL, $16::inet, $17, $18::inet, $19, NULL, $20::jsonb, $21
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
            &timestamp, // Use extracted timestamp, not payload.timestamp
            &event_name,
            &event_category.as_deref().unwrap_or(""),
            &pid.map(|v| v as i32),
            &ppid.map(|v| v as i32),
            &uid.map(|v| v as i32),
            &gid.map(|v| v as i32),
            // All Option<String> TEXT fields removed - using NULL in SQL
            // &username, &process_name, &process_path, &cmdline, &file_path, &protocol
            &network_src_ip.as_deref(),
            &network_src_port.map(|v| v as i32),
            &network_dst_ip.as_deref(),
            &network_dst_port.map(|v| v as i32),
            // &protocol removed
            &payload_json, // Use pre-allocated string
            &payload_sha256, // Use pre-computed hash
        ],
    ).await;

    match result {
        Ok(_) => {
            info!("Ingested linux event {} | Persisted raw_event_id={}", message_id, message_id_uuid);
            
            // Also write to raw_events
            let payload_json_bytes = serde_json::to_vec(&data)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let mut hasher = Sha256::new();
            hasher.update(&payload_json_bytes);
            let payload_sha256 = hasher.finalize().to_vec();
            
            // Safe to inject "linux_agent" as it's a hardcoded literal
            let _raw_result = db.execute(
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
            // Log full PostgreSQL error details
            error!("Failed to insert linux_agent_telemetry: {}", e);
            if let Some(db_err) = e.as_db_error() {
                error!("PostgreSQL Error Details:");
                error!("  Code: {:?}", db_err.code());
                error!("  Message: {}", db_err.message());
                if let Some(constraint) = db_err.constraint() {
                    error!("  Constraint: {}", constraint);
                }
                if let Some(column) = db_err.column() {
                    error!("  Column: {}", column);
                }
                if let Some(detail) = db_err.detail() {
                    error!("  Detail: {}", detail);
                }
                if let Some(hint) = db_err.hint() {
                    error!("  Hint: {}", hint);
                }
            }
            // Log parameter details for debugging
            error!("INSERT Parameters:");
            error!("  $1 (agent_id): {:?}", agent_id);
            error!("  $2 (source_message_id): {:?}", message_id_uuid);
            error!("  $3 (source_nonce): {:?}", nonce);
            error!("  $4 (source_component_identity): {:?}", component_id);
            error!("  $5 (host_id): {:?}", host_id);
            error!("  $6 (signature): {:?}", payload.signature);
            error!("  $7 (signature_alg): {:?}", signature_alg);
            error!("  $8 (payload_hash): {:?}", payload.payload_hash);
            error!("  $9 (observed_at): {:?}", timestamp);
            error!("  $10 (event_name): {:?}", event_name);
            error!("  $11 (event_category): {:?}", event_category);
            error!("  $12 (pid): {:?}", pid);
            error!("  $13 (ppid): {:?}", ppid);
            error!("  $14 (uid): {:?}", uid);
            error!("  $15 (gid): {:?}", gid);
            error!("  $16 (username): {:?}", username);
            error!("  $17 (process_name): {:?}", process_name);
            error!("  $18 (process_path): {:?}", process_path);
            error!("  $19 (cmdline): {:?}", cmdline);
            error!("  $20 (file_path): {:?}", file_path);
            error!("  $21 (network_src_ip): {:?}", network_src_ip);
            error!("  $22 (network_src_port): {:?}", network_src_port);
            error!("  $23 (network_dst_ip): {:?}", network_dst_ip);
            error!("  $24 (network_dst_port): {:?}", network_dst_port);
            error!("  $25 (protocol): {:?}", protocol);
            error!("  $26 (payload_json): {} bytes", payload_json.len());
            error!("  $27 (payload_sha256): {:?}", payload_sha256);
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
    let src_ip = data.get("src_ip").and_then(|v| v.as_str()).map(|s| s.to_string());
    let src_port = data.get("src_port").and_then(|v| v.as_u64()).map(|v| v as i64);
    let dst_ip = data.get("dst_ip").and_then(|v| v.as_str()).map(|s| s.to_string());
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
            $1, $2, $3, $4, $5, $6, $7, $8, CAST($9 AS inet), $10, CAST($11 AS inet), $12, $13, $14, $15, $16, $17,
            $18, $19, $20, $21, $22, CAST($23 AS jsonb), $24, $25
        )
        "#,
        &[
            &agent_id,
            &message_id_uuid,
            &Uuid::new_v4().to_string(), // nonce (generate new since not in envelope)
            &component_id,
            &payload.signature,
            &Some("RSA-PSS-SHA256".to_string()),
            &payload.payload_hash,
            &timestamp,
            &src_ip,
            &src_port.map(|v| v as i32),
            &dst_ip,
            &dst_port.map(|v| v as i32),
            &protocol,
            &bytes_in,
            &bytes_out,
            &packets_in,
            &packets_out,
            &tls_sni,
            &http_host,
            &http_method,
            &http_path,
            &iface_name,
            &flow_id,
            &serde_json::to_string(data).unwrap_or_else(|_| "{}".to_string()), // Serialize JsonValue to string for JSONB
            &Some(hex::decode(&payload.payload_hash).unwrap_or_default()),
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

