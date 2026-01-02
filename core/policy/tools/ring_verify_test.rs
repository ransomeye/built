// Path and File Name : /home/ransomeye/rebuild/core/policy/tools/ring_verify_test.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Isolated signature verification test using ring

use ring::signature::{UnparsedPublicKey, RSA_PSS_2048_8192_SHA256};
use std::fs;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let pubkey_path = if args.len() > 1 {
        &args[1]
    } else {
        "/etc/ransomeye/trust_store/policy_signing.der"
    };
    
    let payload_path = if args.len() > 2 {
        &args[2]
    } else {
        "/tmp/policy_payload.bin"
    };
    
    let sig_path = if args.len() > 3 {
        &args[3]
    } else {
        "/tmp/policy_signature.bin"
    };

    let pubkey = fs::read(pubkey_path)
        .expect(&format!("pubkey read failed from {}", pubkey_path));

    let payload = fs::read(payload_path)
        .expect(&format!("payload read failed from {}", payload_path));

    let sig = fs::read(sig_path)
        .expect(&format!("signature read failed from {}", sig_path));

    let pk = UnparsedPublicKey::new(&RSA_PSS_2048_8192_SHA256, &pubkey);

    match pk.verify(&payload, &sig) {
        Ok(_) => {
            println!("SIGNATURE_VERIFIED_OK");
        }
        Err(e) => {
            println!("SIGNATURE_VERIFY_FAILED: {:?}", e);
            std::process::exit(1);
        }
    }
}

