// Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/tests/signing_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Ed25519 signing and verification tests

use ransomeye_linux_agent::security::signing::EventSigner;

#[test]
fn test_event_signing() {
    let signer = EventSigner::new().unwrap();
    
    let data = b"test event data";
    let signature = signer.sign(data).unwrap();
    
    assert!(!signature.is_empty());
    assert!(signature.len() > 0);
}

#[test]
fn test_signature_verification() {
    let signer = EventSigner::new().unwrap();
    
    let data = b"test event data";
    let signature = signer.sign(data).unwrap();
    let sequence = signer.sequence();
    
    let verified = signer.verify(data, &signature, sequence).unwrap();
    assert!(verified);
}

#[test]
fn test_signature_replay_safety() {
    let signer = EventSigner::new().unwrap();
    
    let data = b"test event data";
    let sig1 = signer.sign(data).unwrap();
    let seq1 = signer.sequence();
    
    let sig2 = signer.sign(data).unwrap();
    let seq2 = signer.sequence();
    
    // Signatures should be different (different sequence numbers)
    assert_ne!(sig1, sig2);
    assert_ne!(seq1, seq2);
    
    // Each signature should verify with its own sequence
    assert!(signer.verify(data, &sig1, seq1).unwrap());
    assert!(signer.verify(data, &sig2, seq2).unwrap());
    
    // Signature should not verify with wrong sequence (replay protection)
    assert!(!signer.verify(data, &sig1, seq2).unwrap());
}

#[test]
fn test_sequence_numbers_increment() {
    let signer = EventSigner::new().unwrap();
    
    let initial_seq = signer.sequence();
    signer.sign(b"data1").unwrap();
    let seq1 = signer.sequence();
    
    signer.sign(b"data2").unwrap();
    let seq2 = signer.sequence();
    
    assert!(seq1 > initial_seq);
    assert!(seq2 > seq1);
}

