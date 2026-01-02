// Path and File Name : /home/ransomeye/rebuild/core/policy/tools/test_resign.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Test tool to re-sign a policy and verify it works

use std::env;
use std::fs;
use std::path::Path;
use serde_yaml;
use ring::signature::{self, RsaKeyPair, UnparsedPublicKey, RSA_PSS_SHA256, RSA_PSS_2048_8192_SHA256};
use ring::rand::SystemRandom;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage: {} <private_key_der> <policy_file>", args[0]);
        std::process::exit(1);
    }
    
    let private_key_path = Path::new(&args[1]);
    let policy_path = Path::new(&args[2]);
    
    // Read private key
    let private_key_der = fs::read(private_key_path)?;
    let key_pair = RsaKeyPair::from_pkcs8(&private_key_der)?;
    
    // Extract public key
    let public_key_der = key_pair.public_key().as_ref().to_vec();
    
    // Read and process policy (same as signing)
    let raw_policy_bytes = fs::read(policy_path)?;
    let policy_content = String::from_utf8(raw_policy_bytes)?;
    let mut policy_data: serde_yaml::Value = serde_yaml::from_str(&policy_content)?;
    
    if let Some(obj) = policy_data.as_mapping_mut() {
        obj.remove("signature");
        obj.remove("signature_hash");
        obj.remove("signature_alg");
        obj.remove("key_id");
    }
    
    let policy_bytes = serde_yaml::to_string(&policy_data)?;
    let policy_bytes_raw = policy_bytes.as_bytes();
    
    // Sign using RSA_PSS_SHA256 (matching verification RSA_PSS_2048_8192_SHA256)
    let rng = SystemRandom::new();
    let mut signature = vec![0u8; key_pair.public_modulus_len()];
    
    key_pair.sign(
        &RSA_PSS_SHA256,
        &rng,
        policy_bytes_raw,
        &mut signature,
    )?;
    
    // Verify the signature we just created
    let public_key = UnparsedPublicKey::new(&RSA_PSS_2048_8192_SHA256, &public_key_der);
    
    match public_key.verify(policy_bytes_raw, &signature) {
        Ok(_) => {
            println!("✓ Re-signature verification PASSED");
            println!("Signature length: {} bytes", signature.len());
            
            // Compute hash
            let mut hasher = Sha256::new();
            hasher.update(policy_bytes_raw);
            let hash = hex::encode(hasher.finalize());
            println!("Payload hash: {}", hash);
            
            Ok(())
        }
        Err(e) => {
            println!("✗ Re-signature verification FAILED: {:?}", e);
            Err(format!("Verification failed: {:?}", e).into())
        }
    }
}

