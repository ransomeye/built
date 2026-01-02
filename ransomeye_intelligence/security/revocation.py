# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/security/revocation.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Checks revocation status for intelligence artifacts - rejects revoked artifacts

"""
Revocation Check: Checks revocation status for intelligence artifacts.
Rejects revoked artifacts.
"""

import os
from pathlib import Path
from typing import Dict, List, Tuple
import json


class IntelligenceRevocation:
    """Checks revocation status for intelligence artifacts."""
    
    # Runtime-safe default (no build-time paths):
    # Use the deployed trust module location under /opt/ransomeye/modules.
    REVOCATION_LIST_PATH = Path(os.environ.get("RANSOMEYE_ROOT", "/opt/ransomeye")) / "modules" / "ransomeye_trust" / "revocation_list.json"
    
    def __init__(self):
        self.revocation_list: Dict = {}
        self._load_revocation_list()
    
    def _load_revocation_list(self) -> None:
        """Load revocation list."""
        if self.REVOCATION_LIST_PATH.exists():
            try:
                with open(self.REVOCATION_LIST_PATH, 'r') as f:
                    self.revocation_list = json.load(f)
            except Exception:
                self.revocation_list = {'revoked_artifacts': []}
        else:
            self.revocation_list = {'revoked_artifacts': []}
    
    def is_revoked(self, artifact_hash: str) -> bool:
        """Check if artifact is revoked."""
        revoked = self.revocation_list.get('revoked_artifacts', [])
        return artifact_hash in revoked
    
    def check_revocation(self, artifact_path: Path) -> Tuple[bool, List[str]]:
        """
        Check revocation status for artifact.
        
        Args:
            artifact_path: Path to artifact
        
        Returns:
            Tuple of (is_valid: bool, errors: List[str])
        """
        errors = []
        
        # Compute artifact hash
        import hashlib
        sha256 = hashlib.sha256()
        with open(artifact_path, 'rb') as f:
            for chunk in iter(lambda: f.read(4096), b''):
                sha256.update(chunk)
        artifact_hash = sha256.hexdigest()
        
        # Check revocation
        if self.is_revoked(artifact_hash):
            errors.append(f"Artifact is revoked: {artifact_path.name}")
        
        return len(errors) == 0, errors


def main():
    """CLI entry point for revocation checker."""
    revocation = IntelligenceRevocation()
    
    # Example check
    artifact_path = Path("/home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack/models/ransomware_behavior.model")
    if artifact_path.exists():
        is_valid, errors = revocation.check_revocation(artifact_path)
        if is_valid:
            print("✓ Artifact not revoked")
        else:
            print("✗ Artifact revocation check failed:")
            for error in errors:
                print(f"  {error}")


if __name__ == '__main__':
    main()

