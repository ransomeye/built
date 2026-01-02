// Path and File Name : /home/ransomeye/rebuild/core/network_scanner/src/persistence.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Database persistence for scan results, assets, port/service mappings, scan deltas

use sqlx::PgPool;
use chrono::{DateTime, Utc};
use serde_json;
use tracing::{error, info, debug};

use crate::errors::ScannerError;
use crate::result::ScanResult;

pub struct ScanPersistence {
    pub(crate) pool: PgPool,
}

impl ScanPersistence {
    pub async fn new() -> Result<Self, ScannerError> {
        // Get database connection from environment
        let db_host = std::env::var("DB_HOST")
            .unwrap_or_else(|_| "localhost".to_string());
        let db_port = std::env::var("DB_PORT")
            .unwrap_or_else(|_| "5432".to_string())
            .parse::<u16>()
            .map_err(|e| ScannerError::InvalidConfiguration(
                format!("Invalid DB_PORT: {}", e)
            ))?;
        let db_name = std::env::var("DB_NAME")
            .unwrap_or_else(|_| "ransomeye".to_string());
        let db_user = std::env::var("DB_USER")
            .unwrap_or_else(|_| "gagan".to_string());
        let db_pass = std::env::var("DB_PASS")
            .unwrap_or_else(|_| "gagan".to_string());
        
        let database_url = format!(
            "postgresql://{}:{}@{}:{}/{}",
            db_user, db_pass, db_host, db_port, db_name
        );
        
        let pool = PgPool::connect(&database_url).await
            .map_err(|e| ScannerError::DatabaseError(
                format!("Failed to connect to database: {}", e)
            ))?;
        
        // Initialize schema
        let persistence = Self { pool };
        persistence.initialize_schema().await?;
        
        Ok(persistence)
    }
    
    /// Initialize database schema
    async fn initialize_schema(&self) -> Result<(), ScannerError> {
        info!("Initializing network scanner persistence schema");
        
        // Scan results table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS scan_results (
                scan_id VARCHAR(36) PRIMARY KEY,
                timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
                scanner_mode VARCHAR(20) NOT NULL,
                asset_ip VARCHAR(45) NOT NULL,
                asset_hostname VARCHAR(255),
                asset_mac VARCHAR(17),
                asset_vendor VARCHAR(255),
                open_ports JSONB NOT NULL DEFAULT '[]'::jsonb,
                services JSONB NOT NULL DEFAULT '[]'::jsonb,
                confidence_score DOUBLE PRECISION NOT NULL,
                hash VARCHAR(64) NOT NULL,
                signature TEXT NOT NULL,
                metadata JSONB,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ScannerError::DatabaseError(
            format!("Failed to create scan_results table: {}", e)
        ))?;
        
        // Assets table (deduplicated)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS scan_assets (
                asset_id SERIAL PRIMARY KEY,
                ip VARCHAR(45) NOT NULL UNIQUE,
                hostname VARCHAR(255),
                mac VARCHAR(17),
                vendor VARCHAR(255),
                first_seen TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                last_seen TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                scan_count INTEGER NOT NULL DEFAULT 1
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ScannerError::DatabaseError(
            format!("Failed to create scan_assets table: {}", e)
        ))?;
        
        // Port/service mappings table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS scan_port_services (
                mapping_id SERIAL PRIMARY KEY,
                asset_ip VARCHAR(45) NOT NULL,
                port INTEGER NOT NULL,
                protocol VARCHAR(10) NOT NULL,
                service_name VARCHAR(255),
                service_version VARCHAR(255),
                first_seen TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                last_seen TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                UNIQUE(asset_ip, port, protocol)
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ScannerError::DatabaseError(
            format!("Failed to create scan_port_services table: {}", e)
        ))?;
        
        // Scan deltas table (what changed since last scan)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS scan_deltas (
                delta_id SERIAL PRIMARY KEY,
                scan_id VARCHAR(36) NOT NULL,
                asset_ip VARCHAR(45) NOT NULL,
                delta_type VARCHAR(20) NOT NULL,
                delta_data JSONB NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ScannerError::DatabaseError(
            format!("Failed to create scan_deltas table: {}", e)
        ))?;
        
        // Create indexes
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_scan_results_asset_ip 
            ON scan_results(asset_ip)
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ScannerError::DatabaseError(
            format!("Failed to create index: {}", e)
        ))?;
        
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_scan_results_timestamp 
            ON scan_results(timestamp)
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ScannerError::DatabaseError(
            format!("Failed to create index: {}", e)
        ))?;
        
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_scan_port_services_asset_ip 
            ON scan_port_services(asset_ip)
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ScannerError::DatabaseError(
            format!("Failed to create index: {}", e)
        ))?;
        
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_scan_deltas_asset_ip 
            ON scan_deltas(asset_ip)
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ScannerError::DatabaseError(
            format!("Failed to create index: {}", e)
        ))?;
        
        info!("Network scanner persistence schema initialized");
        Ok(())
    }
    
    /// Save scan result
    pub async fn save_result(&self, result: &ScanResult) -> Result<(), ScannerError> {
        let open_ports_json = serde_json::to_value(&result.open_ports)
            .map_err(|e| ScannerError::InternalError(
                format!("Failed to serialize open_ports: {}", e)
            ))?;
        
        let services_json = serde_json::to_value(&result.services)
            .map_err(|e| ScannerError::InternalError(
                format!("Failed to serialize services: {}", e)
            ))?;
        
        let metadata_json = result.metadata.as_ref()
            .map(|m| serde_json::to_value(m))
            .transpose()
            .map_err(|e| ScannerError::InternalError(
                format!("Failed to serialize metadata: {}", e)
            ))?;
        
        sqlx::query(
            r#"
            INSERT INTO scan_results (
                scan_id, timestamp, scanner_mode, asset_ip, asset_hostname, asset_mac, asset_vendor,
                open_ports, services, confidence_score, hash, signature, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (scan_id) DO NOTHING
            "#
        )
        .bind(&result.scan_id)
        .bind(&result.timestamp)
        .bind(format!("{:?}", result.scanner_mode))
        .bind(&result.asset.ip)
        .bind(&result.asset.hostname)
        .bind(&result.asset.mac)
        .bind(&result.asset.vendor)
        .bind(&open_ports_json)
        .bind(&services_json)
        .bind(result.confidence_score)
        .bind(&result.hash)
        .bind(&result.signature)
        .bind(&metadata_json)
        .execute(&self.pool)
        .await
        .map_err(|e| ScannerError::DatabaseError(
            format!("Failed to save scan result: {}", e)
        ))?;
        
        // Update or insert asset
        sqlx::query(
            r#"
            INSERT INTO scan_assets (ip, hostname, mac, vendor, last_seen, scan_count)
            VALUES ($1, $2, $3, $4, NOW(), 1)
            ON CONFLICT (ip) DO UPDATE SET
                hostname = COALESCE(EXCLUDED.hostname, scan_assets.hostname),
                mac = COALESCE(EXCLUDED.mac, scan_assets.mac),
                vendor = COALESCE(EXCLUDED.vendor, scan_assets.vendor),
                last_seen = NOW(),
                scan_count = scan_assets.scan_count + 1
            "#
        )
        .bind(&result.asset.ip)
        .bind(&result.asset.hostname)
        .bind(&result.asset.mac)
        .bind(&result.asset.vendor)
        .execute(&self.pool)
        .await
        .map_err(|e| ScannerError::DatabaseError(
            format!("Failed to save asset: {}", e)
        ))?;
        
        // Update port/service mappings
        for service in &result.services {
            sqlx::query(
                r#"
                INSERT INTO scan_port_services (asset_ip, port, protocol, service_name, service_version, last_seen)
                VALUES ($1, $2, $3, $4, $5, NOW())
                ON CONFLICT (asset_ip, port, protocol) DO UPDATE SET
                    service_name = EXCLUDED.service_name,
                    service_version = EXCLUDED.service_version,
                    last_seen = NOW()
                "#
            )
            .bind(&result.asset.ip)
            .bind(service.port as i32)
            .bind(&service.protocol)
            .bind(&service.service_name)
            .bind(&service.version)
            .execute(&self.pool)
            .await
            .map_err(|e| ScannerError::DatabaseError(
                format!("Failed to save port/service mapping: {}", e)
            ))?;
        }
        
        // Compute and save deltas
        self.compute_and_save_deltas(result).await?;
        
        Ok(())
    }
    
    /// Compute and save scan deltas (what changed since last scan)
    async fn compute_and_save_deltas(&self, result: &ScanResult) -> Result<(), ScannerError> {
        // Get previous scan result for this asset
        let previous_result = sqlx::query_as::<_, PreviousScanRow>(
            r#"
            SELECT open_ports, services
            FROM scan_results
            WHERE asset_ip = $1 AND scan_id != $2
            ORDER BY timestamp DESC
            LIMIT 1
            "#
        )
        .bind(&result.asset.ip)
        .bind(&result.scan_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ScannerError::DatabaseError(
            format!("Failed to fetch previous scan: {}", e)
        ))?;
        
        if let Some(previous) = previous_result {
            // Compare ports
            let previous_ports: Vec<u16> = serde_json::from_value(previous.open_ports)
                .unwrap_or_default();
            
            let current_ports: Vec<u16> = result.open_ports.iter().map(|p| p.port).collect();
            
            // Find new ports
            let new_ports: Vec<u16> = current_ports.iter()
                .filter(|p| !previous_ports.contains(p))
                .cloned()
                .collect();
            
            // Find closed ports
            let closed_ports: Vec<u16> = previous_ports.iter()
                .filter(|p| !current_ports.contains(p))
                .cloned()
                .collect();
            
            // Save deltas
            if !new_ports.is_empty() {
                sqlx::query(
                    r#"
                    INSERT INTO scan_deltas (scan_id, asset_ip, delta_type, delta_data)
                    VALUES ($1, $2, 'new_ports', $3)
                    "#
                )
                .bind(&result.scan_id)
                .bind(&result.asset.ip)
                .bind(serde_json::json!({ "ports": new_ports }))
                .execute(&self.pool)
                .await
                .map_err(|e| ScannerError::DatabaseError(
                    format!("Failed to save delta: {}", e)
                ))?;
            }
            
            if !closed_ports.is_empty() {
                sqlx::query(
                    r#"
                    INSERT INTO scan_deltas (scan_id, asset_ip, delta_type, delta_data)
                    VALUES ($1, $2, 'closed_ports', $3)
                    "#
                )
                .bind(&result.scan_id)
                .bind(&result.asset.ip)
                .bind(serde_json::json!({ "ports": closed_ports }))
                .execute(&self.pool)
                .await
                .map_err(|e| ScannerError::DatabaseError(
                    format!("Failed to save delta: {}", e)
                ))?;
            }
        } else {
            // First scan for this asset - mark as new asset
            sqlx::query(
                r#"
                INSERT INTO scan_deltas (scan_id, asset_ip, delta_type, delta_data)
                VALUES ($1, $2, 'new_asset', $3)
                "#
            )
            .bind(&result.scan_id)
            .bind(&result.asset.ip)
            .bind(serde_json::json!({ "ip": result.asset.ip }))
            .execute(&self.pool)
            .await
            .map_err(|e| ScannerError::DatabaseError(
                format!("Failed to save delta: {}", e)
            ))?;
        }
        
        Ok(())
    }
    
    /// Get scan results for an asset
    pub async fn get_asset_results(&self, asset_ip: &str) -> Result<Vec<ScanResult>, ScannerError> {
        let rows = sqlx::query_as::<_, ScanResultRow>(
            r#"
            SELECT scan_id, timestamp, scanner_mode, asset_ip, asset_hostname, asset_mac, asset_vendor,
                   open_ports, services, confidence_score, hash, signature, metadata
            FROM scan_results
            WHERE asset_ip = $1
            ORDER BY timestamp DESC
            "#
        )
        .bind(asset_ip)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ScannerError::DatabaseError(
            format!("Failed to fetch scan results: {}", e)
        ))?;
        
        let results: Result<Vec<ScanResult>, _> = rows.into_iter().map(|r| r.into()).collect();
        results.map_err(|e| ScannerError::DatabaseError(
            format!("Failed to convert scan result: {}", e)
        ))
    }
    
    /// Get scan deltas for an asset
    pub async fn get_asset_deltas(&self, asset_ip: &str) -> Result<Vec<ScanDelta>, ScannerError> {
        let rows = sqlx::query_as::<_, ScanDeltaRow>(
            r#"
            SELECT delta_id, scan_id, asset_ip, delta_type, delta_data, created_at
            FROM scan_deltas
            WHERE asset_ip = $1
            ORDER BY created_at DESC
            "#
        )
        .bind(asset_ip)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ScannerError::DatabaseError(
            format!("Failed to fetch scan deltas: {}", e)
        ))?;
        
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
}

#[derive(sqlx::FromRow)]
struct PreviousScanRow {
    open_ports: serde_json::Value,
    services: serde_json::Value,
}

#[derive(sqlx::FromRow)]
struct ScanResultRow {
    scan_id: String,
    timestamp: DateTime<Utc>,
    scanner_mode: String,
    asset_ip: String,
    asset_hostname: Option<String>,
    asset_mac: Option<String>,
    asset_vendor: Option<String>,
    open_ports: serde_json::Value,
    services: serde_json::Value,
    confidence_score: f64,
    hash: String,
    signature: String,
    metadata: Option<serde_json::Value>,
}

impl From<ScanResultRow> for ScanResult {
    fn from(row: ScanResultRow) -> Self {
        use crate::result::{ScannerMode, Asset, PortInfo, Service, ScanMetadata};
        
        let scanner_mode = match row.scanner_mode.as_str() {
            "Active" => ScannerMode::Active,
            "Passive" => ScannerMode::Passive,
            _ => ScannerMode::Active,
        };
        
        let open_ports: Vec<PortInfo> = serde_json::from_value(row.open_ports)
            .unwrap_or_default();
        
        let services: Vec<Service> = serde_json::from_value(row.services)
            .unwrap_or_default();
        
        let metadata: Option<ScanMetadata> = row.metadata
            .and_then(|m| serde_json::from_value(m).ok());
        
        ScanResult {
            scan_id: row.scan_id,
            timestamp: row.timestamp,
            scanner_mode,
            asset: Asset {
                ip: row.asset_ip,
                hostname: row.asset_hostname,
                mac: row.asset_mac,
                vendor: row.asset_vendor,
            },
            open_ports,
            services,
            confidence_score: row.confidence_score,
            hash: row.hash,
            signature: row.signature,
            metadata,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScanDelta {
    pub delta_id: i32,
    pub scan_id: String,
    pub asset_ip: String,
    pub delta_type: String,
    pub delta_data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct ScanDeltaRow {
    delta_id: i32,
    scan_id: String,
    asset_ip: String,
    delta_type: String,
    delta_data: serde_json::Value,
    created_at: DateTime<Utc>,
}

impl From<ScanDeltaRow> for ScanDelta {
    fn from(row: ScanDeltaRow) -> Self {
        ScanDelta {
            delta_id: row.delta_id,
            scan_id: row.scan_id,
            asset_ip: row.asset_ip,
            delta_type: row.delta_type,
            delta_data: row.delta_data,
            created_at: row.created_at,
        }
    }
}

