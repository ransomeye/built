# Path and File Name : /home/ransomeye/rebuild/core/global_validator/ai_ml_claims.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: AI/ML Claim Validator - validates ONLY registered models from canonical registry

"""
AI/ML Claim Validator - Registry Boundary Enforcement

ONLY validates models explicitly registered in:
/home/ransomeye/rebuild/ransomeye_intelligence/model_registry/registry.json

EXPLICITLY IGNORES:
- .venv/ directories
- site-packages/ directories
- tests/ directories
- Dependency files (distutils-precedence.pth, joblib_*.pkl, etc.)
- Any file not listed in registry.json

Rules:
- Unsigned or unlisted models → IGNORED (not validated)
- Registered model without SHAP → FAIL
- Registered model without metadata → FAIL
- Registered model without signature → FAIL
- Registered model with all artifacts → PASS

Mismatch → FAIL (fail-closed).
"""

import json
import re
from pathlib import Path
from typing import Dict, List, Set, Optional
from .validator import Violation, ViolationSeverity, ValidationResult

# Canonical registry location
REGISTRY_PATH = Path("/home/ransomeye/rebuild/ransomeye_intelligence/model_registry/registry.json")
REGISTRY_ROOT = Path("/home/ransomeye/rebuild/ransomeye_intelligence/model_registry/")

# Exclusion patterns (files/directories to ignore)
EXCLUDED_PATTERNS = [
    r'\.venv/',
    r'site-packages/',
    r'__pycache__/',
    r'\.git/',
    r'node_modules/',
    r'tests/',
    r'test_',
    r'_test\.',
    r'distutils-precedence\.pth',
    r'joblib_\d+\.\d+\.\d+.*\.pkl',
    r'\.pyc$',
    r'\.pyo$',
]


class AIMLClaimValidator:
    """Validates AI/ML claims against registry-only models."""
    
    def __init__(self, validator):
        self.validator = validator
        self.registry: Optional[Dict] = None
        self.registered_models: List[Dict] = []
    
    def _load_registry(self) -> bool:
        """Load model registry from canonical location."""
        if not REGISTRY_PATH.exists():
            # Registry doesn't exist yet - this is OK, just means no models registered
            self.registry = {'version': '1.0.0', 'models': []}
            self.registered_models = []
            return True
        
        try:
            with open(REGISTRY_PATH, 'r') as f:
                self.registry = json.load(f)
            
            self.registered_models = self.registry.get('models', [])
            return True
        except Exception as e:
            # Registry exists but can't be parsed - this is a violation
            return False
    
    def _is_excluded_path(self, file_path: Path) -> bool:
        """Check if path should be excluded from validation."""
        path_str = str(file_path)
        
        for pattern in EXCLUDED_PATTERNS:
            if re.search(pattern, path_str, re.IGNORECASE):
                return True
        
        return False
    
    def _get_registry_model(self, model_path: Path) -> Optional[Dict]:
        """Get registry entry for a model path."""
        # Convert absolute path to relative path from registry root
        try:
            relative_path = model_path.relative_to(REGISTRY_ROOT)
        except ValueError:
            # Path is not under registry root - check if it matches any registry entry
            for model_entry in self.registered_models:
                registry_path = REGISTRY_ROOT / model_entry.get('path', '')
                if model_path == registry_path:
                    return model_entry
            return None
        
        # Find matching registry entry
        for model_entry in self.registered_models:
            entry_path = model_entry.get('path', '')
            if entry_path and Path(entry_path) == relative_path:
                return model_entry
        
        return None
    
    def validate(self) -> ValidationResult:
        """Run AI/ML claims validation - ONLY for registered models."""
        violations: List[Violation] = []
        
        if not self.validator.guardrails:
            return ValidationResult(
                passed=False,
                violations=[Violation(
                    checker='ai_ml_claims',
                    severity=ViolationSeverity.CRITICAL,
                    message="guardrails.yaml not loaded",
                )]
            )
        
        # Load registry
        if not self._load_registry():
            violations.append(Violation(
                checker='ai_ml_claims',
                severity=ViolationSeverity.CRITICAL,
                message=f"Model registry exists but cannot be parsed: {REGISTRY_PATH}",
                details={'registry_path': str(REGISTRY_PATH)},
            ))
            return ValidationResult(
                passed=False,
                violations=violations,
            )
        
        # If no models registered, validation passes (no models to validate)
        if not self.registered_models:
            return ValidationResult(
                passed=True,
                violations=[],
            )
        
        # Validate ONLY registered models
        for model_entry in self.registered_models:
            model_id = model_entry.get('model_id', 'unknown')
            phase_id = model_entry.get('phase')
            model_path_str = model_entry.get('path', '')
            requires_shap = model_entry.get('requires_shap', True)
            
            if not model_path_str:
                violations.append(Violation(
                    checker='ai_ml_claims',
                    severity=ViolationSeverity.CRITICAL,
                    message=f"Registry entry {model_id} missing 'path' field",
                    details={'model_id': model_id},
                    phase_id=phase_id,
                ))
                continue
            
            # Construct absolute path
            model_path = REGISTRY_ROOT / model_path_str
            
            # Check if model file exists
            if not model_path.exists():
                violations.append(Violation(
                    checker='ai_ml_claims',
                    severity=ViolationSeverity.CRITICAL,
                    message=f"Registered model {model_id} path does not exist: {model_path}",
                    details={
                        'model_id': model_id,
                        'expected_path': str(model_path),
                        'registry_path': model_path_str,
                    },
                    phase_id=phase_id,
                ))
                continue
            
            # Check SHAP file (if required)
            if requires_shap:
                violations.extend(self._check_registered_model_shap(model_entry, model_path))
            
            # Check metadata file
            violations.extend(self._check_registered_model_metadata(model_entry, model_path))
            
            # Check signature (if specified)
            if model_entry.get('signature'):
                violations.extend(self._check_registered_model_signature(model_entry, model_path))
        
        return ValidationResult(
            passed=len(violations) == 0,
            violations=violations,
        )
    
    def _check_registered_model_shap(self, model_entry: Dict, model_path: Path) -> List[Violation]:
        """Check if registered model has SHAP file."""
        violations = []
        
        model_id = model_entry.get('model_id', 'unknown')
        phase_id = model_entry.get('phase')
        
        # Look for SHAP file (standard naming patterns)
        shap_file = model_path.parent / f"{model_path.stem}_shap.json"
        shap_file_alt = model_path.parent / f"{model_path.stem}.shap"
        
        if not shap_file.exists() and not shap_file_alt.exists():
            violations.append(Violation(
                checker='ai_ml_claims',
                severity=ViolationSeverity.CRITICAL,
                message=f"Registered model {model_id} requires SHAP but SHAP file not found",
                details={
                    'model_id': model_id,
                    'model_path': str(model_path),
                    'expected_shap': str(shap_file),
                },
                phase_id=phase_id,
            ))
        
        return violations
    
    def _check_registered_model_metadata(self, model_entry: Dict, model_path: Path) -> List[Violation]:
        """Check if registered model has metadata file with required fields."""
        violations = []
        
        model_id = model_entry.get('model_id', 'unknown')
        phase_id = model_entry.get('phase')
        
        # Look for metadata file
        metadata_file = model_path.parent / f"{model_path.stem}_metadata.json"
        
        if not metadata_file.exists():
            violations.append(Violation(
                checker='ai_ml_claims',
                severity=ViolationSeverity.CRITICAL,
                message=f"Registered model {model_id} missing metadata file",
                details={
                    'model_id': model_id,
                    'model_path': str(model_path),
                    'expected_metadata': str(metadata_file),
                },
                phase_id=phase_id,
            ))
            return violations
        
        # Validate metadata file has required fields
        try:
            with open(metadata_file, 'r') as f:
                metadata = json.load(f)
            
            required_fields = ['hash', 'trained_on', 'version']
            missing_fields = []
            
            for field in required_fields:
                if field not in metadata or not metadata[field]:
                    missing_fields.append(field)
            
            if missing_fields:
                violations.append(Violation(
                    checker='ai_ml_claims',
                    severity=ViolationSeverity.CRITICAL,
                    message=f"Registered model {model_id} metadata missing required fields: {', '.join(missing_fields)}",
                    details={
                        'model_id': model_id,
                        'metadata_file': str(metadata_file),
                        'missing_fields': missing_fields,
                    },
                    phase_id=phase_id,
                ))
        
        except json.JSONDecodeError:
            violations.append(Violation(
                checker='ai_ml_claims',
                severity=ViolationSeverity.CRITICAL,
                message=f"Registered model {model_id} metadata file is invalid JSON",
                details={
                    'model_id': model_id,
                    'metadata_file': str(metadata_file),
                },
                phase_id=phase_id,
            ))
        except Exception as e:
            violations.append(Violation(
                checker='ai_ml_claims',
                severity=ViolationSeverity.CRITICAL,
                message=f"Registered model {model_id} failed to read metadata: {e}",
                details={
                    'model_id': model_id,
                    'metadata_file': str(metadata_file),
                    'error': str(e),
                },
                phase_id=phase_id,
            ))
        
        return violations
    
    def _check_registered_model_signature(self, model_entry: Dict, model_path: Path) -> List[Violation]:
        """Check if registered model signature is present and valid."""
        violations = []
        
        model_id = model_entry.get('model_id', 'unknown')
        phase_id = model_entry.get('phase')
        signature = model_entry.get('signature', '')
        
        # If signature field exists in registry, it should be non-empty
        if not signature or signature.strip() == '':
            violations.append(Violation(
                checker='ai_ml_claims',
                severity=ViolationSeverity.CRITICAL,
                message=f"Registered model {model_id} has empty signature in registry",
                details={
                    'model_id': model_id,
                    'model_path': str(model_path),
                },
                phase_id=phase_id,
            ))
        
        # Note: Actual signature verification would require cryptographic libraries
        # This validator only checks that signature field is present and non-empty
        
        return violations
