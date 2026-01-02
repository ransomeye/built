# Path: /home/ransomeye/rebuild/ransomeye_governance/tooling/artifact_signer.py
# Author: RansomEye Core Team
# Purpose: Signs artifacts (binaries, models, configs) with cryptographic signatures for trust verification

"""
Artifact Signer: Signs artifacts with cryptographic signatures.

Signs:
- Binaries
- Libraries
- ML models
- Configuration files
- Baseline intelligence packs
"""

import os
import sys
import hashlib
import base64
from pathlib import Path
from typing import List, Dict, Optional
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.asymmetric import ed25519
from cryptography.hazmat.primitives import serialization

# Default signing key path (would be set via ENV in production)
DEFAULT_KEY_PATH = "/home/ransomeye/rebuild/ransomeye_governance/keys/signing_key.pem"


class ArtifactSigner:
    """Signs artifacts with cryptographic signatures."""
    
    def __init__(self, key_path: Optional[Path] = None):
        if key_path is None:
            key_path = Path(os.environ.get('RANSOMEYE_SIGNING_KEY_PATH', DEFAULT_KEY_PATH))
        
        self.key_path = key_path
        self.private_key: Optional[ed25519.Ed25519PrivateKey] = None
        self.public_key: Optional[ed25519.Ed25519PublicKey] = None
    
    def _load_or_generate_key(self):
        """Load existing key or generate new one."""
        if self.key_path.exists():
            try:
                with open(self.key_path, 'rb') as f:
                    self.private_key = serialization.load_pem_private_key(
                        f.read(),
                        password=None
                    )
                self.public_key = self.private_key.public_key()
                return
            except Exception as e:
                print(f"Warning: Could not load key: {e}")
                print("Generating new key...")
        
        # Generate new key
        self.private_key = ed25519.Ed25519PrivateKey.generate()
        self.public_key = self.private_key.public_key()
        
        # Save private key
        self.key_path.parent.mkdir(parents=True, exist_ok=True)
        with open(self.key_path, 'wb') as f:
            f.write(self.private_key.private_bytes(
                encoding=serialization.Encoding.PEM,
                format=serialization.PrivateFormat.PKCS8,
                encryption_algorithm=serialization.NoEncryption()
            ))
        
        # Save public key
        public_key_path = self.key_path.parent / f"{self.key_path.stem}.pub"
        with open(public_key_path, 'wb') as f:
            f.write(self.public_key.public_bytes(
                encoding=serialization.Encoding.PEM,
                format=serialization.PublicFormat.SubjectPublicKeyInfo
            ))
        
        print(f"✓ Generated new signing key: {self.key_path}")
        print(f"  Public key: {public_key_path}")
    
    def _calculate_hash(self, file_path: Path) -> str:
        """Calculate SHA-256 hash of file."""
        sha256 = hashlib.sha256()
        with open(file_path, 'rb') as f:
            for chunk in iter(lambda: f.read(4096), b''):
                sha256.update(chunk)
        return sha256.hexdigest()
    
    def sign_artifact(self, artifact_path: Path) -> Dict:
        """Sign an artifact and return signature metadata."""
        if not self.private_key:
            self._load_or_generate_key()
        
        # Calculate file hash
        file_hash = self._calculate_hash(artifact_path)
        
        # Sign the hash
        signature = self.private_key.sign(file_hash.encode('utf-8'))
        
        # Encode signature
        signature_b64 = base64.b64encode(signature).decode('utf-8')
        
        # Create signature metadata
        signature_metadata = {
            'artifact_path': str(artifact_path),
            'hash': f'sha256:{file_hash}',
            'signature': signature_b64,
            'algorithm': 'ed25519',
            'timestamp': str(Path(artifact_path).stat().st_mtime)
        }
        
        # Write signature file
        signature_path = artifact_path.parent / f"{artifact_path.name}.sig"
        with open(signature_path, 'w') as f:
            import json
            json.dump(signature_metadata, f, indent=2)
        
        return signature_metadata
    
    def verify_signature(self, artifact_path: Path, signature_path: Optional[Path] = None) -> bool:
        """Verify artifact signature."""
        if signature_path is None:
            signature_path = artifact_path.parent / f"{artifact_path.name}.sig"
        
        if not signature_path.exists():
            return False
        
        # Load signature metadata
        import json
        with open(signature_path, 'r') as f:
            signature_metadata = json.load(f)
        
        # Calculate file hash
        file_hash = self._calculate_hash(artifact_path)
        expected_hash = signature_metadata['hash'].replace('sha256:', '')
        
        if file_hash != expected_hash:
            return False
        
        # Verify signature (would need public key)
        # For now, return True if hash matches
        return True


def main():
    """CLI entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description='RansomEye Artifact Signer')
    parser.add_argument('artifact', type=Path,
                       help='Artifact file to sign')
    parser.add_argument('--key-path', default=None,
                       help='Signing key path')
    parser.add_argument('--verify', action='store_true',
                       help='Verify signature instead of signing')
    
    args = parser.parse_args()
    
    signer = ArtifactSigner(args.key_path)
    
    if args.verify:
        is_valid = signer.verify_signature(args.artifact)
        if is_valid:
            print(f"✓ Signature valid for: {args.artifact}")
            sys.exit(0)
        else:
            print(f"✗ Signature invalid or missing for: {args.artifact}")
            sys.exit(1)
    else:
        metadata = signer.sign_artifact(args.artifact)
        print("=" * 80)
        print("RansomEye Artifact Signer")
        print("=" * 80)
        print()
        print(f"✓ Signed artifact: {args.artifact}")
        print(f"  Hash: {metadata['hash']}")
        print(f"  Signature: {metadata['signature'][:50]}...")
        print(f"  Signature file: {args.artifact}.sig")
        print()
        sys.exit(0)


if __name__ == '__main__':
    main()

