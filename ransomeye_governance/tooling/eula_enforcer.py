# Path: /home/ransomeye/rebuild/ransomeye_governance/tooling/eula_enforcer.py
# Author: RansomEye Core Team
# Purpose: Enforces EULA acceptance - installation/startup fails if EULA not accepted

"""
EULA Enforcer: Enforces EULA acceptance.

Rules:
- Installation MUST fail unless EULA is accepted
- EULA file MUST exist at required path
- Acceptance MUST be logged and auditable
- No EULA = NO INSTALL
"""

import os
import sys
from pathlib import Path
from datetime import datetime
from typing import Optional

# Required EULA path
EULA_PATH = Path("/home/ransomeye/rebuild/ransomeye_governance/eula/EULA.txt")
EULA_ACCEPTANCE_LOG = Path("/var/log/ransomeye/eula_acceptance.log")


class EULAEnforcer:
    """Enforces EULA acceptance."""
    
    def __init__(self, eula_path: Optional[Path] = None):
        self.eula_path = eula_path or EULA_PATH
        self.acceptance_log = EULA_ACCEPTANCE_LOG
    
    def check_eula_exists(self) -> bool:
        """Check if EULA file exists."""
        return self.eula_path.exists()
    
    def log_acceptance(self, user: str = "system", method: str = "installer"):
        """Log EULA acceptance."""
        log_entry = {
            'timestamp': datetime.utcnow().isoformat(),
            'user': user,
            'method': method,
            'eula_path': str(self.eula_path),
            'accepted': True
        }
        
        # Ensure log directory exists
        self.acceptance_log.parent.mkdir(parents=True, exist_ok=True)
        
        # Append to log
        with open(self.acceptance_log, 'a') as f:
            import json
            f.write(json.dumps(log_entry) + '\n')
    
    def check_acceptance(self) -> bool:
        """Check if EULA has been accepted (by checking log)."""
        if not self.acceptance_log.exists():
            return False
        
        # Check if acceptance exists in log
        with open(self.acceptance_log, 'r') as f:
            for line in f:
                try:
                    import json
                    entry = json.loads(line)
                    if entry.get('accepted') and entry.get('eula_path') == str(self.eula_path):
                        return True
                except Exception:
                    continue
        
        return False
    
    def enforce(self, require_acceptance: bool = True) -> bool:
        """Enforce EULA acceptance."""
        # Check if EULA file exists
        if not self.check_eula_exists():
            print(f"ERROR: EULA file not found at: {self.eula_path}")
            return False
        
        # Check if EULA has been accepted
        if require_acceptance and not self.check_acceptance():
            print("ERROR: EULA not accepted. Installation/startup blocked.")
            print(f"EULA file: {self.eula_path}")
            print("Please accept the EULA to proceed.")
            return False
        
        return True
    
    def fail_closed(self, message: str):
        """Fail-closed: exit with error."""
        print(f"\n{'='*80}")
        print("EULA ENFORCEMENT FAILED")
        print(f"{'='*80}")
        print(message)
        print("\nInstallation/startup blocked - EULA not accepted.")
        sys.exit(1)


def main():
    """CLI entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description='RansomEye EULA Enforcer')
    parser.add_argument('--eula-path', default=None,
                       help='Path to EULA file')
    parser.add_argument('--accept', action='store_true',
                       help='Accept EULA and log acceptance')
    parser.add_argument('--check', action='store_true',
                       help='Check if EULA is accepted')
    
    args = parser.parse_args()
    
    eula_path = Path(args.eula_path) if args.eula_path else None
    enforcer = EULAEnforcer(eula_path)
    
    if args.accept:
        enforcer.log_acceptance()
        print("✓ EULA accepted and logged.")
        sys.exit(0)
    
    if args.check:
        is_accepted = enforcer.check_acceptance()
        if is_accepted:
            print("✓ EULA has been accepted.")
            sys.exit(0)
        else:
            print("✗ EULA has not been accepted.")
            sys.exit(1)
    
    # Default: enforce
    is_valid = enforcer.enforce()
    
    if not is_valid:
        enforcer.fail_closed("EULA enforcement failed.")
    
    print("✓ EULA enforcement passed.")
    sys.exit(0)


if __name__ == '__main__':
    main()

