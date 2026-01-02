// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/security/sign_policy.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Ring-based RSA-4096 policy signing tool (build-time)

/*
 * Policy Signing Tool
 * 
 * Signs policy files using ring's RSA-4096 implementation.
 * This is a build-time tool used to generate signatures for policy files.
 */

use ring::signature::{self, RsaKeyPair};
use ring::rand::SystemRandom;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};
use std::fs;
use std::path::Path;

/// Sign policy content using RSA-4096 with ring
/// 
/// # Arguments
/// * `policy_bytes` - The policy content as bytes (without signature fields)
/// * `private_key_der` - The RSA private key in DER format (PKCS#8)
/// 
/// # Returns
/// * `(signature_base64, content_hash)` - Base64-encoded signature and SHA-256 hash
pub fn sign_policy(
    policy_bytes: &[u8],
    private_key_der: &[u8],
) -> Result<(String, String), Box<dyn std::error::Error>> {
    // Load RSA key pair from DER
    let key_pair = RsaKeyPair::from_pkcs8(private_key_der)
        .map_err(|e| format!("Failed to load RSA key pair: {:?}", e))?;
    
    // Verify key size is 4096 bits
    if key_pair.public_modulus_len() != 512 {
        return Err(format!(
            "Key size mismatch: expected 512 bytes (4096 bits), got {} bytes",
            key_pair.public_modulus_len()
        ).into());
    }
    
    // Compute SHA-256 hash of policy content
    let mut hasher = Sha256::new();
    hasher.update(policy_bytes);
    let content_hash = hex::encode(hasher.finalize());
    
    // Sign using RSA_PSS_SHA256 (matches verification algorithm RSA_PSS_2048_8192_SHA256)
    // This performs: SHA-256(policy_bytes) then RSA-PSS signature
    let rng = SystemRandom::new();
    let mut signature = vec![0u8; key_pair.public_modulus_len()];
    
    key_pair.sign(
        &signature::RSA_PSS_SHA256,
        &rng,
        policy_bytes,
        &mut signature,
    ).map_err(|e| format!("Failed to sign policy: {:?}", e))?;
    
    // Encode signature as base64
    let signature_base64 = general_purpose::STANDARD.encode(&signature);
    
    Ok((signature_base64, content_hash))
}

/// Sign a policy file
/// 
/// Reads the policy file as RAW BYTES, removes signature fields, signs the content,
/// and returns the signature and hash.
/// 
/// This ensures byte-exact parity with verification.
pub fn sign_policy_file(
    policy_path: &Path,
    private_key_path: &Path,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    // Step 1: Read policy file as RAW BYTES (fs::read - no string conversion)
    let raw_policy_bytes = fs::read(policy_path)
        .map_err(|e| format!("Failed to read policy file: {}", e))?;
    
    // Step 2: Convert to string for parsing (preserving exact encoding)
    let policy_content = String::from_utf8(raw_policy_bytes)
        .map_err(|e| format!("Failed to convert policy bytes to UTF-8: {}", e))?;
    
    // Step 3: Parse YAML to remove signature fields
    let mut policy_data: serde_yaml::Value = serde_yaml::from_str(&policy_content)
        .map_err(|e| format!("Failed to parse policy YAML: {}", e))?;
    
    // Step 4: Remove signature-related fields
    if let Some(obj) = policy_data.as_mapping_mut() {
        obj.remove("signature");
        obj.remove("signature_hash");
        obj.remove("signature_alg");
        obj.remove("key_id");
    }
    
    // Step 5: Serialize back to YAML (this is what gets signed - must match verification exactly)
    let policy_bytes = serde_yaml::to_string(&policy_data)
        .map_err(|e| format!("Failed to serialize policy: {}", e))?;
    
    // Step 6: Read private key (DER format)
    let private_key_der = fs::read(private_key_path)
        .map_err(|e| format!("Failed to read private key: {}", e))?;
    
    // Step 7: Sign the EXACT serialized bytes
    sign_policy(policy_bytes.as_bytes(), &private_key_der)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ring::signature::{self, RsaKeyPair};
    use ring::rand::SystemRandom;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_sign_and_verify_roundtrip() {
        // Generate a test RSA-4096 key pair
        let rng = SystemRandom::new();
        let pkcs8_bytes = signature::RsaKeyPair::generate_pkcs8(&signature::RSA_KEY_PAIR_4096_BITS, &rng)
            .expect("Failed to generate key pair");
        
        let key_pair = RsaKeyPair::from_pkcs8(pkcs8_bytes.as_ref())
            .expect("Failed to load key pair");
        
        // Extract public key
        let public_key_der = key_pair.public_key().as_ref().to_vec();
        
        // Test policy content
        let policy_content = b"id: test_policy\nversion: 1.0.0\nname: Test Policy";
        
        // Sign the policy
        let (signature_base64, content_hash) = sign_policy(policy_content, pkcs8_bytes.as_ref())
            .expect("Failed to sign policy");
        
        // Verify the signature using ring
        let signature_bytes = general_purpose::STANDARD.decode(&signature_base64)
            .expect("Failed to decode signature");
        
        let public_key = signature::UnparsedPublicKey::new(
            &signature::RSA_PSS_2048_8192_SHA256,
            &public_key_der,
        );
        
        let verify_result = public_key.verify(policy_content, &signature_bytes);
        assert!(verify_result.is_ok(), "Signature verification should succeed");
        
        // Verify hash
        let mut hasher = Sha256::new();
        hasher.update(policy_content);
        let expected_hash = hex::encode(hasher.finalize());
        assert_eq!(content_hash, expected_hash, "Hash should match");
    }
}

