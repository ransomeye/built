# Path and File Name : /home/ransomeye/rebuild/ransomeye_posture_engine/engine/signature_verifier.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Ed25519 signature verifier for telemetry events - database is UNTRUSTED

"""
Signature Verifier

Verifies Ed25519 signatures on telemetry events.
Database is UNTRUSTED - every record MUST be verified.
Fail-closed on missing or invalid signatures.
"""

import logging
import hashlib
import base64
import json
from typing import Dict, Any, Optional
from pathlib import Path

logger = logging.getLogger("ransomeye_posture_engine.signature_verifier")

class SignatureVerificationError(Exception):
    """Signature verification error."""
    pass

class SignatureVerifier:
    """Verifies Ed25519 signatures on telemetry events."""
    
    def __init__(self, trust_store_path: Optional[Path] = None):
        """
        Initialize signature verifier.
        
        Args:
            trust_store_path: Path to trust store containing public keys
        """
        self.trust_store_path = trust_store_path
        self.public_keys: Dict[str, bytes] = {}  # producer_id -> public_key_bytes
        
        if trust_store_path:
            self._load_trust_store()
    
    def _load_trust_store(self):
        """Load public keys from trust store."""
        if not self.trust_store_path or not self.trust_store_path.exists():
            logger.warning(f"Trust store not found: {self.trust_store_path}")
            return
        
        try:
            # Trust store format: JSON file with producer_id -> public_key_b64
            with open(self.trust_store_path, 'r') as f:
                trust_data = json.load(f)
            
            for producer_id, public_key_b64 in trust_data.items():
                try:
                    public_key_bytes = base64.b64decode(public_key_b64)
                    self.public_keys[producer_id] = public_key_bytes
                    logger.info(f"Loaded public key for producer: {producer_id}")
                except Exception as e:
                    logger.error(f"Error loading public key for {producer_id}: {e}")
                    # Fail-closed: continue but log error
        
        except Exception as e:
            logger.error(f"Error loading trust store: {e}")
            # Fail-closed: raise error
            raise RuntimeError(f"Failed to load trust store: {e}")
    
    def verify_telemetry_event(self, 
                               event_data: Dict[str, Any],
                               signature: str,
                               producer_id: str,
                               algorithm: Optional[str] = None) -> bool:
        """
        Verify Ed25519 signature on telemetry event.
        
        Args:
            event_data: Telemetry event data (as dict)
            signature: Base64-encoded Ed25519 signature
            producer_id: Producer identifier (for public key lookup)
            algorithm: Signature algorithm (must be 'Ed25519' or None)
        
        Returns:
            True if signature is valid
        
        Raises:
            SignatureVerificationError: If signature is missing, invalid, or verification fails
        """
        # Fail-closed: signature is mandatory
        if not signature:
            raise SignatureVerificationError(f"Missing signature for producer {producer_id} (FAIL-CLOSED)")
        
        # Fail-closed: Ed25519 is mandatory
        if algorithm and algorithm != 'Ed25519':
            raise SignatureVerificationError(
                f"Invalid algorithm '{algorithm}' - Ed25519 is MANDATORY (RSA is PROHIBITED)"
            )
        
        # Get public key for producer
        if producer_id not in self.public_keys:
            raise SignatureVerificationError(
                f"Public key not found for producer {producer_id} (FAIL-CLOSED)"
            )
        
        public_key_bytes = self.public_keys[producer_id]
        
        # Serialize event data for signing (deterministic JSON)
        event_json = json.dumps(event_data, sort_keys=True, separators=(',', ':'))
        event_bytes = event_json.encode('utf-8')
        
        # Verify Ed25519 signature
        try:
            from cryptography.hazmat.primitives.asymmetric import ed25519
            
            # Decode signature
            signature_bytes = base64.b64decode(signature)
            
            # Reconstruct public key
            public_key = ed25519.Ed25519PublicKey.from_public_bytes(public_key_bytes)
            
            # Verify signature
            public_key.verify(signature_bytes, event_bytes)
            
            logger.debug(f"Ed25519 signature verified for producer {producer_id}")
            return True
        
        except Exception as e:
            logger.error(f"Ed25519 signature verification failed for producer {producer_id}: {e}")
            raise SignatureVerificationError(
                f"Ed25519 signature verification failed: {e} (FAIL-CLOSED)"
            )
    
    def verify_from_database_record(self, 
                                   event_id: str,
                                   event_data: Dict[str, Any],
                                   signature: Optional[str],
                                   producer_id: str,
                                   signature_algorithm: Optional[str],
                                   signature_valid: Optional[bool]) -> bool:
        """
        Verify telemetry event from database record.
        
        Database is UNTRUSTED - signature_valid flag is IGNORED.
        Every record MUST be verified explicitly.
        
        Args:
            event_id: Event identifier
            event_data: Event data
            signature: Signature from database
            producer_id: Producer identifier
            signature_algorithm: Algorithm from database
            signature_valid: Database's signature_valid flag (IGNORED - we verify ourselves)
        
        Returns:
            True if signature is valid
        
        Raises:
            SignatureVerificationError: If verification fails
        """
        # Ignore database's signature_valid flag - we verify ourselves
        # Database is UNTRUSTED
        
        # Verify signature explicitly
        return self.verify_telemetry_event(
            event_data=event_data,
            signature=signature or '',  # Will fail if None
            producer_id=producer_id,
            algorithm=signature_algorithm
        )

