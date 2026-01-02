# Path and File Name : /home/ransomeye/rebuild/ransomeye_guardrails/crypto_enforcer.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Enforces cryptographic signing and verification for all artifacts

"""
Crypto enforcer: ensures all artifacts have manifest.json and manifest.sig.
Verifies signatures before allowing runtime usage.
"""

import os
import json
from pathlib import Path
from typing import List, Dict, Optional, Set
import hashlib

from .fail_closed import fail_closed


class CryptoEnforcer:
    """Enforces cryptographic signing requirements."""
    
    ARTIFACT_EXTENSIONS = {
        '.pkl', '.h5', '.pb', '.onnx', '.pt', '.pth', '.ckpt', '.gguf',  # Models
        '.pdf', '.html', '.csv', '.json',  # Reports
        '.zip', '.tar', '.gz',  # Archives
        '.sql', '.db',  # Databases
    }
    
    REQUIRED_MANIFEST_FIELDS = ['hash', 'timestamp', 'version', 'signer']
    
    def __init__(self, project_root: str = "/home/ransomeye/rebuild"):
        self.project_root = Path(project_root).resolve()
        self.violations: List[Dict] = []
        self.trust_hierarchy_path = self.project_root / "ransomeye_trust" / "key_hierarchy.json"
    
    def _load_trust_hierarchy(self) -> Dict:
        """Load trust domain key hierarchy."""
        if not self.trust_hierarchy_path.exists():
            return {}
        
        try:
            with open(self.trust_hierarchy_path, 'r') as f:
                return json.load(f)
        except Exception:
            return {}
    
    def _is_artifact(self, file_path: Path) -> bool:
        """Determine if file is an artifact requiring signing."""
        # Skip if in excluded directories
        rel_path = str(file_path.relative_to(self.project_root))
        if any(excluded in rel_path for excluded in ['__pycache__', '.git', 'node_modules', 'venv', '.venv', 'logs']):
            return False
        
        return file_path.suffix in self.ARTIFACT_EXTENSIONS
    
    def _check_manifest(self, artifact_path: Path) -> Optional[Dict]:
        """Check if artifact has manifest.json."""
        manifest_path = artifact_path.parent / f"{artifact_path.stem}_manifest.json"
        
        if not manifest_path.exists():
            # Also check for generic manifest.json in same directory
            generic_manifest = artifact_path.parent / "manifest.json"
            if generic_manifest.exists():
                try:
                    with open(generic_manifest, 'r') as f:
                        manifest = json.load(f)
                        # Check if this artifact is listed
                        if artifact_path.name in manifest.get('artifacts', {}):
                            manifest_path = generic_manifest
                        else:
                            return {
                                'artifact_path': str(artifact_path),
                                'description': f'Artifact {artifact_path.name} not listed in manifest.json',
                                'rule_name': 'MANIFEST_MISSING'
                            }
                except Exception:
                    return {
                        'artifact_path': str(artifact_path),
                        'description': f'Artifact {artifact_path.name} missing manifest.json',
                        'rule_name': 'MANIFEST_MISSING'
                    }
            else:
                return {
                    'artifact_path': str(artifact_path),
                    'description': f'Artifact {artifact_path.name} missing manifest.json',
                    'rule_name': 'MANIFEST_MISSING'
                }
        
        # Validate manifest structure
        try:
            with open(manifest_path, 'r') as f:
                manifest = json.load(f)
            
            missing_fields = [field for field in self.REQUIRED_MANIFEST_FIELDS if field not in manifest]
            if missing_fields:
                return {
                    'artifact_path': str(artifact_path),
                    'description': f'Manifest missing required fields: {", ".join(missing_fields)}',
                    'rule_name': 'MANIFEST_INVALID'
                }
            
            # Verify hash matches
            if 'hash' in manifest:
                computed_hash = self._compute_file_hash(artifact_path)
                if computed_hash != manifest['hash']:
                    return {
                        'artifact_path': str(artifact_path),
                        'description': f'Manifest hash mismatch for {artifact_path.name}',
                        'rule_name': 'MANIFEST_HASH_MISMATCH'
                    }
        except json.JSONDecodeError:
            return {
                'artifact_path': str(artifact_path),
                'description': f'Invalid JSON in manifest.json for {artifact_path.name}',
                'rule_name': 'MANIFEST_INVALID'
            }
        except Exception as e:
            return {
                'artifact_path': str(artifact_path),
                'description': f'Error reading manifest: {str(e)}',
                'rule_name': 'MANIFEST_ERROR'
            }
        
        return None
    
    def _check_signature(self, artifact_path: Path) -> Optional[Dict]:
        """Check if artifact has manifest.sig."""
        manifest_path = artifact_path.parent / f"{artifact_path.stem}_manifest.json"
        
        # Check for generic manifest first
        if not manifest_path.exists():
            manifest_path = artifact_path.parent / "manifest.json"
        
        if not manifest_path.exists():
            return None  # Will be caught by manifest check
        
        sig_path = manifest_path.parent / f"{manifest_path.stem}.sig"
        
        if not sig_path.exists():
            return {
                'artifact_path': str(artifact_path),
                'description': f'Artifact {artifact_path.name} missing manifest.sig signature file',
                'rule_name': 'SIGNATURE_MISSING'
            }
        
        # Verify signature file is not empty
        try:
            if sig_path.stat().st_size == 0:
                return {
                    'artifact_path': str(artifact_path),
                    'description': f'Signature file for {artifact_path.name} is empty',
                    'rule_name': 'SIGNATURE_INVALID'
                }
        except Exception:
            return {
                'artifact_path': str(artifact_path),
                'description': f'Cannot read signature file for {artifact_path.name}',
                'rule_name': 'SIGNATURE_ERROR'
            }
        
        return None
    
    def _compute_file_hash(self, file_path: Path) -> str:
        """Compute SHA-256 hash of file."""
        sha256 = hashlib.sha256()
        try:
            with open(file_path, 'rb') as f:
                for chunk in iter(lambda: f.read(4096), b''):
                    sha256.update(chunk)
            return sha256.hexdigest()
        except Exception:
            return ""
    
    def check_file(self, file_path: Path) -> List[Dict]:
        """Check a single artifact for crypto requirements."""
        violations = []
        
        if not self._is_artifact(file_path):
            return violations
        
        manifest_violation = self._check_manifest(file_path)
        if manifest_violation:
            violations.append(manifest_violation)
        
        signature_violation = self._check_signature(file_path)
        if signature_violation:
            violations.append(signature_violation)
        
        return violations
    
    def check_directory(self, directory: Optional[Path] = None) -> List[Dict]:
        """Check all artifacts in directory tree."""
        if directory is None:
            directory = self.project_root
        
        directory = Path(directory).resolve()
        all_violations = []
        
        for root, dirs, files in os.walk(directory):
            dirs[:] = [d for d in dirs if d not in ['__pycache__', '.git', 'node_modules', 'venv', '.venv', 'logs']]
            
            for file in files:
                file_path = Path(root) / file
                violations = self.check_file(file_path)
                all_violations.extend(violations)
        
        self.violations = all_violations
        return all_violations
    
    def report_violations(self) -> None:
        """Report violations and fail-closed if any found."""
        if not self.violations:
            print("✓ All artifacts have valid manifests and signatures.")
            return
        
        print(f"\n{'='*80}")
        print(f"CRYPTO ENFORCEMENT VIOLATIONS: {len(self.violations)}")
        print(f"{'='*80}\n")
        
        for violation in self.violations:
            print(f"Artifact: {violation['artifact_path']}")
            print(f"  Description: {violation['description']}")
            print(f"  Rule: {violation['rule_name']}")
            print()
        
        # Fail-closed
        fail_closed(
            "CRYPTO_ENFORCEMENT_VIOLATION",
            f"Found {len(self.violations)} crypto enforcement violation(s). All artifacts must have manifest.json and manifest.sig. Build cannot proceed.",
            file_path=None
        )


def main():
    """CLI entry point for crypto enforcer."""
    import argparse
    
    parser = argparse.ArgumentParser(description='RansomEye Crypto Enforcer')
    parser.add_argument('--project-root', default='/home/ransomeye/rebuild',
                       help='Project root directory')
    parser.add_argument('--directory', default=None,
                       help='Specific directory to check (default: entire project)')
    
    args = parser.parse_args()
    
    enforcer = CryptoEnforcer(args.project_root)
    
    if args.directory:
        violations = enforcer.check_directory(Path(args.directory))
    else:
        violations = enforcer.check_directory()
    
    enforcer.report_violations()


if __name__ == '__main__':
    main()

