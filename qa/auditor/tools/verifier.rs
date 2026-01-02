// Path and File Name : /home/ransomeye/rebuild/ransomeye_validation/tools/verifier.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Standalone verifier tool binary - validates signatures, certificates, and hashes

use auditor::verifier::Verifier;
use std::env;
use std::path::PathBuf;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: verifier <command> [args...]");
        eprintln!("Commands:");
        eprintln!("  hash <file> <expected_hash> - Verify file hash");
        eprintln!("  signature <data> <sig> - Verify signature");
        eprintln!("  cert <cert_path> - Verify certificate");
        eprintln!("  chain <cert_path> - Verify trust chain");
        eprintln!("  compute <file> - Compute file hash");
        std::process::exit(1);
    }
    
    let trust_store = PathBuf::from("/home/ransomeye/rebuild/trust_store");
    let verifier = Verifier::new(trust_store);
    let command = &args[1];
    
    match command.as_str() {
        "hash" => {
            if args.len() < 4 {
                eprintln!("Usage: verifier hash <file> <expected_hash>");
                std::process::exit(1);
            }
            let file_path = PathBuf::from(&args[2]);
            let expected_hash = &args[3];
            match verifier.verify_file_hash(&file_path, expected_hash) {
                Ok(true) => println!("Hash verification: PASS"),
                Ok(false) => {
                    eprintln!("Hash verification: FAIL");
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Hash verification error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "signature" => {
            if args.len() < 4 {
                eprintln!("Usage: verifier signature <data> <sig>");
                std::process::exit(1);
            }
            let data_path = PathBuf::from(&args[2]);
            let sig_path = PathBuf::from(&args[3]);
            match verifier.verify_signature(&data_path, &sig_path) {
                Ok(true) => println!("Signature verification: PASS"),
                Ok(false) => {
                    eprintln!("Signature verification: FAIL");
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Signature verification error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "cert" => {
            if args.len() < 3 {
                eprintln!("Usage: verifier cert <cert_path>");
                std::process::exit(1);
            }
            let cert_path = PathBuf::from(&args[2]);
            match verifier.verify_certificate(&cert_path) {
                Ok(true) => println!("Certificate verification: PASS"),
                Ok(false) => {
                    eprintln!("Certificate verification: FAIL");
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Certificate verification error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "chain" => {
            if args.len() < 3 {
                eprintln!("Usage: verifier chain <cert_path>");
                std::process::exit(1);
            }
            let cert_path = PathBuf::from(&args[2]);
            match verifier.verify_trust_chain(&cert_path) {
                Ok(true) => println!("Trust chain verification: PASS"),
                Ok(false) => {
                    eprintln!("Trust chain verification: FAIL");
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Trust chain verification error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "compute" => {
            if args.len() < 3 {
                eprintln!("Usage: verifier compute <file>");
                std::process::exit(1);
            }
            let file_path = PathBuf::from(&args[2]);
            match verifier.compute_file_hash(&file_path) {
                Ok(hash) => println!("{}", hash),
                Err(e) => {
                    eprintln!("Hash computation error: {}", e);
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

