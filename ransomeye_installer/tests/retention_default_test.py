# Path and File Name : /home/ransomeye/rebuild/ransomeye_installer/tests/retention_default_test.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Tests retention default application - verifies defaults are applied when user skips

"""
Tests for retention default application.
Verifies defaults are applied when user skips configuration.
"""

import unittest
import tempfile
import os
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from ransomeye_installer.retention.retention_writer import RetentionWriter


class TestRetentionDefaults(unittest.TestCase):
    """Test retention default application."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
        self.config_path = os.path.join(self.temp_dir, 'retention.txt')
    
    def tearDown(self):
        """Clean up test fixtures."""
        import shutil
        shutil.rmtree(self.temp_dir, ignore_errors=True)
    
    def test_defaults_applied(self):
        """Test that defaults are applied when None provided."""
        writer = RetentionWriter(self.config_path)
        path = writer.write_defaults()
        
        self.assertTrue(os.path.exists(path))
        
        # Read and verify defaults
        with open(path, 'r') as f:
            content = f.read()
            self.assertIn('TELEMETRY_RETENTION_MONTHS=6', content)
            self.assertIn('FORENSIC_RETENTION_DAYS=10', content)
            self.assertIn('DISK_MAX_USAGE_PERCENT=80', content)
    
    def test_custom_values(self):
        """Test that custom values are written."""
        writer = RetentionWriter(self.config_path)
        path = writer.write_retention(
            telemetry_months=12,
            forensic_days=30,
            disk_max_percent=75
        )
        
        self.assertTrue(os.path.exists(path))
        
        # Read and verify custom values
        with open(path, 'r') as f:
            content = f.read()
            self.assertIn('TELEMETRY_RETENTION_MONTHS=12', content)
            self.assertIn('FORENSIC_RETENTION_DAYS=30', content)
            self.assertIn('DISK_MAX_USAGE_PERCENT=75', content)


def main():
    """Run tests."""
    unittest.main()


if __name__ == '__main__':
    main()

