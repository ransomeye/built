# Path and File Name : /home/ransomeye/rebuild/core/global_validator/tests/test_registry_boundary.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Tests for AI/ML Registry Boundary Enforcement

"""
Test Suite for AI/ML Registry Boundary Enforcement

These tests verify that:
1. .pkl outside registry → ignored (no violation)
2. Registered model without SHAP → FAIL
3. Registered model without metadata → FAIL
4. Registered model with all artifacts → PASS
"""

import unittest
import tempfile
import shutil
import json
from pathlib import Path
import sys

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent))

from core.global_validator.ai_ml_claims import AIMLClaimValidator
from core.global_validator.validator import GlobalForensicValidator, ViolationSeverity


class TestRegistryBoundary(unittest.TestCase):
    """Test registry boundary enforcement."""
    
    def setUp(self):
        """Set up test environment."""
        self.test_dir = Path(tempfile.mkdtemp(prefix="registry_test_"))
        self.test_project_root = self.test_dir / "rebuild"
        self.test_project_root.mkdir(parents=True)
        
        # Create registry structure
        self.registry_dir = self.test_project_root / "ransomeye_intelligence" / "model_registry"
        self.registry_dir.mkdir(parents=True)
        self.registry_file = self.registry_dir / "registry.json"
        
        # Create guardrails structure
        self.guardrails_dir = self.test_project_root / "core" / "guardrails"
        self.guardrails_dir.mkdir(parents=True)
        self.guardrails_file = self.guardrails_dir / "guardrails.yaml"
        
        # Create minimal guardrails
        guardrails = {
            'allowed_phases': [
                {
                    'id': 3,
                    'name': 'Alert Engine & Policy Manager',
                    'path': str(self.test_project_root / "ransomeye_intelligence"),
                    'status': 'IMPLEMENTED',
                    'installable': True,
                    'runnable': True,
                }
            ]
        }
        
        import yaml
        with open(self.guardrails_file, 'w') as f:
            yaml.dump(guardrails, f)
    
    def tearDown(self):
        """Clean up test environment."""
        shutil.rmtree(self.test_dir, ignore_errors=True)
    
    def test_pkl_outside_registry_ignored(self):
        """Test: .pkl file outside registry → ignored (no violation)."""
        # Create a .pkl file outside registry
        random_pkl = self.test_project_root / "ransomeye_intelligence" / "some_random_model.pkl"
        random_pkl.parent.mkdir(parents=True, exist_ok=True)
        random_pkl.write_bytes(b"fake model data")
        
        # Create empty registry (no models registered)
        registry = {
            'version': '1.0.0',
            'registry_root': str(self.registry_dir) + '/',
            'models': []
        }
        with open(self.registry_file, 'w') as f:
            json.dump(registry, f)
        
        # Create validator
        validator = GlobalForensicValidator(project_root=self.test_project_root)
        ai_ml_checker = AIMLClaimValidator(validator)
        
        # Run validation - should pass (unregistered model ignored)
        result = ai_ml_checker.validate()
        
        # Should pass - no violations for unregistered models
        self.assertTrue(result.passed, "Unregistered models should be ignored")
        self.assertEqual(len(result.violations), 0, "Should have no violations for unregistered models")
    
    def test_registered_model_without_shap_fails(self):
        """Test: Registered model without SHAP → FAIL."""
        # Create a model file
        model_file = self.registry_dir / "test_model.pkl"
        model_file.write_bytes(b"fake model data")
        
        # Register model in registry (without SHAP file)
        registry = {
            'version': '1.0.0',
            'registry_root': str(self.registry_dir) + '/',
            'models': [
                {
                    'model_id': 'test-model-1',
                    'phase': 3,
                    'path': 'test_model.pkl',
                    'hash': 'sha256-hash',
                    'trained_on': 'test-dataset',
                    'version': '1.0.0',
                    'requires_shap': True,
                    'signature': 'ed25519-signature'
                }
            ]
        }
        with open(self.registry_file, 'w') as f:
            json.dump(registry, f)
        
        # Create validator
        validator = GlobalForensicValidator(project_root=self.test_project_root)
        ai_ml_checker = AIMLClaimValidator(validator)
        
        # Run validation - should fail
        result = ai_ml_checker.validate()
        
        # Should fail - missing SHAP
        self.assertFalse(result.passed, "Registered model without SHAP should fail")
        self.assertGreater(len(result.violations), 0, "Should have violations")
        
        violation_messages = [v.message for v in result.violations]
        self.assertTrue(
            any('SHAP' in msg for msg in violation_messages),
            "Should detect missing SHAP file"
        )
    
    def test_registered_model_without_metadata_fails(self):
        """Test: Registered model without metadata → FAIL."""
        # Create a model file
        model_file = self.registry_dir / "test_model.pkl"
        model_file.write_bytes(b"fake model data")
        
        # Create SHAP file
        shap_file = self.registry_dir / "test_model_shap.json"
        shap_file.write_text('{"shap": "values"}')
        
        # Register model in registry (without metadata file)
        registry = {
            'version': '1.0.0',
            'registry_root': str(self.registry_dir) + '/',
            'models': [
                {
                    'model_id': 'test-model-1',
                    'phase': 3,
                    'path': 'test_model.pkl',
                    'hash': 'sha256-hash',
                    'trained_on': 'test-dataset',
                    'version': '1.0.0',
                    'requires_shap': True,
                    'signature': 'ed25519-signature'
                }
            ]
        }
        with open(self.registry_file, 'w') as f:
            json.dump(registry, f)
        
        # Create validator
        validator = GlobalForensicValidator(project_root=self.test_project_root)
        ai_ml_checker = AIMLClaimValidator(validator)
        
        # Run validation - should fail
        result = ai_ml_checker.validate()
        
        # Should fail - missing metadata
        self.assertFalse(result.passed, "Registered model without metadata should fail")
        self.assertGreater(len(result.violations), 0, "Should have violations")
        
        violation_messages = [v.message for v in result.violations]
        self.assertTrue(
            any('metadata' in msg.lower() for msg in violation_messages),
            "Should detect missing metadata file"
        )
    
    def test_registered_model_with_all_artifacts_passes(self):
        """Test: Registered model with all artifacts → PASS."""
        # Create a model file
        model_file = self.registry_dir / "test_model.pkl"
        model_file.write_bytes(b"fake model data")
        
        # Create SHAP file
        shap_file = self.registry_dir / "test_model_shap.json"
        shap_file.write_text('{"shap": "values"}')
        
        # Create metadata file with required fields
        metadata_file = self.registry_dir / "test_model_metadata.json"
        metadata = {
            'hash': 'sha256-hash',
            'trained_on': 'test-dataset',
            'version': '1.0.0'
        }
        metadata_file.write_text(json.dumps(metadata))
        
        # Register model in registry
        registry = {
            'version': '1.0.0',
            'registry_root': str(self.registry_dir) + '/',
            'models': [
                {
                    'model_id': 'test-model-1',
                    'phase': 3,
                    'path': 'test_model.pkl',
                    'hash': 'sha256-hash',
                    'trained_on': 'test-dataset',
                    'version': '1.0.0',
                    'requires_shap': True,
                    'signature': 'ed25519-signature'
                }
            ]
        }
        with open(self.registry_file, 'w') as f:
            json.dump(registry, f)
        
        # Create validator
        validator = GlobalForensicValidator(project_root=self.test_project_root)
        ai_ml_checker = AIMLClaimValidator(validator)
        
        # Run validation - should pass
        result = ai_ml_checker.validate()
        
        # Should pass - all artifacts present
        self.assertTrue(result.passed, "Registered model with all artifacts should pass")
        self.assertEqual(len(result.violations), 0, "Should have no violations")
    
    def test_excluded_paths_ignored(self):
        """Test: Files in excluded paths (.venv, site-packages) → ignored."""
        # Create .pkl file in .venv (should be ignored)
        venv_dir = self.test_project_root / "ransomeye_intelligence" / ".venv" / "lib" / "site-packages"
        venv_dir.mkdir(parents=True, exist_ok=True)
        venv_pkl = venv_dir / "dependency_model.pkl"
        venv_pkl.write_bytes(b"dependency data")
        
        # Create empty registry
        registry = {
            'version': '1.0.0',
            'registry_root': str(self.registry_dir) + '/',
            'models': []
        }
        with open(self.registry_file, 'w') as f:
            json.dump(registry, f)
        
        # Create validator
        validator = GlobalForensicValidator(project_root=self.test_project_root)
        ai_ml_checker = AIMLClaimValidator(validator)
        
        # Run validation - should pass (excluded paths ignored)
        result = ai_ml_checker.validate()
        
        # Should pass - excluded paths ignored
        self.assertTrue(result.passed, "Excluded paths should be ignored")
        self.assertEqual(len(result.violations), 0, "Should have no violations for excluded paths")


if __name__ == '__main__':
    unittest.main()

