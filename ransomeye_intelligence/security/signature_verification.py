# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/security/signature_verification.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Verifies signatures on all intelligence artifacts - fails-closed on invalid signatures

"""
Signature Verification: Verifies signatures on all intelligence artifacts.
Fails-closed on invalid signatures.
"""

import os
from pathlib import Path
from typing import Dict, List, Tuple, Optional
from ransomeye_trust.verify_tool import VerifyTool


class IntelligenceSignatureVerifier:
    """Verifies signatures on intelligence artifacts."""
    
    def __init__(self, trust_dir: Optional[str] = None):
        # Runtime-safe default: trust tooling is deployed under /opt/ransomeye/modules/ransomeye_trust
        if trust_dir is None:
            runtime_root = Path(os.environ.get("RANSOMEYE_ROOT", "/opt/ransomeye"))
            trust_dir = str(runtime_root / "modules" / "ransomeye_trust")
        self.trust_dir = Path(trust_dir)
        self.verify_tool = VerifyTool(str(self.trust_dir))
    
    def verify_baseline_pack(self, pack_dir: Path) -> Tuple[bool, List[str]]:
        """Verify baseline pack signatures."""
        errors = []
        
        # Verify model manifest
        manifest_path = pack_dir / "models" / "model_manifest.json"
        if manifest_path.exists():
            result = self.verify_tool.verify_manifest(manifest_path)
            if not result.get('valid'):
                errors.append("Invalid signature for model manifest")
        
        # Verify SHAP baseline
        shap_path = pack_dir / "shap" / "baseline_shap_values.json"
        if shap_path.exists():
            result = self.verify_tool.verify_manifest(shap_path)
            if not result.get('valid'):
                errors.append("Invalid signature for SHAP baseline")
        
        return len(errors) == 0, errors
    
    def verify_threat_intel(self, intel_path: Path) -> Tuple[bool, List[str]]:
        """Verify threat intelligence signatures."""
        errors = []
        
        manifest_path = intel_path.parent / f"{intel_path.stem}_manifest.json"
        if manifest_path.exists():
            result = self.verify_tool.verify_manifest(manifest_path)
            if not result.get('valid'):
                errors.append(f"Invalid signature for threat intelligence: {intel_path.name}")
        else:
            errors.append(f"Missing manifest for threat intelligence: {intel_path.name}")
        
        return len(errors) == 0, errors
    
    def verify_rag_index(self, index_path: Path) -> Tuple[bool, List[str]]:
        """Verify RAG index signatures."""
        errors = []
        
        manifest_path = index_path / "index_manifest.json"
        if manifest_path.exists():
            result = self.verify_tool.verify_manifest(manifest_path)
            if not result.get('valid'):
                errors.append("Invalid signature for RAG index")
        else:
            errors.append("Missing manifest for RAG index")
        
        return len(errors) == 0, errors


def main():
    """CLI entry point for signature verifier."""
    verifier = IntelligenceSignatureVerifier()
    
    # Example verification
    pack_dir = Path("/home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack")
    is_valid, errors = verifier.verify_baseline_pack(pack_dir)
    
    if is_valid:
        print("✓ Baseline pack signatures valid")
    else:
        print("✗ Baseline pack signature verification failed:")
        for error in errors:
            print(f"  {error}")


if __name__ == '__main__':
    main()

