# Path and File Name : /home/ransomeye/rebuild/ransomeye_posture_engine/tests/test_evaluators.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Unit tests for CIS, STIG, and custom policy evaluators

"""
Unit tests for evaluators.
"""

import unittest
import tempfile
import yaml
from pathlib import Path
from datetime import datetime
from ransomeye_posture_engine.engine.cis_evaluator import CISEvaluator
from ransomeye_posture_engine.engine.stig_evaluator import STIGEvaluator
from ransomeye_posture_engine.engine.custom_policy_evaluator import CustomPolicyEvaluator
from ransomeye_posture_engine.engine.normalizer import PostureFact, PostureCategory

class TestCISEvaluator(unittest.TestCase):
    """Test CIS evaluator."""
    
    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.benchmarks_dir = Path(self.temp_dir)
        self.benchmarks_dir.mkdir(parents=True, exist_ok=True)
        
        # Create sample CIS control
        control_data = {
            'controls': [
                {
                    'id': 'CIS-1.1.1',
                    'title': 'Test Control',
                    'description': 'Test control description',
                    'severity': 'high',
                    'check_type': 'deterministic',
                    'check_logic': {
                        'type': 'absence',
                        'fact_types': ['privileged_execution'],
                    },
                }
            ]
        }
        
        with open(self.benchmarks_dir / 'test_benchmark.yaml', 'w') as f:
            yaml.dump(control_data, f)
        
        self.evaluator = CISEvaluator(self.benchmarks_dir)
    
    def test_load_controls(self):
        """Test loading CIS controls."""
        self.assertGreater(len(self.evaluator.controls), 0)
    
    def test_evaluate_compliant(self):
        """Test evaluation with compliant facts."""
        facts = []  # No violating facts
        results = self.evaluator.evaluate(facts)
        self.assertGreater(len(results), 0)

class TestSTIGEvaluator(unittest.TestCase):
    """Test STIG evaluator."""
    
    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.stig_dir = Path(self.temp_dir)
        self.stig_dir.mkdir(parents=True, exist_ok=True)
        
        # Create sample STIG requirement
        req_data = {
            'requirements': [
                {
                    'id': 'STIG-001',
                    'title': 'Test Requirement',
                    'description': 'Test requirement description',
                    'severity': 'CAT II',
                    'vuln_id': 'V-12345',
                    'check_type': 'deterministic',
                    'check_logic': {
                        'type': 'absence',
                        'fact_types': ['privileged_execution'],
                    },
                }
            ]
        }
        
        with open(self.stig_dir / 'test_stig.yaml', 'w') as f:
            yaml.dump(req_data, f)
        
        self.evaluator = STIGEvaluator(self.stig_dir)
    
    def test_load_requirements(self):
        """Test loading STIG requirements."""
        self.assertGreater(len(self.evaluator.requirements), 0)

class TestCustomPolicyEvaluator(unittest.TestCase):
    """Test custom policy evaluator."""
    
    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.policies_dir = Path(self.temp_dir)
        self.policies_dir.mkdir(parents=True, exist_ok=True)
        
        # Create sample custom policy
        policy_data = {
            'policies': [
                {
                    'policy_id': 'CUSTOM-001',
                    'name': 'Test Policy',
                    'description': 'Test policy description',
                    'severity': 'medium',
                    'enabled': True,
                    'check_logic': {
                        'type': 'threshold',
                        'threshold': 5,
                        'fact_types': ['privileged_execution'],
                    },
                }
            ]
        }
        
        with open(self.policies_dir / 'test_policy.yaml', 'w') as f:
            yaml.dump(policy_data, f)
        
        self.evaluator = CustomPolicyEvaluator(self.policies_dir)
    
    def test_load_policies(self):
        """Test loading custom policies."""
        self.assertGreater(len(self.evaluator.policies), 0)

if __name__ == '__main__':
    unittest.main()

