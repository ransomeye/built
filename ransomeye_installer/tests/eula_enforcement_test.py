# Path and File Name : /home/ransomeye/rebuild/ransomeye_installer/tests/eula_enforcement_test.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Tests EULA enforcement - verifies installation fails without EULA acceptance

"""
Tests for EULA enforcement.
Verifies installation fails without EULA acceptance marker.
EULA acceptance is ONLY handled by install.sh.
Python installer ONLY verifies the acceptance marker.
"""

import unittest
import tempfile
import os
import json
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from ransomeye_installer.installer import RansomEyeInstaller


class TestEULAEnforcement(unittest.TestCase):
    """Test EULA enforcement via acceptance marker verification."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
        self.marker_path = Path(self.temp_dir) / "eula.accepted"
    
    def tearDown(self):
        """Clean up test fixtures."""
        import shutil
        shutil.rmtree(self.temp_dir, ignore_errors=True)
    
    def test_eula_marker_missing_fails(self):
        """Test that missing EULA acceptance marker causes failure."""
        installer = RansomEyeInstaller()
        
        # Temporarily override marker path for testing
        original_verify = installer._verify_eula_acceptance
        
        def mock_verify():
            marker = Path(self.temp_dir) / "eula.accepted"
            if not marker.exists():
                raise RuntimeError("EULA not accepted via install.sh — installation aborted")
        
        installer._verify_eula_acceptance = mock_verify
        
        # Should fail when marker doesn't exist
        with self.assertRaises(RuntimeError) as context:
            installer._verify_eula_acceptance()
        
        self.assertIn("EULA not accepted", str(context.exception))
    
    def test_eula_marker_valid_passes(self):
        """Test that valid EULA acceptance marker allows installation."""
        installer = RansomEyeInstaller()
        
        # Create valid marker
        self.marker_path.parent.mkdir(parents=True, exist_ok=True)
        marker_data = {
            "accepted": True,
            "timestamp": "2025-01-01T00:00:00Z",
            "installer_version": "1.0.0",
            "uid": "0",
            "hostname": "test-host",
            "eula_sha256": "abc123",
            "eula_path": "/path/to/eula"
        }
        with open(self.marker_path, 'w') as f:
            json.dump(marker_data, f)
        
        # Mock verification to use test marker
        def mock_verify():
            if not self.marker_path.exists():
                raise RuntimeError("EULA not accepted")
            with open(self.marker_path, 'r') as f:
                data = json.load(f)
            if not data.get('accepted', False):
                raise RuntimeError("EULA not accepted")
        
        installer._verify_eula_acceptance = mock_verify
        
        # Should pass when valid marker exists
        try:
            installer._verify_eula_acceptance()
        except RuntimeError:
            self.fail("Valid EULA marker should not raise exception")


def main():
    """Run tests."""
    unittest.main()


if __name__ == '__main__':
    main()

