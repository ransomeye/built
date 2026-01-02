// Path and File Name : /home/ransomeye/rebuild/ransomeye_operations/src/installer/summary.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Installation summary - prints installation summary and next steps

use crate::installer::state::InstallState;
use tracing::info;

/// Installation summary printer
pub struct InstallSummary;

impl InstallSummary {
    /// Print installation summary
    pub fn print(state: &InstallState) {
        println!("\n{}", "=".repeat(80));
        println!("RANSOMEYE INSTALLATION COMPLETE");
        println!("{}", "=".repeat(80));
        println!();
        
        println!("Installation ID: {}", state.install_id);
        println!("Installed At: {}", state.installed_at.to_rfc3339());
        println!("Engine Version: {}", state.engine_version);
        println!();
        
        println!("EULA Status: {}", if state.eula_accepted { "ACCEPTED" } else { "NOT ACCEPTED" });
        if let Some(accepted_at) = state.eula_accepted_at {
            println!("EULA Accepted At: {}", accepted_at.to_rfc3339());
        }
        println!();
        
        println!("Retention Policy:");
        println!("  - Telemetry Retention: {} months", state.retention_policy.telemetry_retention_months);
        println!("  - Forensic Retention: {} days", state.retention_policy.forensic_retention_days);
        println!("  - Disk Usage Threshold: {}%", state.retention_policy.disk_max_usage_percent);
        println!();
        
        println!("Cryptographic Identity:");
        println!("  - Identity ID: {}", state.identity.identity_id);
        println!("  - Public Key: {}...", &state.identity.public_key[..16]);
        println!();
        
        println!("{}", "=".repeat(80));
        println!("NEXT STEPS");
        println!("{}", "=".repeat(80));
        println!();
        println!("1. Install systemd units:");
        println!("   sudo cp /home/ransomeye/rebuild/systemd/*.service /etc/systemd/system/");
        println!();
        println!("2. Reload systemd:");
        println!("   sudo systemctl daemon-reload");
        println!();
        println!("3. Enable services (optional):");
        println!("   sudo systemctl enable ransomeye-*");
        println!();
        println!("4. Start services:");
        println!("   sudo systemctl start ransomeye-*");
        println!();
        println!("NOTE: All services are DISABLED by default and will NOT start");
        println!("      until explicitly enabled and started.");
        println!();
        println!("{}", "=".repeat(80));
        
        info!("Installation summary printed");
    }
}

