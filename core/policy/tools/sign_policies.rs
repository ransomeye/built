// Path and File Name : /home/ransomeye/rebuild/core/policy/tools/sign_policies.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Standalone tool to sign policy files using ring RSA-PSS-SHA256

#![cfg(feature = "future-policy")]

use std::path::Path;
use std::fs;
use ring::signature::RsaKeyPair;
use ring::rand::SystemRandom;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};
use serde_yaml;
use serde_json;

fn sign_policy_content(
    policy_bytes: &[u8],
    private_key_der: &[u8],
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let key_pair = RsaKeyPair::from_pkcs8(private_key_der)
        .map_err(|e| format!("Failed to load RSA key pair: {:?}", e))?;
    
    // Verify key size (4096 bits = 512 bytes)
    let modulus_len = key_pair.public().modulus_len();
    if modulus_len != 512 {
        return Err(format!(
            "Key size mismatch: expected 512 bytes (4096 bits), got {} bytes",
            modulus_len
        ).into());
    }
    
    let mut hasher = Sha256::new();
    hasher.update(policy_bytes);
    let content_hash = hex::encode(hasher.finalize());
    
    let rng = SystemRandom::new();
    let mut signature = vec![0u8; modulus_len];
    
    // Use RSA_PSS_SHA256 for signing (matches verification algorithm RSA_PSS_2048_8192_SHA256)
    // RSA-PSS is the only algorithm that supports sign + verify symmetry in ring 0.17.14
    use ring::signature::RSA_PSS_SHA256;
    key_pair.sign(
        &RSA_PSS_SHA256,
        &rng,
        policy_bytes,
        &mut signature,
    ).map_err(|e| format!("Failed to sign policy: {:?}", e))?;
    
    let signature_base64 = general_purpose::STANDARD.encode(&signature);
    
    Ok((signature_base64, content_hash))
}

// Helper: sort JSON keys deterministically (must match policy engine canonicalization)
fn sort_json_value_keys(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            let mut sorted_pairs: Vec<(String, serde_json::Value)> = map
                .iter()
                .map(|(k, v)| {
                    let mut val = v.clone();
                    sort_json_value_keys(&mut val);
                    (k.clone(), val)
                })
                .collect();
            sorted_pairs.sort_by(|a, b| a.0.cmp(&b.0));
            map.clear();
            for (k, v) in sorted_pairs {
                map.insert(k, v);
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr.iter_mut() {
                sort_json_value_keys(item);
            }
        }
        _ => {}
    }
}

/// Canonicalize a policy Value (with signature fields removed) into deterministic JSON.
fn canonicalize_policy_value_for_signing(policy_value: &serde_yaml::Value) -> Result<String, Box<dyn std::error::Error>> {
    let mut json_val = serde_json::to_value(policy_value)?;
    sort_json_value_keys(&mut json_val);
    Ok(serde_json::to_string(&json_val)?)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    // Support both old format (positional) and new format (flags)
    let (private_key_path, policy_path, out_path) = if args.len() >= 3 && !args[1].starts_with('-') {
        // Old format: <private_key> <policy> [out]
        (Path::new(&args[1]), Path::new(&args[2]), None)
    } else {
        // New format: --private-key <key> --policy <policy> [--out <out>]
        let mut private_key: Option<&str> = None;
        let mut policy: Option<&str> = None;
        let mut out: Option<&str> = None;
        
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--private-key" | "-k" => {
                    if i + 1 < args.len() {
                        private_key = Some(&args[i + 1]);
                        i += 2;
                    } else {
                        eprintln!("Error: --private-key requires a value");
                        std::process::exit(1);
                    }
                }
                "--policy" | "-p" => {
                    if i + 1 < args.len() {
                        policy = Some(&args[i + 1]);
                        i += 2;
                    } else {
                        eprintln!("Error: --policy requires a value");
                        std::process::exit(1);
                    }
                }
                "--out" | "-o" => {
                    if i + 1 < args.len() {
                        out = Some(&args[i + 1]);
                        i += 2;
                    } else {
                        eprintln!("Error: --out requires a value");
                        std::process::exit(1);
                    }
                }
                _ => {
                    eprintln!("Unknown argument: {}", args[i]);
                    std::process::exit(1);
                }
            }
        }
        
        if private_key.is_none() || policy.is_none() {
            eprintln!("Usage: {} --private-key <key> --policy <policy> [--out <out>]", args[0]);
            eprintln!("   or: {} <private_key> <policy> [out]", args[0]);
            eprintln!("  --private-key, -k: Path to RSA-4096 private key in DER format (PKCS#8)");
            eprintln!("  --policy, -p: Path to policy YAML file to sign");
            eprintln!("  --out, -o: Optional output path (default: same as policy file)");
            std::process::exit(1);
        }
        
        (Path::new(private_key.unwrap()), Path::new(policy.unwrap()), out.map(Path::new))
    };
    
    let private_key_der = fs::read(private_key_path)
        .map_err(|e| format!("Failed to read private key: {}", e))?;
    
    println!("Signing policy: {}", policy_path.display());
    
    // Step 1: Read policy file as RAW BYTES (fs::read - ensures byte-exact signing)
    let raw_policy_bytes = fs::read(policy_path)?;
    
    // Step 2: Convert to string for parsing (preserving exact encoding)
    let content = String::from_utf8(raw_policy_bytes)
        .map_err(|e| format!("Failed to convert policy bytes to UTF-8: {}", e))?;
    
    // Step 3: Parse YAML
    let mut policy_data: serde_yaml::Value = serde_yaml::from_str(&content)?;
    
    // Extract header comments
    let header_lines: Vec<String> = content
        .lines()
        .take_while(|line| line.trim_start().starts_with('#'))
        .map(|s| s.to_string())
        .collect();
    
    // Remove signature fields
    if let Some(obj) = policy_data.as_mapping_mut() {
        obj.remove("signature");
        obj.remove("signature_hash");
        obj.remove("signature_alg");
        obj.remove("key_id");
    }
    
    // Canonicalize deterministically for signing/verification parity (JSON with sorted keys)
    let canonical = canonicalize_policy_value_for_signing(&policy_data)?;
    let policy_bytes_raw = canonical.as_bytes();
    
    // Sign the policy using RSA-PSS-SHA256 (matches verification algorithm RSA_PSS_2048_8192_SHA256)
    let (signature_base64, hash) = sign_policy_content(policy_bytes_raw, &private_key_der)?;
    
    // Create .payload and .sig files (for isolated verification testing)
    let payload_path = policy_path.with_extension("yaml.payload");
    let sig_path = policy_path.with_extension("yaml.sig");
    
    fs::write(&payload_path, policy_bytes_raw)
        .map_err(|e| format!("Failed to write payload file: {}", e))?;
    
    let signature_bytes = general_purpose::STANDARD.decode(&signature_base64)
        .map_err(|e| format!("Failed to decode signature: {}", e))?;
    fs::write(&sig_path, &signature_bytes)
        .map_err(|e| format!("Failed to write signature file: {}", e))?;
    
    println!("  ✓ Created payload: {}", payload_path.display());
    println!("  ✓ Created signature: {}", sig_path.display());
    
    // Also update policy YAML with signature (for production use)
    if let Some(obj) = policy_data.as_mapping_mut() {
        obj.insert(
            serde_yaml::Value::String("signature".to_string()),
            serde_yaml::Value::String(signature_base64.clone()),
        );
        obj.insert(
            serde_yaml::Value::String("signature_hash".to_string()),
            serde_yaml::Value::String(hash.clone()),
        );
        obj.insert(
            serde_yaml::Value::String("signature_alg".to_string()),
            serde_yaml::Value::String("RSA-4096-PSS-SHA256".to_string()),
        );
        obj.insert(
            serde_yaml::Value::String("key_id".to_string()),
            serde_yaml::Value::String("policy_root_v1".to_string()),
        );
    }
    
    // Serialize updated policy
    let updated_content = serde_yaml::to_string(&policy_data)?;
    
    // Write back with header
    let output_path = out_path.unwrap_or(policy_path);
    let mut final_content = header_lines.join("\n");
    if !final_content.is_empty() {
        final_content.push('\n');
    }
    final_content.push_str(&updated_content);
    
    fs::write(output_path, final_content)?;
    
    println!("  ✓ Signed successfully");
    println!("  Signature hash: {}", &hash[..16]);
    println!("  Updated policy: {}", output_path.display());
    
    Ok(())
}

