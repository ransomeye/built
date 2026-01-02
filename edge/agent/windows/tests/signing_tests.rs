// Path and File Name : /home/ransomeye/rebuild/ransomeye_windows_agent/tests/signing_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Ed25519 signing and verification tests

#[cfg(test)]
mod tests {
    use ransomeye_windows_agent::security::signing::EventSigner;
    
    #[test]
    fn test_signer_creation() {
        let signer = EventSigner::new();
        assert!(signer.is_ok());
    }
    
    #[test]
    fn test_event_signing() {
        let signer = EventSigner::new().unwrap();
        let data = b"test event data";
        
        let signature = signer.sign(data);
        assert!(signature.is_ok());
        
        let sig = signature.unwrap();
        assert!(!sig.is_empty());
    }
    
    #[test]
    fn test_signature_verification() {
        let signer = EventSigner::new().unwrap();
        let data = b"test event data";
        
        let signature = signer.sign(data).unwrap();
        let verified = signer.verify(data, &signature, 0);
        
        assert!(verified.is_ok());
        assert!(verified.unwrap());
    }
    
    #[test]
    fn test_replay_protection() {
        let signer = EventSigner::new().unwrap();
        let data = b"test event data";
        
        let sig1 = signer.sign(data).unwrap();
        let seq1 = signer.sequence();
        
        let sig2 = signer.sign(data).unwrap();
        let seq2 = signer.sequence();
        
        // Sequence numbers should be different
        assert_ne!(seq1, seq2);
        // Signatures should be different due to sequence number
        assert_ne!(sig1, sig2);
    }
    
    #[test]
    fn test_signature_tampering() {
        let signer = EventSigner::new().unwrap();
        let data = b"test event data";
        
        let signature = signer.sign(data).unwrap();
        let tampered_data = b"tampered data";
        
        let verified = signer.verify(tampered_data, &signature, 0);
        assert!(verified.is_ok());
        assert!(!verified.unwrap()); // Should fail verification
    }
}

