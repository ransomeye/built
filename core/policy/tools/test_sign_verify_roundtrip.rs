// Path and File Name : /home/ransomeye/rebuild/core/policy/tools/test_sign_verify_roundtrip.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Test RSA-PSS sign and verify roundtrip

use ring::signature::{RsaKeyPair, UnparsedPublicKey, RSA_PSS_SHA256, RSA_PSS_2048_8192_SHA256};
use ring::rand::SystemRandom;
use std::fs;

fn main() {
    // Load private key
    let private_key_der = fs::read("/etc/ransomeye/keys/policy_signing_private.der")
        .expect("Failed to read private key");
    
    let key_pair = RsaKeyPair::from_pkcs8(&private_key_der)
        .expect("Failed to load key pair");
    
    // Load payload
    let payload = fs::read("/etc/ransomeye/policies/persistence.yaml.payload")
        .expect("Failed to read payload");
    
    println!("Payload length: {} bytes", payload.len());
    println!("Key modulus length: {} bytes", key_pair.public().modulus_len());
    
    // Sign with RSA_PSS_SHA256
    let rng = SystemRandom::new();
    let mut signature = vec![0u8; key_pair.public().modulus_len()];
    
    key_pair.sign(&RSA_PSS_SHA256, &rng, &payload, &mut signature)
        .expect("Failed to sign");
    
    println!("Signature length: {} bytes", signature.len());
    
    // Extract public key
    let public_key_der = key_pair.public().as_ref().to_vec();
    println!("Public key length: {} bytes", public_key_der.len());
    
    // Verify with RSA_PSS_2048_8192_SHA256
    let public_key = UnparsedPublicKey::new(&RSA_PSS_2048_8192_SHA256, &public_key_der);
    
    match public_key.verify(&payload, &signature) {
        Ok(_) => {
            println!("✓ ROUNDTRIP VERIFICATION SUCCESS");
        }
        Err(e) => {
            eprintln!("✗ ROUNDTRIP VERIFICATION FAILED: {:?}", e);
            std::process::exit(1);
        }
    }
    
    // Now try with the trust store public key
    let trust_store_pubkey = fs::read("/etc/ransomeye/trust_store/policy_signing.der")
        .expect("Failed to read trust store public key");
    
    println!("Trust store public key length: {} bytes", trust_store_pubkey.len());
    
    let trust_public_key = UnparsedPublicKey::new(&RSA_PSS_2048_8192_SHA256, &trust_store_pubkey);
    
    match trust_public_key.verify(&payload, &signature) {
        Ok(_) => {
            println!("✓ TRUST STORE VERIFICATION SUCCESS");
        }
        Err(e) => {
            eprintln!("✗ TRUST STORE VERIFICATION FAILED: {:?}", e);
            std::process::exit(1);
        }
    }
    
    // Try with the existing signature file
    let existing_sig = fs::read("/etc/ransomeye/policies/persistence.yaml.sig")
        .expect("Failed to read existing signature");
    
    println!("Existing signature length: {} bytes", existing_sig.len());
    
    match trust_public_key.verify(&payload, &existing_sig) {
        Ok(_) => {
            println!("✓ EXISTING SIGNATURE VERIFICATION SUCCESS");
            println!("SIGNATURE_VERIFIED_OK");
        }
        Err(e) => {
            eprintln!("✗ EXISTING SIGNATURE VERIFICATION FAILED: {:?}", e);
            std::process::exit(1);
        }
    }
}

