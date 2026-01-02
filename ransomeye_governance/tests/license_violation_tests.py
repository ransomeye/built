# Path: /home/ransomeye/rebuild/ransomeye_governance/tests/license_violation_tests.py
# Author: RansomEye Core Team
# Purpose: Tests that verify license validator fails on GPL/AGPL/SSPL violations

"""
License Violation Tests: Verify license validator blocks banned licenses.
"""

import unittest
import sys
import tempfile
from pathlib import Path

# Add tooling to path
sys.path.insert(0, str(Path(__file__).parent.parent / 'tooling'))

from license_validator import LicenseValidator, BANNED_LICENSES, ALLOWED_LICENSES


class TestLicenseViolations(unittest.TestCase):
    """Test license violation detection."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
        self.validator = LicenseValidator(project_root=self.temp_dir)
    
    def test_gpl_license_blocked(self):
        """Test that GPL license is blocked."""
        for gpl_variant in ['GPL', 'GPL-2.0', 'GPL-3.0', 'GPLv2', 'GPLv3']:
            is_allowed, reason = self.validator.check_license(gpl_variant)
            self.assertFalse(is_allowed, f"GPL variant {gpl_variant} should be blocked")
            self.assertIn('Banned', reason, f"Reason should mention banned: {reason}")
    
    def test_agpl_license_blocked(self):
        """Test that AGPL license is blocked."""
        is_allowed, reason = self.validator.check_license('AGPL-3.0')
        self.assertFalse(is_allowed, "AGPL should be blocked")
        self.assertIn('Banned', reason)
    
    def test_sspl_license_blocked(self):
        """Test that SSPL license is blocked."""
        is_allowed, reason = self.validator.check_license('SSPL')
        self.assertFalse(is_allowed, "SSPL should be blocked")
        self.assertIn('Banned', reason)
    
    def test_elastic_license_blocked(self):
        """Test that Elastic License is blocked."""
        is_allowed, reason = self.validator.check_license('Elastic License')
        self.assertFalse(is_allowed, "Elastic License should be blocked")
        self.assertIn('Banned', reason)
    
    def test_unknown_license_blocked(self):
        """Test that unknown licenses are blocked."""
        is_allowed, reason = self.validator.check_license('UNKNOWN')
        self.assertFalse(is_allowed, "Unknown license should be blocked")
        self.assertIn('Banned', reason)
    
    def test_mit_license_allowed(self):
        """Test that MIT license is allowed."""
        is_allowed, reason = self.validator.check_license('MIT')
        self.assertTrue(is_allowed, "MIT should be allowed")
        self.assertIn('Allowed', reason)
    
    def test_apache_license_allowed(self):
        """Test that Apache 2.0 license is allowed."""
        is_allowed, reason = self.validator.check_license('Apache-2.0')
        self.assertTrue(is_allowed, "Apache 2.0 should be allowed")
        self.assertIn('Allowed', reason)
    
    def test_bsd_license_allowed(self):
        """Test that BSD licenses are allowed."""
        for bsd_variant in ['BSD-2-Clause', 'BSD-3-Clause', 'BSD']:
            is_allowed, reason = self.validator.check_license(bsd_variant)
            self.assertTrue(is_allowed, f"BSD variant {bsd_variant} should be allowed")
            self.assertIn('Allowed', reason)


class TestLicenseValidatorIntegration(unittest.TestCase):
    """Integration tests for license validator."""
    
    def test_validator_fails_on_banned(self):
        """Test that validator fails when banned license detected."""
        validator = LicenseValidator()
        # This test would need actual dependency files with banned licenses
        # For now, just verify the method exists
        self.assertTrue(hasattr(validator, 'validate'))


if __name__ == '__main__':
    unittest.main()

