# Path: /home/ransomeye/rebuild/ransomeye_governance/tooling/secret_validator.py
# Author: RansomEye Core Team
# Purpose: Scans codebase for hardcoded secrets, passwords, API keys, and credentials - blocks on detection

"""
Secret Validator: Scans for hardcoded secrets and credentials.

This tool detects:
- Hardcoded passwords
- API keys
- Tokens
- Credentials
- Private keys
- And other sensitive information
"""

import os
import sys
import re
from pathlib import Path
from typing import List, Dict, Optional

# Secret patterns to detect
SECRET_PATTERNS = [
    # Passwords
    (r'password\s*=\s*["\']([^"\']+)["\']', 'Hardcoded password'),
    (r'passwd\s*=\s*["\']([^"\']+)["\']', 'Hardcoded password'),
    (r'pwd\s*=\s*["\']([^"\']+)["\']', 'Hardcoded password'),
    
    # API Keys
    (r'api[_-]?key\s*=\s*["\']([^"\']+)["\']', 'Hardcoded API key'),
    (r'apikey\s*=\s*["\']([^"\']+)["\']', 'Hardcoded API key'),
    
    # Tokens
    (r'token\s*=\s*["\']([^"\']+)["\']', 'Hardcoded token'),
    (r'access[_-]?token\s*=\s*["\']([^"\']+)["\']', 'Hardcoded access token'),
    (r'secret[_-]?key\s*=\s*["\']([^"\']+)["\']', 'Hardcoded secret key'),
    
    # Credentials
    (r'credential\s*=\s*["\']([^"\']+)["\']', 'Hardcoded credential'),
    (r'auth[_-]?token\s*=\s*["\']([^"\']+)["\']', 'Hardcoded auth token'),
    
    # Private keys (partial detection)
    (r'-----BEGIN\s+(RSA\s+)?PRIVATE\s+KEY-----', 'Hardcoded private key'),
    
    # AWS/Azure/GCP keys
    (r'aws[_-]?access[_-]?key[_-]?id\s*=\s*["\']([^"\']+)["\']', 'Hardcoded AWS access key'),
    (r'aws[_-]?secret[_-]?access[_-]?key\s*=\s*["\']([^"\']+)["\']', 'Hardcoded AWS secret key'),
]

# Allowed patterns (false positives to ignore)
ALLOWED_PATTERNS = [
    r'password\s*=\s*["\']\s*["\']',  # Empty password
    r'password\s*=\s*os\.environ\.get',  # ENV variable
    r'password\s*=\s*None',  # None value
    r'#.*password',  # Comments
    r'password\s*=\s*""',  # Empty string
    r'password\s*=\s*\$\{',  # Template variable
]

# File extensions to scan
SCAN_EXTENSIONS = {
    '.py', '.rs', '.c', '.cpp', '.sh', '.yaml', '.yml', '.json', '.toml',
    '.js', '.ts', '.jsx', '.tsx', '.java', '.go'
}

# Excluded patterns
EXCLUDED_PATTERNS = [
    '__pycache__',
    '.git',
    'node_modules',
    'venv',
    '.venv',
    'target',
    'dist',
    'build',
    '.pytest_cache',
    '.mypy_cache',
    'Cargo.lock',
    'package-lock.json',
    # Exclude test files that contain test data (not real secrets)
    '/tests/',
    'test_',
    '_test.py',
]


class SecretValidator:
    """Validates that no secrets are hardcoded."""
    
    def __init__(self, project_root: str = "/home/ransomeye/rebuild"):
        self.project_root = Path(project_root).resolve()
        self.violations: List[Dict] = []
    
    def _should_check_file(self, file_path: Path) -> bool:
        """Determine if file should be checked."""
        # Check extension
        if file_path.suffix not in SCAN_EXTENSIONS:
            return False
        
        # Check excluded patterns
        rel_path = str(file_path.relative_to(self.project_root))
        file_name = file_path.name
        
        for pattern in EXCLUDED_PATTERNS:
            if pattern in rel_path or pattern in file_name:
                return False
        
        # Exclude test files explicitly
        if '/tests/' in rel_path or file_name.startswith('test_') or file_name.endswith('_test.py'):
            return False
        
        return True
    
    def _is_allowed(self, line: str) -> bool:
        """Check if line matches allowed patterns (false positives)."""
        for pattern in ALLOWED_PATTERNS:
            if re.search(pattern, line, re.IGNORECASE):
                return True
        return False
    
    def _scan_file(self, file_path: Path) -> List[Dict]:
        """Scan a single file for secrets."""
        violations = []
        
        try:
            with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                lines = f.readlines()
        except Exception:
            return violations
        
        for line_num, line in enumerate(lines, 1):
            # Skip allowed patterns
            if self._is_allowed(line):
                continue
            
            # Check each secret pattern
            for pattern, description in SECRET_PATTERNS:
                match = re.search(pattern, line, re.IGNORECASE)
                if match:
                    # Extract the secret value (if captured)
                    secret_value = match.group(1) if match.groups() else "<detected>"
                    
                    # Mask the secret in output
                    if len(secret_value) > 10:
                        masked = secret_value[:4] + "..." + secret_value[-4:]
                    else:
                        masked = "***"
                    
                    violations.append({
                        'file_path': str(file_path),
                        'line_number': line_num,
                        'line_content': line.strip(),
                        'description': description,
                        'masked_value': masked
                    })
        
        return violations
    
    def validate(self) -> bool:
        """Run full secret validation."""
        print("=" * 80)
        print("RansomEye Secret Validator")
        print("=" * 80)
        print()
        
        violations = []
        
        for root, dirs, files in os.walk(self.project_root):
            # Filter excluded directories (including tests)
            dirs[:] = [d for d in dirs if not any(pattern in d for pattern in EXCLUDED_PATTERNS) and d != 'tests']
            
            # Skip entire tests directory
            if 'tests' in root:
                continue
            
            for file in files:
                file_path = Path(root) / file
                # Skip test files explicitly
                if 'test' in file.lower() and (file.startswith('test_') or file.endswith('_test.py')):
                    continue
                if self._should_check_file(file_path):
                    file_violations = self._scan_file(file_path)
                    violations.extend(file_violations)
        
        self.violations = violations
        
        if violations:
            print(f"\n{'='*80}")
            print(f"SECRET VIOLATIONS DETECTED: {len(violations)}")
            print(f"{'='*80}\n")
            
            for violation in violations:
                print(f"File: {violation['file_path']}")
                print(f"  Line {violation['line_number']}: {violation['description']}")
                print(f"  Content: {violation['line_content']}")
                print(f"  Value: {violation['masked_value']}")
                print()
            
            return False
        else:
            print("\n✓ No hardcoded secrets detected.")
            return True
    
    def fail_closed(self, message: str):
        """Fail-closed: exit with error."""
        print(f"\n{'='*80}")
        print("SECRET VALIDATION FAILED")
        print(f"{'='*80}")
        print(message)
        print("\nBuild blocked due to hardcoded secret detection.")
        sys.exit(1)


def main():
    """CLI entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description='RansomEye Secret Validator')
    parser.add_argument('--project-root', default='/home/ransomeye/rebuild',
                       help='Project root directory')
    
    args = parser.parse_args()
    
    validator = SecretValidator(args.project_root)
    is_valid = validator.validate()
    
    if not is_valid:
        validator.fail_closed(f"Found {len(validator.violations)} hardcoded secret(s).")
    
    print("\n✓ Secret validation passed.")
    sys.exit(0)


if __name__ == '__main__':
    main()

