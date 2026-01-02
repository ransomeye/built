// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/tests/ring_verify_test.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Test ring signature verification directly

use std::fs;
use ring::signature::{self, UnparsedPublicKey};
use base64::{Engine as _, engine::general_purpose};
use serde_yaml;

#[test]
fn test_ring_verify_persistence_policy() {
    // Step 1: Read policy as RAW BYTES (matching verification flow)
    let raw_policy_bytes = fs::read("policies/persistence.yaml")
        .expect("Failed to read policy");
    
    // Step 2: Convert to string for parsing (preserving exact encoding)
    let content = String::from_utf8(raw_policy_bytes)
        .expect("Failed to convert policy bytes to UTF-8");
    
    // Step 3: Parse YAML to extract signature
    let policy_data: serde_yaml::Value = serde_yaml::from_str(&content)
        .expect("Failed to parse policy");
    
    let signature_b64 = policy_data.get("signature")
        .and_then(|v| v.as_str())
        .expect("No signature found");
    
    // Step 4: Remove signature fields (matching signing process exactly)
    let mut policy_for_verify = policy_data.clone();
    if let Some(obj) = policy_for_verify.as_mapping_mut() {
        obj.remove("signature");
        obj.remove("signature_hash");
        obj.remove("signature_alg");
        obj.remove("key_id");
    }
    
    // Step 5: Serialize (matching signing process exactly - this is what was signed)
    let content_to_verify = serde_yaml::to_string(&policy_for_verify)
        .expect("Failed to serialize");
    
    // Step 6: Load public key (extracted by ring from private key - ensures correct format)
    let public_key_der = fs::read("security/trust_store/policy_root_public_ring_extracted.der")
        .or_else(|_| fs::read("security/trust_store/policy_root_public.der"))
        .expect("Failed to read public key");
    
    // Step 7: Decode signature
    let signature_bytes = general_purpose::STANDARD.decode(signature_b64)
        .expect("Failed to decode signature");
    
    println!("Content length: {} bytes", content_to_verify.len());
    println!("Signature length: {} bytes", signature_bytes.len());
    println!("Public key length: {} bytes", public_key_der.len());
    
    // Step 8: Verify with ring using EXACT bytes that were signed
    // RSA_PSS_2048_8192_SHA256 matches the signing algorithm RSA_PSS_SHA256
    let public_key = UnparsedPublicKey::new(
        &signature::RSA_PSS_2048_8192_SHA256,
        &public_key_der,
    );
    
    let result = public_key.verify(content_to_verify.as_bytes(), &signature_bytes);
    
    match result {
        Ok(_) => {
            println!("✓ Ring verification SUCCESS");
            assert!(true);
        }
        Err(e) => {
            println!("✗ Ring verification FAILED: {:?}", e);
            panic!("Ring verification failed: {:?}", e);
        }
    }
}

#[test]
fn test_byte_exact_verification() {
    // Test that byte-identical policy passes verification
    // This test ensures the verification process matches signing exactly
    
    let raw_policy_bytes = fs::read("policies/persistence.yaml")
        .expect("Failed to read policy");
    
    let content = String::from_utf8(raw_policy_bytes.clone())
        .expect("Failed to convert policy bytes to UTF-8");
    
    let policy_data: serde_yaml::Value = serde_yaml::from_str(&content)
        .expect("Failed to parse policy");
    
    let signature_b64 = policy_data.get("signature")
        .and_then(|v| v.as_str())
        .expect("No signature found");
    
    // Remove signature fields and serialize (matching signing process)
    let mut policy_for_verify = policy_data.clone();
    if let Some(obj) = policy_for_verify.as_mapping_mut() {
        obj.remove("signature");
        obj.remove("signature_hash");
        obj.remove("signature_alg");
        obj.remove("key_id");
    }
    
    let content_to_verify = serde_yaml::to_string(&policy_for_verify)
        .expect("Failed to serialize");
    
    let public_key_der = fs::read("security/trust_store/policy_root_public_ring_extracted.der")
        .or_else(|_| fs::read("security/trust_store/policy_root_public.der"))
        .expect("Failed to read public key");
    
    let signature_bytes = general_purpose::STANDARD.decode(signature_b64)
        .expect("Failed to decode signature");
    
    let public_key = UnparsedPublicKey::new(
        &signature::RSA_PSS_2048_8192_SHA256,
        &public_key_der,
    );
    
    // Byte-identical content should verify successfully
    let result = public_key.verify(content_to_verify.as_bytes(), &signature_bytes);
    assert!(result.is_ok(), "Byte-identical policy must pass verification");
}

#[test]
fn test_whitespace_breaks_verification() {
    // Test that modifying whitespace breaks verification
    let raw_policy_bytes = fs::read("policies/persistence.yaml")
        .expect("Failed to read policy");
    
    let content = String::from_utf8(raw_policy_bytes)
        .expect("Failed to convert policy bytes to UTF-8");
    
    let policy_data: serde_yaml::Value = serde_yaml::from_str(&content)
        .expect("Failed to parse policy");
    
    let signature_b64 = policy_data.get("signature")
        .and_then(|v| v.as_str())
        .expect("No signature found");
    
    let mut policy_for_verify = policy_data.clone();
    if let Some(obj) = policy_for_verify.as_mapping_mut() {
        obj.remove("signature");
        obj.remove("signature_hash");
        obj.remove("signature_alg");
        obj.remove("key_id");
    }
    
    let mut content_to_verify = serde_yaml::to_string(&policy_for_verify)
        .expect("Failed to serialize");
    
    // Modify whitespace (add extra space)
    content_to_verify.push_str(" ");
    
    let public_key_der = fs::read("security/trust_store/policy_root_public_ring_extracted.der")
        .or_else(|_| fs::read("security/trust_store/policy_root_public.der"))
        .expect("Failed to read public key");
    
    let signature_bytes = general_purpose::STANDARD.decode(signature_b64)
        .expect("Failed to decode signature");
    
    let public_key = UnparsedPublicKey::new(
        &signature::RSA_PSS_2048_8192_SHA256,
        &public_key_der,
    );
    
    // Modified content should fail verification
    let result = public_key.verify(content_to_verify.as_bytes(), &signature_bytes);
    assert!(result.is_err(), "Whitespace modification must break verification");
}

#[test]
fn test_tampering_breaks_verification() {
    // Test that tampering ANY byte fails verification
    let raw_policy_bytes = fs::read("policies/persistence.yaml")
        .expect("Failed to read policy");
    
    let content = String::from_utf8(raw_policy_bytes)
        .expect("Failed to convert policy bytes to UTF-8");
    
    let policy_data: serde_yaml::Value = serde_yaml::from_str(&content)
        .expect("Failed to parse policy");
    
    let signature_b64 = policy_data.get("signature")
        .and_then(|v| v.as_str())
        .expect("No signature found");
    
    let mut policy_for_verify = policy_data.clone();
    if let Some(obj) = policy_for_verify.as_mapping_mut() {
        obj.remove("signature");
        obj.remove("signature_hash");
        obj.remove("signature_alg");
        obj.remove("key_id");
    }
    
    let mut content_to_verify = serde_yaml::to_string(&policy_for_verify)
        .expect("Failed to serialize");
    
    // Tamper with content (change a byte)
    if !content_to_verify.is_empty() {
        let mut bytes = content_to_verify.into_bytes();
        bytes[0] = bytes[0].wrapping_add(1); // Modify first byte
        content_to_verify = String::from_utf8(bytes).expect("Invalid UTF-8");
    }
    
    let public_key_der = fs::read("security/trust_store/policy_root_public_ring_extracted.der")
        .or_else(|_| fs::read("security/trust_store/policy_root_public.der"))
        .expect("Failed to read public key");
    
    let signature_bytes = general_purpose::STANDARD.decode(signature_b64)
        .expect("Failed to decode signature");
    
    let public_key = UnparsedPublicKey::new(
        &signature::RSA_PSS_2048_8192_SHA256,
        &public_key_der,
    );
    
    // Tampered content should fail verification
    let result = public_key.verify(content_to_verify.as_bytes(), &signature_bytes);
    assert!(result.is_err(), "Tampering must break verification");
}

