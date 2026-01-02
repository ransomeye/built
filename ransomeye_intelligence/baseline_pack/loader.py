# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack/loader.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Loads baseline intelligence pack models and SHAP - fails-closed if pack is missing or invalid

"""
Baseline Pack Loader: Loads baseline intelligence pack.
Fails-closed if pack is missing, invalid, or unsigned.
"""

import os
import json
import pickle
from pathlib import Path
from typing import Dict, Optional, Any
import sys

from .validator import BaselinePackValidator


class BaselinePackLoader:
    """Loads baseline intelligence pack."""
    
    # Runtime-safe baseline pack directory (no build-time paths).
    BASELINE_PACK_DIR = Path(__file__).resolve().parent
    
    def __init__(self):
        self.validator = BaselinePackValidator()
        self.models: Dict[str, Any] = {}
        self.shap_baselines: Dict[str, Dict] = {}
        self.manifests: Dict[str, Dict] = {}
        self.loaded = False
    
    def _validate_before_load(self) -> bool:
        """Validate baseline pack before loading."""
        is_valid, errors, warnings = self.validator.validate()
        if not is_valid:
            self.validator.report_errors()
            return False
        return True
    
    def _load_model_manifest(self) -> Dict:
        """Load model manifest."""
        manifest_path = self.BASELINE_PACK_DIR / "models" / "model_manifest.json"
        with open(manifest_path, 'r') as f:
            return json.load(f)
    
    def _load_model(self, model_name: str) -> Any:
        """Load a model file."""
        model_path = self.BASELINE_PACK_DIR / "models" / model_name
        with open(model_path, 'rb') as f:
            return pickle.load(f)
    
    def _load_shap_baselines(self) -> Dict:
        """Load SHAP baseline values."""
        shap_path = self.BASELINE_PACK_DIR / "shap" / "baseline_shap_values.json"
        with open(shap_path, 'r') as f:
            return json.load(f)
    
    def load(self) -> bool:
        """
        Load baseline intelligence pack.
        
        Returns:
            True if loaded successfully, False otherwise (and exits)
        """
        # Validate before loading
        if not self._validate_before_load():
            return False
        
        try:
            # Load model manifest
            self.manifests['model'] = self._load_model_manifest()
            
            # Load all models
            for model_info in self.manifests['model'].get('models', []):
                model_name = model_info['name']
                self.models[model_name] = self._load_model(model_name)
            
            # Load SHAP baselines
            self.shap_baselines = self._load_shap_baselines()
            
            self.loaded = True
            return True
        except Exception as e:
            print(f"✗ Error loading baseline pack: {e}", file=sys.stderr)
            sys.exit(1)
    
    def get_model(self, model_name: str) -> Optional[Any]:
        """Get loaded model by name."""
        if not self.loaded:
            return None
        return self.models.get(model_name)
    
    def get_shap_baseline(self, model_name: str) -> Optional[Dict]:
        """Get SHAP baseline for model."""
        if not self.loaded:
            return None
        return self.shap_baselines.get('models', {}).get(model_name)
    
    def is_loaded(self) -> bool:
        """Check if baseline pack is loaded."""
        return self.loaded


def main():
    """CLI entry point for baseline pack loader."""
    loader = BaselinePackLoader()
    
    if loader.load():
        print("✓ Baseline intelligence pack loaded successfully")
        print(f"  Models loaded: {len(loader.models)}")
        print(f"  SHAP baselines loaded: {len(loader.shap_baselines.get('models', {}))}")
    else:
        print("✗ Failed to load baseline intelligence pack", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()

