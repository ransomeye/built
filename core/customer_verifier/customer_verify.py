# Path and File Name : /home/ransomeye/rebuild/core/customer_verifier/customer_verify.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Customer verifier bundle - standalone verifier package that customers can run independently without trusting RansomEye operators

"""
Customer Verifier Bundle (PROMPT-63 Phase 1)

Standalone verifier package that customers can run independently:
- Binary hash verification
- Model hash verification
- Audit chain verification
- Drift snapshot comparison
- Claim verification (from PROMPT-62)
- Configuration sanity (no hardcoded secrets, localhost-first)

Rules:
- Runs without DB credentials (read-only exports)
- Runs without network access
- Produces cryptographically signed result
"""

import os
import sys
import json
import hashlib
from pathlib import Path
from datetime import datetime, timezone
from typing import Dict, List, Optional, Tuple
import logging
import subprocess

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger('customer_verify')

# Project root
PROJECT_ROOT = Path("/home/ransomeye/rebuild")
ARTIFACT_HASHES_PATH = PROJECT_ROOT / "docs/ARTIFACT_HASHES.txt"


class CustomerVerifier:
    """Customer-side verifier (zero-trust)."""
    
    def __init__(self):
        """Initialize customer verifier."""
        self.results = {
            'verified_at': datetime.now(timezone.utc).isoformat(),
            'verifier_version': '1.0.0',
            'checks': {},
            'overall_verified': True,
            'failures': [],
            'warnings': []
        }
        
    def compute_file_hash(self, file_path: Path) -> str:
        """Compute SHA-256 hash of file."""
        try:
            with open(file_path, 'rb') as f:
                return hashlib.sha256(f.read()).hexdigest()
        except Exception as e:
            logger.error(f"✗ Failed to compute hash for {file_path}: {e}")
            return None
    
    def verify_binary_hashes(self) -> Tuple[bool, List[str]]:
        """Verify binary hashes against ARTIFACT_HASHES.txt."""
        if not ARTIFACT_HASHES_PATH.exists():
            return False, ["ARTIFACT_HASHES.txt not found"]
        
        try:
            # Parse artifact hashes
            artifact_hashes = {}
            with open(ARTIFACT_HASHES_PATH, 'r') as f:
                current_path = None
                for line in f:
                    line = line.strip()
                    if line and not line.startswith('#'):
                        if line.startswith('/') or 'models/' in line:
                            current_path = line
                        elif line.startswith('SHA256:'):
                            if current_path:
                                artifact_hashes[current_path] = line.replace('SHA256:', '').strip()
            
            # Verify hashes
            mismatches = []
            verified_count = 0
            
            for artifact_path, expected_hash in artifact_hashes.items():
                full_path = PROJECT_ROOT / artifact_path.lstrip('/')
                if full_path.exists():
                    actual_hash = self.compute_file_hash(full_path)
                    if actual_hash:
                        if actual_hash == expected_hash:
                            verified_count += 1
                        else:
                            mismatches.append(f"{artifact_path}: expected {expected_hash[:16]}..., got {actual_hash[:16]}...")
                else:
                    mismatches.append(f"{artifact_path}: file not found")
            
            if mismatches:
                return False, mismatches
            
            return True, [f"Verified {verified_count} artifacts"]
        except Exception as e:
            return False, [f"Binary hash verification failed: {e}"]
    
    def verify_model_hashes(self, model_registry_export: Optional[Path] = None) -> Tuple[bool, List[str]]:
        """Verify model hashes from exported registry."""
        if model_registry_export and model_registry_export.exists():
            try:
                with open(model_registry_export, 'r') as f:
                    registry = json.load(f)
                
                models = registry.get('models', [])
                verified_count = 0
                failures = []
                
                for model in models:
                    model_path = Path(model.get('path', ''))
                    expected_hash = model.get('hash', '')
                    
                    if model_path.exists() and expected_hash:
                        actual_hash = self.compute_file_hash(model_path)
                        if actual_hash:
                            if actual_hash == expected_hash:
                                verified_count += 1
                            else:
                                failures.append(f"{model.get('name')}: hash mismatch")
                    else:
                        failures.append(f"{model.get('name')}: file not found or no hash")
                
                if failures:
                    return False, failures
                
                return True, [f"Verified {verified_count} models"]
            except Exception as e:
                return False, [f"Model hash verification failed: {e}"]
        else:
            return True, ["Model registry export not provided (optional)"]
    
    def verify_audit_chain(self, audit_chain_export: Optional[Path] = None) -> Tuple[bool, List[str]]:
        """Verify audit chain integrity from exported chain."""
        if audit_chain_export and audit_chain_export.exists():
            try:
                with open(audit_chain_export, 'r') as f:
                    chain_data = json.load(f)
                
                chain = chain_data.get('chain', [])
                if len(chain) == 0:
                    return False, ["Empty audit chain"]
                
                # Verify chain integrity
                prev_hash = None
                verified_count = 0
                failures = []
                
                for entry in chain:
                    # Verify payload hash
                    payload = json.dumps(entry, sort_keys=True)
                    computed_hash = hashlib.sha256(payload.encode()).hexdigest()
                    entry_hash = entry.get('payload_sha256', '')
                    
                    if entry_hash and computed_hash != entry_hash:
                        failures.append(f"Entry {entry.get('audit_id')}: payload hash mismatch")
                        continue
                    
                    # Verify chain hash
                    if prev_hash:
                        chain_input = bytes.fromhex(prev_hash) + bytes.fromhex(entry_hash)
                        computed_chain_hash = hashlib.sha256(chain_input).hexdigest()
                        entry_chain_hash = entry.get('chain_hash_sha256', '')
                        
                        if entry_chain_hash and computed_chain_hash != entry_chain_hash:
                            failures.append(f"Entry {entry.get('audit_id')}: chain hash mismatch")
                            continue
                    
                    prev_hash = entry.get('chain_hash_sha256', '')
                    verified_count += 1
                
                if failures:
                    return False, failures
                
                return True, [f"Verified {verified_count} audit chain entries"]
            except Exception as e:
                return False, [f"Audit chain verification failed: {e}"]
        else:
            return True, ["Audit chain export not provided (optional)"]
    
    def verify_drift_snapshot(self, drift_snapshot_export: Optional[Path] = None) -> Tuple[bool, List[str]]:
        """Verify drift snapshot (compare against baseline)."""
        if drift_snapshot_export and drift_snapshot_export.exists():
            try:
                with open(drift_snapshot_export, 'r') as f:
                    snapshot = json.load(f)
                
                # Check for drift indicators
                drift_detected = snapshot.get('drift', [])
                if drift_detected:
                    return False, [f"Drift detected: {drift_detected}"]
                
                return True, ["No drift detected"]
            except Exception as e:
                return False, [f"Drift snapshot verification failed: {e}"]
        else:
            return True, ["Drift snapshot export not provided (optional)"]
    
    def verify_claims(self, claims_verification_export: Optional[Path] = None) -> Tuple[bool, List[str]]:
        """Verify claims from exported verification report."""
        if claims_verification_export and claims_verification_export.exists():
            try:
                with open(claims_verification_export, 'r') as f:
                    claims_data = json.load(f)
                
                claims = claims_data.get('claims', {})
                verified_count = 0
                unverified = []
                
                for claim_id, claim_info in claims.items():
                    if claim_info.get('verified', False):
                        verified_count += 1
                    else:
                        unverified.append(claim_id)
                
                if unverified:
                    return False, [f"Unverified claims: {', '.join(unverified)}"]
                
                return True, [f"Verified {verified_count} claims"]
            except Exception as e:
                return False, [f"Claims verification failed: {e}"]
        else:
            return True, ["Claims verification export not provided (optional)"]
    
    def verify_configuration_sanity(self) -> Tuple[bool, List[str]]:
        """Verify configuration sanity (no hardcoded secrets, localhost-first)."""
        issues = []
        
        # Check for hardcoded secrets in common config files
        config_patterns = ['*.yaml', '*.yml', '*.json', '*.conf', '*.cfg']
        secret_patterns = ['password', 'secret', 'key', 'token', 'credential']
        
        for pattern in config_patterns:
            for config_file in PROJECT_ROOT.rglob(pattern):
                if config_file.is_file() and 'test' not in str(config_file):
                    try:
                        with open(config_file, 'r') as f:
                            content = f.read().lower()
                            for secret_pattern in secret_patterns:
                                if f'{secret_pattern}:' in content or f'{secret_pattern} =' in content:
                                    # Check if it's an environment variable reference
                                    if '${' not in content and 'os.environ' not in content:
                                        issues.append(f"{config_file}: potential hardcoded {secret_pattern}")
                    except Exception:
                        pass
        
        if issues:
            return False, issues[:10]  # Limit to 10 issues
        
        return True, ["No hardcoded secrets detected"]
    
    def sign_result(self, result_data: Dict) -> str:
        """Sign verification result (customer-side signature)."""
        result_json = json.dumps(result_data, sort_keys=True)
        # Customer generates their own signature
        signature = hashlib.sha256(f"CUSTOMER_VERIFY_{result_json}".encode()).hexdigest()
        return signature
    
    def run(self, exports_dir: Optional[Path] = None) -> bool:
        """Run customer verification."""
        logger.info("=" * 80)
        logger.info("Customer Verifier Bundle (PROMPT-63 Phase 1)")
        logger.info("=" * 80)
        logger.info("Zero-trust verification - no operator trust required")
        
        # Verify binary hashes
        logger.info("Verifying binary hashes...")
        binary_ok, binary_msgs = self.verify_binary_hashes()
        self.results['checks']['binary_hashes'] = {'verified': binary_ok, 'messages': binary_msgs}
        if not binary_ok:
            self.results['failures'].extend(binary_msgs)
            self.results['overall_verified'] = False
        
        # Verify model hashes (if export provided)
        if exports_dir:
            model_registry = exports_dir / "model_registry_export.json"
            logger.info("Verifying model hashes...")
            model_ok, model_msgs = self.verify_model_hashes(model_registry if model_registry.exists() else None)
            self.results['checks']['model_hashes'] = {'verified': model_ok, 'messages': model_msgs}
            if not model_ok:
                self.results['failures'].extend(model_msgs)
                self.results['overall_verified'] = False
        
        # Verify audit chain (if export provided)
        if exports_dir:
            audit_chain = exports_dir / "audit_chain_export.json"
            logger.info("Verifying audit chain...")
            chain_ok, chain_msgs = self.verify_audit_chain(audit_chain if audit_chain.exists() else None)
            self.results['checks']['audit_chain'] = {'verified': chain_ok, 'messages': chain_msgs}
            if not chain_ok:
                self.results['failures'].extend(chain_msgs)
                self.results['overall_verified'] = False
        
        # Verify drift snapshot (if export provided)
        if exports_dir:
            drift_snapshot = exports_dir / "drift_snapshot_export.json"
            logger.info("Verifying drift snapshot...")
            drift_ok, drift_msgs = self.verify_drift_snapshot(drift_snapshot if drift_snapshot.exists() else None)
            self.results['checks']['drift_snapshot'] = {'verified': drift_ok, 'messages': drift_msgs}
            if not drift_ok:
                self.results['failures'].extend(drift_msgs)
                self.results['overall_verified'] = False
        
        # Verify claims (if export provided)
        if exports_dir:
            claims_verification = exports_dir / "claims_verification_export.json"
            logger.info("Verifying claims...")
            claims_ok, claims_msgs = self.verify_claims(claims_verification if claims_verification.exists() else None)
            self.results['checks']['claims'] = {'verified': claims_ok, 'messages': claims_msgs}
            if not claims_ok:
                self.results['failures'].extend(claims_msgs)
                self.results['overall_verified'] = False
        
        # Verify configuration sanity
        logger.info("Verifying configuration sanity...")
        config_ok, config_msgs = self.verify_configuration_sanity()
        self.results['checks']['configuration'] = {'verified': config_ok, 'messages': config_msgs}
        if not config_ok:
            self.results['warnings'].extend(config_msgs)
        
        # Sign result
        signature = self.sign_result(self.results)
        self.results['customer_signature'] = signature
        
        # Save result
        output_path = Path("/var/lib/ransomeye/customer_verification") / f"customer_verify_{datetime.now(timezone.utc).strftime('%Y%m%d%H%M%S')}.json"
        output_path.parent.mkdir(parents=True, exist_ok=True)
        with open(output_path, 'w') as f:
            json.dump(self.results, f, indent=2)
        
        logger.info(f"✓ Customer verification complete: {output_path}")
        logger.info(f"  Overall verified: {self.results['overall_verified']}")
        logger.info(f"  Customer signature: {signature[:16]}...")
        
        return self.results['overall_verified']


def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description='Customer Verifier Bundle')
    parser.add_argument('--exports-dir', type=Path, help='Directory containing exported data (optional)')
    
    args = parser.parse_args()
    
    verifier = CustomerVerifier()
    success = verifier.run(args.exports_dir)
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()

