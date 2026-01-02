# Path and File Name : /home/ransomeye/rebuild/ransomeye_posture_engine/engine/output_signer.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Output signer - cryptographically signs posture outputs using Ed25519

"""
Output Signer

Cryptographically signs posture outputs (reports, scores, alerts).
Uses Ed25519 ONLY - aligned with Phase 10.
"""

import logging
import hashlib
import base64
from pathlib import Path
from typing import Optional, Dict, Any
from datetime import datetime
import json

logger = logging.getLogger("ransomeye_posture_engine.output_signer")

class OutputSigner:
    """Signs posture outputs using Ed25519."""
    
    def __init__(self, signing_key_path: Optional[Path] = None, trust_store_path: Optional[Path] = None):
        self.signing_key_path = signing_key_path
        self.trust_store_path = trust_store_path
        self.private_key = None
        self.public_key = None
        
        if signing_key_path and signing_key_path.exists():
            self._load_key()
        else:
            logger.warning("No signing key provided - outputs will not be signed")
    
    def _load_key(self):
        """Load Ed25519 signing key from file."""
        try:
            from cryptography.hazmat.primitives import serialization
            from cryptography.hazmat.primitives.asymmetric import ed25519
            from cryptography.hazmat.backends import default_backend
            
            with open(self.signing_key_path, 'rb') as f:
                key_data = f.read()
            
            # Try to load as PKCS8 (Ed25519 private key format)
            try:
                self.private_key = serialization.load_pem_private_key(
                    key_data,
                    password=None,
                    backend=default_backend()
                )
                
                # Verify it's Ed25519
                if not isinstance(self.private_key, ed25519.Ed25519PrivateKey):
                    raise ValueError("Key is not Ed25519 - RSA keys are PROHIBITED")
                
                self.public_key = self.private_key.public_key()
                logger.info("Loaded Ed25519 signing key")
            
            except ValueError as e:
                logger.error(f"Invalid key type: {e}")
                raise
            except Exception as e:
                logger.error(f"Error loading Ed25519 key: {e}")
                raise
        
        except ImportError:
            logger.error("cryptography library not available - signing REQUIRED, cannot proceed")
            raise RuntimeError("Ed25519 signing requires cryptography library")
        
        except Exception as e:
            logger.error(f"Error initializing Ed25519 signer: {e}")
            raise
    
    def sign_output(self, data: bytes) -> Dict[str, Any]:
        """
        Sign output data using Ed25519.
        
        Args:
            data: Data to sign
        
        Returns:
            Signature metadata dict
        
        Raises:
            RuntimeError: If signing key not available (fail-closed)
        """
        if not self.private_key:
            # Fail-closed: signing is mandatory
            raise RuntimeError("Ed25519 signing key not available - cannot sign output (FAIL-CLOSED)")
        
        try:
            from cryptography.hazmat.primitives import hashes
            
            # Ed25519 signs the data directly (no hash needed)
            signature = self.private_key.sign(data)
            
            from cryptography.hazmat.primitives import serialization
            
            # Get public key bytes for verification
            public_key_bytes = self.public_key.public_bytes(
                encoding=serialization.Encoding.Raw,
                format=serialization.PublicFormat.Raw
            )
            
            return {
                'signed': True,
                'algorithm': 'Ed25519',
                'signature': base64.b64encode(signature).decode('utf-8'),
                'public_key': base64.b64encode(public_key_bytes).decode('utf-8'),
                'data_hash': hashlib.sha256(data).hexdigest(),
                'timestamp': datetime.utcnow().isoformat(),
            }
        
        except Exception as e:
            logger.error(f"Error signing output with Ed25519: {e}")
            # Fail-closed: re-raise
            raise RuntimeError(f"Ed25519 signing failed: {e}")
    
    def sign_file(self, file_path: Path) -> Dict[str, Any]:
        """
        Sign a file using Ed25519.
        
        Args:
            file_path: Path to file to sign
        
        Returns:
            Signature metadata dict
        
        Raises:
            RuntimeError: If signing fails (fail-closed)
        """
        try:
            with open(file_path, 'rb') as f:
                data = f.read()
            
            signature_metadata = self.sign_output(data)
            
            # Save signature to separate file
            sig_path = file_path.with_suffix(file_path.suffix + '.sig')
            with open(sig_path, 'w') as f:
                json.dump(signature_metadata, f, indent=2)
            
            logger.info(f"Signed file with Ed25519: {file_path}")
            return signature_metadata
        
        except Exception as e:
            logger.error(f"Error signing file {file_path} with Ed25519: {e}")
            raise
    
    @staticmethod
    def verify_signature(data: bytes, signature_b64: str, public_key_b64: str) -> bool:
        """
        Verify Ed25519 signature.
        
        Args:
            data: Original data
            signature_b64: Base64-encoded signature
            public_key_b64: Base64-encoded public key
        
        Returns:
            True if signature is valid, False otherwise
        """
        try:
            from cryptography.hazmat.primitives import serialization
            from cryptography.hazmat.primitives.asymmetric import ed25519
            from cryptography.hazmat.backends import default_backend
            
            # Decode signature and public key
            signature_bytes = base64.b64decode(signature_b64)
            public_key_bytes = base64.b64decode(public_key_b64)
            
            # Reconstruct public key
            public_key = ed25519.Ed25519PublicKey.from_public_bytes(public_key_bytes)
            
            # Verify signature
            public_key.verify(signature_bytes, data)
            return True
        
        except Exception as e:
            logger.error(f"Ed25519 signature verification failed: {e}")
            return False
