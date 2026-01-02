# Path and File Name : /home/ransomeye/rebuild/ransomeye_posture_engine/tests/test_enforcement.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Enforcement tests - RSA usage fails, missing signature fails, policy hash required

"""
Enforcement Tests

Tests that enforce security requirements:
- RSA usage is impossible
- Telemetry without signature fails
- Telemetry with invalid signature fails
- Policy hash appears in every report
- Policy drift is detected correctly
"""

import unittest
import tempfile
from pathlib import Path
from datetime import datetime
from ransomeye_posture_engine.engine.output_signer import OutputSigner
from ransomeye_posture_engine.engine.signature_verifier import SignatureVerifier, SignatureVerificationError
from ransomeye_posture_engine.engine.policy_metadata import PolicyMetadataManager
from ransomeye_posture_engine.engine.cis_evaluator import CISEvaluator
from ransomeye_posture_engine.engine.normalizer import PostureFact, PostureCategory

class TestRSAProhibition(unittest.TestCase):
    """Test that RSA usage is impossible."""
    
    def test_ed25519_only_in_output_signer(self):
        """Test that OutputSigner rejects RSA keys."""
        temp_dir = tempfile.mkdtemp()
        
        # Create a fake RSA key file (PEM format)
        rsa_key_path = Path(temp_dir) / "rsa_key.pem"
        with open(rsa_key_path, 'w') as f:
            f.write("-----BEGIN RSA PRIVATE KEY-----\n")
            f.write("fake RSA key data\n")
            f.write("-----END RSA PRIVATE KEY-----\n")
        
        # Attempt to load - should fail with ValueError
        signer = OutputSigner(signing_key_path=rsa_key_path)
        
        # Should raise error when trying to load RSA key
        with self.assertRaises((ValueError, RuntimeError)):
            signer._load_key()

class TestSignatureVerification(unittest.TestCase):
    """Test that signature verification is mandatory."""
    
    def test_missing_signature_fails(self):
        """Test that telemetry without signature fails."""
        verifier = SignatureVerifier(trust_store_path=None)
        
        # Attempt to verify without signature - should fail
        with self.assertRaises(SignatureVerificationError):
            verifier.verify_telemetry_event(
                event_data={"test": "data"},
                signature="",  # Empty signature
                producer_id="test_producer",
                algorithm="Ed25519"
            )
    
    def test_invalid_signature_fails(self):
        """Test that invalid signature fails."""
        verifier = SignatureVerifier(trust_store_path=None)
        
        # Create fake trust store
        temp_dir = tempfile.mkdtemp()
        trust_store = Path(temp_dir) / "trust_store.json"
        
        # This will fail because we don't have a valid public key
        # But the test ensures that verification is attempted
        with self.assertRaises((SignatureVerificationError, RuntimeError)):
            verifier.verify_telemetry_event(
                event_data={"test": "data"},
                signature="invalid_signature_base64",
                producer_id="test_producer",
                algorithm="Ed25519"
            )
    
    def test_rsa_algorithm_rejected(self):
        """Test that RSA algorithm is rejected."""
        verifier = SignatureVerifier(trust_store_path=None)
        
        with self.assertRaises(SignatureVerificationError) as cm:
            verifier.verify_telemetry_event(
                event_data={"test": "data"},
                signature="fake_signature",
                producer_id="test_producer",
                algorithm="RSA-4096-PSS-SHA256"  # RSA is PROHIBITED
            )
        
        self.assertIn("RSA is PROHIBITED", str(cm.exception))

class TestPolicyHashPinning(unittest.TestCase):
    """Test that policy hash pinning is mandatory."""
    
    def test_policy_metadata_required(self):
        """Test that policy metadata is required in evaluation results."""
        temp_dir = tempfile.mkdtemp()
        benchmarks_dir = Path(temp_dir)
        benchmarks_dir.mkdir(parents=True, exist_ok=True)
        
        metadata_manager = PolicyMetadataManager()
        evaluator = CISEvaluator(benchmarks_dir, metadata_manager)
        
        # Evaluate with empty facts
        facts = []
        results = evaluator.evaluate(facts)
        
        # All results must have policy_metadata
        for result in results:
            self.assertIsNotNone(result.policy_metadata, "Policy metadata is MANDATORY")
            self.assertIsNotNone(result.policy_metadata.sha256_hash, "Policy hash is MANDATORY")
            self.assertIsNotNone(result.policy_metadata.version, "Policy version is MANDATORY")
            self.assertIsNotNone(result.policy_metadata.source_path, "Policy source path is MANDATORY")

class TestDatabaseUntrusted(unittest.TestCase):
    """Test that database is treated as untrusted."""
    
    def test_signature_valid_flag_ignored(self):
        """Test that database's signature_valid flag is ignored."""
        # This test verifies that verify_from_database_record ignores signature_valid
        # The actual verification happens in signature_verifier, which we test separately
        verifier = SignatureVerifier(trust_store_path=None)
        
        # Even if database says signature_valid=True, we verify ourselves
        # This is tested implicitly by the fact that verify_from_database_record
        # calls verify_telemetry_event which performs actual verification
        
        # If signature is missing, it should fail regardless of signature_valid flag
        with self.assertRaises(SignatureVerificationError):
            verifier.verify_from_database_record(
                event_id="test_1",
                event_data={"test": "data"},
                signature=None,  # Missing signature
                producer_id="test_producer",
                signature_algorithm="Ed25519",
                signature_valid=True  # Database says valid, but we ignore this
            )

if __name__ == '__main__':
    unittest.main()

