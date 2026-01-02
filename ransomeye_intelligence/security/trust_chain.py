# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/security/trust_chain.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Validates trust chain for intelligence artifacts - verifies certificate chain

"""
Trust Chain Validation: Validates trust chain for intelligence artifacts.
Verifies certificate chain against Root CA.
"""

import os
from pathlib import Path
from typing import Dict, List, Tuple, Optional
from ransomeye_trust.verify_tool import VerifyTool


class IntelligenceTrustChain:
    """Validates trust chain for intelligence artifacts."""
    
    def __init__(self, trust_dir: Optional[str] = None):
        # Runtime-safe default: trust tooling is deployed under /opt/ransomeye/modules/ransomeye_trust
        if trust_dir is None:
            runtime_root = Path(os.environ.get("RANSOMEYE_ROOT", "/opt/ransomeye"))
            trust_dir = str(runtime_root / "modules" / "ransomeye_trust")
        self.trust_dir = Path(trust_dir)
        self.verify_tool = VerifyTool(str(self.trust_dir))
    
    def validate_trust_chain(self, artifact_path: Path, domain: str = "artifacts") -> Tuple[bool, List[str]]:
        """
        Validate trust chain for artifact.
        
        Args:
            artifact_path: Path to artifact
            domain: Trust domain
        
        Returns:
            Tuple of (is_valid: bool, errors: List[str])
        """
        errors = []
        
        # Verify certificate chain
        try:
            certificate = self.verify_tool.load_certificate(domain)
            if not self.verify_tool.verify_certificate_chain(certificate):
                errors.append(f"Certificate chain validation failed for {artifact_path.name}")
        except Exception as e:
            errors.append(f"Error validating trust chain: {e}")
        
        return len(errors) == 0, errors


def main():
    """CLI entry point for trust chain validator."""
    trust_chain = IntelligenceTrustChain()
    
    # Example validation
    artifact_path = Path("/home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack/models/model_manifest.json")
    is_valid, errors = trust_chain.validate_trust_chain(artifact_path)
    
    if is_valid:
        print("✓ Trust chain validation passed")
    else:
        print("✗ Trust chain validation failed:")
        for error in errors:
            print(f"  {error}")


if __name__ == '__main__':
    main()

