# Path and File Name : /home/ransomeye/rebuild/ransomeye_installer/crypto/identity_generator.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Generates per-install cryptographic identity for RansomEye instance

"""
Identity Generator: Generates per-install cryptographic identity.
Creates unique identity for each RansomEye installation.
"""

import os
import json
from pathlib import Path
from datetime import datetime
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.backends import default_backend
import hashlib


class IdentityGenerator:
    """Generates cryptographic identity for installation."""
    
    IDENTITY_DIR = Path("/home/ransomeye/rebuild/ransomeye_installer/crypto")
    IDENTITY_KEY_FILE = IDENTITY_DIR / "install_identity.key"
    IDENTITY_CERT_FILE = IDENTITY_DIR / "install_identity.crt"
    IDENTITY_METADATA_FILE = IDENTITY_DIR / "install_identity.json"
    
    def __init__(self):
        self.IDENTITY_DIR.mkdir(parents=True, exist_ok=True)
    
    def generate_identity(self) -> dict:
        """
        Generate new installation identity.
        
        Returns:
            Dictionary with identity information
        """
        # Generate RSA-4096 key pair
        private_key = rsa.generate_private_key(
            public_exponent=65537,
            key_size=4096,
            backend=default_backend()
        )
        
        # Get public key
        public_key = private_key.public_key()
        
        # Generate identity hash
        public_key_bytes = public_key.public_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PublicFormat.SubjectPublicKeyInfo
        )
        identity_hash = hashlib.sha256(public_key_bytes).hexdigest()
        
        # Create metadata
        metadata = {
            'identity_hash': identity_hash,
            'timestamp': datetime.utcnow().isoformat(),
            'key_size': 4096,
            'algorithm': 'RSA',
            'public_exponent': 65537
        }
        
        # Save private key
        with open(self.IDENTITY_KEY_FILE, 'wb') as f:
            f.write(private_key.private_bytes(
                encoding=serialization.Encoding.PEM,
                format=serialization.PrivateFormat.PKCS8,
                encryption_algorithm=serialization.NoEncryption()
            ))
        os.chmod(self.IDENTITY_KEY_FILE, 0o600)
        
        # Save public key (as certificate-like format)
        with open(self.IDENTITY_CERT_FILE, 'wb') as f:
            f.write(public_key_bytes)
        os.chmod(self.IDENTITY_CERT_FILE, 0o644)
        
        # Save metadata
        with open(self.IDENTITY_METADATA_FILE, 'w') as f:
            json.dump(metadata, f, indent=2)
        
        return metadata
    
    def identity_exists(self) -> bool:
        """Check if identity already exists."""
        return (self.IDENTITY_KEY_FILE.exists() and 
                self.IDENTITY_CERT_FILE.exists() and 
                self.IDENTITY_METADATA_FILE.exists())
    
    def get_identity_hash(self) -> str:
        """Get existing identity hash."""
        if not self.identity_exists():
            raise ValueError("Identity does not exist")
        
        with open(self.IDENTITY_METADATA_FILE, 'r') as f:
            metadata = json.load(f)
        
        return metadata.get('identity_hash', '')


def main():
    """CLI entry point for identity generator."""
    generator = IdentityGenerator()
    
    if generator.identity_exists():
        print("✓ Identity already exists")
        print(f"  Identity Hash: {generator.get_identity_hash()}")
    else:
        metadata = generator.generate_identity()
        print("✓ Identity generated")
        print(f"  Identity Hash: {metadata['identity_hash']}")
        print(f"  Key File: {generator.IDENTITY_KEY_FILE}")
        print(f"  Cert File: {generator.IDENTITY_CERT_FILE}")


if __name__ == '__main__':
    main()

