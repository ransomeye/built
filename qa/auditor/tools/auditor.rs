// Path and File Name : /home/ransomeye/rebuild/ransomeye_validation/tools/auditor.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Standalone auditor tool binary - validates audit trails for compliance

use auditor::auditor::Auditor;
use std::env;
use std::path::PathBuf;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: auditor <log_path> <command> [retention_years]");
        eprintln!("Commands:");
        eprintln!("  integrity - Check evidence integrity");
        eprintln!("  retention - Check retention enforcement");
        eprintln!("  completeness - Check audit completeness");
        eprintln!("  reproducibility - Check reproducibility");
        eprintln!("  full - Run full audit");
        std::process::exit(1);
    }
    
    let log_path = PathBuf::from(&args[1]);
    let command = &args[2];
    let retention_years = if args.len() > 3 {
        args[3].parse().unwrap_or(7)
    } else {
        7
    };
    
    let mut auditor = Auditor::new(retention_years);
    auditor.load_audit_log(&log_path)?;
    
    match command.as_str() {
        "integrity" => {
            match auditor.audit_evidence_integrity() {
                Ok(_) => println!("Evidence integrity: PASS"),
                Err(e) => {
                    eprintln!("Evidence integrity: FAIL - {}", e);
                    std::process::exit(1);
                }
            }
        }
        "retention" => {
            match auditor.audit_retention() {
                Ok(_) => println!("Retention enforcement: PASS"),
                Err(e) => {
                    eprintln!("Retention enforcement: FAIL - {}", e);
                    std::process::exit(1);
                }
            }
        }
        "completeness" => {
            match auditor.audit_completeness() {
                Ok(_) => println!("Audit completeness: PASS"),
                Err(e) => {
                    eprintln!("Audit completeness: FAIL - {}", e);
                    std::process::exit(1);
                }
            }
        }
        "reproducibility" => {
            match auditor.audit_reproducibility() {
                Ok(_) => println!("Reproducibility: PASS"),
                Err(e) => {
                    eprintln!("Reproducibility: FAIL - {}", e);
                    std::process::exit(1);
                }
            }
        }
        "full" => {
            match auditor.run_full_audit() {
                Ok(result) => {
                    println!("Full audit results:");
                    println!("  Total entries: {}", result.total_entries);
                    println!("  Integrity: {}", if result.integrity_passed { "PASS" } else { "FAIL" });
                    println!("  Retention: {}", if result.retention_passed { "PASS" } else { "FAIL" });
                    println!("  Completeness: {}", if result.completeness_passed { "PASS" } else { "FAIL" });
                    if !result.violations.is_empty() {
                        println!("  Violations:");
                        for violation in &result.violations {
                            println!("    - {}", violation);
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Full audit error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            std::process::exit(1);
        }
    }
    
    Ok(())
}

