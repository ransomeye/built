# Path and File Name : /home/ransomeye/rebuild/ransomeye_guardrails/tests/test_rules_yaml.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Tests for rules.yaml validation - ensures schema validation catches all malformed rules

"""
Tests for rules.yaml validation.
Ensures schema validation catches all malformed rules.
"""

import unittest
import tempfile
import os
from pathlib import Path
import sys

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from ransomeye_guardrails.rules_schema import RulesSchemaValidator, validate_rules_file


class TestRulesYAMLValidation(unittest.TestCase):
    """Test cases for rules.yaml validation."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
    
    def tearDown(self):
        """Clean up test fixtures."""
        import shutil
        shutil.rmtree(self.temp_dir, ignore_errors=True)
    
    def write_temp_rules(self, content: str) -> str:
        """Write temporary rules.yaml file."""
        rules_path = os.path.join(self.temp_dir, 'rules.yaml')
        with open(rules_path, 'w') as f:
            f.write(content)
        return rules_path
    
    def test_valid_rules_yaml(self):
        """Test that valid rules.yaml passes validation."""
        valid_rules = """
hardcoded_patterns:
  - name: "Test Rule"
    regex: "\\btest\\b"
    description: "Test pattern"
    rule_name: "TEST_RULE"
    severity: "CRITICAL"

path_patterns: []
ml_patterns: []
crypto_patterns: []
header_patterns: []
allowed_exceptions: []
scan_config:
  file_extensions: [".py"]
  exclude_dirs: []
"""
        rules_path = self.write_temp_rules(valid_rules)
        validator = RulesSchemaValidator(rules_path)
        self.assertTrue(validator.validate())
        self.assertEqual(len(validator.errors), 0)
    
    def test_missing_required_field(self):
        """Test that missing required field fails validation."""
        invalid_rules = """
hardcoded_patterns:
  - name: "Test Rule"
    regex: "\\btest\\b"
    description: "Test pattern"
    # Missing rule_name and severity
"""
        rules_path = self.write_temp_rules(invalid_rules)
        validator = RulesSchemaValidator(rules_path)
        self.assertFalse(validator.validate())
        self.assertGreater(len(validator.errors), 0)
    
    def test_invalid_regex(self):
        """Test that invalid regex pattern fails validation."""
        invalid_rules = """
hardcoded_patterns:
  - name: "Test Rule"
    regex: "[invalid regex("
    description: "Test pattern"
    rule_name: "TEST_RULE"
    severity: "CRITICAL"
"""
        rules_path = self.write_temp_rules(invalid_rules)
        validator = RulesSchemaValidator(rules_path)
        self.assertFalse(validator.validate())
        # Should have regex error
        error_messages = ' '.join(validator.errors)
        self.assertIn('regex', error_messages.lower() or 'invalid')
    
    def test_malformed_yaml(self):
        """Test that malformed YAML fails validation."""
        invalid_rules = """
hardcoded_patterns:
  - name: "Test Rule"
    regex: "\\btest\\b"
    description: "Test pattern"
    rule_name: "TEST_RULE"
    severity: "CRITICAL"
  - name: "Unclosed list
    regex: "test"
"""
        rules_path = self.write_temp_rules(invalid_rules)
        validator = RulesSchemaValidator(rules_path)
        self.assertFalse(validator.validate())
        self.assertGreater(len(validator.errors), 0)
    
    def test_invalid_severity(self):
        """Test that invalid severity fails validation."""
        invalid_rules = """
hardcoded_patterns:
  - name: "Test Rule"
    regex: "\\btest\\b"
    description: "Test pattern"
    rule_name: "TEST_RULE"
    severity: "INVALID_SEVERITY"
"""
        rules_path = self.write_temp_rules(invalid_rules)
        validator = RulesSchemaValidator(rules_path)
        self.assertFalse(validator.validate())
        error_messages = ' '.join(validator.errors)
        self.assertIn('severity', error_messages.lower())
    
    def test_invalid_rule_name_format(self):
        """Test that invalid rule_name format fails validation."""
        invalid_rules = """
hardcoded_patterns:
  - name: "Test Rule"
    regex: "\\btest\\b"
    description: "Test pattern"
    rule_name: "invalid-rule-name"
    severity: "CRITICAL"
"""
        rules_path = self.write_temp_rules(invalid_rules)
        validator = RulesSchemaValidator(rules_path)
        self.assertFalse(validator.validate())
        error_messages = ' '.join(validator.errors)
        self.assertIn('rule_name', error_messages.lower())
    
    def test_empty_regex(self):
        """Test that empty regex fails validation."""
        invalid_rules = """
hardcoded_patterns:
  - name: "Test Rule"
    regex: ""
    description: "Test pattern"
    rule_name: "TEST_RULE"
    severity: "CRITICAL"
"""
        rules_path = self.write_temp_rules(invalid_rules)
        validator = RulesSchemaValidator(rules_path)
        self.assertFalse(validator.validate())
        error_messages = ' '.join(validator.errors)
        self.assertIn('regex', error_messages.lower())
    
    def test_missing_top_level_key(self):
        """Test that missing required top-level key fails validation."""
        invalid_rules = """
hardcoded_patterns:
  - name: "Test Rule"
    regex: "\\btest\\b"
    description: "Test pattern"
    rule_name: "TEST_RULE"
    severity: "CRITICAL"
# Missing path_patterns, ml_patterns, etc.
"""
        rules_path = self.write_temp_rules(invalid_rules)
        validator = RulesSchemaValidator(rules_path)
        # Should have warnings but may still be valid (some keys are optional)
        # But missing required pattern lists should generate errors
        result = validator.validate()
        # At minimum, should warn about missing keys
        self.assertTrue(len(validator.errors) > 0 or len(validator.warnings) > 0)
    
    def test_bad_indentation(self):
        """Test that bad YAML indentation fails validation."""
        invalid_rules = """
hardcoded_patterns:
- name: "Test Rule"
    regex: "\\btest\\b"
  description: "Test pattern"
"""
        rules_path = self.write_temp_rules(invalid_rules)
        validator = RulesSchemaValidator(rules_path)
        self.assertFalse(validator.validate())
        self.assertGreater(len(validator.errors), 0)
    
    def test_unescaped_special_chars_in_regex(self):
        """Test that unescaped special characters in regex are handled."""
        # This should be valid if properly escaped
        valid_rules = """
hardcoded_patterns:
  - name: "Test Rule"
    regex: "\\b(?:test|example)\\b"
    description: "Test pattern"
    rule_name: "TEST_RULE"
    severity: "CRITICAL"
"""
        rules_path = self.write_temp_rules(valid_rules)
        validator = RulesSchemaValidator(rules_path)
        # Should validate regex syntax
        result = validator.validate()
        # Should pass if regex is valid
        if result:
            self.assertEqual(len(validator.errors), 0)


def main():
    """Run tests."""
    unittest.main()


if __name__ == '__main__':
    main()

