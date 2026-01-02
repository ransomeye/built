// Path and File Name : /home/ransomeye/rebuild/qa/validation/src/main.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: System-wide validation main entry point

use std::path::PathBuf;
use tracing::{info, error};
use tracing_subscriber;

use ransomeye_validation::{SystemValidator, ValidationError};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Get output directory from environment or use default
    let output_dir = std::env::var("RANSOMEYE_VALIDATION_OUTPUT_DIR")
        .map(|s| PathBuf::from(s))
        .unwrap_or_else(|_| PathBuf::from("./validation_reports"));
    
    info!("System Validation Output Directory: {:?}", output_dir);
    
    // Create validator
    let validator = SystemValidator::new(output_dir);
    
    // Run validation
    match validator.run_full_validation().await {
        Ok(result) => {
            match result.overall_decision {
                ransomeye_validation::reports::GoNoGoDecision::Go => {
                    info!("System validation completed successfully - GO");
                    std::process::exit(0);
                }
                ransomeye_validation::reports::GoNoGoDecision::NoGo { reason, .. } => {
                    error!("System validation failed - NO-GO: {}", reason);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            error!("System validation error: {}", e);
            std::process::exit(1);
        }
    }
}

