// Path and File Name : /home/ransomeye/rebuild/core/policy/tools/extract_payload.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tool to extract exact payload bytes from a policy file (matching signing process)

use std::env;
use std::fs;
use std::path::Path;
use serde_yaml;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <policy_file> <output_file>", args[0]);
        eprintln!("  policy_file: Path to policy YAML file");
        eprintln!("  output_file: Path to write payload bytes");
        std::process::exit(1);
    }
    
    let policy_path = Path::new(&args[1]);
    let output_path = Path::new(&args[2]);
    
    // Step 1: Read policy file as RAW BYTES (fs::read - no string conversion)
    let raw_policy_bytes = fs::read(policy_path)
        .map_err(|e| format!("Failed to read policy file: {}", e))?;
    
    // Step 2: Convert to string for parsing (preserving exact encoding)
    let policy_content = String::from_utf8(raw_policy_bytes)
        .map_err(|e| format!("Failed to convert policy bytes to UTF-8: {}", e))?;
    
    // Step 3: Parse YAML to remove signature fields
    let mut policy_data: serde_yaml::Value = serde_yaml::from_str(&policy_content)
        .map_err(|e| format!("Failed to parse policy YAML: {}", e))?;
    
    // Step 4: Remove signature-related fields (matching sign_policy.rs exactly)
    if let Some(obj) = policy_data.as_mapping_mut() {
        obj.remove("signature");
        obj.remove("signature_hash");
        obj.remove("signature_alg");
        obj.remove("key_id");
    }
    
    // Step 5: Serialize back to YAML (this is what gets signed - must match verification exactly)
    let policy_bytes = serde_yaml::to_string(&policy_data)
        .map_err(|e| format!("Failed to serialize policy: {}", e))?;
    
    // Step 6: Write exact bytes that were signed
    fs::write(output_path, policy_bytes.as_bytes())
        .map_err(|e| format!("Failed to write payload: {}", e))?;
    
    println!("Payload extracted: {} bytes", policy_bytes.as_bytes().len());
    Ok(())
}

