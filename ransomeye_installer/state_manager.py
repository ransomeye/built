# Path and File Name : /home/ransomeye/rebuild/ransomeye_installer/state_manager.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Manages install/upgrade state machine with signed state validation

"""
State Manager: Manages install and upgrade state machine.
Enforces signed state validation and prevents tampering.
"""

import os
import json
from pathlib import Path
from typing import Dict, Optional
from datetime import datetime
import hashlib

from ransomeye_trust.sign_tool import SignTool
from ransomeye_trust.verify_tool import VerifyTool


class StateManager:
    """Manages RansomEye installation state."""
    
    STATE_FILE = Path("/home/ransomeye/rebuild/ransomeye_installer/config/install_state.json")
    STATE_DIR = STATE_FILE.parent
    VALID_STATES = ['INSTALLED', 'UPGRADED', 'VALID']
    REQUIRED_FIELDS = ['state', 'timestamp', 'version', 'eula_accepted', 'retention_configured', 'identity_generated']
    
    def __init__(self, trust_dir: str = "/home/ransomeye/rebuild/ransomeye_trust"):
        self.trust_dir = Path(trust_dir)
        self.sign_tool = SignTool(str(self.trust_dir))
        self.verify_tool = VerifyTool(str(self.trust_dir))
        self.STATE_DIR.mkdir(parents=True, exist_ok=True)
    
    def _compute_state_hash(self, state: Dict) -> str:
        """Compute SHA-256 hash of state data."""
        state_str = json.dumps(state, sort_keys=True)
        return hashlib.sha256(state_str.encode()).hexdigest()
    
    def create_state(self, version: str, eula_accepted: bool, retention_configured: bool, 
                    identity_generated: bool, state: str = 'INSTALLED') -> Dict:
        """
        Create installation state.
        
        Args:
            version: RansomEye version
            eula_accepted: Whether EULA was accepted
            retention_configured: Whether retention was configured
            identity_generated: Whether crypto identity was generated
            state: State value (INSTALLED, UPGRADED, VALID)
        
        Returns:
            State dictionary
        """
        if state not in self.VALID_STATES:
            raise ValueError(f"Invalid state: {state}. Must be one of {self.VALID_STATES}")
        
        install_state = {
            'state': state,
            'timestamp': datetime.utcnow().isoformat(),
            'version': version,
            'eula_accepted': eula_accepted,
            'retention_configured': retention_configured,
            'identity_generated': identity_generated,
            'hash': None  # Will be computed
        }
        
        # Compute hash
        install_state['hash'] = self._compute_state_hash(install_state)
        
        return install_state
    
    def save_state(self, state: Dict) -> Path:
        """
        Save state to file and sign it.
        
        Fail-closed: if signing fails, the state file is removed and an exception is raised.
        
        Args:
            state: State dictionary
        
        Returns:
            Path to saved state file
        
        Raises:
            RuntimeError: If signing fails (fail-closed)
        """
        # Save state
        with open(self.STATE_FILE, 'w') as f:
            json.dump(state, f, indent=2)
        
        # Sign state file (fail-closed)
        try:
            metadata = {
                'timestamp': datetime.utcnow().isoformat(),
                'version': state.get('version', '1.0.0'),
                'type': 'install_state'
            }
            self.sign_tool.create_and_sign_manifest(self.STATE_FILE, metadata, 'config')
        except Exception as e:
            # Fail-closed: if signing fails, remove the unsigned state and raise
            if self.STATE_FILE.exists():
                self.STATE_FILE.unlink()
            raise RuntimeError(f"Failed to sign installation state (fail-closed): {e}")
        
        return self.STATE_FILE
    
    def load_state(self) -> Optional[Dict]:
        """
        Load and verify installation state.
        
        Returns:
            State dictionary if valid, None otherwise
        """
        if not self.STATE_FILE.exists():
            return None
        
        try:
            # Verify signature
            verify_result = self.verify_tool.verify_manifest(
                self.STATE_FILE.parent / f"{self.STATE_FILE.stem}_manifest.json"
            )
            
            if not verify_result.get('valid'):
                return None
            
            # Load state
            with open(self.STATE_FILE, 'r') as f:
                state = json.load(f)
            
            # Validate required fields
            for field in self.REQUIRED_FIELDS:
                if field not in state:
                    return None
            
            # Verify hash
            computed_hash = self._compute_state_hash(state)
            if computed_hash != state.get('hash'):
                return None
            
            return state
        except Exception:
            return None
    
    def is_state_valid(self) -> bool:
        """
        Check if installation state is valid.
        
        Returns:
            True if state is valid and signed
        """
        state = self.load_state()
        if state is None:
            return False
        
        # Check state value
        if state.get('state') not in self.VALID_STATES:
            return False
        
        # Check required conditions
        if not state.get('eula_accepted', False):
            return False
        
        if not state.get('retention_configured', False):
            return False
        
        if not state.get('identity_generated', False):
            return False
        
        return True
    
    def update_state(self, **updates) -> Dict:
        """
        Update installation state.
        
        Args:
            **updates: Fields to update
        
        Returns:
            Updated state dictionary
        """
        current_state = self.load_state()
        if current_state is None:
            raise ValueError("No existing state to update")
        
        # Update fields
        current_state.update(updates)
        current_state['timestamp'] = datetime.utcnow().isoformat()
        
        # Recompute hash
        current_state['hash'] = self._compute_state_hash(current_state)
        
        # Save updated state
        self.save_state(current_state)
        
        return current_state
    
    def get_state(self) -> Optional[Dict]:
        """Get current state without validation."""
        return self.load_state()

