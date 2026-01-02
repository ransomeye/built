# Path and File Name : /home/ransomeye/rebuild/core/install_state/state_signer.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Ed25519 signing and verification for install state

"""
Install State Cryptographic Signing

Provides Ed25519 signature generation and verification for install_state.json
to ensure tamper-proof installation state tracking.
"""

import os
import sys
import json
import hashlib
from pathlib import Path
from typing import Tuple, Optional

try:
    from cryptography.hazmat.primitives.asymmetric.ed25519 import (
        Ed25519PrivateKey,
        Ed25519PublicKey
    )
    from cryptography.hazmat.primitives import serialization
    from cryptography.exceptions import InvalidSignature
except ImportError:
    print("ERROR: cryptography package not installed", file=sys.stderr)
    sys.exit(1)


class StateSignerError(Exception):
    """Raised when signing or verification fails"""
    pass


def load_signing_keypair(private_key_path: str) -> Tuple[Ed25519PrivateKey, Ed25519PublicKey]:
    """
    Load Ed25519 keypair from PEM file.
    
    Args:
        private_key_path: Path to PEM-encoded private key
        
    Returns:
        Tuple of (private_key, public_key)
        
    Raises:
        StateSignerError: If key cannot be loaded
    """
    try:
        key_path = Path(private_key_path)
        
        if not key_path.exists():
            raise StateSignerError(f"Private key not found: {private_key_path}")
        
        with open(key_path, 'rb') as f:
            private_key = serialization.load_pem_private_key(
                f.read(),
                password=None
            )
        
        if not isinstance(private_key, Ed25519PrivateKey):
            raise StateSignerError("Key is not Ed25519 format")
        
        public_key = private_key.public_key()
        
        return private_key, public_key
    
    except Exception as e:
        raise StateSignerError(f"Failed to load signing key: {e}")


def load_public_key(public_key_path: str) -> Ed25519PublicKey:
    """
    Load Ed25519 public key from PEM file.
    
    Args:
        public_key_path: Path to PEM-encoded public key
        
    Returns:
        Ed25519PublicKey instance
        
    Raises:
        StateSignerError: If key cannot be loaded
    """
    try:
        key_path = Path(public_key_path)
        
        if not key_path.exists():
            raise StateSignerError(f"Public key not found: {public_key_path}")
        
        with open(key_path, 'rb') as f:
            public_key = serialization.load_pem_public_key(f.read())
        
        if not isinstance(public_key, Ed25519PublicKey):
            raise StateSignerError("Key is not Ed25519 format")
        
        return public_key
    
    except Exception as e:
        raise StateSignerError(f"Failed to load public key: {e}")


def compute_public_key_fingerprint(public_key: Ed25519PublicKey) -> str:
    """
    Compute SHA256 fingerprint of public key.
    
    Args:
        public_key: Ed25519PublicKey instance
        
    Returns:
        Hex-encoded SHA256 hash
    """
    public_bytes = public_key.public_bytes(
        encoding=serialization.Encoding.Raw,
        format=serialization.PublicFormat.Raw
    )
    
    return hashlib.sha256(public_bytes).hexdigest()


def sign_state_file(
    state_file_path: str,
    signature_output_path: str,
    private_key_path: str
) -> str:
    """
    Sign install_state.json with Ed25519 private key.
    
    Args:
        state_file_path: Path to install_state.json
        signature_output_path: Path to write signature file
        private_key_path: Path to private key PEM
        
    Returns:
        Public key fingerprint (hex)
        
    Raises:
        StateSignerError: If signing fails
    """
    try:
        # Load signing key
        private_key, public_key = load_signing_keypair(private_key_path)
        
        # Read state file
        state_path = Path(state_file_path)
        if not state_path.exists():
            raise StateSignerError(f"State file not found: {state_file_path}")
        
        with open(state_path, 'rb') as f:
            state_data = f.read()
        
        # Sign
        signature = private_key.sign(state_data)
        
        # Write signature
        sig_path = Path(signature_output_path)
        sig_path.parent.mkdir(parents=True, exist_ok=True)
        
        with open(sig_path, 'wb') as f:
            f.write(signature)
        
        # Make signature read-only
        os.chmod(sig_path, 0o444)
        
        # Compute fingerprint
        fingerprint = compute_public_key_fingerprint(public_key)
        
        print(f"✓ Signed install_state.json")
        print(f"  Signature: {signature_output_path}")
        print(f"  Fingerprint: {fingerprint}")
        
        return fingerprint
    
    except Exception as e:
        raise StateSignerError(f"Failed to sign state file: {e}")


def verify_state_signature(
    state_file_path: str,
    signature_file_path: str,
    public_key_path: str
) -> bool:
    """
    Verify install_state.json signature.
    
    Args:
        state_file_path: Path to install_state.json
        signature_file_path: Path to signature file
        public_key_path: Path to public key PEM
        
    Returns:
        True if signature valid, False otherwise
    """
    try:
        # Load public key
        public_key = load_public_key(public_key_path)
        
        # Read state file
        state_path = Path(state_file_path)
        if not state_path.exists():
            print(f"ERROR: State file not found: {state_file_path}", file=sys.stderr)
            return False
        
        with open(state_path, 'rb') as f:
            state_data = f.read()
        
        # Read signature
        sig_path = Path(signature_file_path)
        if not sig_path.exists():
            print(f"ERROR: Signature file not found: {signature_file_path}", file=sys.stderr)
            return False
        
        with open(sig_path, 'rb') as f:
            signature = f.read()
        
        # Verify
        public_key.verify(signature, state_data)
        
        return True
    
    except InvalidSignature:
        print("ERROR: Invalid signature on install_state.json", file=sys.stderr)
        return False
    
    except Exception as e:
        print(f"ERROR: Signature verification failed: {e}", file=sys.stderr)
        return False


def generate_installer_identity_hash() -> str:
    """
    Generate installer identity hash from system characteristics.
    
    Returns:
        SHA256 hex hash
    """
    identity_components = [
        str(os.getuid()),
        str(os.getgid()),
        os.uname().nodename,
        os.uname().machine
    ]
    
    identity_string = '|'.join(identity_components)
    
    return hashlib.sha256(identity_string.encode()).hexdigest()


if __name__ == '__main__':
    print("Install State Signer - RansomEye")
    print("This module is not meant to be run directly.")
    sys.exit(1)

