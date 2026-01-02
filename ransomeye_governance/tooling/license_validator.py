# Path: /home/ransomeye/rebuild/ransomeye_governance/tooling/license_validator.py
# Author: RansomEye Core Team
# Purpose: Validates all dependencies for license compliance - blocks GPL/AGPL/SSPL and enforces allowed licenses only

"""
License Validator: Enforces RansomEye license policy.

This tool scans all dependencies (Rust crates, Python packages, etc.) and blocks
any banned licenses (GPL, AGPL, SSPL) while allowing only permissive licenses.
"""

import os
import sys
import json
import subprocess
from pathlib import Path
from typing import List, Dict, Optional, Set
import re

# Allowed licenses (permissive only)
ALLOWED_LICENSES = {
    'MIT', 'MIT License', 'MIT/X11',
    'BSD-2-Clause', 'BSD-3-Clause', 'BSD',
    'Apache-2.0', 'Apache 2.0', 'Apache License 2.0',
    'PSF', 'Python Software Foundation License',
    'CC0', 'CC0-1.0', 'Public Domain',
}

# Conditionally allowed (with restrictions)
CONDITIONALLY_ALLOWED = {
    'LGPL-2.1', 'LGPL-3.0', 'LGPL',
}

# BANNED licenses (absolute prohibition)
BANNED_LICENSES = {
    'GPL', 'GPL-2.0', 'GPL-3.0', 'GPLv2', 'GPLv3',
    'AGPL', 'AGPL-3.0', 'AGPLv3',
    'SSPL', 'Server Side Public License',
    'Elastic License', 'Elastic License 2.0',
    'MongoDB SSPL',
}

# Unknown/custom licenses are also banned
UNKNOWN_LICENSE = 'UNKNOWN'


class LicenseValidator:
    """Validates licenses for all dependencies."""
    
    def __init__(self, project_root: str = "/home/ransomeye/rebuild"):
        self.project_root = Path(project_root).resolve()
        self.violations: List[Dict] = []
        self.allowed_deps: List[Dict] = []
        self.banned_deps: List[Dict] = []
    
    def scan_rust_dependencies(self) -> List[Dict]:
        """Scan Rust dependencies from Cargo.toml and Cargo.lock."""
        violations = []
        
        cargo_toml = self.project_root / "Cargo.toml"
        cargo_lock = self.project_root / "Cargo.lock"
        
        if not cargo_toml.exists():
            return violations
        
        try:
            # Parse Cargo.toml for direct dependencies
            with open(cargo_toml, 'r') as f:
                content = f.read()
            
            # Extract dependency names (simplified - real implementation would use toml parser)
            dep_pattern = r'\[dependencies\.([^\]]+)\]'
            deps = re.findall(dep_pattern, content)
            
            for dep_name in deps:
                # Check license (would need to query crates.io or parse Cargo.toml properly)
                # For now, we'll flag as needing manual check
                violations.append({
                    'type': 'rust',
                    'name': dep_name,
                    'license': 'NEEDS_CHECK',
                    'status': 'pending'
                })
        
        except Exception as e:
            violations.append({
                'type': 'rust',
                'name': 'SCAN_ERROR',
                'error': str(e),
                'status': 'error'
            })
        
        return violations
    
    def scan_python_dependencies(self) -> List[Dict]:
        """Scan Python dependencies from requirements.txt."""
        violations = []
        
        requirements_txt = self.project_root / "requirements.txt"
        
        if not requirements_txt.exists():
            return violations
        
        try:
            with open(requirements_txt, 'r') as f:
                lines = f.readlines()
            
            for line in lines:
                line = line.strip()
                if not line or line.startswith('#'):
                    continue
                
                # Extract package name
                package_name = line.split('==')[0].split('>=')[0].split('<=')[0].split('@')[0].strip()
                
                # Try to get license info (would need pip-licenses or similar)
                # For now, flag as needing check
                violations.append({
                    'type': 'python',
                    'name': package_name,
                    'license': 'NEEDS_CHECK',
                    'status': 'pending'
                })
        
        except Exception as e:
            violations.append({
                'type': 'python',
                'name': 'SCAN_ERROR',
                'error': str(e),
                'status': 'error'
            })
        
        return violations
    
    def check_license(self, license_str: str) -> tuple[bool, str]:
        """
        Check if license is allowed, banned, or conditionally allowed.
        Returns (is_allowed, reason)
        """
        if not license_str or license_str.upper() == UNKNOWN_LICENSE:
            return False, "Unknown license - BANNED"
        
        license_upper = license_str.upper()
        
        # Check banned licenses
        for banned in BANNED_LICENSES:
            if banned.upper() in license_upper:
                return False, f"Banned license detected: {license_str}"
        
        # Check allowed licenses
        for allowed in ALLOWED_LICENSES:
            if allowed.upper() in license_upper:
                return True, f"Allowed license: {license_str}"
        
        # Check conditionally allowed
        for cond in CONDITIONALLY_ALLOWED:
            if cond.upper() in license_upper:
                return False, f"Conditionally allowed license requires manual review: {license_str}"
        
        # Unknown/custom license
        return False, f"Unknown or custom license - BANNED: {license_str}"
    
    def validate(self) -> bool:
        """Run full license validation."""
        print("=" * 80)
        print("RansomEye License Validator")
        print("=" * 80)
        print()
        
        # Scan dependencies
        print("Scanning Rust dependencies...")
        rust_violations = self.scan_rust_dependencies()
        
        print("Scanning Python dependencies...")
        python_violations = self.scan_python_dependencies()
        
        all_violations = rust_violations + python_violations
        
        # Check each violation
        for violation in all_violations:
            if violation.get('license') == 'NEEDS_CHECK':
                # In real implementation, would query package registry
                # For now, flag as needing manual review
                self.violations.append({
                    'dependency': violation['name'],
                    'type': violation['type'],
                    'issue': 'License needs manual verification',
                    'status': 'REVIEW_REQUIRED'
                })
        
        # Report results
        if self.violations:
            print(f"\n{'='*80}")
            print(f"LICENSE VIOLATIONS DETECTED: {len(self.violations)}")
            print(f"{'='*80}\n")
            
            for violation in self.violations:
                print(f"Dependency: {violation['dependency']}")
                print(f"  Type: {violation['type']}")
                print(f"  Issue: {violation['issue']}")
                print(f"  Status: {violation['status']}")
                print()
            
            return False
        else:
            print("\n✓ All dependencies comply with license policy.")
            return True
    
    def fail_closed(self, message: str):
        """Fail-closed: exit with error."""
        print(f"\n{'='*80}")
        print("LICENSE VALIDATION FAILED")
        print(f"{'='*80}")
        print(message)
        print("\nBuild blocked due to license policy violation.")
        sys.exit(1)


def main():
    """CLI entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description='RansomEye License Validator')
    parser.add_argument('--project-root', default='/home/ransomeye/rebuild',
                       help='Project root directory')
    parser.add_argument('--fail-on-review', action='store_true',
                       help='Fail if manual review required')
    
    args = parser.parse_args()
    
    validator = LicenseValidator(args.project_root)
    is_valid = validator.validate()
    
    if not is_valid:
        if args.fail_on_review:
            validator.fail_closed("License validation failed - manual review required.")
        else:
            print("\n⚠ License validation requires manual review.")
            print("Use --fail-on-review to fail on review requirements.")
            sys.exit(1)
    
    print("\n✓ License validation passed.")
    sys.exit(0)


if __name__ == '__main__':
    main()

