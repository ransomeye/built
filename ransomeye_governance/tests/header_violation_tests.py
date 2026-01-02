# Path: /home/ransomeye/rebuild/ransomeye_governance/tests/header_violation_tests.py
# Author: RansomEye Core Team
# Purpose: Tests that verify header validator fails on missing headers

"""
Header Violation Tests: Verify header validator blocks files without mandatory headers.
"""

import unittest
import sys
import tempfile
from pathlib import Path

# Add tooling to path
sys.path.insert(0, str(Path(__file__).parent.parent / 'tooling'))

from header_validator import HeaderValidator


class TestHeaderViolations(unittest.TestCase):
    """Test header violation detection."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = Path(tempfile.mkdtemp())
        self.validator = HeaderValidator(project_root=str(self.temp_dir))
    
    def test_file_without_header_fails(self):
        """Test that file without header is detected."""
        test_file = self.temp_dir / "test.py"
        test_file.write_text("print('hello')\n")
        
        violation = self.validator.check_file(test_file)
        self.assertIsNotNone(violation, "File without header should be detected")
        self.assertIn('missing_components', violation)
    
    def test_file_with_partial_header_fails(self):
        """Test that file with partial header is detected."""
        test_file = self.temp_dir / "test.py"
        test_file.write_text("# Path: /test.py\nprint('hello')\n")
        
        violation = self.validator.check_file(test_file)
        self.assertIsNotNone(violation, "File with partial header should be detected")
        self.assertIn('Author', violation['missing_components'])
    
    def test_file_with_complete_header_passes(self):
        """Test that file with complete header passes."""
        test_file = self.temp_dir / "test.py"
        test_file.write_text(
            "# Path: /test.py\n"
            "# Author: RansomEye Core Team\n"
            "# Purpose: Test file\n"
            "print('hello')\n"
        )
        
        violation = self.validator.check_file(test_file)
        self.assertIsNone(violation, "File with complete header should pass")
    
    def test_file_with_wrong_author_fails(self):
        """Test that file with wrong author is detected."""
        test_file = self.temp_dir / "test.py"
        test_file.write_text(
            "# Path: /test.py\n"
            "# Author: Wrong Author\n"
            "# Purpose: Test file\n"
            "print('hello')\n"
        )
        
        violation = self.validator.check_file(test_file)
        self.assertIsNotNone(violation, "File with wrong author should be detected")
        self.assertIn('Author', violation['missing_components'])


class TestHeaderValidatorIntegration(unittest.TestCase):
    """Integration tests for header validator."""
    
    def test_validator_checks_directory(self):
        """Test that validator checks entire directory."""
        temp_dir = Path(tempfile.mkdtemp())
        validator = HeaderValidator(project_root=str(temp_dir))
        
        # Create file without header
        test_file = temp_dir / "test.py"
        test_file.write_text("print('hello')\n")
        
        violations = validator.check_directory()
        self.assertGreater(len(violations), 0, "Should detect violations in directory")


if __name__ == '__main__':
    unittest.main()

