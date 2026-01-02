# Path and File Name : /home/ransomeye/rebuild/core/ai/models/sign_manifest.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Sign model manifest with RSA-4096 signature

"""
Sign model manifest with RSA-4096 signature.
The signature process matches the Rust loader's verification:
1. Remove signature field from manifest
2. Compute SHA256 hash
3. Sign hash with RSA-PSS-SHA256
4. Base64 encode signature
"""

import os
import json
import hashlib
import base64
from pathlib import Path
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import rsa, padding
from cryptography.hazmat.backends import default_backend

def load_or_generate_key(private_key_path: Path, public_key_path: Path):
    """Load existing key or generate new RSA-4096 key pair."""
    if private_key_path.exists() and public_key_path.exists():
        print(f"Loading existing keys from {private_key_path}")
        with open(private_key_path, 'rb') as f:
            private_key = serialization.load_pem_private_key(
                f.read(), password=None, backend=default_backend()
            )
        with open(public_key_path, 'rb') as f:
            public_key = serialization.load_pem_public_key(
                f.read(), backend=default_backend()
            )
        return private_key, public_key
    else:
        print(f"Generating new RSA-4096 key pair...")
        private_key = rsa.generate_private_key(
            public_exponent=65537,
            key_size=4096,
            backend=default_backend()
        )
        public_key = private_key.public_key()
        
        # Save private key (PKCS8 format for ring compatibility)
        private_pem = private_key.private_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PrivateFormat.PKCS8,
            encryption_algorithm=serialization.NoEncryption()
        )
        with open(private_key_path, 'wb') as f:
            f.write(private_pem)
        os.chmod(private_key_path, 0o600)
        
        # Save public key
        public_pem = public_key.public_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PublicFormat.SubjectPublicKeyInfo
        )
        with open(public_key_path, 'wb') as f:
            f.write(public_pem)
        
        print(f"Keys saved: {private_key_path}, {public_key_path}")
        return private_key, public_key

def sign_manifest(manifest_path: Path, private_key_path: Path):
    """Sign manifest using RSA-PSS-SHA256 (matches Rust verification)."""
    # Load manifest
    with open(manifest_path, 'r') as f:
        manifest = json.load(f)
    
    # Create manifest copy without signature
    manifest_for_signing = manifest.copy()
    manifest_for_signing.pop('signature', None)
    
    # Serialize to JSON (compact, no whitespace for consistent hashing)
    manifest_json = json.dumps(manifest_for_signing, sort_keys=True, separators=(',', ':'))
    manifest_bytes = manifest_json.encode('utf-8')
    
    # Compute SHA256 hash
    manifest_hash = hashlib.sha256(manifest_bytes).digest()
    
    # Load private key
    with open(private_key_path, 'rb') as f:
        private_key = serialization.load_pem_private_key(
            f.read(), password=None, backend=default_backend()
        )
    
    # Sign hash with RSA-PSS-SHA256 (matches RSA_PSS_2048_8192_SHA256 in Rust)
    signature = private_key.sign(
        manifest_hash,
        padding.PSS(
            mgf=padding.MGF1(hashes.SHA256()),
            salt_length=padding.PSS.MAX_LENGTH
        ),
        hashes.SHA256()
    )
    
    # Base64 encode signature
    signature_b64 = base64.b64encode(signature).decode('utf-8')
    
    # Update manifest with signature
    manifest['signature'] = signature_b64
    
    # Save signed manifest
    with open(manifest_path, 'w') as f:
        json.dump(manifest, f, indent=2)
    
    print(f"Manifest signed: {manifest_path}")
    print(f"Signature: {signature_b64[:64]}...")
    return signature_b64

def main():
    """Main signing function."""
    models_dir = Path(os.environ.get("RANSOMEYE_AI_MODELS_DIR", 
                                     "/home/ransomeye/rebuild/core/ai/models"))
    manifest_path = models_dir / "models.manifest.json"
    
    # Key paths
    private_key_path = models_dir / "model_signing_key.pem"
    public_key_path = models_dir / "model_signing_key.pub"
    
    if not manifest_path.exists():
        print(f"Error: Manifest not found: {manifest_path}")
        return 1
    
    # Load or generate keys
    private_key, public_key = load_or_generate_key(private_key_path, public_key_path)
    
    # Sign manifest
    signature = sign_manifest(manifest_path, private_key_path)
    
    print(f"\n✅ Manifest signed successfully!")
    print(f"   Public key: {public_key_path}")
    print(f"   Signature: {signature[:32]}...")
    
    return 0

if __name__ == "__main__":
    exit(main())

