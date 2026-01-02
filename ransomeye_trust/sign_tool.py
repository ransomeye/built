# Path and File Name : /home/ransomeye/rebuild/ransomeye_trust/sign_tool.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Generic signer for RansomEye artifacts - supports RSA-4096 for artifacts and Ed25519 for config domain

"""
Sign Tool: Generic signer for RansomEye artifacts.
Supports RSA-4096 for artifacts and Ed25519 for config domain.
Signs manifest.json files and creates manifest.sig files.
"""

import os
import json
import hashlib
from pathlib import Path
from typing import Dict, Tuple, Any, Optional, List, Union
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import padding, ed25519, rsa
from cryptography.hazmat.backends import default_backend


class SignTool:
    """Generic signer for RansomEye artifacts."""
    
    def __init__(self, trust_dir: str = "/home/ransomeye/rebuild/ransomeye_trust"):
        self.trust_dir = Path(trust_dir)
        self.keys_dir = self.trust_dir / "keys"
    
    def load_signing_key(self, domain: str = "artifacts") -> tuple:
        """
        Load signing key for a trust domain.
        
        Supports:
        - RSA-4096 for artifacts, agents, updates, reports domains
        - Ed25519 for config domain
        
        Returns:
            Tuple of (private_key, domain_name)
        """
        key_path = self.keys_dir / f"{domain}_signing.key"
        
        if not key_path.exists():
            raise FileNotFoundError(f"Signing key not found: {key_path}")
        
        with open(key_path, 'rb') as f:
            private_key = serialization.load_pem_private_key(
                f.read(),
                password=None,
                backend=default_backend()
            )
        
        return private_key, domain
    
    def sign_data(self, data: bytes, private_key, domain: str = "artifacts") -> bytes:
        """
        Sign data using appropriate algorithm based on key type.
        
        - Ed25519 for config domain (direct signing, no padding)
        - RSA-4096 with PSS padding for other domains
        
        Args:
            data: Data to sign
            private_key: Private key for signing
            domain: Trust domain (used to determine algorithm)
        
        Returns:
            Signature bytes
        """
        # Check if key is Ed25519 (config domain uses Ed25519)
        if isinstance(private_key, ed25519.Ed25519PrivateKey):
            # Ed25519: direct signing, no padding needed
            signature = private_key.sign(data)
            return signature
        elif isinstance(private_key, rsa.RSAPrivateKey):
            # RSA-4096: use PSS padding
            signature = private_key.sign(
                data,
                padding.PSS(
                    mgf=padding.MGF1(hashes.SHA256()),
                    salt_length=padding.PSS.MAX_LENGTH
                ),
                hashes.SHA256()
            )
            return signature
        else:
            raise ValueError(f"Unsupported key type for domain '{domain}': {type(private_key)}")
    
    def sign_file(self, file_path: Path, domain: str = "artifacts") -> Path:
        """
        Sign a file and create .sig file.
        
        Args:
            file_path: Path to file to sign
            domain: Trust domain for signing key
        
        Returns:
            Path to signature file
        """
        private_key, _ = self.load_signing_key(domain)
        
        # Read file content
        with open(file_path, 'rb') as f:
            file_data = f.read()
        
        # Sign file (pass domain for algorithm selection)
        signature = self.sign_data(file_data, private_key, domain=domain)
        
        # Save signature
        sig_path = file_path.parent / f"{file_path.stem}.sig"
        with open(sig_path, 'wb') as f:
            f.write(signature)
        
        os.chmod(sig_path, 0o644)
        return sig_path
    
    def sign_manifest(self, manifest_path: Path, domain: str = "artifacts") -> Path:
        """
        Sign manifest.json and create manifest.sig.
        
        Args:
            manifest_path: Path to manifest.json
            domain: Trust domain for signing key
        
        Returns:
            Path to manifest.sig
        """
        return self.sign_file(manifest_path, domain)
    
    def create_and_sign_manifest(self, artifact_path: Path, metadata: Dict, domain: str = "artifacts") -> tuple:
        """
        Create manifest.json for artifact and sign it.
        
        Args:
            artifact_path: Path to artifact file
            metadata: Additional metadata for manifest
            domain: Trust domain for signing key
        
        Returns:
            Tuple of (manifest_path, sig_path)
        """
        # Compute file hash
        sha256 = hashlib.sha256()
        with open(artifact_path, 'rb') as f:
            for chunk in iter(lambda: f.read(4096), b''):
                sha256.update(chunk)
        file_hash = sha256.hexdigest()
        
        # Create manifest
        manifest = {
            'hash': file_hash,
            'timestamp': metadata.get('timestamp', str(Path(artifact_path).stat().st_mtime)),
            'version': metadata.get('version', '1.0.0'),
            'signer': domain,
            'artifact': str(artifact_path.name),
            **{k: v for k, v in metadata.items() if k not in ['timestamp', 'version']}
        }
        
        # Save manifest
        manifest_path = artifact_path.parent / f"{artifact_path.stem}_manifest.json"
        with open(manifest_path, 'w') as f:
            json.dump(manifest, f, indent=2)
        
        # Sign manifest
        sig_path = self.sign_manifest(manifest_path, domain)
        
        return manifest_path, sig_path


def main():
    """CLI entry point for sign tool."""
    import argparse
    from datetime import datetime
    
    parser = argparse.ArgumentParser(description='RansomEye Sign Tool')
    parser.add_argument('file', type=Path, help='File to sign')
    parser.add_argument('--domain', default='artifacts', help='Trust domain')
    parser.add_argument('--manifest', action='store_true', help='Create and sign manifest.json')
    parser.add_argument('--version', default='1.0.0', help='Version for manifest')
    
    args = parser.parse_args()
    
    signer = SignTool()
    
    if args.manifest:
        metadata = {
            'timestamp': datetime.utcnow().isoformat(),
            'version': args.version
        }
        manifest_path, sig_path = signer.create_and_sign_manifest(args.file, metadata, args.domain)
        print(f"Created manifest: {manifest_path}")
        print(f"Created signature: {sig_path}")
    else:
        sig_path = signer.sign_file(args.file, args.domain)
        print(f"Signed file: {sig_path}")


if __name__ == '__main__':
    main()

