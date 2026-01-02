# Path and File Name : /home/ransomeye/rebuild/ransomeye_trust/bootstrap_config_signing.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Bootstrap Ed25519 config signing keys during installation - generates keypair if missing, enforces permissions, creates trust metadata

"""
Config Signing Key Bootstrap: Generates Ed25519 keypair for configuration signing.

This module is called during installation to bootstrap the config signing key
if it doesn't exist. It enforces strict permissions and creates trust metadata.
"""

import os
import sys
import json
import hashlib
from pathlib import Path
from datetime import datetime
from typing import Dict, Optional, Tuple
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric import ed25519
from cryptography.hazmat.backends import default_backend


class ConfigSigningBootstrap:
    """Bootstrap Ed25519 config signing keys."""
    
    def __init__(self, trust_dir: str = "/home/ransomeye/rebuild/ransomeye_trust"):
        self.trust_dir = Path(trust_dir)
        self.keys_dir = self.trust_dir / "keys"
        self.private_key_path = self.keys_dir / "config_signing.key"
        self.public_key_path = self.keys_dir / "config_signing.pub"
        self.bootstrap_metadata_path = Path("/var/lib/ransomeye/trust_bootstrap.json")
    
    def key_exists(self) -> bool:
        """Check if config signing key already exists."""
        return self.private_key_path.exists() and self.public_key_path.exists()
    
    def verify_permissions(self) -> Tuple[bool, str]:
        """
        Verify key permissions are correct.
        
        Returns:
            Tuple of (is_valid: bool, error_message: str)
        """
        if not self.private_key_path.exists():
            return False, "Private key does not exist"
        
        if not self.public_key_path.exists():
            return False, "Public key does not exist"
        
        # Check private key permissions (must be 600)
        private_key_stat = os.stat(self.private_key_path)
        private_key_mode = oct(private_key_stat.st_mode)[-3:]
        if private_key_mode != '600':
            return False, f"Private key permissions incorrect: {private_key_mode} (expected 600)"
        
        # Check public key permissions (must be 644)
        public_key_stat = os.stat(self.public_key_path)
        public_key_mode = oct(public_key_stat.st_mode)[-3:]
        if public_key_mode != '644':
            return False, f"Public key permissions incorrect: {public_key_mode} (expected 644)"
        
        # Check directory permissions (must be 700)
        keys_dir_stat = os.stat(self.keys_dir)
        keys_dir_mode = oct(keys_dir_stat.st_mode)[-3:]
        if keys_dir_mode != '700':
            return False, f"Keys directory permissions incorrect: {keys_dir_mode} (expected 700)"
        
        return True, "Permissions verified"
    
    def generate_keypair(self) -> Dict:
        """
        Generate Ed25519 keypair.
        
        Returns:
            Dictionary with key metadata including fingerprint
        """
        # Ensure keys directory exists
        self.keys_dir.mkdir(parents=True, exist_ok=True)
        
        # Set directory permissions to 700
        os.chmod(self.keys_dir, 0o700)
        
        # Generate Ed25519 keypair
        private_key = ed25519.Ed25519PrivateKey.generate()
        public_key = private_key.public_key()
        
        # Serialize private key (PEM format, no passphrase)
        private_key_pem = private_key.private_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PrivateFormat.PKCS8,
            encryption_algorithm=serialization.NoEncryption()
        )
        
        # Serialize public key (PEM format)
        public_key_pem = public_key.public_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PublicFormat.SubjectPublicKeyInfo
        )
        
        # Write private key
        with open(self.private_key_path, 'wb') as f:
            f.write(private_key_pem)
        
        # Write public key
        with open(self.public_key_path, 'wb') as f:
            f.write(public_key_pem)
        
        # Enforce permissions
        os.chmod(self.private_key_path, 0o600)
        os.chmod(self.public_key_path, 0o644)
        
        # Compute public key fingerprint (SHA-256 of public key bytes)
        public_key_bytes = public_key.public_bytes(
            encoding=serialization.Encoding.Raw,
            format=serialization.PublicFormat.Raw
        )
        fingerprint = hashlib.sha256(public_key_bytes).hexdigest()
        
        # Create metadata
        metadata = {
            'algorithm': 'Ed25519',
            'fingerprint': fingerprint,
            'fingerprint_format': 'SHA-256',
            'created_at': datetime.utcnow().isoformat(),
            'private_key_path': str(self.private_key_path),
            'public_key_path': str(self.public_key_path),
            'key_size': 256,  # Ed25519 is 256-bit
            'bootstrap_version': '1.0.0'
        }
        
        return metadata
    
    def save_bootstrap_metadata(self, metadata: Dict, installer_version: str = "1.0.0") -> Path:
        """
        Save bootstrap metadata to /var/lib/ransomeye/trust_bootstrap.json.
        
        Args:
            metadata: Key metadata dictionary
            installer_version: Installer version that created the keys
        
        Returns:
            Path to metadata file
        """
        bootstrap_data = {
            'bootstrap_timestamp': datetime.utcnow().isoformat(),
            'installer_version': installer_version,
            'key_algorithm': metadata['algorithm'],
            'public_key_fingerprint': metadata['fingerprint'],
            'fingerprint_format': metadata['fingerprint_format'],
            'key_created_at': metadata['created_at'],
            'private_key_path': metadata['private_key_path'],
            'public_key_path': metadata['public_key_path'],
            'key_size_bits': metadata['key_size']
        }
        
        # Ensure directory exists
        self.bootstrap_metadata_path.parent.mkdir(parents=True, exist_ok=True)
        
        # Write metadata (read-only after creation)
        with open(self.bootstrap_metadata_path, 'w') as f:
            json.dump(bootstrap_data, f, indent=2)
        
        # Set read-only permissions (444)
        os.chmod(self.bootstrap_metadata_path, 0o444)
        
        return self.bootstrap_metadata_path
    
    def bootstrap(self, installer_version: str = "1.0.0") -> Tuple[bool, Optional[Dict], str]:
        """
        Bootstrap config signing keys if missing.
        
        Args:
            installer_version: Installer version
        
        Returns:
            Tuple of (success: bool, metadata: Optional[Dict], message: str)
        """
        # Check if key exists
        if self.key_exists():
            # Verify permissions
            is_valid, error = self.verify_permissions()
            if not is_valid:
                return False, None, f"Existing key found but permissions invalid: {error}"
            
            # Load existing metadata if available
            metadata = None
            if self.bootstrap_metadata_path.exists():
                try:
                    with open(self.bootstrap_metadata_path, 'r') as f:
                        metadata = json.load(f)
                except Exception:
                    pass
            
            return True, metadata, "Config signing key found and verified"
        
        # Generate new keypair
        try:
            metadata = self.generate_keypair()
            self.save_bootstrap_metadata(metadata, installer_version)
            
            # Verify permissions after generation
            is_valid, error = self.verify_permissions()
            if not is_valid:
                return False, None, f"Key generated but permissions invalid: {error}"
            
            return True, metadata, "Config signing key generated successfully"
        except Exception as e:
            return False, None, f"Failed to generate config signing key: {str(e)}"


def main():
    """CLI entry point for config signing key bootstrap."""
    import argparse
    
    parser = argparse.ArgumentParser(description='Bootstrap RansomEye config signing keys')
    parser.add_argument('--trust-dir', default='/home/ransomeye/rebuild/ransomeye_trust',
                       help='Trust directory path')
    parser.add_argument('--installer-version', default='1.0.0',
                       help='Installer version')
    parser.add_argument('--check-only', action='store_true',
                       help='Only check if key exists, do not generate')
    
    args = parser.parse_args()
    
    bootstrap = ConfigSigningBootstrap(trust_dir=args.trust_dir)
    
    if args.check_only:
        if bootstrap.key_exists():
            is_valid, error = bootstrap.verify_permissions()
            if is_valid:
                print("✓ Config signing key found and verified")
                sys.exit(0)
            else:
                print(f"✗ Config signing key found but invalid: {error}", file=sys.stderr)
                sys.exit(1)
        else:
            print("✗ Config signing key not found", file=sys.stderr)
            sys.exit(1)
    
    # Bootstrap keys
    success, metadata, message = bootstrap.bootstrap(installer_version=args.installer_version)
    
    if success:
        if metadata:
            print(f"✓ {message}")
            print(f"  Algorithm: {metadata['algorithm']}")
            print(f"  Fingerprint: {metadata['fingerprint']}")
        else:
            print(f"✓ {message}")
        sys.exit(0)
    else:
        print(f"✗ {message}", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()

