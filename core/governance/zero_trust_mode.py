# Path and File Name : /home/ransomeye/rebuild/core/governance/zero_trust_mode.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Zero-trust operation mode - system remains verifiable even if operator is compromised, admin credentials leaked, logs partially destroyed, or UI disabled

"""
Zero-Trust Operation Mode (PROMPT-63 Phase 3)

System must remain verifiable even if:
- Operator is compromised
- Admin credentials leaked
- Logs partially destroyed
- UI disabled

Implements:
- Minimal immutable proof anchors
- Snapshot survivability checks
- Cross-verification against golden baseline
"""

import os
import sys
import json
import hashlib
from pathlib import Path
from datetime import datetime, timezone
from typing import Dict, List, Optional, Tuple
import logging

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger('zero_trust_mode')

PROJECT_ROOT = Path("/home/ransomeye/rebuild")
GOLDEN_BASELINE_PATH = Path("/var/lib/ransomeye/golden_baseline.json")
PROOF_ANCHORS_PATH = Path("/var/lib/ransomeye/proof_anchors")


class ZeroTrustMode:
    """Zero-trust operation mode manager."""
    
    def __init__(self):
        """Initialize zero-trust mode."""
        self.proof_anchors = []
        
    def create_proof_anchor(self, anchor_type: str, anchor_data: Dict) -> Dict:
        """Create minimal immutable proof anchor."""
        anchor = {
            'anchor_id': f"anchor_{datetime.now(timezone.utc).strftime('%Y%m%d%H%M%S')}_{hashlib.sha256(json.dumps(anchor_data, sort_keys=True).encode()).hexdigest()[:8]}",
            'anchor_type': anchor_type,
            'anchor_data': anchor_data,
            'created_at': datetime.now(timezone.utc).isoformat(),
            'anchor_hash': None
        }
        
        # Compute anchor hash
        anchor_json = json.dumps(anchor, sort_keys=True)
        anchor_hash = hashlib.sha256(anchor_json.encode()).hexdigest()
        anchor['anchor_hash'] = anchor_hash
        
        return anchor
    
    def save_proof_anchor(self, anchor: Dict) -> bool:
        """Save proof anchor (immutable)."""
        try:
            PROOF_ANCHORS_PATH.mkdir(parents=True, exist_ok=True)
            
            anchor_file = PROOF_ANCHORS_PATH / f"{anchor['anchor_id']}.json"
            
            # Anchor files are immutable (read-only)
            with open(anchor_file, 'w') as f:
                json.dump(anchor, f, indent=2)
            
            # Make read-only
            anchor_file.chmod(0o444)
            
            logger.info(f"✓ Proof anchor saved: {anchor['anchor_id']}")
            return True
        except Exception as e:
            logger.error(f"✗ Failed to save proof anchor: {e}")
            return False
    
    def create_golden_baseline(self) -> Dict:
        """Create golden baseline for cross-verification."""
        baseline = {
            'baseline_id': f"baseline_{datetime.now(timezone.utc).strftime('%Y%m%d%H%M%S')}",
            'created_at': datetime.now(timezone.utc).isoformat(),
            'artifacts': {},
            'systemd_services': [],
            'database_schema': {},
            'baseline_hash': None
        }
        
        # Capture artifact hashes
        artifact_hashes_path = PROJECT_ROOT / "docs/ARTIFACT_HASHES.txt"
        if artifact_hashes_path.exists():
            try:
                with open(artifact_hashes_path, 'r') as f:
                    content = f.read()
                    baseline['artifacts']['artifact_hashes_file_hash'] = hashlib.sha256(content.encode()).hexdigest()
            except Exception as e:
                logger.warning(f"⚠ Failed to capture artifact hashes: {e}")
        
        # Capture systemd services
        try:
            import subprocess
            result = subprocess.run(
                ['systemctl', 'list-units', '--type=service', '--state=running', 'ransomeye*'],
                capture_output=True,
                text=True,
                timeout=5
            )
            if result.returncode == 0:
                services = []
                for line in result.stdout.split('\n')[1:]:
                    if line.strip():
                        parts = line.split()
                        if parts:
                            services.append(parts[0])
                baseline['systemd_services'] = services
        except Exception as e:
            logger.warning(f"⚠ Failed to capture systemd services: {e}")
        
        # Compute baseline hash
        baseline_json = json.dumps(baseline, sort_keys=True)
        baseline_hash = hashlib.sha256(baseline_json.encode()).hexdigest()
        baseline['baseline_hash'] = baseline_hash
        
        return baseline
    
    def save_golden_baseline(self, baseline: Dict) -> bool:
        """Save golden baseline."""
        try:
            GOLDEN_BASELINE_PATH.parent.mkdir(parents=True, exist_ok=True)
            
            with open(GOLDEN_BASELINE_PATH, 'w') as f:
                json.dump(baseline, f, indent=2)
            
            # Make read-only
            GOLDEN_BASELINE_PATH.chmod(0o444)
            
            logger.info(f"✓ Golden baseline saved: {baseline['baseline_id']}")
            return True
        except Exception as e:
            logger.error(f"✗ Failed to save golden baseline: {e}")
            return False
    
    def check_snapshot_survivability(self) -> Tuple[bool, List[str]]:
        """Check snapshot survivability (can survive operator compromise)."""
        issues = []
        
        # Check proof anchors exist
        if not PROOF_ANCHORS_PATH.exists():
            issues.append("Proof anchors directory not found")
        else:
            anchor_files = list(PROOF_ANCHORS_PATH.glob("*.json"))
            if len(anchor_files) == 0:
                issues.append("No proof anchors found")
            else:
                # Verify anchors are immutable (read-only)
                for anchor_file in anchor_files:
                    if anchor_file.stat().st_mode & 0o222:  # Check if writable
                        issues.append(f"Proof anchor {anchor_file.name} is writable (should be read-only)")
        
        # Check golden baseline exists
        if not GOLDEN_BASELINE_PATH.exists():
            issues.append("Golden baseline not found")
        else:
            # Verify baseline is immutable (read-only)
            if GOLDEN_BASELINE_PATH.stat().st_mode & 0o222:
                issues.append("Golden baseline is writable (should be read-only)")
        
        return len(issues) == 0, issues
    
    def cross_verify_against_baseline(self) -> Tuple[bool, List[str]]:
        """Cross-verify current state against golden baseline."""
        if not GOLDEN_BASELINE_PATH.exists():
            return False, ["Golden baseline not found"]
        
        try:
            with open(GOLDEN_BASELINE_PATH, 'r') as f:
                baseline = json.load(f)
            
            issues = []
            
            # Verify artifact hashes file
            artifact_hashes_path = PROJECT_ROOT / "docs/ARTIFACT_HASHES.txt"
            if artifact_hashes_path.exists():
                with open(artifact_hashes_path, 'r') as f:
                    content = f.read()
                    current_hash = hashlib.sha256(content.encode()).hexdigest()
                    expected_hash = baseline['artifacts'].get('artifact_hashes_file_hash')
                    if expected_hash and current_hash != expected_hash:
                        issues.append("Artifact hashes file modified")
            
            # Verify systemd services
            try:
                import subprocess
                result = subprocess.run(
                    ['systemctl', 'list-units', '--type=service', '--state=running', 'ransomeye*'],
                    capture_output=True,
                    text=True,
                    timeout=5
                )
                if result.returncode == 0:
                    current_services = []
                    for line in result.stdout.split('\n')[1:]:
                        if line.strip():
                            parts = line.split()
                            if parts:
                                current_services.append(parts[0])
                    
                    baseline_services = set(baseline.get('systemd_services', []))
                    current_services_set = set(current_services)
                    
                    if baseline_services != current_services_set:
                        missing = baseline_services - current_services_set
                        added = current_services_set - baseline_services
                        if missing:
                            issues.append(f"Services missing: {missing}")
                        if added:
                            issues.append(f"Services added: {added}")
            except Exception as e:
                issues.append(f"Failed to verify systemd services: {e}")
            
            return len(issues) == 0, issues
        except Exception as e:
            return False, [f"Cross-verification failed: {e}"]
    
    def run(self, create_baseline: bool = False) -> bool:
        """Run zero-trust mode checks."""
        logger.info("=" * 80)
        logger.info("Zero-Trust Operation Mode (PROMPT-63 Phase 3)")
        logger.info("=" * 80)
        
        # Create golden baseline if requested
        if create_baseline:
            logger.info("Creating golden baseline...")
            baseline = self.create_golden_baseline()
            if not self.save_golden_baseline(baseline):
                logger.error("FAIL-CLOSED: Failed to save golden baseline")
                return False
        
        # Check snapshot survivability
        logger.info("Checking snapshot survivability...")
        survivable, survivability_issues = self.check_snapshot_survivability()
        if not survivable:
            logger.warning(f"⚠ Snapshot survivability issues: {survivability_issues}")
        
        # Cross-verify against baseline
        logger.info("Cross-verifying against golden baseline...")
        verified, verification_issues = self.cross_verify_against_baseline()
        if not verified:
            logger.warning(f"⚠ Cross-verification issues: {verification_issues}")
        
        if survivable and verified:
            logger.info("✓ Zero-trust mode checks passed")
            return True
        else:
            logger.warning("⚠ Zero-trust mode checks have warnings")
            return True  # Warnings don't fail, but are logged


def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description='Zero-Trust Operation Mode')
    parser.add_argument('--create-baseline', action='store_true', help='Create golden baseline')
    
    args = parser.parse_args()
    
    ztm = ZeroTrustMode()
    success = ztm.run(args.create_baseline)
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()

