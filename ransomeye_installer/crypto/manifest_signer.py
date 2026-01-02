# Path and File Name : /home/ransomeye/rebuild/ransomeye_installer/crypto/manifest_signer.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Cryptographic signing of install_manifest.json using Ed25519

"""
Manifest Signer: Cryptographically signs install_manifest.json using Ed25519.

Security Properties:
- Algorithm: Ed25519 (EdDSA over Curve25519)
- Key size: 256 bits (32 bytes)
- Signature size: 64 bytes
- Deterministic signatures (no RNG dependency)
- Fail-closed on any error

Key Material:
- Private key: /var/lib/ransomeye/keys/manifest_signing.key (600, root-owned)
- Public key: /var/lib/ransomeye/keys/manifest_signing.pub (644)
- Signature: /var/lib/ransomeye/install_manifest.sig (644)
"""

import os
import sys
import json
from pathlib import Path
from typing import Tuple, Optional

try:
    from cryptography.hazmat.primitives.asymmetric import ed25519
    from cryptography.hazmat.primitives import serialization
except ImportError:
    print("ERROR: cryptography library not installed", file=sys.stderr)
    print("  Install with: pip3 install cryptography", file=sys.stderr)
    sys.exit(1)


class ManifestSigner:
    """Cryptographic signer for install_manifest.json."""
    
    KEY_DIR = Path("/var/lib/ransomeye/keys")
    PRIVATE_KEY_PATH = KEY_DIR / "manifest_signing.key"
    PUBLIC_KEY_PATH = KEY_DIR / "manifest_signing.pub"
    MANIFEST_PATH = Path("/var/lib/ransomeye/install_manifest.json")
    SIGNATURE_PATH = Path("/var/lib/ransomeye/install_manifest.sig")
    
    def __init__(self):
        """Initialize manifest signer."""
        pass
    
    def generate_keypair(self) -> Tuple[ed25519.Ed25519PrivateKey, ed25519.Ed25519PublicKey]:
        """
        Generate Ed25519 keypair.
        
        Returns:
            Tuple of (private_key, public_key)
        
        Raises:
            RuntimeError: If key generation fails
        """
        try:
            # Generate private key
            private_key = ed25519.Ed25519PrivateKey.generate()
            
            # Derive public key
            public_key = private_key.public_key()
            
            return private_key, public_key
        except Exception as e:
            raise RuntimeError(f"Failed to generate Ed25519 keypair: {e}")
    
    def save_keypair(self, private_key: ed25519.Ed25519PrivateKey, 
                     public_key: ed25519.Ed25519PublicKey) -> None:
        """
        Save keypair to disk with correct permissions.
        
        Args:
            private_key: Ed25519 private key
            public_key: Ed25519 public key
        
        Raises:
            RuntimeError: If save fails
        """
        try:
            # Ensure key directory exists
            self.KEY_DIR.mkdir(parents=True, exist_ok=True, mode=0o755)
            
            # Serialize private key (PEM format, no encryption - protected by filesystem perms)
            private_pem = private_key.private_bytes(
                encoding=serialization.Encoding.PEM,
                format=serialization.PrivateFormat.PKCS8,
                encryption_algorithm=serialization.NoEncryption()
            )
            
            # Write private key (600 permissions)
            self.PRIVATE_KEY_PATH.write_bytes(private_pem)
            os.chmod(self.PRIVATE_KEY_PATH, 0o600)
            
            # Ensure root ownership
            os.chown(self.PRIVATE_KEY_PATH, 0, 0)
            
            # Serialize public key (PEM format)
            public_pem = public_key.public_bytes(
                encoding=serialization.Encoding.PEM,
                format=serialization.PublicFormat.SubjectPublicKeyInfo
            )
            
            # Write public key (644 permissions)
            self.PUBLIC_KEY_PATH.write_bytes(public_pem)
            os.chmod(self.PUBLIC_KEY_PATH, 0o644)
            
            print(f"[SIGNER] Keypair saved:")
            print(f"  Private key: {self.PRIVATE_KEY_PATH} (600)")
            print(f"  Public key: {self.PUBLIC_KEY_PATH} (644)")
            
        except Exception as e:
            raise RuntimeError(f"Failed to save keypair: {e}")
    
    def load_private_key(self) -> ed25519.Ed25519PrivateKey:
        """
        Load private key from disk.
        
        Returns:
            Ed25519 private key
        
        Raises:
            RuntimeError: If load fails or key doesn't exist
        """
        if not self.PRIVATE_KEY_PATH.exists():
            raise RuntimeError(f"Private key not found: {self.PRIVATE_KEY_PATH}")
        
        try:
            private_pem = self.PRIVATE_KEY_PATH.read_bytes()
            private_key = serialization.load_pem_private_key(
                private_pem,
                password=None
            )
            
            if not isinstance(private_key, ed25519.Ed25519PrivateKey):
                raise RuntimeError("Key is not Ed25519 format")
            
            return private_key
        except Exception as e:
            raise RuntimeError(f"Failed to load private key: {e}")
    
    def ensure_keypair_exists(self) -> None:
        """
        Ensure keypair exists. Generate if missing, reuse if present.
        
        Raises:
            RuntimeError: If keypair cannot be ensured
        """
        if self.PRIVATE_KEY_PATH.exists() and self.PUBLIC_KEY_PATH.exists():
            print(f"[SIGNER] Reusing existing keypair")
            print(f"  Private key: {self.PRIVATE_KEY_PATH}")
            print(f"  Public key: {self.PUBLIC_KEY_PATH}")
            
            # Verify private key is valid
            try:
                self.load_private_key()
                print(f"[SIGNER] Existing private key validated")
            except Exception as e:
                raise RuntimeError(f"Existing private key is invalid: {e}")
        else:
            print(f"[SIGNER] Generating new Ed25519 keypair...")
            private_key, public_key = self.generate_keypair()
            self.save_keypair(private_key, public_key)
            print(f"[SIGNER] Keypair generated successfully")
    
    def sign_manifest(self) -> None:
        """
        Sign install_manifest.json and write signature.
        
        Raises:
            RuntimeError: If signing fails
        """
        # Validate manifest exists
        if not self.MANIFEST_PATH.exists():
            raise RuntimeError(f"Manifest not found: {self.MANIFEST_PATH}")
        
        # Load private key
        try:
            private_key = self.load_private_key()
        except Exception as e:
            raise RuntimeError(f"Cannot load private key for signing: {e}")
        
        # Read and canonicalize manifest
        try:
            with open(self.MANIFEST_PATH, 'r') as f:
                manifest_data = json.load(f)
            
            # Serialize deterministically (sorted keys, no whitespace)
            manifest_canonical = json.dumps(manifest_data, sort_keys=True, separators=(',', ':'))
            manifest_bytes = manifest_canonical.encode('utf-8')
            
            print(f"[SIGNER] Manifest size: {len(manifest_bytes)} bytes")
            
        except Exception as e:
            raise RuntimeError(f"Failed to read manifest: {e}")
        
        # Sign manifest
        try:
            signature = private_key.sign(manifest_bytes)
            
            if len(signature) != 64:
                raise RuntimeError(f"Invalid signature length: {len(signature)} (expected 64)")
            
            print(f"[SIGNER] Signature computed: {len(signature)} bytes")
            
        except Exception as e:
            raise RuntimeError(f"Failed to sign manifest: {e}")
        
        # Write signature
        try:
            self.SIGNATURE_PATH.write_bytes(signature)
            os.chmod(self.SIGNATURE_PATH, 0o644)
            
            print(f"[SIGNER] Signature written: {self.SIGNATURE_PATH} (644)")
            
        except Exception as e:
            raise RuntimeError(f"Failed to write signature: {e}")
    
    def get_public_key_fingerprint(self) -> str:
        """
        Get SHA-256 fingerprint of public key.
        
        Returns:
            Hex-encoded fingerprint
        """
        import hashlib
        
        public_pem = self.PUBLIC_KEY_PATH.read_bytes()
        fingerprint = hashlib.sha256(public_pem).hexdigest()
        return fingerprint


def main():
    """CLI entry point for manifest signer."""
    import argparse
    
    parser = argparse.ArgumentParser(
        description='Sign install_manifest.json with Ed25519',
        formatter_class=argparse.RawDescriptionHelpFormatter
    )
    
    parser.add_argument(
        '--generate-keys',
        action='store_true',
        help='Generate new keypair (or reuse existing)'
    )
    
    parser.add_argument(
        '--sign',
        action='store_true',
        help='Sign manifest'
    )
    
    parser.add_argument(
        '--fingerprint',
        action='store_true',
        help='Show public key fingerprint'
    )
    
    args = parser.parse_args()
    
    signer = ManifestSigner()
    
    try:
        if args.generate_keys:
            signer.ensure_keypair_exists()
        
        if args.sign:
            signer.sign_manifest()
        
        if args.fingerprint:
            fingerprint = signer.get_public_key_fingerprint()
            print(f"Public key fingerprint (SHA-256):")
            print(f"  {fingerprint}")
        
        if not (args.generate_keys or args.sign or args.fingerprint):
            # Default: ensure keys and sign
            signer.ensure_keypair_exists()
            signer.sign_manifest()
        
        sys.exit(0)
        
    except Exception as e:
        print(f"ERROR: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()

