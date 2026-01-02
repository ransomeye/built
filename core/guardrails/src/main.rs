// Path and File Name : /home/ransomeye/rebuild/core/guardrails/src/main.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Binary entry point for guardrails enforcement - called by installer, systemd, CI

use clap::{Parser, Subcommand};
use ransomeye_guardrails::*;
use std::process;

#[derive(Parser)]
#[command(name = "ransomeye-guardrails")]
#[command(about = "RansomEye Guardrails Enforcement Engine - Phase 0 root-of-trust")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Verify guardrails specification signature
    Verify,
    
    /// Enforce guardrails (for installer)
    Enforce {
        /// Context: installer, service, model, policy, runtime, ci
        #[arg(short, long)]
        context: String,
        
        /// Additional context data (service name, file path, etc.)
        #[arg(short, long)]
        data: Option<String>,
    },
    
    /// CI validation
    CI,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Verify => {
            println!("Verifying guardrails specification...");
            let loader = GuardrailLoader::default();
            match GuardrailVerifier::verify_and_load(&loader) {
                Ok(_) => {
                    println!("✓ Guardrails specification verified");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("✗ Verification failed: {}", e);
                    Err(e)
                }
            }
        }
        
        Commands::Enforce { context, data } => {
            let enforcement_context = match context.as_str() {
                "installer" => EnforcementContext::Installer,
                "service" => {
                    let service_name = match data {
                        Some(name) => name,
                        None => {
                            eprintln!("Error: Service name required for service context");
                            process::exit(1);
                        }
                    };
                    EnforcementContext::ServiceStart { service_name }
                }
                "model" => {
                    let model_path = match data {
                        Some(path) => path,
                        None => {
                            eprintln!("Error: Model path required for model context");
                            process::exit(1);
                        }
                    };
                    EnforcementContext::ModelLoad { model_path }
                }
                "policy" => {
                    let policy_path = match data {
                        Some(path) => path,
                        None => {
                            eprintln!("Error: Policy path required for policy context");
                            process::exit(1);
                        }
                    };
                    EnforcementContext::PolicyLoad { policy_path }
                }
                "runtime" => EnforcementContext::Runtime,
                "ci" => EnforcementContext::CI,
                _ => {
                    eprintln!("Unknown context: {}", context);
                    eprintln!("Valid contexts: installer, service, model, policy, runtime, ci");
                    process::exit(1);
                }
            };

            match GuardrailEnforcer::new() {
                Ok(enforcer) => {
                    enforcer.enforce(enforcement_context)
                }
                Err(e) => {
                    eprintln!("✗ Failed to initialize guardrail enforcer: {}", e);
                    Err(e)
                }
            }
        }
        
        Commands::CI => {
            println!("Running CI guardrail validation...");
            match CIValidator::new() {
                Ok(validator) => validator.validate(),
                Err(e) => {
                    eprintln!("✗ Failed to initialize CI validator: {}", e);
                    Err(e)
                }
            }
        }
    };

    match result {
        Ok(_) => {
            process::exit(0);
        }
        Err(e) => {
            eprintln!("GUARDRAIL VIOLATION: {}", e);
            eprintln!("System will not proceed. This is fail-closed behavior.");
            process::exit(1);
        }
    }
}

