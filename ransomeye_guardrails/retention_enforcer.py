# Path and File Name : /home/ransomeye/rebuild/ransomeye_guardrails/retention_enforcer.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Enforces data retention policies and validates retention configuration

"""
Retention enforcer: validates retention policy configuration and enforces retention rules.
Ensures retention.txt is valid and policies are enforceable.
"""

import os
import re
from pathlib import Path
from typing import Dict, Optional, List
from datetime import datetime, timedelta

from .fail_closed import fail_closed


class RetentionEnforcer:
    """Enforces data retention policies."""
    
    REQUIRED_RETENTION_VARS = [
        'TELEMETRY_RETENTION_MONTHS',
        'FORENSIC_RETENTION_DAYS',
        'DISK_MAX_USAGE_PERCENT'
    ]
    
    DEFAULT_VALUES = {
        'TELEMETRY_RETENTION_MONTHS': 6,
        'FORENSIC_RETENTION_DAYS': 10,
        'DISK_MAX_USAGE_PERCENT': 80
    }
    
    MIN_AI_RETENTION_YEARS = 2
    
    def __init__(self, project_root: str = "/home/ransomeye/rebuild", retention_config_path: Optional[str] = None):
        self.project_root = Path(project_root).resolve()
        if retention_config_path:
            self.retention_config_path = Path(retention_config_path)
        else:
            self.retention_config_path = self.project_root / "config" / "retention.txt"
        self.violations: List[Dict] = []
        self.retention_config: Dict = {}
    
    def _parse_retention_config(self) -> Dict:
        """Parse retention.txt configuration file."""
        config = {}
        
        if not self.retention_config_path.exists():
            # Use defaults if file doesn't exist
            return self.DEFAULT_VALUES.copy()
        
        try:
            with open(self.retention_config_path, 'r') as f:
                for line in f:
                    line = line.strip()
                    if not line or line.startswith('#'):
                        continue
                    
                    # Parse KEY=VALUE format
                    if '=' in line:
                        key, value = line.split('=', 1)
                        key = key.strip()
                        value = value.strip()
                        
                        # Validate and convert
                        if key in self.REQUIRED_RETENTION_VARS:
                            try:
                                if 'PERCENT' in key:
                                    config[key] = int(value)
                                    if not (0 <= config[key] <= 100):
                                        self.violations.append({
                                            'rule_name': 'RETENTION_CONFIG_INVALID',
                                            'description': f'{key} must be between 0 and 100, got {value}',
                                            'file_path': str(self.retention_config_path)
                                        })
                                elif 'MONTHS' in key:
                                    config[key] = int(value)
                                    if config[key] < 0:
                                        self.violations.append({
                                            'rule_name': 'RETENTION_CONFIG_INVALID',
                                            'description': f'{key} must be non-negative, got {value}',
                                            'file_path': str(self.retention_config_path)
                                        })
                                elif 'DAYS' in key:
                                    config[key] = int(value)
                                    if config[key] < 0:
                                        self.violations.append({
                                            'rule_name': 'RETENTION_CONFIG_INVALID',
                                            'description': f'{key} must be non-negative, got {value}',
                                            'file_path': str(self.retention_config_path)
                                        })
                                else:
                                    config[key] = value
                            except ValueError:
                                self.violations.append({
                                    'rule_name': 'RETENTION_CONFIG_INVALID',
                                    'description': f'Invalid value for {key}: {value} (must be numeric)',
                                    'file_path': str(self.retention_config_path)
                                })
        except Exception as e:
            self.violations.append({
                'rule_name': 'RETENTION_CONFIG_ERROR',
                'description': f'Error reading retention.txt: {str(e)}',
                'file_path': str(self.retention_config_path)
            })
            return self.DEFAULT_VALUES.copy()
        
        # Ensure all required vars are present
        for var in self.REQUIRED_RETENTION_VARS:
            if var not in config:
                config[var] = self.DEFAULT_VALUES[var]
        
        return config
    
    def _validate_retention_config(self) -> bool:
        """Validate retention configuration."""
        self.retention_config = self._parse_retention_config()
        
        # Check for violations
        if self.violations:
            return False
        
        # Validate values are reasonable
        if self.retention_config.get('TELEMETRY_RETENTION_MONTHS', 0) > 84:  # 7 years max
            self.violations.append({
                'rule_name': 'RETENTION_CONFIG_INVALID',
                'description': 'TELEMETRY_RETENTION_MONTHS exceeds 7-year maximum (84 months)',
                'file_path': str(self.retention_config_path)
            })
            return False
        
        if self.retention_config.get('DISK_MAX_USAGE_PERCENT', 0) < 50:
            self.violations.append({
                'rule_name': 'RETENTION_CONFIG_INVALID',
                'description': 'DISK_MAX_USAGE_PERCENT should be at least 50%',
                'file_path': str(self.retention_config_path)
            })
            return False
        
        return True
    
    def _check_retention_enforcement_code(self, directory: Path) -> List[Dict]:
        """Check that retention enforcement code exists."""
        violations = []
        
        # Check for retention enforcement modules
        required_modules = [
            'ransomeye_retention/retention_parser.py',
            'ransomeye_retention/disk_monitor.py',
            'ransomeye_retention/telemetry_retention.py',
            'ransomeye_retention/forensic_retention.py',
            'ransomeye_retention/ai_retention_guard.py',
        ]
        
        for module_rel_path in required_modules:
            module_path = self.project_root / module_rel_path
            if not module_path.exists():
                violations.append({
                    'rule_name': 'RETENTION_ENFORCEMENT_MISSING',
                    'description': f'Required retention enforcement module missing: {module_rel_path}',
                    'file_path': str(module_path)
                })
        
        return violations
    
    def check_configuration(self) -> List[Dict]:
        """Check retention configuration validity."""
        if not self._validate_retention_config():
            return self.violations
        
        # Check enforcement code exists
        enforcement_violations = self._check_retention_enforcement_code(self.project_root)
        self.violations.extend(enforcement_violations)
        
        return self.violations
    
    def get_retention_config(self) -> Dict:
        """Get parsed retention configuration."""
        if not self.retention_config:
            self.retention_config = self._parse_retention_config()
        return self.retention_config.copy()
    
    def report_violations(self) -> None:
        """Report violations and fail-closed if any found."""
        if not self.violations:
            print("✓ Retention configuration is valid.")
            print(f"  TELEMETRY_RETENTION_MONTHS: {self.retention_config.get('TELEMETRY_RETENTION_MONTHS')}")
            print(f"  FORENSIC_RETENTION_DAYS: {self.retention_config.get('FORENSIC_RETENTION_DAYS')}")
            print(f"  DISK_MAX_USAGE_PERCENT: {self.retention_config.get('DISK_MAX_USAGE_PERCENT')}%")
            return
        
        print(f"\n{'='*80}")
        print(f"RETENTION ENFORCEMENT VIOLATIONS: {len(self.violations)}")
        print(f"{'='*80}\n")
        
        for violation in self.violations:
            print(f"Rule: {violation['rule_name']}")
            print(f"  Description: {violation['description']}")
            if 'file_path' in violation:
                print(f"  File: {violation['file_path']}")
            print()
        
        # Fail-closed
        fail_closed(
            "RETENTION_ENFORCEMENT_VIOLATION",
            f"Found {len(self.violations)} retention enforcement violation(s). Retention policy must be valid and enforceable. Build cannot proceed.",
            file_path=None
        )


def main():
    """CLI entry point for retention enforcer."""
    import argparse
    
    parser = argparse.ArgumentParser(description='RansomEye Retention Enforcer')
    parser.add_argument('--project-root', default='/home/ransomeye/rebuild',
                       help='Project root directory')
    parser.add_argument('--retention-config', default=None,
                       help='Path to retention.txt (default: config/retention.txt)')
    parser.add_argument('--dry-run', action='store_true',
                       help='Dry run mode (validate only, no enforcement)')
    
    args = parser.parse_args()
    
    enforcer = RetentionEnforcer(args.project_root, args.retention_config)
    violations = enforcer.check_configuration()
    
    enforcer.report_violations()


if __name__ == '__main__':
    main()

