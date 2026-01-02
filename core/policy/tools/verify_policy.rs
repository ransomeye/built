// Path and File Name : /home/ransomeye/rebuild/core/policy/tools/verify_policy.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tool to verify policy signatures using ring

use std::env;
use std::fs;
use std::path::Path;
use serde_yaml;
use ring::signature::{UnparsedPublicKey, RSA_PSS_2048_8192_SHA256};
use base64::{Engine as _, engine::general_purpose};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage: {} <policy_file> <trust_store_dir>", args[0]);
        eprintln!("  policy_file: Path to policy YAML file");
        eprintln!("  trust_store_dir: Path to trust store directory");
        std::process::exit(1);
    }
    
    let policy_path = Path::new(&args[1]);
    let trust_store_dir = Path::new(&args[2]);
    
    // Read policy file
    let raw_policy_bytes = fs::read(policy_path)?;
    let policy_content = String::from_utf8(raw_policy_bytes)?;
    
    // Parse YAML
    let mut policy_data: serde_yaml::Value = serde_yaml::from_str(&policy_content)?;
    
    // Extract signature
    let signature_base64 = policy_data
        .as_mapping()
        .and_then(|m| m.get("signature"))
        .and_then(|v| v.as_str())
        .ok_or("Policy file does not contain signature field")?;
    
    // Remove signature fields for payload extraction
    if let Some(obj) = policy_data.as_mapping_mut() {
        obj.remove("signature");
        obj.remove("signature_hash");
        obj.remove("signature_alg");
        obj.remove("key_id");
    }
    
    // Serialize to YAML (this is what was signed)
    let policy_bytes = serde_yaml::to_string(&policy_data)?;
    let policy_bytes_raw = policy_bytes.as_bytes();
    
    // Decode signature
    let signature_bytes = general_purpose::STANDARD.decode(signature_base64.trim())?;
    
    // Load public key from trust store
    let public_key_path = trust_store_dir.join("policy_signing.der");
    let public_key_bytes = fs::read(&public_key_path)
        .map_err(|e| format!("Failed to read public key from {:?}: {}", public_key_path, e))?;
    
    // Verify signature
    let public_key = UnparsedPublicKey::new(&RSA_PSS_2048_8192_SHA256, &public_key_bytes);
    
    match public_key.verify(policy_bytes_raw, &signature_bytes) {
        Ok(_) => {
            println!("✓ Policy signature verified successfully");
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Policy signature verification failed: {:?}", e);
            std::process::exit(1);
        }
    }
}

