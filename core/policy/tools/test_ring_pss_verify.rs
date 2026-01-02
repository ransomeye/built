// Path and File Name : /home/ransomeye/rebuild/core/policy/tools/test_ring_pss_verify.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Isolated ring verification test for RSA-PSS-SHA256 signatures

use ring::signature::{UnparsedPublicKey, RSA_PSS_2048_8192_SHA256};
use std::fs;

fn main() {
    let pubkey = fs::read("/etc/ransomeye/trust_store/policy_signing.der")
        .expect("Failed to read public key");
    
    let payload = fs::read("/etc/ransomeye/policies/persistence.yaml.payload")
        .expect("Failed to read payload");
    
    let sig = fs::read("/etc/ransomeye/policies/persistence.yaml.sig")
        .expect("Failed to read signature");

    println!("Public key length: {} bytes", pubkey.len());
    println!("Payload length: {} bytes", payload.len());
    println!("Signature length: {} bytes", sig.len());

    let key = UnparsedPublicKey::new(&RSA_PSS_2048_8192_SHA256, &pubkey);
    match key.verify(&payload, &sig) {
        Ok(_) => {
            println!("SIGNATURE_VERIFIED_OK");
        }
        Err(e) => {
            eprintln!("VERIFY FAILED: {:?}", e);
            std::process::exit(1);
        }
    }
}

