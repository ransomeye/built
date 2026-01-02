// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/security/bin/sign_policies.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Binary tool to sign policy files using ring

use std::env;
use std::path::{Path, PathBuf};
use std::fs;
use serde_yaml;
use ransomeye_policy::security::sign_policy::sign_policy_file;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage: {} <private_key_der> <policy_file> [policy_file2 ...]", args[0]);
        eprintln!("  private_key_der: Path to RSA-4096 private key in DER format (PKCS#8)");
        eprintln!("  policy_file: Path to policy YAML file to sign");
        std::process::exit(1);
    }
    
    let private_key_path = Path::new(&args[1]);
    let policy_files = &args[2..];
    
    for policy_file in policy_files {
        let policy_path = Path::new(policy_file);
        
        println!("Signing policy: {}", policy_path.display());
        
        match sign_policy_file(policy_path, private_key_path) {
            Ok((signature, hash)) => {
                // Read the policy file
                let mut content = fs::read_to_string(policy_path)?;
                
                // Parse YAML
                let mut policy_data: serde_yaml::Value = serde_yaml::from_str(&content)?;
                
                // Update signature fields
                if let Some(obj) = policy_data.as_mapping_mut() {
                    obj.insert(
                        serde_yaml::Value::String("signature".to_string()),
                        serde_yaml::Value::String(signature.clone()),
                    );
                    obj.insert(
                        serde_yaml::Value::String("signature_hash".to_string()),
                        serde_yaml::Value::String(hash.clone()),
                    );
                    obj.insert(
                        serde_yaml::Value::String("signature_alg".to_string()),
                        serde_yaml::Value::String("RSA-4096-SHA256".to_string()),
                    );
                    obj.insert(
                        serde_yaml::Value::String("key_id".to_string()),
                        serde_yaml::Value::String("policy_root_v1".to_string()),
                    );
                }
                
                // Preserve header comments
                let header_lines: Vec<String> = content
                    .lines()
                    .take_while(|line| line.trim_start().starts_with('#'))
                    .map(|s| s.to_string())
                    .collect();
                
                // Serialize updated policy
                let updated_content = serde_yaml::to_string(&policy_data)?;
                
                // Write back with header
                let mut final_content = header_lines.join("\n");
                if !final_content.is_empty() {
                    final_content.push('\n');
                }
                final_content.push_str(&updated_content);
                
                fs::write(policy_path, final_content)?;
                
                println!("  ✓ Signed successfully");
                println!("  Signature hash: {}", &hash[..16]);
            }
            Err(e) => {
                eprintln!("  ✗ Failed to sign: {}", e);
                return Err(e);
            }
        }
    }
    
    Ok(())
}

