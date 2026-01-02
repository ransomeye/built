# Path and File Name : /home/ransomeye/rebuild/ransomeye_guardrails/main.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Main entry point for running all guardrails checks

"""
Main entry point for RansomEye guardrails enforcement.
Runs all guardrail checks and fails-closed on any violation.
"""

import sys
from pathlib import Path

# CRITICAL: Runtime check MUST run first, before any other imports
from .runtime_check import validate_runtime

# Validate runtime before proceeding
validate_runtime()

from .scanner import GuardrailScanner
from .header_enforcer import HeaderEnforcer
from .env_enforcer import EnvEnforcer
from .ml_enforcer import MLEnforcer
from .crypto_enforcer import CryptoEnforcer
from .retention_enforcer import RetentionEnforcer


def main():
    """Run all guardrail checks."""
    # Runtime check already validated at import time
    project_root = "/home/ransomeye/rebuild"
    rules_path = Path(project_root) / "ransomeye_guardrails" / "rules.yaml"
    
    print("="*80)
    print("RANSOMEYE GLOBAL GUARDRAILS ENFORCEMENT")
    print("="*80)
    print()
    
    # 1. Static Scanner
    print("[1/6] Running static scanner...")
    scanner = GuardrailScanner(str(rules_path), project_root)
    scanner.scan_directory()
    scanner.report_violations()
    
    # 2. Header Enforcer
    print("\n[2/6] Running header enforcer...")
    header_enforcer = HeaderEnforcer(project_root)
    header_enforcer.check_directory()
    header_enforcer.report_violations()
    
    # 3. ENV Enforcer
    print("\n[3/6] Running ENV enforcer...")
    env_enforcer = EnvEnforcer(project_root)
    env_enforcer.check_directory()
    env_enforcer.report_violations()
    
    # 4. ML Enforcer
    print("\n[4/6] Running ML enforcer...")
    ml_enforcer = MLEnforcer(project_root)
    ml_enforcer.check_directory()
    ml_enforcer.report_violations()
    
    # 5. Crypto Enforcer
    print("\n[5/6] Running crypto enforcer...")
    crypto_enforcer = CryptoEnforcer(project_root)
    crypto_enforcer.check_directory()
    crypto_enforcer.report_violations()
    
    # 6. Retention Enforcer
    print("\n[6/6] Running retention enforcer...")
    retention_enforcer = RetentionEnforcer(project_root)
    retention_enforcer.check_configuration()
    retention_enforcer.report_violations()
    
    print("\n" + "="*80)
    print("✓ ALL GUARDRAILS CHECKS PASSED")
    print("="*80)
    print("\nBuild can proceed.")


if __name__ == '__main__':
    try:
        main()
    except SystemExit:
        raise
    except Exception as e:
        print(f"\n✗ Fatal error in guardrails enforcement: {e}", file=sys.stderr)
        sys.exit(1)

