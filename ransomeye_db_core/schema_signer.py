# Path and File Name: /home/ransomeye/rebuild/ransomeye_db_core/schema_signer.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Signs database schema using Ed25519 (reuses manifest signing keys)

"""
Database Schema Signer: Cryptographically signs schema.sql

CRITICAL:
- Reuses manifest signing keys (no new keypair)
- Computes SHA256 hash of schema.sql
- Signs hash with Ed25519
- Creates schema.hash and schema.sig files
"""

import sys
import hashlib
from pathlib import Path
from typing import Tuple, Optional

# Import manifest signer to reuse keys
try:
    sys.path.insert(0, str(Path(__file__).parent.parent))
    from ransomeye_installer.crypto.manifest_signer import ManifestSigner
except ImportError as e:
    print(f"ERROR: Cannot import ManifestSigner: {e}", file=sys.stderr)
    sys.exit(1)


class SchemaSigner:
    """Signs database schema files."""
    
    def __init__(self, schema_dir: Optional[Path] = None):
        """
        Initialize schema signer.
        
        Args:
            schema_dir: Directory containing schema.sql (default: ./schema)
        """
        if schema_dir is None:
            schema_dir = Path(__file__).parent / "schema"
        
        self.schema_dir = Path(schema_dir)
        self.schema_file = self.schema_dir / "schema.sql"
        self.hash_file = self.schema_dir / "schema.hash"
        self.sig_file = self.schema_dir / "schema.sig"
        
        # Reuse manifest signer for keys
        self.manifest_signer = ManifestSigner()
    
    def compute_schema_hash(self) -> str:
        """
        Compute SHA256 hash of schema.sql.
        
        Returns:
            Hex-encoded SHA256 hash
        
        Raises:
            FileNotFoundError: If schema.sql not found
        """
        if not self.schema_file.exists():
            raise FileNotFoundError(f"Schema file not found: {self.schema_file}")
        
        with open(self.schema_file, 'rb') as f:
            schema_content = f.read()
        
        sha256_hash = hashlib.sha256(schema_content).hexdigest()
        return sha256_hash
    
    def sign_schema(self) -> Tuple[str, str]:
        """
        Sign schema file.
        
        Workflow:
        1. Compute SHA256 hash of schema.sql
        2. Write hash to schema.hash
        3. Sign hash using Ed25519 (manifest signing key)
        4. Write signature to schema.sig
        
        Returns:
            Tuple of (hash, signature)
        
        Raises:
            FileNotFoundError: If schema.sql not found
            RuntimeError: If signing fails
        """
        # Ensure signing keys exist
        self.manifest_signer.ensure_keypair_exists()
        
        # Compute hash
        print(f"[SCHEMA SIGNER] Computing SHA256 hash of {self.schema_file}")
        schema_hash = self.compute_schema_hash()
        print(f"[SCHEMA SIGNER] Schema hash: {schema_hash}")
        
        # Write hash to file
        with open(self.hash_file, 'w') as f:
            f.write(schema_hash)
        print(f"[SCHEMA SIGNER] Hash written to: {self.hash_file}")
        
        # Sign hash (not the schema file itself - sign the hash)
        # This follows the same pattern as manifest signing
        print(f"[SCHEMA SIGNER] Signing schema hash with Ed25519")
        
        # Load signing key
        private_key = self.manifest_signer.load_private_key()
        
        # Sign the hash
        from cryptography.hazmat.primitives import serialization
        signature = private_key.sign(schema_hash.encode('utf-8'))
        signature_hex = signature.hex()
        
        # Write signature to file
        with open(self.sig_file, 'w') as f:
            f.write(signature_hex)
        print(f"[SCHEMA SIGNER] Signature written to: {self.sig_file}")
        
        return schema_hash, signature_hex
    
    def verify_schema_signature(self) -> bool:
        """
        Verify schema signature.
        
        Workflow:
        1. Read schema.hash
        2. Read schema.sig
        3. Verify signature against hash using public key
        4. Recompute schema hash and compare
        
        Returns:
            True if signature valid and hash matches, False otherwise
        """
        # Check files exist
        if not self.schema_file.exists():
            print(f"ERROR: Schema file not found: {self.schema_file}", file=sys.stderr)
            return False
        
        if not self.hash_file.exists():
            print(f"ERROR: Hash file not found: {self.hash_file}", file=sys.stderr)
            return False
        
        if not self.sig_file.exists():
            print(f"ERROR: Signature file not found: {self.sig_file}", file=sys.stderr)
            return False
        
        # Read stored hash
        with open(self.hash_file, 'r') as f:
            stored_hash = f.read().strip()
        
        # Read signature
        with open(self.sig_file, 'r') as f:
            signature_hex = f.read().strip()
        
        # Recompute hash
        current_hash = self.compute_schema_hash()
        
        # Verify hash matches
        if current_hash != stored_hash:
            print(f"ERROR: Schema hash mismatch", file=sys.stderr)
            print(f"  Stored hash:  {stored_hash}", file=sys.stderr)
            print(f"  Current hash: {current_hash}", file=sys.stderr)
            return False
        
        # Verify signature
        try:
            # Load public key from file
            from cryptography.hazmat.primitives import serialization
            from cryptography.hazmat.primitives.asymmetric import ed25519
            
            public_key_path = self.manifest_signer.PUBLIC_KEY_PATH
            if not public_key_path.exists():
                print(f"ERROR: Public key not found: {public_key_path}", file=sys.stderr)
                return False
            
            public_pem = public_key_path.read_bytes()
            public_key = serialization.load_pem_public_key(public_pem)
            
            if not isinstance(public_key, ed25519.Ed25519PublicKey):
                print(f"ERROR: Key is not Ed25519 format", file=sys.stderr)
                return False
            
            signature = bytes.fromhex(signature_hex)
            
            # Verify signature
            public_key.verify(signature, stored_hash.encode('utf-8'))
            print(f"✓ Schema signature verified successfully")
            return True
        
        except Exception as e:
            print(f"ERROR: Signature verification failed: {e}", file=sys.stderr)
            return False


def main():
    """CLI entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description="Sign or verify database schema")
    parser.add_argument('--sign', action='store_true', help="Sign schema.sql")
    parser.add_argument('--verify', action='store_true', help="Verify schema signature")
    parser.add_argument('--schema-dir', type=str, help="Schema directory (default: ./schema)")
    
    args = parser.parse_args()
    
    if not args.sign and not args.verify:
        parser.error("Must specify --sign or --verify")
    
    signer = SchemaSigner(schema_dir=args.schema_dir)
    
    if args.sign:
        try:
            schema_hash, signature = signer.sign_schema()
            print()
            print("=" * 80)
            print("SCHEMA SIGNING COMPLETE")
            print("=" * 80)
            print(f"Schema file: {signer.schema_file}")
            print(f"Hash file: {signer.hash_file}")
            print(f"Signature file: {signer.sig_file}")
            print(f"SHA256: {schema_hash}")
            print("=" * 80)
            sys.exit(0)
        except Exception as e:
            print(f"ERROR: Schema signing failed: {e}", file=sys.stderr)
            sys.exit(1)
    
    if args.verify:
        if signer.verify_schema_signature():
            print("✓ Schema signature verification PASSED")
            sys.exit(0)
        else:
            print("✗ Schema signature verification FAILED", file=sys.stderr)
            sys.exit(1)


if __name__ == '__main__':
    main()

