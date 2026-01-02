# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/ai_registry/verification.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Verifies AI model integrity - checks signatures, hashes, and dependencies

"""
AI Model Verification: Verifies AI model integrity.
Checks signatures, hashes, and dependencies.
"""

import os
import hashlib
from pathlib import Path
from typing import Dict, List, Tuple
from ransomeye_trust.verify_tool import VerifyTool


class AIModelVerifier:
    """Verifies AI model integrity."""
    
    def __init__(self, trust_dir: str = "/home/ransomeye/rebuild/ransomeye_trust"):
        self.trust_dir = Path(trust_dir)
        self.verify_tool = VerifyTool(str(self.trust_dir))
    
    def _compute_file_hash(self, file_path: Path) -> str:
        """Compute SHA-256 hash of file."""
        sha256 = hashlib.sha256()
        with open(file_path, 'rb') as f:
            for chunk in iter(lambda: f.read(4096), b''):
                sha256.update(chunk)
        return sha256.hexdigest()
    
    def verify_model(self, model_path: Path, expected_hash: Optional[str] = None) -> Tuple[bool, List[str]]:
        """
        Verify model integrity.
        
        Args:
            model_path: Path to model file
            expected_hash: Expected hash (optional)
        
        Returns:
            Tuple of (is_valid: bool, errors: List[str])
        """
        errors = []
        
        # Check file exists
        if not model_path.exists():
            errors.append(f"Model file not found: {model_path}")
            return False, errors
        
        # Verify signature
        manifest_path = model_path.parent / f"{model_path.stem}_manifest.json"
        if manifest_path.exists():
            result = self.verify_tool.verify_manifest(manifest_path)
            if not result.get('valid'):
                errors.append(f"Invalid signature for model: {model_path.name}")
        else:
            errors.append(f"Missing manifest for model: {model_path.name}")
        
        # Verify hash if provided
        if expected_hash:
            computed_hash = self._compute_file_hash(model_path)
            if computed_hash != expected_hash:
                errors.append(f"Hash mismatch for model: {model_path.name}")
        
        return len(errors) == 0, errors
    
    def verify_model_dependencies(self, model_entry: Dict, registry) -> Tuple[bool, List[str]]:
        """
        Verify model dependencies.
        
        Args:
            model_entry: Model registry entry
            registry: AI registry instance
        
        Returns:
            Tuple of (is_valid: bool, errors: List[str])
        """
        errors = []
        dependencies = model_entry.get('dependencies', [])
        
        for dep_name in dependencies:
            dep_model = registry.get_model(dep_name)
            if not dep_model:
                errors.append(f"Dependency not found: {dep_name}")
                continue
            
            if not dep_model.get('active', True):
                errors.append(f"Dependency not active: {dep_name}")
        
        return len(errors) == 0, errors


def main():
    """CLI entry point for model verifier."""
    verifier = AIModelVerifier()
    
    # Example verification
    model_path = Path("/home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack/models/ransomware_behavior.model")
    if model_path.exists():
        is_valid, errors = verifier.verify_model(model_path)
        if is_valid:
            print("✓ Model verification passed")
        else:
            print("✗ Model verification failed:")
            for error in errors:
                print(f"  {error}")


if __name__ == '__main__':
    main()

