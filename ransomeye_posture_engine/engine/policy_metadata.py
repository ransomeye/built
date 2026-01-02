# Path and File Name : /home/ransomeye/rebuild/ransomeye_posture_engine/engine/policy_metadata.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Policy metadata manager - computes and tracks policy hashes and versions

"""
Policy Metadata Manager

Computes SHA-256 hashes for policy files and tracks versions.
Every policy evaluation MUST include policy hash, version, and source path.
"""

import logging
import hashlib
from pathlib import Path
from typing import Dict, List, Optional
from dataclasses import dataclass
from datetime import datetime

logger = logging.getLogger("ransomeye_posture_engine.policy_metadata")

@dataclass
class PolicyMetadata:
    """Policy metadata with hash and version."""
    policy_id: str
    policy_type: str  # "cis", "stig", "custom"
    source_path: Path
    version: str
    sha256_hash: str
    loaded_at: datetime

class PolicyMetadataManager:
    """Manages policy metadata (hash, version, source path)."""
    
    def __init__(self):
        self.metadata: Dict[str, PolicyMetadata] = {}  # policy_id -> metadata
    
    def compute_policy_hash(self, policy_path: Path) -> str:
        """
        Compute SHA-256 hash of policy file.
        
        Args:
            policy_path: Path to policy file
        
        Returns:
            SHA-256 hash as hex string
        """
        try:
            with open(policy_path, 'rb') as f:
                content = f.read()
            
            hash_obj = hashlib.sha256(content)
            return hash_obj.hexdigest()
        
        except Exception as e:
            logger.error(f"Error computing hash for {policy_path}: {e}")
            raise
    
    def register_policy(self, 
                       policy_id: str,
                       policy_type: str,
                       source_path: Path,
                       version: Optional[str] = None) -> PolicyMetadata:
        """
        Register a policy with metadata.
        
        Args:
            policy_id: Policy identifier
            policy_type: Type of policy (cis, stig, custom)
            source_path: Path to policy file
            version: Policy version (if None, uses file mtime)
        
        Returns:
            PolicyMetadata object
        """
        # Compute hash
        sha256_hash = self.compute_policy_hash(source_path)
        
        # Get version (use file mtime if not provided)
        if version is None:
            mtime = source_path.stat().st_mtime
            version = str(int(mtime))
        
        metadata = PolicyMetadata(
            policy_id=policy_id,
            policy_type=policy_type,
            source_path=source_path,
            version=version,
            sha256_hash=sha256_hash,
            loaded_at=datetime.utcnow(),
        )
        
        self.metadata[policy_id] = metadata
        logger.info(f"Registered policy {policy_id}: hash={sha256_hash[:16]}..., version={version}")
        
        return metadata
    
    def get_metadata(self, policy_id: str) -> Optional[PolicyMetadata]:
        """Get metadata for a policy."""
        return self.metadata.get(policy_id)
    
    def get_all_metadata(self) -> List[PolicyMetadata]:
        """Get all policy metadata."""
        return list(self.metadata.values())
    
    def detect_policy_drift(self, policy_id: str, source_path: Path) -> bool:
        """
        Detect if policy file has changed (drift detection).
        
        Args:
            policy_id: Policy identifier
            source_path: Current path to policy file
        
        Returns:
            True if policy has changed (drift detected)
        """
        if policy_id not in self.metadata:
            # New policy - not drift
            return False
        
        old_metadata = self.metadata[policy_id]
        
        # Check if path changed
        if old_metadata.source_path != source_path:
            logger.warning(f"Policy {policy_id} source path changed: {old_metadata.source_path} -> {source_path}")
            return True
        
        # Check if hash changed
        current_hash = self.compute_policy_hash(source_path)
        if current_hash != old_metadata.sha256_hash:
            logger.warning(f"Policy {policy_id} hash changed: {old_metadata.sha256_hash[:16]}... -> {current_hash[:16]}...")
            return True
        
        return False

