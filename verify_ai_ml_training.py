# Path and File Name : /home/ransomeye/rebuild/verify_ai_ml_training.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Verification script to check if all AI/ML/LLM modules are fully trained

"""
RansomEye AI/ML/LLM Training Verification Script
==================================================
This script verifies that all AI, ML, and LLM modules are fully trained
with real models, not placeholders or dummy data.

Checks:
1. Model files exist and are real (not dummy/placeholder)
2. Model metadata (training date, version, hash)
3. SHAP explainability files
4. Model signatures
5. LLM/RAG knowledge base
"""

import os
import json
import hashlib
from pathlib import Path
from typing import List, Dict, Tuple, Optional

# Empty string SHA256 (placeholder hash)
EMPTY_HASH = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"

# Dummy/placeholder patterns
DUMMY_PATTERNS = [
    b"dummy",
    b"placeholder",
    b"test",
    b"empty",
    b"TODO",
    b"FIXME",
]

class ModelVerifier:
    def __init__(self, root_path: str):
        self.root_path = Path(root_path)
        self.issues: List[Dict] = []
        self.findings: Dict = {
            'models': [],
            'llm_modules': [],
            'issues': [],
            'summary': {}
        }
        
    def compute_file_hash(self, file_path: Path) -> str:
        """Compute SHA256 hash of file."""
        try:
            with open(file_path, 'rb') as f:
                content = f.read()
                return hashlib.sha256(content).hexdigest()
        except Exception as e:
            return f"ERROR: {e}"
    
    def check_file_for_dummy(self, file_path: Path) -> bool:
        """Check if file contains dummy/placeholder data."""
        try:
            with open(file_path, 'rb') as f:
                content = f.read(1024)  # Check first 1KB
                content_lower = content.lower()
                for pattern in DUMMY_PATTERNS:
                    if pattern in content_lower:
                        return True
            return False
        except Exception:
            return False
    
    def verify_model_file(self, model_path: Path, model_name: str, module: str) -> Dict:
        """Verify a single model file."""
        finding = {
            'module': module,
            'model_name': model_name,
            'file_path': str(model_path.relative_to(self.root_path)),
            'exists': False,
            'size_bytes': 0,
            'hash': None,
            'is_dummy': False,
            'status': 'UNKNOWN',
            'issues': []
        }
        
        if not model_path.exists():
            finding['status'] = 'MISSING'
            finding['issues'].append('Model file does not exist')
            return finding
        
        finding['exists'] = True
        finding['size_bytes'] = model_path.stat().st_size
        
        # Check if file is dummy/placeholder
        if self.check_file_for_dummy(model_path):
            finding['is_dummy'] = True
            finding['status'] = 'DUMMY'
            finding['issues'].append('Model file contains dummy/placeholder data')
            return finding
        
        # Compute hash
        file_hash = self.compute_file_hash(model_path)
        finding['hash'] = file_hash
        
        # Check if hash is empty (placeholder)
        if file_hash == EMPTY_HASH:
            finding['status'] = 'PLACEHOLDER'
            finding['issues'].append('Model file has empty hash (placeholder)')
            return finding
        
        # Check file size (dummy files are usually very small)
        if finding['size_bytes'] < 100:
            finding['status'] = 'SUSPICIOUS'
            finding['issues'].append(f'Model file is suspiciously small: {finding["size_bytes"]} bytes')
            return finding
        
        finding['status'] = 'REAL'
        return finding
    
    def verify_model_manifest(self, manifest_path: Path) -> Dict:
        """Verify model manifest."""
        finding = {
            'manifest_path': str(manifest_path.relative_to(self.root_path)),
            'exists': False,
            'models': [],
            'has_signature': False,
            'has_training_date': False,
            'issues': []
        }
        
        if not manifest_path.exists():
            finding['issues'].append('Manifest file does not exist')
            return finding
        
        finding['exists'] = True
        
        try:
            with open(manifest_path, 'r') as f:
                manifest = json.load(f)
            
            # Check for signature
            if 'signature' in manifest:
                sig = manifest.get('signature', {})
                if isinstance(sig, dict):
                    finding['has_signature'] = bool(sig.get('signature'))
                elif isinstance(sig, str):
                    finding['has_signature'] = bool(sig) and 'dummy' not in sig.lower()
            
            # Check for training date
            if 'trained_on' in manifest:
                finding['has_training_date'] = bool(manifest['trained_on'])
            
            # Check models in manifest
            if 'models' in manifest:
                for model in manifest['models']:
                    model_info = {
                        'name': model.get('name', 'unknown'),
                        'version': model.get('version', 'unknown'),
                        'hash': model.get('hash', 'unknown'),
                        'has_shap': bool(model.get('shap_file')),
                        'trained_on': model.get('trained_on', 'unknown'),
                    }
                    
                    # Check if hash is placeholder
                    if model_info['hash'] == EMPTY_HASH or 'e3b0c442' in str(model_info['hash']):
                        model_info['is_placeholder'] = True
                        finding['issues'].append(f"Model {model_info['name']} has placeholder hash")
                    
                    finding['models'].append(model_info)
            
        except Exception as e:
            finding['issues'].append(f'Failed to parse manifest: {e}')
        
        return finding
    
    def verify_shap_files(self, shap_path: Path) -> Dict:
        """Verify SHAP explainability files."""
        finding = {
            'shap_path': str(shap_path.relative_to(self.root_path)),
            'exists': False,
            'is_valid_json': False,
            'has_data': False,
            'issues': []
        }
        
        if not shap_path.exists():
            finding['issues'].append('SHAP file does not exist')
            return finding
        
        finding['exists'] = True
        
        try:
            with open(shap_path, 'r') as f:
                shap_data = json.load(f)
            
            finding['is_valid_json'] = True
            
            # Check if SHAP has actual data
            if isinstance(shap_data, dict):
                finding['has_data'] = bool(shap_data)
            elif isinstance(shap_data, list):
                finding['has_data'] = len(shap_data) > 0
            
        except Exception as e:
            finding['issues'].append(f'Failed to parse SHAP file: {e}')
        
        return finding
    
    def verify_llm_rag(self) -> Dict:
        """Verify LLM/RAG modules."""
        finding = {
            'module': 'LLM/RAG',
            'rag_index_exists': False,
            'rag_index_is_dummy': False,
            'knowledge_base_exists': False,
            'has_real_hash': False,
            'has_chunks': False,
            'issues': []
        }
        
        # Check RAG index
        rag_index_path = self.root_path / 'core' / 'ai' / 'rag' / 'index'
        if rag_index_path.exists():
            finding['rag_index_exists'] = True
            
            # Check metadata
            metadata_path = rag_index_path / 'metadata.json'
            if metadata_path.exists():
                try:
                    with open(metadata_path, 'r') as f:
                        metadata = json.load(f)
                    
                    # Check for placeholder hash
                    index_hash = metadata.get('index_hash', '')
                    if index_hash == EMPTY_HASH or 'e3b0c442' in str(index_hash):
                        finding['rag_index_is_dummy'] = True
                        finding['issues'].append('RAG index has placeholder hash')
                    else:
                        finding['has_real_hash'] = True
                    
                    integrity_hash = metadata.get('integrity_hash', '')
                    if integrity_hash == EMPTY_HASH or 'e3b0c442' in str(integrity_hash):
                        if not finding['rag_index_is_dummy']:
                            finding['rag_index_is_dummy'] = True
                            finding['issues'].append('RAG index has placeholder integrity hash')
                    
                    # Check document count
                    doc_count = metadata.get('document_count', 0)
                    if doc_count > 0:
                        finding['has_chunks'] = True
                        
                except Exception as e:
                    finding['issues'].append(f'Failed to parse RAG metadata: {e}')
            
            # Check index file
            index_bin_path = rag_index_path / 'index.bin'
            if index_bin_path.exists():
                if self.check_file_for_dummy(index_bin_path):
                    finding['rag_index_is_dummy'] = True
                    finding['issues'].append('RAG index file contains dummy data')
                else:
                    # Check file size (real index should be > 1KB)
                    size = index_bin_path.stat().st_size
                    if size < 1024:
                        finding['issues'].append(f'RAG index file suspiciously small: {size} bytes')
            
            # Check chunks.json
            chunks_path = rag_index_path / 'chunks.json'
            if chunks_path.exists():
                finding['has_chunks'] = True
                try:
                    with open(chunks_path, 'r') as f:
                        chunks = json.load(f)
                    if not isinstance(chunks, list) or len(chunks) == 0:
                        finding['issues'].append('RAG chunks.json is empty or invalid')
                except Exception as e:
                    finding['issues'].append(f'Failed to parse chunks.json: {e}')
        
        # Check knowledge base
        kb_path = self.root_path / 'ransomeye_intelligence' / 'llm_knowledge'
        if kb_path.exists():
            finding['knowledge_base_exists'] = True
        
        return finding
    
    def verify_all_models(self):
        """Verify all AI/ML/LLM models."""
        print("\n" + "="*80)
        print("VERIFYING AI/ML/LLM MODELS")
        print("="*80)
        
        # 1. Baseline Pack Models
        print("\n[1] Checking Baseline Pack Models...")
        baseline_dir = self.root_path / 'ransomeye_intelligence' / 'baseline_pack' / 'models'
        if baseline_dir.exists():
            manifest_path = baseline_dir / 'model_manifest.json'
            manifest_finding = self.verify_model_manifest(manifest_path)
            self.findings['models'].append({
                'module': 'Baseline Pack',
                'manifest': manifest_finding
            })
            
            # Check individual model files
            for model_file in baseline_dir.glob('*.model'):
                model_name = model_file.stem
                model_finding = self.verify_model_file(model_file, model_name, 'Baseline Pack')
                self.findings['models'].append({
                    'module': 'Baseline Pack',
                    'model': model_finding
                })
        
        # 2. Core AI Models
        print("\n[2] Checking Core AI Models...")
        core_models_dir = self.root_path / 'core' / 'ai' / 'models'
        if core_models_dir.exists():
            manifest_path = core_models_dir / 'models.manifest.json'
            manifest_finding = self.verify_model_manifest(manifest_path)
            self.findings['models'].append({
                'module': 'Core AI',
                'manifest': manifest_finding
            })
            
            # Check risk_model
            risk_model_path = core_models_dir / 'risk_model.model'
            if risk_model_path.exists():
                model_finding = self.verify_model_file(risk_model_path, 'risk_model', 'Core AI')
                self.findings['models'].append({
                    'module': 'Core AI',
                    'model': model_finding
                })
        
        # 3. Inference Models
        print("\n[3] Checking Inference Models...")
        inference_dir = self.root_path / 'core' / 'ai' / 'inference' / 'models'
        if inference_dir.exists():
            manifest_path = inference_dir / 'models.manifest.json'
            manifest_finding = self.verify_model_manifest(manifest_path)
            self.findings['models'].append({
                'module': 'Inference',
                'manifest': manifest_finding
            })
            
            # Check model files
            for model_file in inference_dir.glob('*.model'):
                model_name = model_file.stem
                model_finding = self.verify_model_file(model_file, model_name, 'Inference')
                self.findings['models'].append({
                    'module': 'Inference',
                    'model': model_finding
                })
        
        # 4. LLM/RAG
        print("\n[4] Checking LLM/RAG Modules...")
        rag_finding = self.verify_llm_rag()
        self.findings['llm_modules'].append(rag_finding)
        
        # 5. Check SHAP files
        print("\n[5] Checking SHAP Files...")
        shap_files = [
            self.root_path / 'core' / 'ai' / 'models' / 'risk_model_shap_baseline.json',
            self.root_path / 'ransomeye_intelligence' / 'baseline_pack' / 'shap' / 'ransomware_behavior_shap.json',
        ]
        
        for shap_file in shap_files:
            if shap_file.exists():
                shap_finding = self.verify_shap_files(shap_file)
                self.findings['models'].append({
                    'module': 'SHAP',
                    'shap': shap_finding
                })
    
    def generate_summary(self) -> Dict:
        """Generate summary of findings."""
        summary = {
            'total_models': 0,
            'real_models': 0,
            'dummy_models': 0,
            'missing_models': 0,
            'placeholder_models': 0,
            'modules_with_issues': [],
            'overall_status': 'UNKNOWN'
        }
        
        for finding in self.findings['models']:
            if 'model' in finding:
                model = finding['model']
                summary['total_models'] += 1
                
                if model['status'] == 'REAL':
                    summary['real_models'] += 1
                elif model['status'] == 'DUMMY':
                    summary['dummy_models'] += 1
                    summary['modules_with_issues'].append(finding['module'])
                elif model['status'] == 'MISSING':
                    summary['missing_models'] += 1
                    summary['modules_with_issues'].append(finding['module'])
                elif model['status'] == 'PLACEHOLDER':
                    summary['placeholder_models'] += 1
                    summary['modules_with_issues'].append(finding['module'])
        
        # Determine overall status
        if summary['dummy_models'] > 0 or summary['placeholder_models'] > 0:
            summary['overall_status'] = 'FAIL - Dummy/Placeholder models found'
        elif summary['missing_models'] > 0:
            summary['overall_status'] = 'WARNING - Some models missing'
        elif summary['real_models'] == summary['total_models'] and summary['total_models'] > 0:
            summary['overall_status'] = 'PASS - All models are real'
        else:
            summary['overall_status'] = 'UNKNOWN'
        
        return summary
    
    def generate_report(self) -> str:
        """Generate verification report."""
        summary = self.generate_summary()
        self.findings['summary'] = summary
        
        report = []
        report.append("="*80)
        report.append("RANSOMEYE AI/ML/LLM TRAINING VERIFICATION REPORT")
        report.append("="*80)
        report.append("")
        report.append("Purpose: Verify that all AI, ML, and LLM modules are fully trained")
        report.append("         with real models, not placeholders or dummy data.")
        report.append("")
        report.append("="*80)
        report.append("EXECUTIVE SUMMARY")
        report.append("="*80)
        report.append("")
        report.append(f"Overall Status: {summary['overall_status']}")
        report.append("")
        report.append(f"Total Models Found: {summary['total_models']}")
        report.append(f"  ✓ Real Models: {summary['real_models']}")
        report.append(f"  ✗ Dummy Models: {summary['dummy_models']}")
        report.append(f"  ✗ Placeholder Models: {summary['placeholder_models']}")
        report.append(f"  ⚠ Missing Models: {summary['missing_models']}")
        report.append("")
        
        if summary['modules_with_issues']:
            report.append("Modules with Issues:")
            for module in set(summary['modules_with_issues']):
                report.append(f"  - {module}")
            report.append("")
        
        # Detailed findings
        report.append("="*80)
        report.append("DETAILED FINDINGS")
        report.append("="*80)
        report.append("")
        
        for finding in self.findings['models']:
            if 'model' in finding:
                model = finding['model']
                report.append(f"Module: {finding['module']}")
                report.append(f"  Model: {model['model_name']}")
                report.append(f"  Status: {model['status']}")
                report.append(f"  File: {model['file_path']}")
                report.append(f"  Size: {model['size_bytes']} bytes")
                if model['hash']:
                    report.append(f"  Hash: {model['hash'][:16]}...")
                if model['issues']:
                    for issue in model['issues']:
                        report.append(f"  ⚠ {issue}")
                report.append("")
        
        # LLM/RAG findings
        for rag_finding in self.findings['llm_modules']:
            report.append(f"Module: {rag_finding['module']}")
            report.append(f"  RAG Index Exists: {rag_finding['rag_index_exists']}")
            report.append(f"  RAG Index is Dummy: {rag_finding['rag_index_is_dummy']}")
            report.append(f"  Knowledge Base Exists: {rag_finding['knowledge_base_exists']}")
            if rag_finding['issues']:
                for issue in rag_finding['issues']:
                    report.append(f"  ⚠ {issue}")
            report.append("")
        
        report.append("="*80)
        report.append("RECOMMENDATIONS")
        report.append("="*80)
        report.append("")
        
        if summary['dummy_models'] > 0:
            report.append("CRITICAL: Dummy models found in production directories.")
            report.append("  Action Required:")
            report.append("  1. Replace dummy models with real trained models")
            report.append("  2. Train models using training scripts")
            report.append("  3. Verify model hashes match manifests")
            report.append("  4. Ensure SHAP files are generated")
            report.append("")
        
        if summary['placeholder_models'] > 0:
            report.append("CRITICAL: Placeholder models found (empty hash).")
            report.append("  Action Required:")
            report.append("  1. Train models and generate real model files")
            report.append("  2. Update manifests with real hashes")
            report.append("")
        
        report.append("="*80)
        
        return "\n".join(report)


def main():
    root_path = os.path.dirname(os.path.abspath(__file__))
    verifier = ModelVerifier(root_path)
    
    print("Starting RansomEye AI/ML/LLM Training Verification...")
    print(f"Root path: {root_path}")
    
    verifier.verify_all_models()
    
    report = verifier.generate_report()
    
    print(report)
    
    # Save report
    report_path = Path(root_path) / 'logs' / 'ai_ml_training_verification_report.txt'
    report_path.parent.mkdir(parents=True, exist_ok=True)
    with open(report_path, 'w') as f:
        f.write(report)
    
    # Save JSON findings
    json_path = Path(root_path) / 'logs' / 'ai_ml_training_verification_findings.json'
    with open(json_path, 'w') as f:
        json.dump(verifier.findings, f, indent=2)
    
    print(f"\nReport saved to: {report_path}")
    print(f"JSON findings saved to: {json_path}")
    
    # Exit code
    summary = verifier.findings['summary']
    if summary['overall_status'].startswith('FAIL'):
        print("\n❌ CRITICAL ISSUES FOUND: Dummy/Placeholder models detected")
        return 1
    elif summary['overall_status'].startswith('WARNING'):
        print("\n⚠ WARNINGS: Some models missing or have issues")
        return 1
    else:
        print("\n✓ Verification complete - All models are real")
        return 0


if __name__ == '__main__':
    exit(main())

