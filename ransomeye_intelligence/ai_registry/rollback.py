# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/ai_registry/rollback.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: AI model rollback - reverts to previous model version on failure

"""
AI Model Rollback: Reverts to previous model version on failure.
Ensures system can recover from model failures.
"""

import os
import json
from pathlib import Path
from typing import Dict, List, Optional
from datetime import datetime
from .registry import AIRegistry


class AIModelRollback:
    """Manages AI model rollback."""
    
    ROLLBACK_HISTORY_FILE = Path("/home/ransomeye/rebuild/ransomeye_intelligence/ai_registry/rollback_history.json")
    
    def __init__(self):
        self.registry = AIRegistry()
        self.ROLLBACK_HISTORY_FILE.parent.mkdir(parents=True, exist_ok=True)
        self.history: List[Dict] = []
        self._load_history()
    
    def _load_history(self) -> None:
        """Load rollback history."""
        if self.ROLLBACK_HISTORY_FILE.exists():
            try:
                with open(self.ROLLBACK_HISTORY_FILE, 'r') as f:
                    self.history = json.load(f)
            except Exception:
                self.history = []
        else:
            self.history = []
    
    def _save_history(self) -> None:
        """Save rollback history."""
        with open(self.ROLLBACK_HISTORY_FILE, 'w') as f:
            json.dump(self.history, f, indent=2)
    
    def rollback_model(self, model_name: str, reason: str) -> bool:
        """
        Rollback model to previous version.
        
        Args:
            model_name: Model name
            reason: Rollback reason
        
        Returns:
            True if rollback successful
        """
        # Get current model
        current_model = self.registry.get_model(model_name)
        if not current_model:
            return False
        
        # Get all versions
        all_models = [m for m in self.registry.list_models() if m['name'] == model_name]
        all_models.sort(key=lambda m: m['registered'], reverse=True)
        
        if len(all_models) < 2:
            return False  # No previous version
        
        # Deactivate current version
        current_model['active'] = False
        self.registry._save_registry()
        
        # Activate previous version
        previous_model = all_models[1]
        previous_model['active'] = True
        self.registry._save_registry()
        
        # Log rollback
        rollback_entry = {
            'model_name': model_name,
            'from_version': current_model['version'],
            'to_version': previous_model['version'],
            'reason': reason,
            'timestamp': datetime.utcnow().isoformat()
        }
        self.history.append(rollback_entry)
        self._save_history()
        
        return True
    
    def get_rollback_history(self, model_name: Optional[str] = None) -> List[Dict]:
        """Get rollback history."""
        if model_name:
            return [h for h in self.history if h['model_name'] == model_name]
        return self.history


def main():
    """CLI entry point for rollback."""
    rollback = AIModelRollback()
    
    history = rollback.get_rollback_history()
    print(f"Rollback history: {len(history)} entries")
    for entry in history:
        print(f"  {entry['model_name']}: {entry['from_version']} -> {entry['to_version']} ({entry['reason']})")


if __name__ == '__main__':
    main()

