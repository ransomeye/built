# Path and File Name : /home/ransomeye/rebuild/ransomeye_guardrails/phase6_validator.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Phase 6 validation - AI/ML/LLM training, explainability & fail-closed governance

"""
Phase 6 Validator: Validates AI/ML/LLM training, explainability, and fail-closed governance.
"""

import os
import sys
import json
import re
from pathlib import Path
from typing import List, Dict, Set, Optional
from collections import defaultdict

from .fail_closed import fail_closed


class Phase6Validator:
    """Validates Phase 6 requirements."""
    
    def __init__(self, project_root: str = "/home/ransomeye/rebuild"):
        self.project_root = Path(project_root).resolve()
        self.violations: List[Dict] = []
        self.models: Dict[str, Dict] = {}
        self.training_scripts: Set[str] = set()
        self.shap_files: Set[str] = set()
        self.incremental_scripts: Set[str] = set()
    
    def validate_all(self) -> bool:
        """Run all validations."""
        print("=" * 80)
        print("PHASE 6 VALIDATION: AI/ML/LLM Training, Explainability & Fail-Closed Governance")
        print("=" * 80)
        print()
        
        # 1. Model Inventory & Classification
        print("1. Model Inventory & Classification...")
        self._inventory_models()
        print(f"   ✓ Found {len(self.models)} models")
        print()
        
        # 2. Training Pipeline Enforcement
        print("2. Training Pipeline Enforcement...")
        self._validate_training_pipelines()
        print()
        
        # 3. SHAP Explainability
        print("3. SHAP Explainability...")
        self._validate_shap()
        print()
        
        # 4. Model Integrity & Signing
        print("4. Model Integrity & Signing...")
        self._validate_model_signing()
        print()
        
        # 5. Resource Governance
        print("5. Resource Governance...")
        self._validate_resource_governance()
        print()
        
        # 6. LLM/RAG Safety
        print("6. LLM/RAG Safety...")
        self._validate_llm_rag_safety()
        print()
        
        # Report results
        if self.violations:
            print("=" * 80)
            print(f"VALIDATION FAILED: {len(self.violations)} violation(s)")
            print("=" * 80)
            for violation in self.violations:
                print(f"  - {violation['rule']}: {violation['description']}")
                if 'file' in violation:
                    print(f"    File: {violation['file']}")
            print()
            return False
        else:
            print("=" * 80)
            print("✓ PHASE 6 VALIDATION PASSED")
            print("=" * 80)
            print()
            return True
    
    def _inventory_models(self):
        """Enumerate all models."""
        model_extensions = {'.pkl', '.h5', '.pb', '.onnx', '.pt', '.pth', '.ckpt', '.gguf', '.safetensors'}
        
        for root, dirs, files in os.walk(self.project_root):
            dirs[:] = [d for d in dirs if d not in ['__pycache__', '.git', 'node_modules', 'venv', '.venv', 'target']]
            
            for file in files:
                file_path = Path(root) / file
                if file_path.suffix in model_extensions:
                    rel_path = str(file_path.relative_to(self.project_root))
                    self.models[rel_path] = {
                        'path': file_path,
                        'name': file,
                        'extension': file_path.suffix,
                        'type': self._classify_model(file_path)
                    }
    
    def _classify_model(self, model_path: Path) -> str:
        """Classify model type."""
        name_lower = model_path.name.lower()
        
        if 'vocabulary' in name_lower:
            return 'vocabulary'
        elif 'behavior' in name_lower or 'classifier' in name_lower:
            return 'ml_classifier'
        elif 'anomaly' in name_lower:
            return 'ml_anomaly'
        elif 'calibration' in name_lower:
            return 'ml_calibration'
        elif 'embedding' in name_lower or 'rag' in name_lower:
            return 'embedding'
        elif model_path.suffix in {'.gguf', '.safetensors'}:
            return 'llm'
        else:
            return 'unknown'
    
    def _validate_training_pipelines(self):
        """Validate training pipelines."""
        # Find training scripts
        for root, dirs, files in os.walk(self.project_root):
            dirs[:] = [d for d in dirs if d not in ['__pycache__', '.git', 'node_modules', 'venv', '.venv', 'target']]
            
            for file in files:
                if file.endswith('.py'):
                    file_path = Path(root) / file
                    try:
                        with open(file_path, 'r', encoding='utf-8') as f:
                            content = f.read().lower()
                        
                        # Check for training indicators
                        if any(indicator in content for indicator in ['def train', '.fit(', 'train_from_scratch']):
                            if re.search(r'(?:def|class).*train|\.fit\(|train_from_scratch', content):
                                rel_path = str(file_path.relative_to(self.project_root))
                                self.training_scripts.add(rel_path)
                        
                        # Check for incremental learning
                        if any(indicator in content for indicator in ['incremental', 'continual', 'update_model']):
                            if re.search(r'incremental|continual|update_model', content):
                                rel_path = str(file_path.relative_to(self.project_root))
                                self.incremental_scripts.add(rel_path)
                    except Exception:
                        continue
        
        print(f"   ✓ Found {len(self.training_scripts)} training scripts")
        print(f"   ✓ Found {len(self.incremental_scripts)} incremental learning scripts")
        
        # Check each model has training script
        for rel_path, model_info in self.models.items():
            if model_info['type'] == 'vocabulary':
                continue  # Vocabulary has build script
            
            model_path = model_info['path']
            model_dir = model_path.parent
            
            # Check for training script
            has_training = False
            for train_script in self.training_scripts:
                train_path = self.project_root / train_script
                if train_path.parent == model_dir or train_path.parent.parent == model_dir:
                    has_training = True
                    break
            
            if not has_training:
                self.violations.append({
                    'rule': 'NO_TRAINING_SCRIPT',
                    'description': f'Model {model_path.name} has no associated training script',
                    'file': str(model_path)
                })
        
        # Check for incremental learning support
        if len(self.incremental_scripts) == 0:
            self.violations.append({
                'rule': 'NO_INCREMENTAL_LEARNING',
                'description': 'No incremental learning scripts found',
                'file': None
            })
    
    def _validate_shap(self):
        """Validate SHAP explainability."""
        # Find SHAP files
        for root, dirs, files in os.walk(self.project_root):
            dirs[:] = [d for d in dirs if d not in ['__pycache__', '.git', 'node_modules', 'venv', '.venv', 'target']]
            
            for file in files:
                file_path = Path(root) / file
                file_lower = file.lower()
                
                if 'shap' in file_lower or 'explain' in file_lower:
                    rel_path = str(file_path.relative_to(self.project_root))
                    self.shap_files.add(rel_path)
        
        print(f"   ✓ Found {len(self.shap_files)} SHAP-related files")
        
        # Check Rust SHAP implementation
        shap_rust_files = []
        for root, dirs, files in os.walk(self.project_root):
            dirs[:] = [d for d in dirs if d not in ['__pycache__', '.git', 'node_modules', 'venv', '.venv', 'target']]
            
            for file in files:
                if file.endswith('.rs'):
                    file_path = Path(root) / file
                    try:
                        with open(file_path, 'r', encoding='utf-8') as f:
                            content = f.read()
                        
                        if 'shap' in content.lower() or 'explain' in content.lower():
                            shap_rust_files.append(str(file_path.relative_to(self.project_root)))
                    except Exception:
                        continue
        
        if len(shap_rust_files) == 0:
            self.violations.append({
                'rule': 'NO_SHAP_IMPLEMENTATION',
                'description': 'No SHAP implementation found in Rust code',
                'file': None
            })
        else:
            print(f"   ✓ Found {len(shap_rust_files)} SHAP Rust files")
    
    def _validate_model_signing(self):
        """Validate model signing."""
        # Check for Ed25519 signing support
        ed25519_files = []
        for root, dirs, files in os.walk(self.project_root):
            dirs[:] = [d for d in dirs if d not in ['__pycache__', '.git', 'node_modules', 'venv', '.venv', 'target']]
            
            for file in files:
                if file.endswith(('.rs', '.py')):
                    file_path = Path(root) / file
                    try:
                        with open(file_path, 'r', encoding='utf-8') as f:
                            content = f.read()
                        
                        if 'ed25519' in content.lower() and ('model' in content.lower() or 'sign' in content.lower()):
                            ed25519_files.append(str(file_path.relative_to(self.project_root)))
                    except Exception:
                        continue
        
        if len(ed25519_files) == 0:
            self.violations.append({
                'rule': 'NO_ED25519_MODEL_SIGNING',
                'description': 'No Ed25519 model signing implementation found',
                'file': None
            })
        else:
            print(f"   ✓ Found {len(ed25519_files)} Ed25519 model signing files")
    
    def _validate_resource_governance(self):
        """Validate resource governance."""
        # Check for CPU/memory limits
        resource_files = []
        for root, dirs, files in os.walk(self.project_root):
            dirs[:] = [d for d in dirs if d not in ['__pycache__', '.git', 'node_modules', 'venv', '.venv', 'target']]
            
            for file in files:
                if file.endswith(('.rs', '.py')):
                    file_path = Path(root) / file
                    try:
                        with open(file_path, 'r', encoding='utf-8') as f:
                            content = f.read()
                        
                        if any(term in content.lower() for term in ['memory_limit', 'cpu_limit', 'resource', 'governor']):
                            if 'ai' in str(file_path).lower() or 'model' in str(file_path).lower():
                                resource_files.append(str(file_path.relative_to(self.project_root)))
                    except Exception:
                        continue
        
        if len(resource_files) == 0:
            self.violations.append({
                'rule': 'NO_RESOURCE_GOVERNANCE',
                'description': 'No resource governance (CPU/memory limits) found for AI models',
                'file': None
            })
        else:
            print(f"   ✓ Found {len(resource_files)} resource governance files")
    
    def _validate_llm_rag_safety(self):
        """Validate LLM/RAG safety."""
        # Check for deterministic prompts and bounded context
        llm_files = []
        for root, dirs, files in os.walk(self.project_root):
            dirs[:] = [d for d in dirs if d not in ['__pycache__', '.git', 'node_modules', 'venv', '.venv', 'target']]
            
            for file in files:
                if file.endswith(('.rs', '.py')):
                    file_path = Path(root) / file
                    try:
                        with open(file_path, 'r', encoding='utf-8') as f:
                            content = f.read()
                        
                        if any(term in content.lower() for term in ['rag', 'llm', 'copilot', 'assistant']):
                            llm_files.append(str(file_path.relative_to(self.project_root)))
                    except Exception:
                        continue
        
        print(f"   ✓ Found {len(llm_files)} LLM/RAG files")
        
        # Check for context window bounds
        has_bounds = False
        for file_path_str in llm_files:
            file_path = self.project_root / file_path_str
            try:
                with open(file_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                
                if any(term in content.lower() for term in ['max_tokens', 'context_window', 'bounded', 'limit']):
                    has_bounds = True
                    break
            except Exception:
                continue
        
        if not has_bounds:
            self.violations.append({
                'rule': 'NO_CONTEXT_BOUNDS',
                'description': 'No context window bounds found for LLM/RAG',
                'file': None
            })


def main():
    """CLI entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description='RansomEye Phase 6 Validator')
    parser.add_argument('--project-root', default='/home/ransomeye/rebuild',
                       help='Project root directory')
    
    args = parser.parse_args()
    
    validator = Phase6Validator(args.project_root)
    success = validator.validate_all()
    
    if not success:
        fail_closed(
            "PHASE6_VALIDATION_FAILED",
            f"Phase 6 validation failed with {len(validator.violations)} violation(s)",
            file_path=None
        )
        sys.exit(1)
    
    sys.exit(0)


if __name__ == '__main__':
    main()

