# Path: /home/ransomeye/rebuild/ransomeye_governance/tests/ai_baseline_presence_tests.py
# Author: RansomEye Core Team
# Purpose: Tests that verify AI baseline enforcer fails if baseline artifacts are missing

"""
AI Baseline Presence Tests: Verify AI baseline enforcer blocks AI startup without baseline.
"""

import unittest
import sys
import tempfile
import json
from pathlib import Path

# Add tooling to path
sys.path.insert(0, str(Path(__file__).parent.parent / 'tooling'))

from ai_baseline_enforcer import AIBaselineEnforcer


class TestAIBaselinePresence(unittest.TestCase):
    """Test AI baseline presence enforcement."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = Path(tempfile.mkdtemp())
        self.baseline_path = self.temp_dir / "baseline_pack"
    
    def test_missing_baseline_fails(self):
        """Test that missing baseline pack fails."""
        enforcer = AIBaselineEnforcer(baseline_pack_path=self.baseline_path)
        
        presence_ok = enforcer.check_baseline_presence()
        self.assertFalse(presence_ok, "Missing baseline should fail")
        self.assertGreater(len(enforcer.violations), 0)
    
    def test_missing_baseline_components_fails(self):
        """Test that missing baseline components fail."""
        # Create baseline directory but no components
        self.baseline_path.mkdir(parents=True, exist_ok=True)
        
        enforcer = AIBaselineEnforcer(baseline_pack_path=self.baseline_path)
        
        components_ok = enforcer.check_baseline_components()
        self.assertFalse(components_ok, "Missing components should fail")
        self.assertGreater(len(enforcer.violations), 0)
    
    def test_missing_signature_fails(self):
        """Test that missing signature fails."""
        # Create baseline directory with some components
        self.baseline_path.mkdir(parents=True, exist_ok=True)
        (self.baseline_path / "models").mkdir()
        (self.baseline_path / "models" / "ransomware_behavior.pkl").touch()
        
        enforcer = AIBaselineEnforcer(baseline_pack_path=self.baseline_path)
        
        signature_ok = enforcer.check_baseline_signature()
        self.assertFalse(signature_ok, "Missing signature should fail")
    
    def test_missing_metadata_fails(self):
        """Test that missing metadata fails."""
        # Create baseline directory with signature
        self.baseline_path.mkdir(parents=True, exist_ok=True)
        (self.baseline_path / "signature.sig").touch()
        
        enforcer = AIBaselineEnforcer(baseline_pack_path=self.baseline_path)
        
        metadata_ok = enforcer.check_baseline_metadata()
        self.assertFalse(metadata_ok, "Missing metadata should fail")
    
    def test_invalid_metadata_fails(self):
        """Test that invalid metadata fails."""
        # Create baseline directory with invalid metadata
        self.baseline_path.mkdir(parents=True, exist_ok=True)
        (self.baseline_path / "signature.sig").touch()
        
        metadata_file = self.baseline_path / "metadata.json"
        metadata_file.write_text('{"invalid": "metadata"}\n')
        
        enforcer = AIBaselineEnforcer(baseline_pack_path=self.baseline_path)
        
        metadata_ok = enforcer.check_baseline_metadata()
        self.assertFalse(metadata_ok, "Invalid metadata should fail")
    
    def test_complete_baseline_passes(self):
        """Test that complete baseline passes."""
        # Create complete baseline structure
        self.baseline_path.mkdir(parents=True, exist_ok=True)
        
        # Create required components
        (self.baseline_path / "models").mkdir()
        (self.baseline_path / "models" / "ransomware_behavior.pkl").touch()
        (self.baseline_path / "models" / "anomaly_detection.pkl").touch()
        
        (self.baseline_path / "shap").mkdir()
        (self.baseline_path / "shap" / "baseline_distributions.json").touch()
        (self.baseline_path / "shap" / "explainers.pkl").touch()
        
        (self.baseline_path / "llm").mkdir()
        (self.baseline_path / "llm" / "rag_index").touch()
        
        (self.baseline_path / "intelligence").mkdir()
        (self.baseline_path / "intelligence" / "ioc_database.json").touch()
        
        # Create valid metadata
        metadata = {
            'version': '1.0.0',
            'hash': 'sha256:test',
            'trained_on': '2025-01-01',
            'signature': 'test_sig'
        }
        (self.baseline_path / "metadata.json").write_text(json.dumps(metadata))
        
        # Create signature
        (self.baseline_path / "signature.sig").touch()
        
        enforcer = AIBaselineEnforcer(baseline_pack_path=self.baseline_path)
        
        is_valid = enforcer.enforce()
        self.assertTrue(is_valid, "Complete baseline should pass")


class TestAIBaselineEnforcerIntegration(unittest.TestCase):
    """Integration tests for AI baseline enforcer."""
    
    def test_enforcer_fails_on_missing_baseline(self):
        """Test that enforcer fails when baseline is missing."""
        temp_dir = Path(tempfile.mkdtemp())
        baseline_path = temp_dir / "nonexistent_baseline"
        
        enforcer = AIBaselineEnforcer(baseline_pack_path=baseline_path)
        
        is_valid = enforcer.enforce()
        self.assertFalse(is_valid, "Should fail when baseline is missing")


if __name__ == '__main__':
    unittest.main()

