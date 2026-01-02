// Path and File Name : /home/ransomeye/rebuild/ransomeye_operations/src/main.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Main entry point - CLI interface for RansomEye operations, installer, uninstaller, and lifecycle management

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{info, error};

mod installer;
mod uninstaller;
mod lifecycle;
mod errors;

use errors::OperationsError;

#[derive(Parser)]
#[command(name = "ransomeye_operations")]
#[command(about = "RansomEye Unified Installer, Uninstaller & Operations")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install RansomEye
    Install {
        /// Accept EULA
        #[arg(long)]
        accept_eula: bool,
        /// Engine version
        #[arg(long, default_value = "1.0.0")]
        engine_version: String,
    },
    /// Uninstall RansomEye
    Uninstall {
        /// Remove evidence
        #[arg(long)]
        remove_evidence: bool,
        /// Use secure deletion
        #[arg(long)]
        secure_delete: bool,
        /// Confirm uninstallation
        #[arg(long)]
        confirm: bool,
    },
    /// Start services
    Start {
        /// Service name (optional, starts all if not specified)
        service: Option<String>,
    },
    /// Stop services
    Stop {
        /// Service name (optional, stops all if not specified)
        service: Option<String>,
    },
    /// Restart services
    Restart {
        /// Service name (optional, restarts all if not specified)
        service: Option<String>,
    },
    /// Check service status
    Status {
        /// Service name (optional, checks all if not specified)
        service: Option<String>,
    },
}

fn main() -> Result<(), OperationsError> {
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    let project_root = "/home/ransomeye/rebuild";
    
    match cli.command {
        Commands::Install { accept_eula, engine_version } => {
            if !accept_eula {
                println!("ERROR: EULA acceptance is required. Use --accept-eula flag.");
                return Err(OperationsError::EulaNotAccepted);
            }
            
            info!("Starting installation");
            let installer = installer::Installer::new(project_root);
            installer.install(accept_eula, None, &engine_version)?;
            println!("Installation complete");
        }
        Commands::Uninstall { remove_evidence, secure_delete, confirm } => {
            if !confirm {
                println!("ERROR: Uninstallation requires confirmation. Use --confirm flag.");
                return Err(OperationsError::UninstallVerificationFailed(
                    "Uninstallation requires explicit confirmation".to_string()
                ));
            }
            
            info!("Starting uninstallation");
            let uninstaller = uninstaller::Uninstaller::new(project_root);
            uninstaller.uninstall(remove_evidence, secure_delete, confirm)?;
            println!("Uninstallation complete");
        }
        Commands::Start { service } => {
            let state_path = format!("{}/ransomeye_installer/config/install_state.json", project_root);
            let keys_dir = format!("{}/ransomeye_installer/keys", project_root);
            let starter = lifecycle::ServiceStarter::new(&state_path, &keys_dir);
            
            if let Some(service_name) = service {
                starter.start_service(&service_name)?;
            } else {
                starter.start_all()?;
            }
        }
        Commands::Stop { service } => {
            let state_path = format!("{}/ransomeye_installer/config/install_state.json", project_root);
            let keys_dir = format!("{}/ransomeye_installer/keys", project_root);
            let stopper = lifecycle::ServiceStopper::new(&state_path, &keys_dir);
            
            if let Some(service_name) = service {
                stopper.stop_service(&service_name)?;
            } else {
                stopper.stop_all()?;
            }
        }
        Commands::Restart { service } => {
            let state_path = format!("{}/ransomeye_installer/config/install_state.json", project_root);
            let keys_dir = format!("{}/ransomeye_installer/keys", project_root);
            let restarter = lifecycle::ServiceRestarter::new(&state_path, &keys_dir);
            
            if let Some(service_name) = service {
                restarter.restart_service(&service_name)?;
            } else {
                restarter.restart_all()?;
            }
        }
        Commands::Status { service } => {
            let state_path = format!("{}/ransomeye_installer/config/install_state.json", project_root);
            let keys_dir = format!("{}/ransomeye_installer/keys", project_root);
            let checker = lifecycle::ServiceStatusChecker::new(&state_path, &keys_dir);
            
            if let Some(service_name) = service {
                let status = checker.check_service(&service_name)?;
                println!("Service {}: {:?}", service_name, status);
            } else {
                let statuses = checker.check_all_services()?;
                for (name, status) in statuses {
                    println!("{}: {:?}", name, status);
                }
            }
        }
    }
    
    Ok(())
}

