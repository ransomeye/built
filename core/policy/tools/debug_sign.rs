// Path and File Name : /home/ransomeye/rebuild/core/policy/tools/debug_sign.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Debug tool to show what was signed and verify hash

use std::env;
use std::fs;
use std::path::Path;
use serde_yaml;
use sha2::{Sha256, Digest};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <policy_file>", args[0]);
        eprintln!("  policy_file: Path to policy YAML file");
        std::process::exit(1);
    }
    
    let policy_path = Path::new(&args[1]);
    
    // Step 1: Read policy file as RAW BYTES
    let raw_policy_bytes = fs::read(policy_path)
        .map_err(|e| format!("Failed to read policy file: {}", e))?;
    
    // Step 2: Convert to string for parsing
    let policy_content = String::from_utf8(raw_policy_bytes)
        .map_err(|e| format!("Failed to convert policy bytes to UTF-8: {}", e))?;
    
    // Step 3: Parse YAML to remove signature fields
    let mut policy_data: serde_yaml::Value = serde_yaml::from_str(&policy_content)
        .map_err(|e| format!("Failed to parse policy YAML: {}", e))?;
    
    // Step 4: Extract signature_hash before removing it
    let expected_hash = policy_data
        .as_mapping()
        .and_then(|m| m.get("signature_hash"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    // Step 5: Remove signature-related fields
    if let Some(obj) = policy_data.as_mapping_mut() {
        obj.remove("signature");
        obj.remove("signature_hash");
        obj.remove("signature_alg");
        obj.remove("key_id");
    }
    
    // Step 6: Serialize back to YAML (this is what gets signed)
    let policy_bytes = serde_yaml::to_string(&policy_data)
        .map_err(|e| format!("Failed to serialize policy: {}", e))?;
    
    // Step 7: Compute hash
    let mut hasher = Sha256::new();
    hasher.update(policy_bytes.as_bytes());
    let computed_hash = hex::encode(hasher.finalize());
    
    println!("Policy file: {}", policy_path.display());
    println!("Payload length: {} bytes", policy_bytes.as_bytes().len());
    println!("Computed hash: {}", computed_hash);
    if let Some(ref exp_hash) = expected_hash {
        println!("Expected hash: {}", exp_hash);
        if computed_hash == *exp_hash {
            println!("✓ Hash matches!");
        } else {
            println!("✗ Hash mismatch!");
        }
    }
    
    println!("\nFirst 200 bytes of payload:");
    let preview = if policy_bytes.len() > 200 {
        &policy_bytes[..200]
    } else {
        &policy_bytes
    };
    println!("{}", preview);
    if policy_bytes.len() > 200 {
        println!("... (truncated)");
    }
    
    Ok(())
}

