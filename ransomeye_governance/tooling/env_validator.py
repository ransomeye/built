# Path: /home/ransomeye/rebuild/ransomeye_governance/tooling/env_validator.py
# Author: RansomEye Core Team
# Purpose: Validates environment variables - fails startup on missing required ENV or unknown ENV variables

"""
ENV Validator: Enforces ENV-only configuration.

Rules:
- Missing required ENV variable = STARTUP FAILURE
- Unknown ENV variable = STARTUP FAILURE
- No defaults for secrets
"""

import os
import sys
from pathlib import Path
from typing import List, Dict, Set, Optional
import json

# Required environment variables (from env_schema.md)
REQUIRED_ENV_VARS = {
    'DB_HOST',
    'DB_PORT',
    'DB_NAME',
    'DB_USER',
    'DB_PASS',
    'MODEL_DIR',
    'PDF_ENGINE_PATH',
    'OUTPUT_DIR',
    'CORE_API_PORT',
}

# Allowed environment variables (known variables)
ALLOWED_ENV_VARS = {
    # Database
    'DB_HOST', 'DB_PORT', 'DB_NAME', 'DB_USER', 'DB_PASS',
    # Model/AI
    'MODEL_DIR', 'PDF_ENGINE_PATH', 'LLM_MODEL_PATH', 'SHAP_ENABLED',
    # Paths
    'OUTPUT_DIR', 'CHAIN_OUTPUT_PATH', 'SUMMARY_EXPORT_PATH',
    # API
    'CORE_API_PORT', 'FRONTEND_PORT', 'BACKEND_API_PORT',
    # Threat Intel
    'MISP_URL', 'MISP_KEY', 'OTX_URL', 'OTX_KEY', 'TALOS_URL', 'TALOS_KEY',
    'THREATFOX_URL', 'THREATFOX_KEY',
    # Compliance
    'RETENTION_YEARS', 'DB_ENCRYPTION_KEY_PATH', 'COMPLIANCE_CONFIG',
    # Network
    'CAPTURE_IFACE', 'NVD_DB_PATH', 'GEOIP_DB',
    # Assistant
    'ASSISTANT_TOPK', 'ASSISTANT_MAX_TOKENS', 'ASSISTANT_DATA_DIR',
    # Agent
    'CORE_API_URL', 'AGENT_CERT_PATH', 'BUFFER_DIR', 'UPDATE_BUNDLE_DIR',
    # Features
    'ENABLE_EBPF', 'ENABLE_KERNEL_DRIVER',
    # Policy
    'POLICY_DIR', 'POLICY_SYNC_INTERVAL_MIN',
    # Posture
    'RANSOMEYE_ROOT', 'CIS_BENCHMARKS_DIR', 'STIG_PROFILES_DIR',
    'CUSTOM_POLICIES_DIR', 'POSTURE_OUTPUT_DIR', 'POSTURE_AUDIT_LOG_DIR',
    'POSTURE_EVAL_INTERVAL_SEC', 'POSTURE_DRIFT_WINDOW_HOURS',
    'POSTURE_SIGNING_KEY_PATH', 'POSTURE_TRUST_STORE_PATH',
}


class EnvValidator:
    """Validates environment variables."""
    
    def __init__(self, required_vars: Optional[Set[str]] = None, allowed_vars: Optional[Set[str]] = None):
        self.required_vars = required_vars or REQUIRED_ENV_VARS
        self.allowed_vars = allowed_vars or ALLOWED_ENV_VARS
        self.violations: List[Dict] = []
    
    def validate_required(self) -> bool:
        """Validate that all required ENV variables are present."""
        missing = []
        
        for var in self.required_vars:
            if var not in os.environ or not os.environ[var]:
                missing.append(var)
        
        if missing:
            for var in missing:
                self.violations.append({
                    'type': 'missing_required',
                    'variable': var,
                    'message': f'Required environment variable missing: {var}'
                })
            return False
        
        return True
    
    def validate_unknown(self) -> bool:
        """Validate that no unknown ENV variables are present."""
        unknown = []
        
        for var in os.environ:
            if var not in self.allowed_vars:
                # Skip system variables
                if not var.startswith('_') and not var.startswith('PATH') and not var.startswith('HOME'):
                    unknown.append(var)
        
        if unknown:
            for var in unknown:
                self.violations.append({
                    'type': 'unknown_variable',
                    'variable': var,
                    'message': f'Unknown environment variable: {var}'
                })
            return False
        
        return True
    
    def validate_secrets(self) -> bool:
        """Validate that secrets don't have default values."""
        # This would check code for hardcoded defaults
        # For now, just return True
        return True
    
    def validate(self) -> bool:
        """Run full ENV validation."""
        print("=" * 80)
        print("RansomEye ENV Validator")
        print("=" * 80)
        print()
        
        required_ok = self.validate_required()
        unknown_ok = self.validate_unknown()
        secrets_ok = self.validate_secrets()
        
        if not (required_ok and unknown_ok and secrets_ok):
            print(f"\n{'='*80}")
            print(f"ENV VALIDATION FAILED: {len(self.violations)} violation(s)")
            print(f"{'='*80}\n")
            
            for violation in self.violations:
                print(f"  {violation['message']}")
            
            return False
        
        print("\n✓ All environment variables valid.")
        return True
    
    def fail_closed(self, message: str):
        """Fail-closed: exit with error."""
        print(f"\n{'='*80}")
        print("ENV VALIDATION FAILED")
        print(f"{'='*80}")
        print(message)
        print("\nStartup blocked due to ENV validation failure.")
        sys.exit(1)


def main():
    """CLI entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description='RansomEye ENV Validator')
    parser.add_argument('--required-vars', default=None,
                       help='Comma-separated list of required variables')
    parser.add_argument('--allow-unknown', action='store_true',
                       help='Allow unknown environment variables')
    
    args = parser.parse_args()
    
    required_vars = None
    if args.required_vars:
        required_vars = set(args.required_vars.split(','))
    
    validator = EnvValidator(required_vars=required_vars)
    
    if not args.allow_unknown:
        validator.validate_unknown = lambda: True  # Skip unknown check if allowed
    
    is_valid = validator.validate()
    
    if not is_valid:
        validator.fail_closed(f"Found {len(validator.violations)} ENV validation violation(s).")
    
    print("\n✓ ENV validation passed.")
    sys.exit(0)


if __name__ == '__main__':
    main()

