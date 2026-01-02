// Path and File Name : /home/ransomeye/rebuild/core/engine/orchestrator/src/retention_main.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Standalone retention enforcer service binary (periodic runtime purge) with dry-run and fail-closed validation.

use std::process;

use tracing::{error, info};

#[path = "lib.rs"]
mod orchestrator;

use orchestrator::db::{CoreDb, DbConfig};
use orchestrator::retention_enforcer::{RetentionEnforcer, RetentionEnforcerConfig};

fn usage_and_exit() -> ! {
    eprintln!("RansomEye Retention Enforcer");
    eprintln!("");
    eprintln!("USAGE:");
    eprintln!("  ransomeye_retention_enforcer --dry-run");
    eprintln!("  ransomeye_retention_enforcer --live");
    eprintln!("");
    eprintln!("NOTES:");
    eprintln!("  - Default is FAIL-SAFE: you MUST explicitly choose --live to delete rows.");
    eprintln!("  - DB env vars are required: DB_HOST, DB_PORT, DB_NAME, DB_USER, DB_PASS");
    process::exit(2);
}

fn arg_flag(name: &str) -> bool {
    std::env::args().any(|a| a == name)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let dry_run = arg_flag("--dry-run");
    let live = arg_flag("--live");
    if (dry_run && live) || (!dry_run && !live) {
        usage_and_exit();
    }

    let cfg = match DbConfig::from_env_strict() {
        Ok(c) => c,
        Err(e) => {
            error!("{e}");
            process::exit(1);
        }
    };

    let db = match CoreDb::connect_strict(&cfg).await {
        Ok(db) => db,
        Err(e) => {
            error!("FAIL-CLOSED: DB connect failed: {e}");
            process::exit(1);
        }
    };

    // Register component for audit attribution (best-effort fail-closed: if this fails, we still abort).
    let build_hash = std::env::var("RANSOMEYE_BUILD_HASH").ok();
    let version = std::env::var("RANSOMEYE_VERSION").ok();
    let instance_id = std::env::var("RANSOMEYE_INSTANCE_ID").ok();
    let component_id = match db
        .upsert_component(
            "db_core",
            "ransomeye_retention_enforcer",
            instance_id.as_deref(),
            build_hash.as_deref(),
            version.as_deref(),
        )
        .await
    {
        Ok(id) => id,
        Err(e) => {
            error!("FAIL-CLOSED: Cannot upsert component identity for retention enforcer: {e}");
            process::exit(1);
        }
    };

    let enforcer_cfg = match RetentionEnforcerConfig::from_env() {
        Ok(c) => c,
        Err(e) => {
            error!("{e}");
            process::exit(1);
        }
    };
    let enforcer = RetentionEnforcer::new(enforcer_cfg.clone());

    info!(
        "Retention enforcer starting (mode={}, batch_size={}, max_batches_per_table={})",
        if dry_run { "DRY-RUN" } else { "LIVE" },
        enforcer_cfg.batch_size,
        enforcer_cfg.max_batches_per_table
    );

    let (audit_id, results) = match enforcer.enforce(&db, Some(component_id), dry_run).await {
        Ok(r) => r,
        Err(e) => {
            error!("{e}");
            // Best-effort: attempt to log failure reason into immutable audit.
            let _ = db
                .insert_immutable_audit_log(
                    Some(component_id),
                    "runtime_retention_failed",
                    "other",
                    Some(component_id),
                    &serde_json::json!({"event":"runtime_retention_failed","error": e}),
                )
                .await;
            process::exit(1);
        }
    };

    // Print high-signal summary to stdout (systemd journal picks this up).
    let mut total_would_purge: i64 = 0;
    let mut total_deleted: i64 = 0;
    for r in &results {
        if let Some(n) = r.dry_run_rows_older {
            total_would_purge += n;
        }
        total_deleted += r.deleted_rows;
    }

    info!("Retention run complete: audit_id={}", audit_id);
    info!(
        "Totals: would_purge_rows={} deleted_rows={} tables={}",
        total_would_purge,
        total_deleted,
        results.len()
    );

    // Exit 0 on success.
    process::exit(0);
}


