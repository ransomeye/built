# Path and File Name : /home/ransomeye/rebuild/core/global_validator/manifest_verifier.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Cryptographic verification of install_manifest.json signature using Ed25519

"""
Manifest Verifier: Verifies Ed25519 signature of install_manifest.json.

Security Properties:
- FAIL-CLOSED on ANY error
- NO fallback behavior
- NO best-effort warnings
- Verification MUST succeed before manifest is trusted

Verification Order (MANDATORY):
1. Check manifest exists
2. Check signature exists
3. Check public key exists
4. Load public key
5. Read manifest (canonicalized)
6. Verify signature
7. ONLY THEN: trust manifest content
"""

import sys
import json
from pathlib import Path
from typing import Tuple

try:
    from cryptography.hazmat.primitives.asymmetric import ed25519
    from cryptography.hazmat.primitives import serialization
    from cryptography.exceptions import InvalidSignature
except ImportError:
    print("ERROR: cryptography library not installed", file=sys.stderr)
    print("  Install with: pip3 install cryptography", file=sys.stderr)
    sys.exit(1)


class ManifestVerifier:
    """Cryptographic verifier for install_manifest.json."""
    
    PUBLIC_KEY_PATH = Path("/var/lib/ransomeye/keys/manifest_signing.pub")
    MANIFEST_PATH = Path("/var/lib/ransomeye/install_manifest.json")
    SIGNATURE_PATH = Path("/var/lib/ransomeye/install_manifest.sig")
    
    def __init__(self):
        """Initialize manifest verifier."""
        self.errors = []
    
    def verify_signature(self) -> Tuple[bool, str]:
        """
        Verify manifest signature.
        
        Returns:
            Tuple of (is_valid, error_message)
            - (True, "Signature valid") on success
            - (False, error_message) on failure
        
        FAIL-CLOSED: Any error returns False
        """
        # Check 1: Manifest exists
        if not self.MANIFEST_PATH.exists():
            return False, f"Manifest not found: {self.MANIFEST_PATH}"
        
        # Check 2: Signature exists
        if not self.SIGNATURE_PATH.exists():
            return False, f"Signature not found: {self.SIGNATURE_PATH}"
        
        # Check 3: Public key exists
        if not self.PUBLIC_KEY_PATH.exists():
            return False, f"Public key not found: {self.PUBLIC_KEY_PATH}"
        
        # Check 4: Load public key
        try:
            public_pem = self.PUBLIC_KEY_PATH.read_bytes()
            public_key = serialization.load_pem_public_key(public_pem)
            
            if not isinstance(public_key, ed25519.Ed25519PublicKey):
                return False, "Public key is not Ed25519 format"
            
        except Exception as e:
            return False, f"Failed to load public key: {e}"
        
        # Check 5: Read and canonicalize manifest
        try:
            with open(self.MANIFEST_PATH, 'r') as f:
                manifest_data = json.load(f)
            
            # Serialize deterministically (must match signing format)
            manifest_canonical = json.dumps(manifest_data, sort_keys=True, separators=(',', ':'))
            manifest_bytes = manifest_canonical.encode('utf-8')
            
        except Exception as e:
            return False, f"Failed to read manifest: {e}"
        
        # Check 6: Read signature
        try:
            signature = self.SIGNATURE_PATH.read_bytes()
            
            if len(signature) != 64:
                return False, f"Invalid signature length: {len(signature)} (expected 64)"
            
        except Exception as e:
            return False, f"Failed to read signature: {e}"
        
        # Check 7: Verify signature
        try:
            public_key.verify(signature, manifest_bytes)
            
            # If we reach here, signature is valid
            return True, "Signature valid"
            
        except InvalidSignature:
            return False, "Signature verification FAILED - manifest has been tampered with"
        except Exception as e:
            return False, f"Signature verification error: {e}"
    
    def verify_or_abort(self) -> None:
        """
        Verify signature and abort on failure.
        
        FAIL-CLOSED: Calls sys.exit(1) on any verification failure.
        """
        is_valid, message = self.verify_signature()
        
        if is_valid:
            print(f"✓ Manifest signature verified: {message}")
        else:
            print(f"FATAL: Manifest signature verification FAILED", file=sys.stderr)
            print(f"  {message}", file=sys.stderr)
            print(f"", file=sys.stderr)
            print(f"  Manifest: {self.MANIFEST_PATH}", file=sys.stderr)
            print(f"  Signature: {self.SIGNATURE_PATH}", file=sys.stderr)
            print(f"  Public key: {self.PUBLIC_KEY_PATH}", file=sys.stderr)
            print(f"", file=sys.stderr)
            print(f"  This indicates:", file=sys.stderr)
            print(f"    - Manifest has been modified after signing", file=sys.stderr)
            print(f"    - Signature file is missing or corrupted", file=sys.stderr)
            print(f"    - Public key is missing or corrupted", file=sys.stderr)
            print(f"", file=sys.stderr)
            print(f"  Action required: Reinstall RansomEye to restore integrity", file=sys.stderr)
            print(f"", file=sys.stderr)
            sys.exit(1)


def main():
    """CLI entry point for manifest verifier."""
    verifier = ManifestVerifier()
    
    is_valid, message = verifier.verify_signature()
    
    if is_valid:
        print(f"✓ Manifest signature verification PASSED")
        print(f"  {message}")
        sys.exit(0)
    else:
        print(f"✗ Manifest signature verification FAILED", file=sys.stderr)
        print(f"  {message}", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()

