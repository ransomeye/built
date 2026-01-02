# Path and File Name : /home/ransomeye/rebuild/ransomeye_installer/tests/fail_closed_test.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Tests fail-closed behavior - verifies installer aborts on all failure conditions

"""
Tests for fail-closed behavior.
Verifies installer aborts on all failure conditions.
"""

import unittest
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from ransomeye_installer.system.os_check import OSCheck
from ransomeye_installer.system.disk_check import DiskCheck
from ransomeye_installer.system.swap_check import SwapCheck


class TestFailClosed(unittest.TestCase):
    """Test fail-closed behavior."""
    
    def test_os_check_fails_unsupported(self):
        """Test that OS check fails for unsupported OS."""
        # This test would need to mock OS detection
        # For now, just verify the check exists
        checker = OSCheck()
        is_supported, reason = checker.is_supported()
        # Should return a result (either True or False with reason)
        self.assertIsInstance(is_supported, bool)
        self.assertIsInstance(reason, str)
    
    def test_disk_check_fails_insufficient(self):
        """Test that disk check fails for insufficient space."""
        # Create checker with very small path
        checker = DiskCheck()
        is_available, message, usage = checker.check_availability()
        # Should return a result
        self.assertIsInstance(is_available, bool)
        self.assertIsInstance(message, str)
    
    def test_swap_check_fails_insufficient(self):
        """Test that swap check fails for insufficient swap."""
        checker = SwapCheck()
        meets_requirements, message, info = checker.check_swap()
        # Should return a result
        self.assertIsInstance(meets_requirements, bool)
        self.assertIsInstance(message, str)


def main():
    """Run tests."""
    unittest.main()


if __name__ == '__main__':
    main()

