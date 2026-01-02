# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/ai_registry/registry.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: AI model registry - tracks all AI models, versions, and dependencies

"""
AI Registry: Tracks all AI models, versions, and dependencies.
Ensures model integrity and version compatibility.
"""

import os
import json
from pathlib import Path
from typing import Dict, List, Optional
from datetime import datetime
import hashlib


class AIRegistry:
    """AI model registry."""
    
    REGISTRY_FILE = Path("/home/ransomeye/rebuild/ransomeye_intelligence/ai_registry/registry.json")
    REGISTRY_DIR = REGISTRY_FILE.parent
    
    def __init__(self):
        self.REGISTRY_DIR.mkdir(parents=True, exist_ok=True)
        self.registry: Dict = {}
        self._load_registry()
    
    def _load_registry(self) -> None:
        """Load registry from file."""
        if self.REGISTRY_FILE.exists():
            try:
                with open(self.REGISTRY_FILE, 'r') as f:
                    self.registry = json.load(f)
            except Exception:
                self.registry = {'models': [], 'version': '1.0.0'}
        else:
            self.registry = {'models': [], 'version': '1.0.0'}
    
    def _save_registry(self) -> None:
        """Save registry to file."""
        with open(self.REGISTRY_FILE, 'w') as f:
            json.dump(self.registry, f, indent=2)
    
    def register_model(self, model_name: str, model_path: Path, version: str, 
                      dependencies: List[str], metadata: Dict) -> None:
        """
        Register a model in the registry.
        
        Args:
            model_name: Model name
            model_path: Path to model file
            version: Model version
            dependencies: List of dependency model names
            metadata: Model metadata
        """
        # Compute model hash
        model_hash = self._compute_file_hash(model_path)
        
        model_entry = {
            'name': model_name,
            'path': str(model_path),
            'version': version,
            'hash': model_hash,
            'dependencies': dependencies,
            'metadata': metadata,
            'registered': datetime.utcnow().isoformat(),
            'active': True
        }
        
        # Add to registry
        if 'models' not in self.registry:
            self.registry['models'] = []
        
        # Remove existing entry if present
        self.registry['models'] = [m for m in self.registry['models'] if m['name'] != model_name]
        
        # Add new entry
        self.registry['models'].append(model_entry)
        
        self._save_registry()
    
    def get_model(self, model_name: str, version: Optional[str] = None) -> Optional[Dict]:
        """
        Get model from registry.
        
        Args:
            model_name: Model name
            version: Optional version (default: latest)
        
        Returns:
            Model entry or None
        """
        models = [m for m in self.registry.get('models', []) if m['name'] == model_name and m.get('active', True)]
        
        if not models:
            return None
        
        if version:
            models = [m for m in models if m['version'] == version]
        
        if not models:
            return None
        
        # Return latest version
        return max(models, key=lambda m: m['registered'])
    
    def list_models(self) -> List[Dict]:
        """List all registered models."""
        return [m for m in self.registry.get('models', []) if m.get('active', True)]
    
    def _compute_file_hash(self, file_path: Path) -> str:
        """Compute SHA-256 hash of file."""
        sha256 = hashlib.sha256()
        with open(file_path, 'rb') as f:
            for chunk in iter(lambda: f.read(4096), b''):
                sha256.update(chunk)
        return sha256.hexdigest()


def main():
    """CLI entry point for AI registry."""
    registry = AIRegistry()
    
    models = registry.list_models()
    print(f"Registered models: {len(models)}")
    for model in models:
        print(f"  {model['name']} v{model['version']}")


if __name__ == '__main__':
    main()

