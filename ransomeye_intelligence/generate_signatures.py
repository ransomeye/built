# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/generate_signatures.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Generates cryptographic signatures for all intelligence packs

"""
Signature Generator: Generates cryptographic signatures for all intelligence packs.
Uses RSA-4096-PSS-SHA256 as specified.
"""

import os
import sys
import json
import hashlib
import base64
from pathlib import Path
from datetime import datetime
from typing import Dict, Tuple
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import rsa, padding
from cryptography.hazmat.backends import default_backend

INTELLIGENCE_DIR = Path("/home/ransomeye/rebuild/ransomeye_intelligence")
TRUST_DIR = Path("/home/ransomeye/rebuild/ransomeye_trust")
KEYS_DIR = TRUST_DIR / "keys"


def generate_signing_key(key_name: str) -> Tuple[rsa.RSAPrivateKey, bytes]:
    """
    Generate RSA-4096 signing key pair.
    
    Returns:
        Tuple of (private_key, public_key_bytes)
    """
    private_key = rsa.generate_private_key(
        public_exponent=65537,
        key_size=4096,
        backend=default_backend()
    )
    
    public_key = private_key.public_key()
    public_key_bytes = public_key.public_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PublicFormat.SubjectPublicKeyInfo
    )
    
    return private_key, public_key_bytes


def load_or_generate_key(key_name: str) -> Tuple[rsa.RSAPrivateKey, bytes]:
    """
    Load existing key or generate new one.
    
    Returns:
        Tuple of (private_key, public_key_bytes)
    """
    KEYS_DIR.mkdir(parents=True, exist_ok=True)
    
    private_key_path = KEYS_DIR / f"{key_name}_signing.key"
    public_key_path = KEYS_DIR / f"{key_name}_signing.pub"
    
    if private_key_path.exists() and public_key_path.exists():
        # Load existing key
        with open(private_key_path, 'rb') as f:
            private_key = serialization.load_pem_private_key(
                f.read(),
                password=None,
                backend=default_backend()
            )
        
        with open(public_key_path, 'rb') as f:
            public_key_bytes = f.read()
        
        return private_key, public_key_bytes
    else:
        # Generate new key
        private_key, public_key_bytes = generate_signing_key(key_name)
        
        # Save private key
        with open(private_key_path, 'wb') as f:
            f.write(private_key.private_bytes(
                encoding=serialization.Encoding.PEM,
                format=serialization.PrivateFormat.PKCS8,
                encryption_algorithm=serialization.NoEncryption()
            ))
        os.chmod(private_key_path, 0o600)
        
        # Save public key
        with open(public_key_path, 'wb') as f:
            f.write(public_key_bytes)
        os.chmod(public_key_path, 0o644)
        
        print(f"  ✓ Generated new signing key: {key_name}")
        return private_key, public_key_bytes


def sign_data(data: bytes, private_key: rsa.RSAPrivateKey) -> bytes:
    """
    Sign data using RSA-4096-PSS-SHA256.
    
    Returns:
        Signature bytes
    """
    signature = private_key.sign(
        data,
        padding.PSS(
            mgf=padding.MGF1(hashes.SHA256()),
            salt_length=padding.PSS.MAX_LENGTH
        ),
        hashes.SHA256()
    )
    return signature


def sign_directory(directory: Path, key_name: str, pack_name: str) -> Tuple[str, str]:
    """
    Sign all files in a directory and create pack signature.
    
    Returns:
        Tuple of (signature_base64, public_key_base64)
    """
    # Load or generate key
    private_key, public_key_bytes = load_or_generate_key(key_name)
    
    # Collect all files to sign
    files_to_sign = []
    for file_path in directory.rglob("*"):
        if file_path.is_file() and not file_path.name.endswith(('.sig', '.pub')):
            files_to_sign.append(file_path)
    
    # Sort files for deterministic signing
    files_to_sign.sort(key=lambda p: str(p.relative_to(directory)))
    
    # Compute combined hash
    combined_hash = hashlib.sha256()
    for file_path in files_to_sign:
        with open(file_path, 'rb') as f:
            file_data = f.read()
        combined_hash.update(file_path.relative_to(directory).as_posix().encode())
        combined_hash.update(file_data)
    
    pack_hash = combined_hash.digest()
    
    # Sign pack hash
    signature = sign_data(pack_hash, private_key)
    
    # Encode
    signature_base64 = base64.b64encode(signature).decode('utf-8')
    public_key_base64 = base64.b64encode(public_key_bytes).decode('utf-8')
    
    return signature_base64, public_key_base64


def sign_baseline_pack() -> None:
    """Sign baseline intelligence pack."""
    print("Signing baseline intelligence pack...")
    
    pack_dir = INTELLIGENCE_DIR / "baseline_pack"
    sig_dir = pack_dir / "signatures"
    sig_dir.mkdir(parents=True, exist_ok=True)
    
    signature, public_key = sign_directory(pack_dir, "baseline_pack", "Baseline Intelligence Pack")
    
    # Save signature
    sig_path = sig_dir / "baseline_pack.sig"
    with open(sig_path, 'w') as f:
        f.write(signature)
    
    # Save public key
    pub_path = sig_dir / "baseline_pack.pub"
    with open(pub_path, 'w') as f:
        f.write(public_key)
    
    print(f"  ✓ Signature saved: {sig_path}")
    print(f"  ✓ Public key saved: {pub_path}")
    print()


def sign_threat_intel_pack() -> None:
    """Sign threat intelligence pack."""
    print("Signing threat intelligence pack...")
    
    pack_dir = INTELLIGENCE_DIR / "threat_intel"
    sig_dir = pack_dir / "signatures"
    sig_dir.mkdir(parents=True, exist_ok=True)
    
    signature, public_key = sign_directory(pack_dir, "threat_intel", "Threat Intelligence Pack")
    
    # Save signature
    sig_path = sig_dir / "intel_pack.sig"
    with open(sig_path, 'w') as f:
        f.write(signature)
    
    print(f"  ✓ Signature saved: {sig_path}")
    print()


def sign_rag_pack() -> None:
    """Sign RAG knowledge pack."""
    print("Signing RAG knowledge pack...")
    
    pack_dir = INTELLIGENCE_DIR / "llm_knowledge"
    sig_dir = pack_dir / "signatures"
    sig_dir.mkdir(parents=True, exist_ok=True)
    
    signature, public_key = sign_directory(pack_dir, "rag_pack", "RAG Knowledge Pack")
    
    # Save signature
    sig_path = sig_dir / "rag_pack.sig"
    with open(sig_path, 'w') as f:
        f.write(signature)
    
    print(f"  ✓ Signature saved: {sig_path}")
    print()


def main():
    """Generate signatures for all packs."""
    print("=" * 80)
    print("RansomEye Intelligence - Cryptographic Signature Generation")
    print("=" * 80)
    print()
    print("Algorithm: RSA-4096-PSS-SHA256")
    print()
    
    # Ensure trust directory exists
    TRUST_DIR.mkdir(parents=True, exist_ok=True)
    KEYS_DIR.mkdir(parents=True, exist_ok=True)
    
    # Sign all packs
    sign_baseline_pack()
    sign_threat_intel_pack()
    sign_rag_pack()
    
    print("=" * 80)
    print("✓ Signature generation complete")
    print("=" * 80)


if __name__ == '__main__':
    main()

