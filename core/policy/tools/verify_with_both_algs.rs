// Path and File Name : /home/ransomeye/rebuild/core/policy/tools/verify_with_both_algs.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Test verification with different algorithms to find the issue

use ring::signature::{UnparsedPublicKey, RSA_PSS_2048_8192_SHA256};
use std::fs;

fn main() {
    let pubkey = fs::read("/etc/ransomeye/trust_store/policy_signing.der")
        .expect("pubkey read failed");

    let payload = fs::read("/tmp/policy_payload.bin")
        .expect("payload read failed");

    let sig = fs::read("/tmp/policy_signature.bin")
        .expect("signature read failed");

    println!("Public key length: {} bytes", pubkey.len());
    println!("Payload length: {} bytes", payload.len());
    println!("Signature length: {} bytes", sig.len());
    
    // Try with RSA_PSS_2048_8192_SHA256 (current verification algorithm)
    let pk = UnparsedPublicKey::new(&RSA_PSS_2048_8192_SHA256, &pubkey);
    
    println!("\nTrying verification with RSA_PSS_2048_8192_SHA256...");
    match pk.verify(&payload, &sig) {
        Ok(_) => {
            println!("✓ SIGNATURE_VERIFIED_OK with RSA_PSS_2048_8192_SHA256");
        }
        Err(e) => {
            println!("✗ SIGNATURE_VERIFY_FAILED with RSA_PSS_2048_8192_SHA256: {:?}", e);
            
            // Check if signature length is correct
            if sig.len() != 512 {
                println!("ERROR: Signature length is {} bytes, expected 512 bytes (4096 bits)", sig.len());
            }
            
            // Check payload hash
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(&payload);
            let hash = hex::encode(hasher.finalize());
            println!("Payload SHA-256: {}", hash);
        }
    }
}

