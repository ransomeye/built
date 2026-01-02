# Path: /home/ransomeye/rebuild/ransomeye_governance/tests/secret_violation_tests.py
# Author: RansomEye Core Team
# Purpose: Tests that verify secret validator fails on hardcoded secrets

"""
Secret Violation Tests: Verify secret validator blocks hardcoded secrets.
"""

import unittest
import sys
import tempfile
from pathlib import Path

# Add tooling to path
sys.path.insert(0, str(Path(__file__).parent.parent / 'tooling'))

from secret_validator import SecretValidator


class TestSecretViolations(unittest.TestCase):
    """Test secret violation detection."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = Path(tempfile.mkdtemp())
        self.validator = SecretValidator(project_root=str(self.temp_dir))
    
    def test_hardcoded_password_detected(self):
        """Test that hardcoded password is detected."""
        test_file = self.temp_dir / "test.py"
        test_file.write_text('password = "secret123"\n')
        
        violations = self.validator._scan_file(test_file)
        self.assertGreater(len(violations), 0, "Hardcoded password should be detected")
        self.assertIn('password', violations[0]['description'].lower())
    
    def test_hardcoded_api_key_detected(self):
        """Test that hardcoded API key is detected."""
        test_file = self.temp_dir / "test.py"
        test_file.write_text('api_key = "sk-1234567890"\n')
        
        violations = self.validator._scan_file(test_file)
        self.assertGreater(len(violations), 0, "Hardcoded API key should be detected")
    
    def test_hardcoded_token_detected(self):
        """Test that hardcoded token is detected."""
        test_file = self.temp_dir / "test.py"
        test_file.write_text('token = "abc123xyz"\n')
        
        violations = self.validator._scan_file(test_file)
        self.assertGreater(len(violations), 0, "Hardcoded token should be detected")
    
    def test_env_variable_allowed(self):
        """Test that environment variable usage is allowed."""
        test_file = self.temp_dir / "test.py"
        test_file.write_text('password = os.environ.get("DB_PASS")\n')
        
        violations = self.validator._scan_file(test_file)
        self.assertEqual(len(violations), 0, "ENV variable usage should be allowed")
    
    def test_empty_password_allowed(self):
        """Test that empty password is allowed."""
        test_file = self.temp_dir / "test.py"
        test_file.write_text('password = ""\n')
        
        violations = self.validator._scan_file(test_file)
        self.assertEqual(len(violations), 0, "Empty password should be allowed")


class TestSecretValidatorIntegration(unittest.TestCase):
    """Integration tests for secret validator."""
    
    def test_validator_scans_directory(self):
        """Test that validator scans entire directory."""
        temp_dir = Path(tempfile.mkdtemp())
        validator = SecretValidator(project_root=str(temp_dir))
        
        # Create file with hardcoded secret
        test_file = temp_dir / "test.py"
        test_file.write_text('password = "secret123"\n')
        
        is_valid = validator.validate()
        self.assertFalse(is_valid, "Should detect secrets in directory")


if __name__ == '__main__':
    unittest.main()

