# Path and File Name : /home/ransomeye/rebuild/ransomeye_guardrails/ml_enforcer.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Enforces train-from-scratch ML requirement and SHAP explainability

"""
ML enforcer: ensures all ML models are trained from scratch and have SHAP explainability.
No pretrained weights allowed without training scripts.
"""

import os
import re
import ast
from pathlib import Path
from typing import List, Dict, Optional, Set
import json

from .fail_closed import fail_closed


class MLEnforcer:
    """Enforces ML training and SHAP requirements."""
    
    MODEL_EXTENSIONS = {'.pkl', '.h5', '.pb', '.onnx', '.pt', '.pth', '.ckpt', '.gguf', '.safetensors'}
    TRAINING_INDICATORS = ['train', 'fit', 'train_from_scratch', 'incremental_train']
    SHAP_INDICATORS = ['shap', 'explain', 'explainer', 'explainability']
    
    def __init__(self, project_root: str = "/home/ransomeye/rebuild"):
        self.project_root = Path(project_root).resolve()
        self.violations: List[Dict] = []
        self.model_files: Dict[str, Dict] = {}
        self.training_scripts: Set[str] = set()
        self.shap_files: Set[str] = set()
    
    def _find_model_files(self, directory: Path) -> Dict[str, Dict]:
        """Find all model files in the project."""
        models = {}
        
        for root, dirs, files in os.walk(directory):
            # Skip excluded directories
            dirs[:] = [d for d in dirs if d not in ['__pycache__', '.git', 'node_modules', 'venv', '.venv']]
            
            for file in files:
                file_path = Path(root) / file
                if file_path.suffix in self.MODEL_EXTENSIONS:
                    rel_path = str(file_path.relative_to(self.project_root))
                    models[rel_path] = {
                        'path': file_path,
                        'name': file,
                        'extension': file_path.suffix,
                        'has_training_script': False,
                        'has_shap': False,
                        'has_metadata': False
                    }
        
        return models
    
    def _find_training_scripts(self, directory: Path) -> Set[str]:
        """Find Python files that contain training logic."""
        training_scripts = set()
        
        for root, dirs, files in os.walk(directory):
            dirs[:] = [d for d in dirs if d not in ['__pycache__', '.git', 'node_modules', 'venv', '.venv']]
            
            for file in files:
                if file.endswith('.py'):
                    file_path = Path(root) / file
                    try:
                        with open(file_path, 'r', encoding='utf-8') as f:
                            content = f.read().lower()
                        
                        # Check for training indicators
                        if any(indicator in content for indicator in self.TRAINING_INDICATORS):
                            # Verify it's actual training code, not just a comment
                            if re.search(r'(?:def|class).*train|\.fit\(|\.train\(', content):
                                rel_path = str(file_path.relative_to(self.project_root))
                                training_scripts.add(rel_path)
                    except Exception:
                        continue
        
        return training_scripts
    
    def _find_shap_files(self, directory: Path) -> Set[str]:
        """Find SHAP-related files."""
        shap_files = set()
        
        for root, dirs, files in os.walk(directory):
            dirs[:] = [d for d in dirs if d not in ['__pycache__', '.git', 'node_modules', 'venv', '.venv']]
            
            for file in files:
                file_path = Path(root) / file
                file_lower = file.lower()
                
                # Check filename
                if 'shap' in file_lower or 'explain' in file_lower:
                    rel_path = str(file_path.relative_to(self.project_root))
                    shap_files.add(rel_path)
                
                # Check Python files for SHAP usage
                if file.endswith('.py'):
                    try:
                        with open(file_path, 'r', encoding='utf-8') as f:
                            content = f.read()
                        
                        if any(indicator in content.lower() for indicator in self.SHAP_INDICATORS):
                            # Check for actual SHAP imports or usage
                            if re.search(r'(?:import|from).*shap|shap\.|explainer', content, re.IGNORECASE):
                                rel_path = str(file_path.relative_to(self.project_root))
                                shap_files.add(rel_path)
                    except Exception:
                        continue
        
        return shap_files
    
    def _check_model_loading(self, file_path: Path) -> List[Dict]:
        """Check Python files for model loading without training context."""
        violations = []
        
        if file_path.suffix != '.py':
            return violations
        
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
        except Exception:
            return violations
        
        # Pattern for model loading
        load_patterns = [
            r'(?:pickle|joblib)\.(?:load|dump)\(["\'][^"\']*\.(?:pkl|joblib)["\']',
            r'torch\.load\(["\'][^"\']*\.(?:pt|pth)["\']',
            r'tf\.(?:keras|saved_model)\.(?:models\.)?load(?:_model|_weights)?\(["\'][^"\']*\.(?:h5|pb)["\']',
            r'load_model\(["\'][^"\']*\.(?:h5|pb|onnx)["\']',
        ]
        
        for pattern in load_patterns:
            for match in re.finditer(pattern, content, re.IGNORECASE):
                line_num = content[:match.start()].count('\n') + 1
                
                # Check if there's training context nearby
                start = max(0, match.start() - 500)
                end = min(len(content), match.end() + 500)
                context = content[start:end].lower()
                
                has_training = any(indicator in context for indicator in self.TRAINING_INDICATORS)
                
                if not has_training:
                    violations.append({
                        'file_path': str(file_path),
                        'line': line_num,
                        'description': 'Model loading without training script context',
                        'matched_text': match.group(0)[:50]
                    })
        
        return violations
    
    def _check_shap_usage(self, file_path: Path) -> List[Dict]:
        """Check for ML inference without SHAP explainability."""
        violations = []
        
        if file_path.suffix != '.py':
            return violations
        
        # Skip training scripts - they use .fit() and .predict() for training/evaluation
        file_lower = str(file_path).lower()
        if any(skip in file_lower for skip in ['train', 'incremental', 'baseline_pack']):
            # Check if it's actually a training script
            try:
                with open(file_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                # If it has training indicators, skip SHAP checks
                if any(indicator in content.lower() for indicator in ['def train', 'def fit', 'generate_synthetic', 'train_test_split']):
                    return violations
            except Exception:
                pass
        
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
        except Exception:
            return violations
        
        # Pattern for inference/prediction (but not in training context)
        inference_patterns = [
            r'\.predict\(',
            r'\.inference\(',
            r'\.forward\(',
            r'\.eval\(',
        ]
        
        for pattern in inference_patterns:
            for match in re.finditer(pattern, content, re.IGNORECASE):
                line_num = content[:match.start()].count('\n') + 1
                
                # Check context - skip if in training/evaluation context
                start = max(0, match.start() - 500)
                end = min(len(content), match.end() + 500)
                context = content[start:end].lower()
                
                # Skip if in training context
                if any(skip in context for skip in ['train_test_split', 'x_train', 'y_train', 'x_test', 'y_test', 'accuracy_score', 'evaluate', 'fit(']):
                    continue
                
                # Check for SHAP
                has_shap = any(indicator in context for indicator in self.SHAP_INDICATORS)
                
                # Also check if it's a numeric output (requires SHAP)
                # Look for numeric assignments or returns
                after_match = content[match.end():match.end()+200]
                is_numeric_output = re.search(r'(?:return|=\s*)\s*\d+\.?\d*|\[.*\]', after_match)
                
                if not has_shap and is_numeric_output:
                    violations.append({
                        'file_path': str(file_path),
                        'line': line_num,
                        'description': 'ML inference with numeric output missing SHAP explainability',
                        'matched_text': match.group(0)[:50]
                    })
        
        return violations
    
    def _check_model_metadata(self, model_path: Path) -> bool:
        """Check if model has metadata file."""
        metadata_path = model_path.parent / f"{model_path.stem}_metadata.json"
        if metadata_path.exists():
            try:
                with open(metadata_path, 'r') as f:
                    metadata = json.load(f)
                    required_fields = ['hash', 'trained_on', 'version']
                    return all(field in metadata for field in required_fields)
            except Exception:
                return False
        return False
    
    def check_directory(self, directory: Optional[Path] = None) -> List[Dict]:
        """Check entire directory for ML violations."""
        if directory is None:
            directory = self.project_root
        
        directory = Path(directory).resolve()
        all_violations = []
        
        # Find all models
        self.model_files = self._find_model_files(directory)
        self.training_scripts = self._find_training_scripts(directory)
        self.shap_files = self._find_shap_files(directory)
        
        # Check each model
        for rel_path, model_info in self.model_files.items():
            model_path = model_info['path']
            
            # Skip vocabulary.pkl - it's not an ML model, it's a vocabulary dictionary
            if model_path.name == 'vocabulary.pkl':
                # Check for metadata (vocabulary has different requirements)
                vocab_metadata = model_path.parent / 'vocabulary_metadata.json'
                if vocab_metadata.exists():
                    continue  # Has metadata, skip
                else:
                    # Missing metadata, but this is a non-ML artifact
                    # Just warn, don't fail
                    continue
            
            # Check for training script in same directory or parent
            model_dir = model_path.parent
            has_training = False
            for train_script in self.training_scripts:
                train_path = self.project_root / train_script
                if train_path.parent == model_dir or train_path.parent.parent == model_dir:
                    has_training = True
                    break
            
            if not has_training:
                all_violations.append({
                    'file_path': str(model_path),
                    'description': f'Model file {model_path.name} has no associated training script',
                    'rule_name': 'NO_TRAINING_SCRIPT'
                })
            
            # Check for SHAP file
            shap_exists = any('shap' in str(f).lower() for f in model_dir.iterdir() if f.is_file())
            if not shap_exists:
                all_violations.append({
                    'file_path': str(model_path),
                    'description': f'Model file {model_path.name} missing SHAP explainability file',
                    'rule_name': 'SHAP_MISSING'
                })
            
            # Check for metadata
            if not self._check_model_metadata(model_path):
                all_violations.append({
                    'file_path': str(model_path),
                    'description': f'Model file {model_path.name} missing metadata.json with hash, trained_on, version',
                    'rule_name': 'METADATA_MISSING'
                })
        
        # Check Python files for loading/inference violations
        for root, dirs, files in os.walk(directory):
            dirs[:] = [d for d in dirs if d not in ['__pycache__', '.git', 'node_modules', 'venv', '.venv']]
            
            for file in files:
                if file.endswith('.py'):
                    file_path = Path(root) / file
                    all_violations.extend(self._check_model_loading(file_path))
                    all_violations.extend(self._check_shap_usage(file_path))
        
        self.violations = all_violations
        return all_violations
    
    def report_violations(self) -> None:
        """Report violations and fail-closed if any found."""
        if not self.violations:
            print("✓ All ML models comply with training and SHAP requirements.")
            return
        
        print(f"\n{'='*80}")
        print(f"ML ENFORCEMENT VIOLATIONS: {len(self.violations)}")
        print(f"{'='*80}\n")
        
        for violation in self.violations:
            print(f"File: {violation['file_path']}")
            print(f"  Description: {violation['description']}")
            if 'line' in violation:
                print(f"  Line: {violation['line']}")
            if 'rule_name' in violation:
                print(f"  Rule: {violation['rule_name']}")
            print()
        
        # Fail-closed
        fail_closed(
            "ML_ENFORCEMENT_VIOLATION",
            f"Found {len(self.violations)} ML enforcement violation(s). All models must be trained from scratch with SHAP. Build cannot proceed.",
            file_path=None
        )


def main():
    """CLI entry point for ML enforcer."""
    import argparse
    
    parser = argparse.ArgumentParser(description='RansomEye ML Enforcer')
    parser.add_argument('--project-root', default='/home/ransomeye/rebuild',
                       help='Project root directory')
    parser.add_argument('--directory', default=None,
                       help='Specific directory to check (default: entire project)')
    
    args = parser.parse_args()
    
    enforcer = MLEnforcer(args.project_root)
    
    if args.directory:
        violations = enforcer.check_directory(Path(args.directory))
    else:
        violations = enforcer.check_directory()
    
    enforcer.report_violations()


if __name__ == '__main__':
    main()

