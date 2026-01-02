# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/intelligence_controller.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Main intelligence controller - ensures AI never starts empty, validates baseline pack, manages intelligence subsystems

"""
Intelligence Controller: Main controller for Intelligence Plane.
Ensures AI never starts empty, validates baseline pack, manages subsystems.
Fails-closed if baseline pack is missing or invalid.
"""

import sys
from pathlib import Path
from typing import Dict, Optional

from .baseline_pack.validator import BaselinePackValidator
from .baseline_pack.loader import BaselinePackLoader
from .security.signature_verification import IntelligenceSignatureVerifier
from .security.trust_chain import IntelligenceTrustChain
from .security.revocation import IntelligenceRevocation


class IntelligenceController:
    """Main controller for Intelligence Plane."""
    
    def __init__(self):
        self.baseline_validator = BaselinePackValidator()
        self.baseline_loader = BaselinePackLoader()
        self.signature_verifier = IntelligenceSignatureVerifier()
        self.trust_chain = IntelligenceTrustChain()
        self.revocation = IntelligenceRevocation()
        self.initialized = False
    
    def initialize(self) -> bool:
        """
        Initialize Intelligence Plane.
        Validates baseline pack and all intelligence artifacts.
        Fails-closed if validation fails.
        
        Returns:
            True if initialized successfully, False otherwise (and exits)
        """
        # Step 1: Validate baseline pack
        is_valid, errors, warnings = self.baseline_validator.validate()
        if not is_valid:
            print("✗ Baseline intelligence pack validation failed. AI cannot start.", file=sys.stderr)
            self.baseline_validator.report_errors()
            return False
        
        # Step 2: Verify signatures
        # Runtime-safe baseline pack path (no build-time paths):
        # The baseline pack resides inside the installed module directory at runtime.
        pack_dir = Path(__file__).resolve().parent / "baseline_pack"
        is_valid, errors = self.signature_verifier.verify_baseline_pack(pack_dir)
        if not is_valid:
            print("✗ Baseline pack signature verification failed. AI cannot start.", file=sys.stderr)
            for error in errors:
                print(f"  {error}", file=sys.stderr)
            sys.exit(1)
        
        # Step 3: Load baseline pack
        if not self.baseline_loader.load():
            print("✗ Failed to load baseline intelligence pack. AI cannot start.", file=sys.stderr)
            sys.exit(1)
        
        # Step 4: Verify trust chain
        manifest_path = pack_dir / "models" / "model_manifest.json"
        is_valid, errors = self.trust_chain.validate_trust_chain(manifest_path)
        if not is_valid:
            print("✗ Trust chain validation failed. AI cannot start.", file=sys.stderr)
            for error in errors:
                print(f"  {error}", file=sys.stderr)
            sys.exit(1)
        
        self.initialized = True
        print("✓ Intelligence Plane initialized successfully")
        return True
    
    def is_initialized(self) -> bool:
        """Check if Intelligence Plane is initialized."""
        return self.initialized
    
    def get_model(self, model_name: str):
        """Get model from baseline pack."""
        if not self.initialized:
            return None
        return self.baseline_loader.get_model(model_name)
    
    def get_shap_baseline(self, model_name: str) -> Optional[Dict]:
        """Get SHAP baseline for model."""
        if not self.initialized:
            return None
        return self.baseline_loader.get_shap_baseline(model_name)


def main():
    """CLI entry point for intelligence controller."""
    controller = IntelligenceController()
    
    if controller.initialize():
        print("✓ Intelligence Plane ready")
        sys.exit(0)
    else:
        print("✗ Intelligence Plane initialization failed", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()

