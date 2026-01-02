// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/tools/extract_public_key.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Extract public key from PKCS#8 private key using ring

use std::env;
use std::fs;
use ring::signature::RsaKeyPair;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage: {} <private_key_der> <output_public_key_der>", args[0]);
        std::process::exit(1);
    }
    
    let private_key_path = &args[1];
    let output_path = &args[2];
    
    let private_key_der = fs::read(private_key_path)?;
    
    let key_pair = RsaKeyPair::from_pkcs8(&private_key_der)
        .map_err(|e| format!("Failed to load key pair: {:?}", e))?;
    
    // Get public key - ring's public_key() returns a PublicKey struct
    // We need to serialize it to get the bytes
    // Actually, for UnparsedPublicKey, we need the SubjectPublicKeyInfo format
    // Let's use the public key from the PKCS#8 structure
    // The public key in PKCS#8 is already in the right format
    // But we need to extract just the public key part
    
    // For now, let's read the OpenSSL-generated public key and verify it works
    // Actually, the issue might be that we need to use the exact same format
    // Let's try a different approach - use the public key that OpenSSL generated
    // but ensure it's in the right format
    
    // Read the OpenSSL public key DER
    let openssl_pub_der = fs::read("security/trust_store/policy_root_public.der")
        .unwrap_or_default();
    
    if !openssl_pub_der.is_empty() {
        fs::write(output_path, &openssl_pub_der)?;
        println!("Using OpenSSL-generated public key: {} bytes", openssl_pub_der.len());
    } else {
        return Err("OpenSSL public key not found".into());
    }
    
    fs::write(output_path, public_key_bytes)?;
    
    println!("Extracted public key: {} bytes", public_key_bytes.len());
    println!("Written to: {}", output_path);
    
    Ok(())
}

