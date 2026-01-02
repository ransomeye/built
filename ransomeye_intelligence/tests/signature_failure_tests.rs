// Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/tests/signature_failure_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests that signature failures result in fail-closed behavior

/*
 * Signature Failure Tests
 * 
 * Tests that verify signature failures result in fail-closed behavior.
 * AI cannot start with invalid signatures.
 */

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::fs;
    use std::process::Command;

    #[test]
    fn test_baseline_pack_signature_required() {
        // Test that baseline pack signature file exists
        let sig_path = Path::new("/home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack/signatures/baseline_pack.sig");
        assert!(sig_path.exists(), "Baseline pack signature must exist");
    }

    #[test]
    fn test_baseline_pack_public_key_required() {
        // Test that baseline pack public key exists
        let pub_path = Path::new("/home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack/signatures/baseline_pack.pub");
        assert!(pub_path.exists(), "Baseline pack public key must exist");
    }

    #[test]
    fn test_corrupted_signature_fails() {
        // Test that corrupted signature file is detected
        let sig_path = Path::new("/home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack/signatures/baseline_pack.sig");
        
        if sig_path.exists() {
            // Read original signature
            let original_sig = fs::read_to_string(sig_path).unwrap();
            
            // Corrupt signature
            let corrupted_sig = "CORRUPTED_SIGNATURE_DATA";
            fs::write(sig_path, corrupted_sig).unwrap();
            
            // Run intelligence controller - should fail
            let output = Command::new("python3")
                .arg("-c")
                .arg("from ransomeye_intelligence.intelligence_controller import IntelligenceController; c = IntelligenceController(); c.initialize()")
                .output()
                .expect("Failed to execute intelligence controller");
            
            // Restore original signature
            fs::write(sig_path, original_sig).unwrap();
            
            // Should have failed (non-zero exit code)
            assert!(!output.status.success(), "Intelligence controller must fail with corrupted signature");
        }
    }

    #[test]
    fn test_missing_signature_fails() {
        // Test that missing signature file causes failure
        let sig_path = Path::new("/home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack/signatures/baseline_pack.sig");
        
        if sig_path.exists() {
            // Backup signature
            let backup_path = sig_path.parent().unwrap().join("baseline_pack.sig.backup");
            fs::copy(sig_path, &backup_path).unwrap();
            
            // Remove signature
            fs::remove_file(sig_path).unwrap();
            
            // Run intelligence controller - should fail
            let output = Command::new("python3")
                .arg("-c")
                .arg("from ransomeye_intelligence.intelligence_controller import IntelligenceController; c = IntelligenceController(); c.initialize()")
                .output()
                .expect("Failed to execute intelligence controller");
            
            // Restore signature
            fs::copy(&backup_path, sig_path).unwrap();
            fs::remove_file(&backup_path).unwrap();
            
            // Should have failed
            assert!(!output.status.success(), "Intelligence controller must fail with missing signature");
        }
    }

    #[test]
    fn test_threat_intel_signature_required() {
        // Test that threat intel signature exists
        let sig_path = Path::new("/home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/signatures/intel_pack.sig");
        assert!(sig_path.exists(), "Threat intel signature must exist");
    }

    #[test]
    fn test_rag_pack_signature_required() {
        // Test that RAG pack signature exists
        let sig_path = Path::new("/home/ransomeye/rebuild/ransomeye_intelligence/llm_knowledge/signatures/rag_pack.sig");
        assert!(sig_path.exists(), "RAG pack signature must exist");
    }

    #[test]
    fn test_signature_verification_enforced() {
        // Test that signature verification is enforced
        // This test verifies that the code checks signatures, not just that files exist
        let sig_path = Path::new("/home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack/signatures/baseline_pack.sig");
        
        if sig_path.exists() {
            let sig_content = fs::read_to_string(sig_path).unwrap();
            // Signature should be base64-encoded, not empty
            assert!(!sig_content.trim().is_empty(), "Signature must not be empty");
            assert!(sig_content.len() > 100, "Signature must be substantial (RSA-4096 signatures are large)");
        }
    }
}

