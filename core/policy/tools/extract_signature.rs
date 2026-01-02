// Path and File Name : /home/ransomeye/rebuild/core/policy/tools/extract_signature.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tool to extract raw signature bytes from a policy file (base64 decode)

use std::env;
use std::fs;
use std::path::Path;
use serde_yaml;
use base64::{Engine as _, engine::general_purpose};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <policy_file> <output_file>", args[0]);
        eprintln!("  policy_file: Path to policy YAML file");
        eprintln!("  output_file: Path to write raw signature bytes");
        std::process::exit(1);
    }
    
    let policy_path = Path::new(&args[1]);
    let output_path = Path::new(&args[2]);
    
    // Read policy file
    let policy_content = fs::read_to_string(policy_path)
        .map_err(|e| format!("Failed to read policy file: {}", e))?;
    
    // Parse YAML
    let policy_data: serde_yaml::Value = serde_yaml::from_str(&policy_content)
        .map_err(|e| format!("Failed to parse policy YAML: {}", e))?;
    
    // Extract signature field
    let signature_base64 = policy_data
        .as_mapping()
        .and_then(|m| m.get("signature"))
        .and_then(|v| v.as_str())
        .ok_or("Policy file does not contain signature field")?;
    
    // Decode base64 signature to raw bytes
    let signature_bytes = general_purpose::STANDARD.decode(signature_base64.trim())
        .map_err(|e| format!("Failed to decode signature: {}", e))?;
    
    // Write raw signature bytes
    fs::write(output_path, &signature_bytes)
        .map_err(|e| format!("Failed to write signature: {}", e))?;
    
    println!("Signature extracted: {} bytes", signature_bytes.len());
    Ok(())
}

