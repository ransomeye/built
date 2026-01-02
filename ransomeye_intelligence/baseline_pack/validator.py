# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack/validator.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Validates baseline intelligence pack - ensures all models, SHAP, and signatures are present and valid

"""
Baseline Pack Validator: Validates baseline intelligence pack.
Ensures all models, SHAP, metadata, and signatures are present and valid.
Fails-closed if validation fails.
"""

import os
import json
import hashlib
import os
from pathlib import Path
from typing import Dict, List, Tuple, Optional
import sys

from ransomeye_trust.verify_tool import VerifyTool


class BaselinePackValidator:
    """Validates baseline intelligence pack."""
    
    # Runtime-safe baseline pack directory (no build-time paths).
    BASELINE_PACK_DIR = Path(__file__).resolve().parent
    REQUIRED_MODELS = [
        "ransomware_behavior.model",
        "anomaly_baseline.model",
        "confidence_calibration.model"
    ]
    REQUIRED_FILES = [
        "models/model_manifest.json",
        "shap/shap_schema.json",
        "shap/baseline_shap_values.json",
        "metadata/training_manifest.json",
        "metadata/feature_schema.json",
        "metadata/license_manifest.json"
    ]
    
    def __init__(self, trust_dir: str | None = None):
        # Runtime-safe trust module directory (no build-time paths).
        # Intelligence pack verification uses the shared trust tooling module deployed to:
        #   /opt/ransomeye/modules/ransomeye_trust
        if trust_dir is None:
            runtime_root = Path(os.environ.get("RANSOMEYE_ROOT", "/opt/ransomeye"))
            trust_dir = str(runtime_root / "modules" / "ransomeye_trust")
        self.trust_dir = Path(trust_dir)
        self.verify_tool = VerifyTool(str(self.trust_dir))
        self.errors: List[str] = []
        self.warnings: List[str] = []
    
    def _check_file_exists(self, file_path: Path) -> bool:
        """Check if file exists."""
        if not file_path.exists():
            self.errors.append(f"Required file missing: {file_path}")
            return False
        return True
    
    def _check_model_files(self) -> bool:
        """Check all model files exist."""
        valid = True
        for model_name in self.REQUIRED_MODELS:
            model_path = self.BASELINE_PACK_DIR / "models" / model_name
            if not self._check_file_exists(model_path):
                valid = False
        return valid
    
    def _check_manifest_files(self) -> bool:
        """Check all manifest files exist."""
        valid = True
        for file_rel_path in self.REQUIRED_FILES:
            file_path = self.BASELINE_PACK_DIR / file_rel_path
            if not self._check_file_exists(file_path):
                valid = False
        return valid
    
    def _validate_model_manifest(self) -> bool:
        """Validate model manifest structure."""
        manifest_path = self.BASELINE_PACK_DIR / "models" / "model_manifest.json"
        
        if not manifest_path.exists():
            self.errors.append("Model manifest not found")
            return False
        
        try:
            with open(manifest_path, 'r') as f:
                manifest = json.load(f)
        except Exception as e:
            self.errors.append(f"Error reading model manifest: {e}")
            return False
        
        # Validate required fields
        required_fields = ['manifest_version', 'pack_name', 'pack_version', 'models', 'signature']
        for field in required_fields:
            if field not in manifest:
                self.errors.append(f"Model manifest missing required field: {field}")
                return False
        
        # Validate models list
        if not isinstance(manifest.get('models'), list):
            self.errors.append("Model manifest 'models' must be a list")
            return False
        
        # Validate each model
        model_names = [m.get('name') for m in manifest.get('models', [])]
        for required_model in self.REQUIRED_MODELS:
            if required_model not in model_names:
                self.errors.append(f"Required model missing from manifest: {required_model}")
                return False
        
        # Validate SHAP requirements
        for model in manifest.get('models', []):
            if not model.get('shap_required', False):
                self.errors.append(f"Model {model.get('name')} must require SHAP")
                return False
            if not model.get('shap_file'):
                self.errors.append(f"Model {model.get('name')} missing SHAP file reference")
                return False
        
        return True
    
    def _validate_shap_schema(self) -> bool:
        """Validate SHAP schema."""
        schema_path = self.BASELINE_PACK_DIR / "shap" / "shap_schema.json"
        
        if not schema_path.exists():
            self.errors.append("SHAP schema not found")
            return False
        
        try:
            with open(schema_path, 'r') as f:
                schema = json.load(f)
        except Exception as e:
            self.errors.append(f"Error reading SHAP schema: {e}")
            return False
        
        # Validate required fields
        required_fields = ['schema_version', 'shap_format', 'required_fields']
        for field in required_fields:
            if field not in schema:
                self.errors.append(f"SHAP schema missing required field: {field}")
                return False
        
        return True
    
    def _validate_training_manifest(self) -> bool:
        """Validate training manifest."""
        manifest_path = self.BASELINE_PACK_DIR / "metadata" / "training_manifest.json"
        
        if not manifest_path.exists():
            self.errors.append("Training manifest not found")
            return False
        
        try:
            with open(manifest_path, 'r') as f:
                manifest = json.load(f)
        except Exception as e:
            self.errors.append(f"Error reading training manifest: {e}")
            return False
        
        # Validate no customer data
        if manifest.get('customer_data_used', True):
            self.errors.append("Training manifest indicates customer data was used (forbidden)")
            return False
        
        # Validate training methodology
        methodology = manifest.get('training_methodology', '')
        if methodology not in ['synthetic_and_redteam', 'synthetic_only']:
            self.errors.append(f"Invalid training methodology: {methodology}")
            return False
        
        return True
    
    def _validate_signatures(self) -> bool:
        """Validate all signatures."""
        files_to_verify = [
            self.BASELINE_PACK_DIR / "models" / "model_manifest.json",
            self.BASELINE_PACK_DIR / "shap" / "baseline_shap_values.json",
            self.BASELINE_PACK_DIR / "metadata" / "training_manifest.json",
            self.BASELINE_PACK_DIR / "metadata" / "license_manifest.json"
        ]
        
        valid = True
        for file_path in files_to_verify:
            # These JSON files are the manifest objects themselves; verification uses a detached
            # signature file at: <stem>.sig (see VerifyTool.verify_manifest contract).
            result = self.verify_tool.verify_manifest(file_path)
            if not result.get('valid'):
                self.errors.append(f"Invalid or missing signature for {file_path.name}: {result.get('error', 'unknown error')}")
                valid = False
        
        return valid
    
    def validate(self) -> Tuple[bool, List[str], List[str]]:
        """
        Validate entire baseline pack.
        
        Returns:
            Tuple of (is_valid: bool, errors: List[str], warnings: List[str])
        """
        self.errors = []
        self.warnings = []
        
        # Check model files
        if not self._check_model_files():
            return False, self.errors, self.warnings
        
        # Check manifest files
        if not self._check_manifest_files():
            return False, self.errors, self.warnings
        
        # Validate model manifest
        if not self._validate_model_manifest():
            return False, self.errors, self.warnings
        
        # Validate SHAP schema
        if not self._validate_shap_schema():
            return False, self.errors, self.warnings
        
        # Validate training manifest
        if not self._validate_training_manifest():
            return False, self.errors, self.warnings
        
        # Validate signatures
        if not self._validate_signatures():
            return False, self.errors, self.warnings
        
        return len(self.errors) == 0, self.errors, self.warnings
    
    def report_errors(self) -> None:
        """Report validation errors and fail-closed if any found."""
        if not self.errors:
            if self.warnings:
                print("Baseline pack validation warnings:", file=sys.stderr)
                for warning in self.warnings:
                    print(f"  WARNING: {warning}", file=sys.stderr)
            return
        
        print("="*80, file=sys.stderr)
        print("BASELINE PACK VALIDATION FAILED", file=sys.stderr)
        print("="*80, file=sys.stderr)
        print(f"Errors: {len(self.errors)}", file=sys.stderr)
        print("", file=sys.stderr)
        
        for i, error in enumerate(self.errors, 1):
            print(f"{i}. {error}", file=sys.stderr)
        
        print("="*80, file=sys.stderr)
        
        from ransomeye_guardrails.fail_closed import fail_closed
        fail_closed(
            "BASELINE_PACK_VALIDATION_FAILED",
            f"Found {len(self.errors)} validation error(s) in baseline intelligence pack. AI cannot start without valid baseline pack.",
            file_path=None
        )


def main():
    """CLI entry point for baseline pack validator."""
    validator = BaselinePackValidator()
    is_valid, errors, warnings = validator.validate()
    
    if is_valid:
        print("✓ Baseline intelligence pack validation passed.")
        if warnings:
            print("\nWarnings:")
            for warning in warnings:
                print(f"  {warning}")
    else:
        validator.report_errors()


if __name__ == '__main__':
    main()

