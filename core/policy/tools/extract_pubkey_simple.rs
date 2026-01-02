// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/tools/extract_pubkey_simple.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details: Extract public key using ring

use std::env;
use std::fs;
use ring::signature::{RsaKeyPair, KeyPair};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <private_key_der> <output_public_key_der>", args[0]);
        return;
    }
    
    let private_key_der = fs::read(&args[1]).expect("Failed to read private key");
    let key_pair = RsaKeyPair::from_pkcs8(&private_key_der).expect("Failed to load key pair");
    
    let public_key_bytes = key_pair.public_key().as_ref();
    
    fs::write(&args[2], public_key_bytes).expect("Failed to write public key");
    println!("Extracted {} bytes", public_key_bytes.len());
}

