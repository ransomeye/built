// Path and File Name : /home/ransomeye/rebuild/core/response_playbooks/src/persistence.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Database persistence for playbook executions, rollback states, nonce tracking, audit trail

use sqlx::{PgPool, Postgres, Transaction};
use chrono::{DateTime, Utc};
use serde_json;
use tracing::{error, info, debug};

use crate::errors::PlaybookError;
use crate::executor::ExecutionContext;
use crate::rollback::RollbackState;

pub struct PlaybookPersistence {
    pool: PgPool,
}

impl PlaybookPersistence {
    pub async fn new() -> Result<Self, PlaybookError> {
        // Get database connection from environment
        let db_host = std::env::var("DB_HOST")
            .unwrap_or_else(|_| "localhost".to_string());
        let db_port = std::env::var("DB_PORT")
            .unwrap_or_else(|_| "5432".to_string())
            .parse::<u16>()
            .map_err(|e| PlaybookError::ConfigurationError(
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
            .map_err(|e| PlaybookError::DatabaseError(
                format!("Failed to connect to database: {}", e)
            ))?;
        
        // Initialize schema
        let persistence = Self { pool };
        persistence.initialize_schema().await?;
        
        Ok(persistence)
    }
    
    /// Initialize database schema
    async fn initialize_schema(&self) -> Result<(), PlaybookError> {
        info!("Initializing playbook persistence schema");
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS playbook_executions (
                execution_id VARCHAR(36) PRIMARY KEY,
                playbook_id VARCHAR(36) NOT NULL,
                state VARCHAR(20) NOT NULL,
                started_at TIMESTAMP WITH TIME ZONE NOT NULL,
                completed_at TIMESTAMP WITH TIME ZONE,
                current_step INTEGER NOT NULL DEFAULT 0,
                step_results JSONB NOT NULL DEFAULT '{}'::jsonb,
                nonce VARCHAR(36) NOT NULL UNIQUE,
                policy_decision_id VARCHAR(36),
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| PlaybookError::DatabaseError(
            format!("Failed to create playbook_executions table: {}", e)
        ))?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS playbook_rollback_states (
                rollback_id VARCHAR(36) PRIMARY KEY,
                execution_id VARCHAR(36) NOT NULL,
                playbook_id VARCHAR(36) NOT NULL,
                started_at TIMESTAMP WITH TIME ZONE NOT NULL,
                completed_at TIMESTAMP WITH TIME ZONE,
                rollback_step_results JSONB NOT NULL DEFAULT '[]'::jsonb,
                status VARCHAR(20) NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| PlaybookError::DatabaseError(
            format!("Failed to create playbook_rollback_states table: {}", e)
        ))?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS playbook_nonces (
                nonce VARCHAR(36) PRIMARY KEY,
                used_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                execution_id VARCHAR(36)
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| PlaybookError::DatabaseError(
            format!("Failed to create playbook_nonces table: {}", e)
        ))?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS playbook_audit_log (
                audit_id SERIAL PRIMARY KEY,
                execution_id VARCHAR(36),
                rollback_id VARCHAR(36),
                event_type VARCHAR(50) NOT NULL,
                event_data JSONB NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| PlaybookError::DatabaseError(
            format!("Failed to create playbook_audit_log table: {}", e)
        ))?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS playbook_safe_halt_state (
                halt_id SERIAL PRIMARY KEY,
                rollback_id VARCHAR(36) NOT NULL,
                error_message TEXT,
                entered_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                resolved_at TIMESTAMP WITH TIME ZONE,
                is_active BOOLEAN NOT NULL DEFAULT TRUE
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| PlaybookError::DatabaseError(
            format!("Failed to create playbook_safe_halt_state table: {}", e)
        ))?;
        
        // Create indexes
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_playbook_executions_playbook_id 
            ON playbook_executions(playbook_id)
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| PlaybookError::DatabaseError(
            format!("Failed to create index: {}", e)
        ))?;
        
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_playbook_executions_state 
            ON playbook_executions(state)
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| PlaybookError::DatabaseError(
            format!("Failed to create index: {}", e)
        ))?;
        
        info!("Playbook persistence schema initialized");
        Ok(())
    }
    
    /// Save execution context
    pub async fn save_execution(&self, context: &ExecutionContext) -> Result<(), PlaybookError> {
        let step_results_json = serde_json::to_value(&context.step_results)
            .map_err(|e| PlaybookError::InternalError(
                format!("Failed to serialize step_results: {}", e)
            ))?;
        
        sqlx::query(
            r#"
            INSERT INTO playbook_executions (
                execution_id, playbook_id, state, started_at, completed_at,
                current_step, step_results, nonce, policy_decision_id, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
            ON CONFLICT (execution_id) DO UPDATE SET
                state = EXCLUDED.state,
                completed_at = EXCLUDED.completed_at,
                current_step = EXCLUDED.current_step,
                step_results = EXCLUDED.step_results,
                updated_at = NOW()
            "#
        )
        .bind(&context.execution_id)
        .bind(&context.playbook_id)
        .bind(format!("{:?}", context.state))
        .bind(&context.started_at)
        .bind(&context.completed_at)
        .bind(context.current_step as i32)
        .bind(&step_results_json)
        .bind(&context.nonce)
        .bind(&context.policy_decision_id)
        .execute(&self.pool)
        .await
        .map_err(|e| PlaybookError::DatabaseError(
            format!("Failed to save execution: {}", e)
        ))?;
        
        // Audit log
        self.audit_log(
            Some(&context.execution_id),
            None,
            "execution_updated",
            &serde_json::json!({
                "state": format!("{:?}", context.state),
                "current_step": context.current_step
            })
        ).await?;
        
        Ok(())
    }
    
    /// Load execution context
    pub async fn load_execution(&self, execution_id: &str) -> Result<Option<ExecutionContext>, PlaybookError> {
        let row = sqlx::query_as::<_, ExecutionRow>(
            r#"
            SELECT execution_id, playbook_id, state, started_at, completed_at,
                   current_step, step_results, nonce, policy_decision_id
            FROM playbook_executions
            WHERE execution_id = $1
            "#
        )
        .bind(execution_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PlaybookError::DatabaseError(
            format!("Failed to load execution: {}", e)
        ))?;
        
        if let Some(row) = row {
            Ok(Some(row.into()))
        } else {
            Ok(None)
        }
    }
    
    /// Mark nonce as used (replay protection)
    pub async fn mark_nonce_used(&self, nonce: &str) -> Result<(), PlaybookError> {
        sqlx::query(
            r#"
            INSERT INTO playbook_nonces (nonce, used_at)
            VALUES ($1, NOW())
            ON CONFLICT (nonce) DO NOTHING
            "#
        )
        .bind(nonce)
        .execute(&self.pool)
        .await
        .map_err(|e| PlaybookError::DatabaseError(
            format!("Failed to mark nonce as used: {}", e)
        ))?;
        
        Ok(())
    }
    
    /// Check if nonce is already used
    pub async fn is_nonce_used(&self, nonce: &str) -> Result<bool, PlaybookError> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM playbook_nonces WHERE nonce = $1
            "#
        )
        .bind(nonce)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| PlaybookError::DatabaseError(
            format!("Failed to check nonce: {}", e)
        ))?;
        
        Ok(count > 0)
    }
    
    /// Save rollback state
    pub async fn save_rollback_state(&self, state: &RollbackState) -> Result<(), PlaybookError> {
        let rollback_step_results_json = serde_json::to_value(&state.rollback_step_results)
            .map_err(|e| PlaybookError::InternalError(
                format!("Failed to serialize rollback_step_results: {}", e)
            ))?;
        
        sqlx::query(
            r#"
            INSERT INTO playbook_rollback_states (
                rollback_id, execution_id, playbook_id, started_at, completed_at,
                rollback_step_results, status, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
            ON CONFLICT (rollback_id) DO UPDATE SET
                completed_at = EXCLUDED.completed_at,
                rollback_step_results = EXCLUDED.rollback_step_results,
                status = EXCLUDED.status,
                updated_at = NOW()
            "#
        )
        .bind(&state.rollback_id)
        .bind(&state.execution_id)
        .bind(&state.playbook_id)
        .bind(&state.started_at)
        .bind(&state.completed_at)
        .bind(&rollback_step_results_json)
        .bind(format!("{:?}", state.status))
        .execute(&self.pool)
        .await
        .map_err(|e| PlaybookError::DatabaseError(
            format!("Failed to save rollback state: {}", e)
        ))?;
        
        // Audit log
        self.audit_log(
            Some(&state.execution_id),
            Some(&state.rollback_id),
            "rollback_updated",
            &serde_json::json!({
                "status": format!("{:?}", state.status)
            })
        ).await?;
        
        Ok(())
    }
    
    /// Load rollback state
    pub async fn load_rollback_state(&self, rollback_id: &str) -> Result<Option<RollbackState>, PlaybookError> {
        let row = sqlx::query_as::<_, RollbackRow>(
            r#"
            SELECT rollback_id, execution_id, playbook_id, started_at, completed_at,
                   rollback_step_results, status
            FROM playbook_rollback_states
            WHERE rollback_id = $1
            "#
        )
        .bind(rollback_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PlaybookError::DatabaseError(
            format!("Failed to load rollback state: {}", e)
        ))?;
        
        if let Some(row) = row {
            Ok(Some(row.into()))
        } else {
            Ok(None)
        }
    }
    
    /// Mark safe-halt state
    pub async fn mark_safe_halt(&self) -> Result<(), PlaybookError> {
        sqlx::query(
            r#"
            INSERT INTO playbook_safe_halt_state (rollback_id, error_message, is_active)
            VALUES ($1, $2, TRUE)
            "#
        )
        .bind("system_halt")
        .bind("Rollback failure - system entered safe-halt state")
        .execute(&self.pool)
        .await
        .map_err(|e| PlaybookError::DatabaseError(
            format!("Failed to mark safe-halt: {}", e)
        ))?;
        
        Ok(())
    }
    
    /// Check if system is in safe-halt state
    pub async fn is_safe_halt(&self) -> Result<bool, PlaybookError> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM playbook_safe_halt_state 
            WHERE is_active = TRUE
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| PlaybookError::DatabaseError(
            format!("Failed to check safe-halt state: {}", e)
        ))?;
        
        Ok(count > 0)
    }
    
    /// Audit log entry
    async fn audit_log(
        &self,
        execution_id: Option<&str>,
        rollback_id: Option<&str>,
        event_type: &str,
        event_data: &serde_json::Value,
    ) -> Result<(), PlaybookError> {
        sqlx::query(
            r#"
            INSERT INTO playbook_audit_log (execution_id, rollback_id, event_type, event_data)
            VALUES ($1, $2, $3, $4)
            "#
        )
        .bind(execution_id)
        .bind(rollback_id)
        .bind(event_type)
        .bind(event_data)
        .execute(&self.pool)
        .await
        .map_err(|e| PlaybookError::DatabaseError(
            format!("Failed to write audit log: {}", e)
        ))?;
        
        Ok(())
    }
}

// Helper structs for database rows
#[derive(sqlx::FromRow)]
struct ExecutionRow {
    execution_id: String,
    playbook_id: String,
    state: String,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    current_step: i32,
    step_results: serde_json::Value,
    nonce: String,
    policy_decision_id: Option<String>,
}

impl From<ExecutionRow> for ExecutionContext {
    fn from(row: ExecutionRow) -> Self {
        use crate::executor::{ExecutionState, StepResult};
        use std::collections::HashMap;
        
        let state = match row.state.as_str() {
            "Pending" => ExecutionState::Pending,
            "DryRun" => ExecutionState::DryRun,
            "Running" => ExecutionState::Running,
            "Failed" => ExecutionState::Failed,
            "RolledBack" => ExecutionState::RolledBack,
            "Completed" => ExecutionState::Completed,
            _ => ExecutionState::Pending,
        };
        
        let step_results: HashMap<usize, StepResult> = serde_json::from_value(row.step_results)
            .unwrap_or_default();
        
        ExecutionContext {
            execution_id: row.execution_id,
            playbook_id: row.playbook_id,
            state,
            started_at: row.started_at,
            completed_at: row.completed_at,
            current_step: row.current_step as usize,
            step_results,
            nonce: row.nonce,
            policy_decision_id: row.policy_decision_id,
        }
    }
}

#[derive(sqlx::FromRow)]
struct RollbackRow {
    rollback_id: String,
    execution_id: String,
    playbook_id: String,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    rollback_step_results: serde_json::Value,
    status: String,
}

impl From<RollbackRow> for RollbackState {
    fn from(row: RollbackRow) -> Self {
        use crate::rollback::{RollbackStatus, RollbackStepResult};
        
        let status = match row.status.as_str() {
            "Pending" => RollbackStatus::Pending,
            "Running" => RollbackStatus::Running,
            "Completed" => RollbackStatus::Completed,
            "Failed" => RollbackStatus::Failed,
            "SafeHalt" => RollbackStatus::SafeHalt,
            _ => RollbackStatus::Pending,
        };
        
        let rollback_step_results: Vec<RollbackStepResult> = serde_json::from_value(row.rollback_step_results)
            .unwrap_or_default();
        
        RollbackState {
            rollback_id: row.rollback_id,
            execution_id: row.execution_id,
            playbook_id: row.playbook_id,
            started_at: row.started_at,
            completed_at: row.completed_at,
            rollback_step_results,
            status,
        }
    }
}

