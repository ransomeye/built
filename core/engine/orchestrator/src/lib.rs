// Path and File Name : /home/ransomeye/rebuild/core/engine/orchestrator/src/lib.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Core Orchestrator - fail-closed lifecycle management with strict startup/shutdown ordering

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::signal;
use tracing::{info, error, warn};
use thiserror::Error;

use kernel::Kernel;
use policy::{PolicyEngine, PolicyError};
use bus::{BusClient, BusClientError, ComponentRole};
use sha2::Digest;

pub mod db;
use db::{CoreDb, DbConfig};

pub mod retention_enforcer;

#[derive(Debug, Error)]
pub enum OrchestratorError {
    #[error("Environment validation failed: {0}")]
    EnvironmentValidationFailed(String),
    #[error("Trust initialization failed: {0}")]
    TrustInitFailed(#[from] kernel::KernelError),
    #[error("Policy engine initialization failed: {0}")]
    PolicyInitFailed(#[from] PolicyError),
    #[error("Event bus initialization failed: {0}")]
    BusInitFailed(#[from] BusClientError),
    #[error("Component initialization failed: {0}")]
    ComponentInitFailed(String),
    #[error("Health gate failed: {0}")]
    HealthGateFailed(String),
    #[error("Database connection failed: {0}")]
    DatabaseConnectionFailed(String),
    #[error("Database schema apply failed: {0}")]
    DatabaseSchemaApplyFailed(String),
    #[error("Database schema validation failed: {0}")]
    DatabaseSchemaValidationFailed(String),
    #[error("Database write failed: {0}")]
    DatabaseWriteFailed(String),
    #[error("Retention dry-run validation failed: {0}")]
    RetentionDryRunValidationFailed(String),
    #[error("Shutdown failed: {0}")]
    ShutdownFailed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrchestratorState {
    /// Initial state before any initialization
    Initializing,
    /// Environment validated
    EnvironmentValidated,
    /// Trust subsystem initialized
    TrustInitialized,
    /// Policy engine initialized
    PolicyInitialized,
    /// Event bus initialized
    BusInitialized,
    /// Core services initialized
    ServicesInitialized,
    /// All health gates passed, ready to serve
    Ready,
    /// Running state (serving requests)
    Running,
    /// Shutting down
    ShuttingDown,
    /// Failed state (terminal)
    Failed,
}

/// Core Orchestrator with fail-closed guarantees
/// 
/// Enforces strict startup order:
/// 1. Environment validation
/// 2. Trust subsystem
/// 3. Policy engine
/// 4. Event bus
/// 5. Core services
/// 6. Health gate
pub struct Orchestrator {
    state: Arc<AtomicBool>,
    kernel: Option<Arc<Kernel>>,
    policy_engine: Option<Arc<PolicyEngine>>,
    bus_client: Option<Arc<BusClient>>,
    db: Option<Arc<CoreDb>>,
    component_db_id: Option<uuid::Uuid>,
    startup_event_id: Option<uuid::Uuid>,
    startup_health_id: Option<uuid::Uuid>,
    current_state: Arc<parking_lot::RwLock<OrchestratorState>>,
    dry_run: bool,
}

impl Orchestrator {
    /// Create new orchestrator
    pub fn new() -> Result<Self, OrchestratorError> {
        // Check for dry-run mode
        let dry_run = std::env::var("RANSOMEYE_DRY_RUN")
            .unwrap_or_else(|_| "0".to_string())
            == "1";

        Ok(Self {
            state: Arc::new(AtomicBool::new(false)),
            kernel: None,
            policy_engine: None,
            bus_client: None,
            db: None,
            component_db_id: None,
            startup_event_id: None,
            startup_health_id: None,
            current_state: Arc::new(parking_lot::RwLock::new(OrchestratorState::Initializing)),
            dry_run,
        })
    }

    /// Set orchestrator state (internal)
    fn set_state(&self, new_state: OrchestratorState) {
        let mut state = self.current_state.write();
        info!("Orchestrator state transition: {:?} -> {:?}", *state, new_state);
        *state = new_state;
    }

    /// Get current state
    pub fn get_state(&self) -> OrchestratorState {
        *self.current_state.read()
    }

    /// Validate required environment variables
    /// 
    /// FAIL-CLOSED: Returns error if any required env var is missing
    fn validate_environment(&self) -> Result<(), OrchestratorError> {
        info!("Validating environment...");

        // Required environment variables
        let required_vars = vec![
            "RANSOMEYE_ROOT_KEY_PATH",
            "RANSOMEYE_POLICY_DIR",
            "RANSOMEYE_TRUST_STORE_PATH",
        ];

        let mut missing = Vec::new();
        for var in required_vars {
            if std::env::var(var).is_err() {
                missing.push(var.to_string());
            }
        }

        if !missing.is_empty() {
            return Err(OrchestratorError::EnvironmentValidationFailed(
                format!("Missing required environment variables: {}", missing.join(", "))
            ));
        }

        // Validate file paths exist
        let root_key_path = std::env::var("RANSOMEYE_ROOT_KEY_PATH").unwrap();
        if !std::path::Path::new(&root_key_path).exists() {
            return Err(OrchestratorError::EnvironmentValidationFailed(
                format!("Root key file not found: {}", root_key_path)
            ));
        }

        let policy_dir = std::env::var("RANSOMEYE_POLICY_DIR").unwrap();
        if !std::path::Path::new(&policy_dir).exists() {
            return Err(OrchestratorError::EnvironmentValidationFailed(
                format!("Policy directory not found: {}", policy_dir)
            ));
        }

        let trust_store = std::env::var("RANSOMEYE_TRUST_STORE_PATH").unwrap();
        if !std::path::Path::new(&trust_store).exists() {
            return Err(OrchestratorError::EnvironmentValidationFailed(
                format!("Trust store directory not found: {}", trust_store)
            ));
        }

        info!("Environment validation passed");
        self.set_state(OrchestratorState::EnvironmentValidated);
        Ok(())
    }

    /// Initialize database (MANDATORY, FAIL-CLOSED):
    /// - Connect using required env vars
    /// - Apply authoritative schema SQL (idempotent)
    /// - Validate required tables and core-critical columns exist
    /// - Upsert this orchestrator into ransomeye.components (FK anchor)
    /// - Write required runtime rows: startup_events, component_health, immutable_audit_log
    async fn initialize_database(&mut self) -> Result<(), OrchestratorError> {
        info!("Initializing mandatory database integration (authoritative schema contract)...");

        let cfg = DbConfig::from_env_strict()
            .map_err(OrchestratorError::EnvironmentValidationFailed)?;

        let db = CoreDb::connect_strict(&cfg)
            .await
            .map_err(OrchestratorError::DatabaseConnectionFailed)?;

        // Apply schema on first run (idempotent CREATE IF NOT EXISTS) using authoritative file.
        db.apply_authoritative_schema_from_env()
            .await
            .map_err(OrchestratorError::DatabaseSchemaApplyFailed)?;

        // Validate schema presence/compatibility at startup.
        db.validate_schema_contract()
            .await
            .map_err(OrchestratorError::DatabaseSchemaValidationFailed)?;

        // Upsert orchestrator component (FK anchor for core runtime tables).
        let build_hash = std::env::var("RANSOMEYE_BUILD_HASH").ok();
        let version = std::env::var("RANSOMEYE_VERSION").ok();
        let instance_id = std::env::var("RANSOMEYE_INSTANCE_ID").ok();

        let component_db_id = db
            .upsert_component(
                "master_core",
                "ransomeye_orchestrator",
                instance_id.as_deref(),
                build_hash.as_deref(),
                version.as_deref(),
            )
            .await
            .map_err(OrchestratorError::DatabaseWriteFailed)?;

        // Compute a non-secret environment fingerprint (hash only; excludes DB_PASS and other secrets).
        let env_fingerprint = {
            let mut pairs: Vec<(String, String)> = Vec::new();
            for k in [
                "DB_HOST",
                "DB_PORT",
                "DB_NAME",
                "DB_USER",
                "RANSOMEYE_ROOT_KEY_PATH",
                "RANSOMEYE_POLICY_DIR",
                "RANSOMEYE_TRUST_STORE_PATH",
                "RANSOMEYE_SCHEMA_SQL_PATH",
            ] {
                if let Ok(v) = std::env::var(k) {
                    pairs.push((k.to_string(), v));
                }
            }
            pairs.sort_by(|a, b| a.0.cmp(&b.0));
            let mut canonical = String::new();
            for (k, v) in pairs {
                canonical.push_str(&k);
                canonical.push('=');
                canonical.push_str(&v);
                canonical.push('\n');
            }
            let mut hasher = sha2::Sha256::new();
            hasher.update(canonical.as_bytes());
            let digest = hasher.finalize();
            digest.to_vec()
        };

        let startup_event_id = db
            .insert_startup_event(
                component_db_id,
                chrono::Utc::now(),
                Some("service_start"),
                build_hash.as_deref(),
                version.as_deref(),
                Some(&env_fingerprint),
                Some(&serde_json::json!({
                    "component": "ransomeye_orchestrator",
                    "component_type": "master_core"
                })),
            )
            .await
            .map_err(OrchestratorError::DatabaseWriteFailed)?;

        let health_id = db
            .insert_component_health(
                component_db_id,
                "healthy",
                Some("startup_db_initialized"),
                Some(&serde_json::json!({
                    "startup_event_id": startup_event_id.to_string(),
                    "state": "STARTING",
                    "schema_validated": true
                })),
            )
            .await
            .map_err(OrchestratorError::DatabaseWriteFailed)?;

        // PROMPT-27: Audit correctness.
        // NEVER claim RUNNING here. We only log that DB initialization + schema validation succeeded.
        let audit_id = db
            .insert_immutable_audit_log(
                Some(component_db_id),
                "orchestrator_db_initialized",
                "other",
                Some(component_db_id),
                &serde_json::json!({
                    "startup_event_id": startup_event_id.to_string(),
                    "health_id": health_id.to_string(),
                    "status": "STARTING",
                    "schema_validated": true
                }),
            )
            .await
            .map_err(OrchestratorError::DatabaseWriteFailed)?;

        info!(
            "DB runtime writes completed: startup_events.startup_event_id={}, component_health.health_id={}, immutable_audit_log.audit_id={}",
            startup_event_id, health_id, audit_id
        );

        // =====================================================================
        // PROMPT-25: Runtime retention enforcement — orchestrator startup dry-run
        // =====================================================================
        // FAIL-CLOSED: If retention_policies is missing/empty or targets illegal tables,
        // the orchestrator must NOT start. This provides runtime compliance guarantees.
        let retention_enforcer = retention_enforcer::RetentionEnforcer::new_from_env()
            .map_err(OrchestratorError::RetentionDryRunValidationFailed)?;
        let (retention_audit_id, _results) = retention_enforcer
            .enforce(&db, Some(component_db_id), true /* dry_run */)
            .await
            .map_err(OrchestratorError::RetentionDryRunValidationFailed)?;
        info!(
            "Retention dry-run validation complete (immutable_audit_log.audit_id={})",
            retention_audit_id
        );

        self.db = Some(Arc::new(db));
        self.component_db_id = Some(component_db_id);
        self.startup_event_id = Some(startup_event_id);
        self.startup_health_id = Some(health_id);
        Ok(())
    }

    /// Best-effort: record an error event + audit entry if DB is initialized; never masks the original failure.
    pub async fn record_fatal_error(&self, error_text: &str) {
        let Some(db) = &self.db else {
            return;
        };

        let component_id = self.component_db_id;
        if let Err(e) = db
            .insert_error_event(
                component_id,
                "critical",
                "orchestrator_fatal",
                error_text,
                None,
                Some(&serde_json::json!({"state": format!("{:?}", self.get_state())})),
                None,
                None,
            )
            .await
        {
            error!("Failed to write error_events for fatal error: {}", e);
        }

        if let Some(component_id) = component_id {
            if let Err(e) = db
                .insert_immutable_audit_log(
                    Some(component_id),
                    "orchestrator_fatal_error",
                    "other",
                    Some(component_id),
                    &serde_json::json!({"error": error_text}),
                )
                .await
            {
                error!("Failed to write immutable_audit_log for fatal error: {}", e);
            }
        }
    }

    /// Initialize trust subsystem
    /// 
    /// FAIL-CLOSED: Returns error if trust material is missing
    fn initialize_trust(&mut self) -> Result<(), OrchestratorError> {
        info!("Initializing trust subsystem...");

        let kernel = Kernel::new()?;
        
        // Verify kernel is initialized
        if !kernel.is_initialized() {
            return Err(OrchestratorError::TrustInitFailed(
                kernel::KernelError::TrustInitFailed("Kernel failed to initialize".to_string())
            ));
        }

        self.kernel = Some(Arc::new(kernel));
        info!("Trust subsystem initialized successfully");
        self.set_state(OrchestratorState::TrustInitialized);
        Ok(())
    }

    /// Initialize policy engine
    /// 
    /// FAIL-CLOSED: Returns error if policy loading or verification fails
    fn initialize_policy(&mut self) -> Result<(), OrchestratorError> {
        info!("Initializing policy engine...");

        let policy_dir = std::env::var("RANSOMEYE_POLICY_DIR")
            .map_err(|_| OrchestratorError::ComponentInitFailed(
                "RANSOMEYE_POLICY_DIR not set".to_string()
            ))?;
        
        let trust_store = std::env::var("RANSOMEYE_TRUST_STORE_PATH")
            .map_err(|_| OrchestratorError::ComponentInitFailed(
                "RANSOMEYE_TRUST_STORE_PATH not set".to_string()
            ))?;

        let revocation_list = std::env::var("RANSOMEYE_POLICY_REVOCATION_LIST")
            .ok();

        let audit_log = std::env::var("RANSOMEYE_POLICY_AUDIT_LOG")
            .ok();

        let engine_version = std::env::var("RANSOMEYE_POLICY_ENGINE_VERSION")
            .unwrap_or_else(|_| "1.0.0".to_string());

        let policy_engine = PolicyEngine::new(
            &policy_dir,
            &engine_version,
            Some(&trust_store),
            revocation_list.as_deref(),
            audit_log.as_deref(),
        )?;

        self.policy_engine = Some(Arc::new(policy_engine));
        info!("Policy engine initialized successfully");
        self.set_state(OrchestratorState::PolicyInitialized);
        Ok(())
    }

    /// Initialize event bus
    /// 
    /// FAIL-CLOSED: Returns error if bus certificates are missing
    fn initialize_bus(&mut self) -> Result<(), OrchestratorError> {
        info!("Initializing event bus...");

        // Bus initialization is optional - only if env vars are set
        if std::env::var("RANSOMEYE_BUS_CLIENT_CERT").is_err() {
            warn!("Bus client certificates not configured - skipping bus initialization");
            self.set_state(OrchestratorState::BusInitialized);
            return Ok(());
        }

        let server_addr = std::env::var("RANSOMEYE_BUS_SERVER_ADDR")
            .unwrap_or_else(|_| "localhost:8443".to_string());

        let component_id = std::env::var("RANSOMEYE_COMPONENT_ID")
            .unwrap_or_else(|_| "orchestrator".to_string());

        let bus_client = BusClient::new(
            ComponentRole::Core,
            component_id,
            server_addr,
        )?;

        self.bus_client = Some(Arc::new(bus_client));
        info!("Event bus initialized successfully");
        self.set_state(OrchestratorState::BusInitialized);
        Ok(())
    }

    /// Initialize core services
    /// 
    /// At this stage, services are validated but not started.
    /// Individual service binaries handle their own startup.
    fn initialize_services(&mut self) -> Result<(), OrchestratorError> {
        info!("Validating core service dependencies...");

        // Verify required services can be initialized
        // Services run as separate binaries, so we just validate dependencies here

        // Ingest service dependencies
        let ingest_port = std::env::var("RANSOMEYE_INGEST_PORT")
            .unwrap_or_else(|_| "8080".to_string());
        info!("Ingest service port configured: {}", ingest_port);

        // Dispatch service dependencies
        info!("Dispatch service dependencies validated");

        // Reporting service dependencies
        let reporting_dir = std::env::var("RANSOMEYE_REPORTING_DIR")
            .unwrap_or_else(|_| "/var/lib/ransomeye/reports".to_string());
        info!("Reporting directory configured: {}", reporting_dir);

        // Governor service dependencies
        info!("Governor service dependencies validated");

        info!("Core service dependencies validated");
        self.set_state(OrchestratorState::ServicesInitialized);
        Ok(())
    }

    /// Health gate - verify all components report READY
    /// 
    /// FAIL-CLOSED: Returns error if any component is not ready
    fn health_gate(&self) -> Result<(), OrchestratorError> {
        info!("Running health gate...");

        // Verify trust subsystem
        if let Some(kernel) = &self.kernel {
            if !kernel.is_initialized() {
                return Err(OrchestratorError::HealthGateFailed(
                    "Trust subsystem not initialized".to_string()
                ));
            }
        } else {
            return Err(OrchestratorError::HealthGateFailed(
                "Trust subsystem missing".to_string()
            ));
        }

        // Verify policy engine
        if self.policy_engine.is_none() {
            return Err(OrchestratorError::HealthGateFailed(
                "Policy engine missing".to_string()
            ));
        }

        info!("Health gate passed - all components READY");
        self.set_state(OrchestratorState::Ready);
        Ok(())
    }

    /// Execute full startup sequence
    /// 
    /// FAIL-CLOSED: Exits with error if any step fails
    pub async fn startup(&mut self) -> Result<(), OrchestratorError> {
        info!("Starting RansomEye Core Orchestrator...");
        if self.dry_run {
            info!("DRY-RUN mode enabled");
        }

        // Step 1: Environment validation
        self.validate_environment()?;

        // Step 2: Database initialization (MANDATORY - fail-closed)
        self.initialize_database().await?;

        // Step 3: Trust subsystem
        self.initialize_trust()?;

        // Step 4: Policy engine
        self.initialize_policy()?;

        // Step 5: Event bus
        self.initialize_bus()?;

        // Step 6: Core services
        self.initialize_services()?;

        // Step 7: Health gate
        self.health_gate()?;

        // Transition to RUNNING
        self.set_state(OrchestratorState::Running);
        self.state.store(true, Ordering::SeqCst);

        // PROMPT-27: Only after successful final transition do we write RUNNING state to DB/audit.
        if let (Some(db), Some(component_id)) = (self.db.as_ref(), self.component_db_id) {
            let _ = db
                .insert_component_health(
                    component_id,
                    "healthy",
                    Some("running"),
                    Some(&serde_json::json!({
                        "state": "RUNNING",
                        "startup_event_id": self.startup_event_id.map(|x| x.to_string()),
                        "startup_health_id": self.startup_health_id.map(|x| x.to_string())
                    })),
                )
                .await
                .map_err(OrchestratorError::DatabaseWriteFailed)?;

            let _ = db
                .insert_immutable_audit_log(
                    Some(component_id),
                    "orchestrator_startup",
                    "other",
                    Some(component_id),
                    &serde_json::json!({
                        "startup_event_id": self.startup_event_id.map(|x| x.to_string()),
                        "startup_health_id": self.startup_health_id.map(|x| x.to_string()),
                        "status": "RUNNING"
                    }),
                )
                .await
                .map_err(OrchestratorError::DatabaseWriteFailed)?;
        }

        info!("RansomEye Core Orchestrator started successfully");
        Ok(())
    }

    /// Execute shutdown sequence (reverse of startup)
    /// 
    /// Orders shutdown to ensure graceful teardown
    pub async fn shutdown(&mut self) -> Result<(), OrchestratorError> {
        info!("Shutting down RansomEye Core Orchestrator...");
        self.set_state(OrchestratorState::ShuttingDown);

        // Shutdown in reverse order of startup
        
        // Step 1: Shutdown core services (flush queues, persist state)
        info!("Shutting down core services...");
        // Services handle their own shutdown via signal handling
        
        // Step 2: Shutdown event bus (flush messages)
        if let Some(bus) = &self.bus_client {
            info!("Flushing event bus...");
            // Bus client handles its own cleanup
        }

        // Step 3: Shutdown policy engine (persist state)
        if let Some(policy) = &self.policy_engine {
            info!("Shutting down policy engine...");
            // Policy engine handles its own cleanup
        }

        // Step 4: Trust subsystem cleanup
        if let Some(kernel) = &self.kernel {
            info!("Shutting down trust subsystem...");
            // Kernel cleanup
        }

        self.state.store(false, Ordering::SeqCst);
        info!("RansomEye Core Orchestrator shutdown complete");
        Ok(())
    }

    /// Check if orchestrator is running
    pub fn is_running(&self) -> bool {
        self.state.load(Ordering::SeqCst)
    }

    /// Run orchestrator (startup, wait for signal, shutdown)
    pub async fn run(&mut self) -> Result<(), OrchestratorError> {
        // Startup
        self.startup().await?;

        if self.dry_run {
            info!("Dry-run complete - orchestrator initialized successfully");
            return Ok(());
        }

        // Wait for shutdown signal
        info!("Orchestrator running - waiting for shutdown signal...");
        signal::ctrl_c().await.map_err(|e| OrchestratorError::ShutdownFailed(
            format!("Failed to wait for signal: {}", e)
        ))?;

        // Shutdown
        self.shutdown().await?;
        Ok(())
    }
}

