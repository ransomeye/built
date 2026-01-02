# Path and File Name : /home/ransomeye/rebuild/ransomeye_installer/crypto/keystore.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Secure key storage and management for installation identity

"""
Keystore: Secure key storage and management.
Manages installation identity keys with proper permissions.
"""

import os
from pathlib import Path
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.backends import default_backend


class Keystore:
    """Secure key storage manager."""
    
    KEYSTORE_DIR = Path("/home/ransomeye/rebuild/ransomeye_installer/crypto")
    
    def __init__(self):
        self.KEYSTORE_DIR.mkdir(parents=True, exist_ok=True)
    
    def load_identity_key(self) -> bytes:
        """
        Load installation identity private key.
        
        Returns:
            Private key bytes
        """
        key_file = self.KEYSTORE_DIR / "install_identity.key"
        
        if not key_file.exists():
            raise FileNotFoundError(f"Identity key not found: {key_file}")
        
        with open(key_file, 'rb') as f:
            return f.read()
    
    def verify_key_permissions(self) -> bool:
        """Verify key files have correct permissions."""
        key_file = self.KEYSTORE_DIR / "install_identity.key"
        cert_file = self.KEYSTORE_DIR / "install_identity.crt"
        
        if not key_file.exists() or not cert_file.exists():
            return False
        
        # Check key file permissions (should be 600)
        key_stat = os.stat(key_file)
        key_mode = oct(key_stat.st_mode)[-3:]
        if key_mode != '600':
            return False
        
        # Check cert file permissions (should be 644)
        cert_stat = os.stat(cert_file)
        cert_mode = oct(cert_stat.st_mode)[-3:]
        if cert_mode != '644':
            return False
        
        return True


def main():
    """CLI entry point for keystore."""
    keystore = Keystore()
    
    try:
        key = keystore.load_identity_key()
        print("✓ Identity key loaded")
        
        if keystore.verify_key_permissions():
            print("✓ Key permissions correct")
        else:
            print("✗ Key permissions incorrect", file=sys.stderr)
            sys.exit(1)
    except FileNotFoundError as e:
        print(f"✗ {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    import sys
    main()

