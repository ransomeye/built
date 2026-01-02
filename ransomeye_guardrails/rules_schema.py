# Path and File Name : /home/ransomeye/rebuild/ransomeye_guardrails/rules_schema.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Validates rules.yaml structure and regex patterns - fails-closed on any validation error

"""
Rules Schema Validator: Validates rules.yaml structure and regex patterns.
Fails-closed on any validation error.
"""

import sys
import re
from pathlib import Path
from typing import Dict, List, Any, Optional
import yaml

from .fail_closed import fail_closed


class RulesSchemaValidator:
    """Validates rules.yaml structure and content."""
    
    REQUIRED_PATTERN_FIELDS = ['name', 'regex', 'description', 'rule_name', 'severity']
    VALID_SEVERITIES = ['CRITICAL', 'HIGH', 'MEDIUM', 'LOW']
    REQUIRED_TOP_LEVEL_KEYS = [
        'hardcoded_patterns',
        'path_patterns',
        'ml_patterns',
        'crypto_patterns',
        'header_patterns',
        'allowed_exceptions',
        'scan_config'
    ]
    
    def __init__(self, rules_path: str):
        self.rules_path = Path(rules_path)
        self.errors: List[str] = []
        self.warnings: List[str] = []
    
    def validate_file_exists(self) -> bool:
        """Validate rules.yaml file exists."""
        if not self.rules_path.exists():
            self.errors.append(f"Rules file not found: {self.rules_path}")
            return False
        return True
    
    def validate_yaml_syntax(self) -> Optional[Dict]:
        """Validate YAML syntax is correct."""
        try:
            with open(self.rules_path, 'r') as f:
                content = f.read()
            
            # Try to parse YAML
            try:
                rules = yaml.safe_load(content)
            except yaml.YAMLError as e:
                self.errors.append(f"YAML syntax error: {str(e)}")
                return None
            
            if rules is None:
                self.errors.append("YAML file is empty or contains only comments")
                return None
            
            return rules
        except Exception as e:
            self.errors.append(f"Error reading rules file: {str(e)}")
            return None
    
    def validate_regex_pattern(self, pattern: str, rule_name: str) -> bool:
        """Validate regex pattern is syntactically correct."""
        if not pattern or not isinstance(pattern, str):
            self.errors.append(f"Rule '{rule_name}': regex pattern is empty or not a string")
            return False
        
        try:
            # Try to compile the regex
            re.compile(pattern)
            return True
        except re.error as e:
            self.errors.append(f"Rule '{rule_name}': Invalid regex pattern '{pattern[:50]}...': {str(e)}")
            return False
    
    def validate_pattern_rule(self, rule: Dict, rule_type: str, index: int) -> bool:
        """Validate a single pattern rule."""
        rule_name = rule.get('rule_name', f'{rule_type}[{index}]')
        valid = True
        
        # Check required fields
        for field in self.REQUIRED_PATTERN_FIELDS:
            if field not in rule:
                self.errors.append(f"Rule '{rule_name}': Missing required field '{field}'")
                valid = False
        
        # Validate regex pattern
        if 'regex' in rule:
            if not self.validate_regex_pattern(rule['regex'], rule_name):
                valid = False
        else:
            # Fallback: check for 'pattern' field (legacy)
            if 'pattern' in rule:
                self.warnings.append(f"Rule '{rule_name}': Using legacy 'pattern' field, should use 'regex'")
                if not self.validate_regex_pattern(rule['pattern'], rule_name):
                    valid = False
            else:
                self.errors.append(f"Rule '{rule_name}': Missing 'regex' field")
                valid = False
        
        # Validate severity
        if 'severity' in rule:
            if rule['severity'] not in self.VALID_SEVERITIES:
                self.errors.append(f"Rule '{rule_name}': Invalid severity '{rule['severity']}', must be one of {self.VALID_SEVERITIES}")
                valid = False
        
        # Validate name is non-empty
        if 'name' in rule:
            if not rule['name'] or not isinstance(rule['name'], str):
                self.errors.append(f"Rule '{rule_name}': 'name' field must be a non-empty string")
                valid = False
        
        # Validate rule_name format
        if 'rule_name' in rule:
            if not re.match(r'^[A-Z_][A-Z0-9_]*$', rule['rule_name']):
                self.errors.append(f"Rule '{rule_name}': 'rule_name' must match pattern [A-Z_][A-Z0-9_]*")
                valid = False
        
        return valid
    
    def validate_pattern_list(self, rules: Dict, key: str) -> bool:
        """Validate a list of pattern rules."""
        if key not in rules:
            self.errors.append(f"Missing required top-level key: '{key}'")
            return False
        
        pattern_list = rules[key]
        if not isinstance(pattern_list, list):
            self.errors.append(f"'{key}' must be a list")
            return False
        
        if len(pattern_list) == 0:
            self.warnings.append(f"'{key}' is empty (no patterns defined)")
        
        valid = True
        for i, rule in enumerate(pattern_list):
            if not isinstance(rule, dict):
                self.errors.append(f"'{key}[{i}]': Must be a dictionary")
                valid = False
                continue
            
            if not self.validate_pattern_rule(rule, key, i):
                valid = False
        
        return valid
    
    def validate_allowed_exceptions(self, rules: Dict) -> bool:
        """Validate allowed_exceptions structure."""
        if 'allowed_exceptions' not in rules:
            self.warnings.append("'allowed_exceptions' not defined (no exceptions will be allowed)")
            return True
        
        exceptions = rules['allowed_exceptions']
        if not isinstance(exceptions, list):
            self.errors.append("'allowed_exceptions' must be a list")
            return False
        
        valid = True
        for i, exception in enumerate(exceptions):
            if not isinstance(exception, dict):
                self.errors.append(f"'allowed_exceptions[{i}]': Must be a dictionary")
                valid = False
                continue
            
            if 'file_patterns' not in exception:
                self.errors.append(f"'allowed_exceptions[{i}]': Missing 'file_patterns' field")
                valid = False
            
            if 'rules' not in exception:
                self.errors.append(f"'allowed_exceptions[{i}]': Missing 'rules' field")
                valid = False
        
        return valid
    
    def validate_scan_config(self, rules: Dict) -> bool:
        """Validate scan_config structure."""
        if 'scan_config' not in rules:
            self.warnings.append("'scan_config' not defined (using defaults)")
            return True
        
        config = rules['scan_config']
        if not isinstance(config, dict):
            self.errors.append("'scan_config' must be a dictionary")
            return False
        
        valid = True
        
        if 'file_extensions' in config:
            if not isinstance(config['file_extensions'], list):
                self.errors.append("'scan_config.file_extensions' must be a list")
                valid = False
        
        if 'exclude_dirs' in config:
            if not isinstance(config['exclude_dirs'], list):
                self.errors.append("'scan_config.exclude_dirs' must be a list")
                valid = False
        
        return valid
    
    def validate(self) -> bool:
        """
        Validate entire rules.yaml file.
        
        Returns:
            True if valid, False otherwise
        """
        self.errors = []
        self.warnings = []
        
        # Check file exists
        if not self.validate_file_exists():
            return False
        
        # Validate YAML syntax
        rules = self.validate_yaml_syntax()
        if rules is None:
            return False
        
        # Validate top-level structure
        for key in self.REQUIRED_TOP_LEVEL_KEYS:
            if key not in rules:
                if key in ['allowed_exceptions', 'scan_config']:
                    # These are optional
                    continue
                else:
                    self.errors.append(f"Missing required top-level key: '{key}'")
        
        # Validate pattern lists
        pattern_keys = ['hardcoded_patterns', 'path_patterns', 'ml_patterns', 'crypto_patterns', 'header_patterns']
        for key in pattern_keys:
            if key in rules:
                self.validate_pattern_list(rules, key)
        
        # Validate allowed_exceptions
        self.validate_allowed_exceptions(rules)
        
        # Validate scan_config
        self.validate_scan_config(rules)
        
        return len(self.errors) == 0
    
    def report_errors(self) -> None:
        """Report validation errors and fail-closed if any found."""
        if not self.errors:
            if self.warnings:
                print("Rules validation warnings:", file=sys.stderr)
                for warning in self.warnings:
                    print(f"  WARNING: {warning}", file=sys.stderr)
            return
        
        print("="*80, file=sys.stderr)
        print("RULES.YAML VALIDATION FAILED", file=sys.stderr)
        print("="*80, file=sys.stderr)
        print(f"File: {self.rules_path}", file=sys.stderr)
        print(f"Errors: {len(self.errors)}", file=sys.stderr)
        print("", file=sys.stderr)
        
        for i, error in enumerate(self.errors, 1):
            print(f"{i}. {error}", file=sys.stderr)
        
        if self.warnings:
            print("", file=sys.stderr)
            print("Warnings:", file=sys.stderr)
            for warning in self.warnings:
                print(f"  WARNING: {warning}", file=sys.stderr)
        
        print("="*80, file=sys.stderr)
        
        fail_closed(
            "RULES_YAML_VALIDATION_FAILED",
            f"Found {len(self.errors)} validation error(s) in rules.yaml. Rules file must be valid before scanning can proceed.",
            file_path=str(self.rules_path)
        )


def validate_rules_file(rules_path: str) -> bool:
    """
    Validate rules.yaml file.
    
    Args:
        rules_path: Path to rules.yaml file
    
    Returns:
        True if valid, False otherwise (and exits on failure)
    """
    validator = RulesSchemaValidator(rules_path)
    is_valid = validator.validate()
    
    if not is_valid:
        validator.report_errors()
    
    return is_valid


def main():
    """CLI entry point for rules schema validator."""
    import argparse
    
    parser = argparse.ArgumentParser(description='RansomEye Rules Schema Validator')
    parser.add_argument('--rules', default='/home/ransomeye/rebuild/ransomeye_guardrails/rules.yaml',
                       help='Path to rules.yaml file')
    
    args = parser.parse_args()
    
    validator = RulesSchemaValidator(args.rules)
    is_valid = validator.validate()
    
    if is_valid:
        print("✓ Rules validation passed.")
        if validator.warnings:
            print("\nWarnings:")
            for warning in validator.warnings:
                print(f"  {warning}")
    else:
        validator.report_errors()


if __name__ == '__main__':
    main()

