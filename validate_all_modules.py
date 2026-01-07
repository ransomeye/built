# Path and File Name : /home/ransomeye/rebuild/validate_all_modules.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Comprehensive validation script to check build status and trained models for all AI/ML/LLM modules

"""
RansomEye Comprehensive Module Validation Script
================================================
Validates:
- All modules are built and installed
- All AI/ML/LLM models are trained
- Model metadata (version, hash, training date)
- SHAP explainability files
- Systemd services status
- Database connectivity
- Threat intelligence feeds
"""

import os
import sys
import json
import pickle
import hashlib
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Optional, Tuple
import subprocess

PROJECT_ROOT = Path("/home/ransomeye/rebuild")

# Module definitions with expected models
MODULE_DEFINITIONS = {
    'baseline_pack': {
        'path': PROJECT_ROOT / 'ransomeye_intelligence' / 'baseline_pack' / 'models',
        'models': [
            'ransomware_behavior.model',
            'anomaly_detector.model',
            'confidence_calibrator.model'
        ],
        'shap_dir': PROJECT_ROOT / 'ransomeye_intelligence' / 'baseline_pack' / 'shap',
        'metadata': 'model_manifest.json'
    },
    'risk_model': {
        'path': PROJECT_ROOT / 'ransomeye_ai_core' / 'models',
        'models': ['risk_predictor.model'],
        'shap_dir': PROJECT_ROOT / 'ransomeye_ai_core' / 'shap',
        'metadata': 'risk_model_metadata.json'
    },
    'threat_correlation': {
        'path': PROJECT_ROOT / 'ransomeye_threat_correlation' / 'models',
        'models': ['confidence_predictor.model'],
        'shap_dir': PROJECT_ROOT / 'ransomeye_threat_correlation' / 'shap',
        'metadata': 'correlation_metadata.json'
    },
    'forensic_malware_dna': {
        'path': PROJECT_ROOT / 'ransomeye_forensic' / 'models',
        'models': ['malware_dna.model'],
        'shap_dir': PROJECT_ROOT / 'ransomeye_forensic' / 'shap',
        'metadata': 'forensic_metadata.json'
    },
    'threat_intel_trust': {
        'path': PROJECT_ROOT / 'ransomeye_threat_intel_engine' / 'models',
        'models': ['trust_scorer.model', 'ioc_clusterer.model'],
        'shap_dir': PROJECT_ROOT / 'ransomeye_threat_intel_engine' / 'shap',
        'metadata': 'trust_scoring_metadata.json'
    },
    'dpi_probe_classifier': {
        'path': PROJECT_ROOT / 'ransomeye_dpi_probe' / 'models',
        'models': ['asset_classifier.model'],
        'shap_dir': PROJECT_ROOT / 'ransomeye_dpi_probe' / 'shap',
        'metadata': 'dpi_metadata.json'
    },
    'threat_classifier_continuous': {
        'path': PROJECT_ROOT / 'ransomeye_intelligence' / 'threat_intel' / 'models',
        'models': ['threat_classifier_continuous.model'],
        'shap_dir': PROJECT_ROOT / 'ransomeye_intelligence' / 'threat_intel' / 'shap',
        'metadata': 'threat_classifier_continuous_metrics.json'
    },
    'rag_index': {
        'path': PROJECT_ROOT / 'ransomeye_ai_assistant' / 'rag_index',
        'models': ['index.faiss', 'index.pkl'],  # RAG index files
        'shap_dir': None,
        'metadata': 'rag_metadata.json'
    }
}

SYSTEMD_SERVICES = [
    'ransomeye-master-core.service',
    'ransomeye-ai-core.service',
    'ransomeye-alert-engine.service',
    'ransomeye-threat-intel.service',
    'ransomeye-continuous-training.timer',
    'ransomeye-auto-evolution.timer'
]


class ModuleValidator:
    """Comprehensive module validator."""
    
    def __init__(self):
        self.results = {
            'modules': {},
            'systemd': {},
            'database': {},
            'feeds': {},
            'summary': {}
        }
    
    def check_model_exists(self, model_path: Path) -> Tuple[bool, Optional[Dict]]:
        """Check if model exists and get metadata."""
        if not model_path.exists():
            return False, None
        
        try:
            # Try to load model
            with open(model_path, 'rb') as f:
                model_data = f.read()
            
            # Compute hash
            model_hash = hashlib.sha256(model_data).hexdigest()
            
            # Get file stats
            stat = model_path.stat()
            
            return True, {
                'path': str(model_path),
                'size_bytes': stat.st_size,
                'size_mb': round(stat.st_size / (1024 * 1024), 2),
                'hash': model_hash,
                'modified': datetime.fromtimestamp(stat.st_mtime).isoformat()
            }
        except Exception as e:
            return False, {'error': str(e)}
    
    def check_shap_file(self, model_name: str, shap_dir: Path) -> Tuple[bool, Optional[str]]:
        """Check if SHAP file exists for model."""
        if not shap_dir or not shap_dir.exists():
            return False, None
        
        # Look for SHAP files matching model name
        shap_files = list(shap_dir.glob(f"*{model_name}*shap*.json"))
        if shap_files:
            return True, str(shap_files[0])
        
        return False, None
    
    def check_metadata(self, metadata_path: Path) -> Tuple[bool, Optional[Dict]]:
        """Check if metadata file exists and load it."""
        if not metadata_path.exists():
            return False, None
        
        try:
            with open(metadata_path, 'r') as f:
                metadata = json.load(f)
            return True, metadata
        except Exception as e:
            return False, {'error': str(e)}
    
    def validate_module(self, module_name: str, module_def: Dict) -> Dict:
        """Validate a single module."""
        result = {
            'module_name': module_name,
            'path_exists': module_def['path'].exists(),
            'models': {},
            'shap_files': {},
            'metadata': {},
            'status': 'unknown'
        }
        
        # Check each model
        all_models_exist = True
        for model_name in module_def['models']:
            model_path = module_def['path'] / model_name
            exists, info = self.check_model_exists(model_path)
            result['models'][model_name] = {
                'exists': exists,
                'info': info
            }
            if not exists:
                all_models_exist = False
            
            # Check SHAP file
            if module_def.get('shap_dir'):
                shap_exists, shap_path = self.check_shap_file(model_name, module_def['shap_dir'])
                result['shap_files'][model_name] = {
                    'exists': shap_exists,
                    'path': shap_path
                }
        
        # Check metadata
        if module_def.get('metadata'):
            metadata_path = module_def['path'] / module_def['metadata']
            metadata_exists, metadata_info = self.check_metadata(metadata_path)
            result['metadata'] = {
                'exists': metadata_exists,
                'info': metadata_info
            }
        
        # Determine overall status
        if all_models_exist:
            result['status'] = 'complete'
        elif any(m['exists'] for m in result['models'].values()):
            result['status'] = 'partial'
        else:
            result['status'] = 'missing'
        
        return result
    
    def check_systemd_service(self, service_name: str) -> Dict:
        """Check systemd service status."""
        try:
            result = subprocess.run(
                ['systemctl', 'is-active', service_name],
                capture_output=True,
                text=True,
                timeout=5
            )
            is_active = result.stdout.strip() == 'active'
            
            result_enabled = subprocess.run(
                ['systemctl', 'is-enabled', service_name],
                capture_output=True,
                text=True,
                timeout=5
            )
            is_enabled = result_enabled.stdout.strip() == 'enabled'
            
            return {
                'exists': True,
                'active': is_active,
                'enabled': is_enabled,
                'status': 'active' if is_active else 'inactive'
            }
        except subprocess.TimeoutExpired:
            return {'exists': False, 'error': 'timeout'}
        except FileNotFoundError:
            return {'exists': False, 'error': 'systemctl not found'}
        except Exception as e:
            return {'exists': False, 'error': str(e)}
    
    def check_database(self) -> Dict:
        """Check database connectivity."""
        try:
            import psycopg2
            conn = psycopg2.connect(
                host=os.getenv('DB_HOST', 'localhost'),
                port=int(os.getenv('DB_PORT', 5432)),
                database=os.getenv('DB_NAME', 'ransomeye'),
                user=os.getenv('DB_USER', 'gagan'),
                password=os.getenv('DB_PASS', 'gagan')
            )
            cursor = conn.cursor()
            cursor.execute("SELECT version();")
            version = cursor.fetchone()[0]
            cursor.close()
            conn.close()
            
            return {
                'connected': True,
                'version': version.split(',')[0] if version else 'unknown'
            }
        except Exception as e:
            return {
                'connected': False,
                'error': str(e)
            }
    
    def check_threat_intel_feeds(self) -> Dict:
        """Check threat intelligence feed cache."""
        cache_dir = PROJECT_ROOT / 'ransomeye_intelligence' / 'threat_intel' / 'cache'
        result = {
            'cache_dir_exists': cache_dir.exists(),
            'feeds': {}
        }
        
        if cache_dir.exists():
            for feed_dir in cache_dir.iterdir():
                if feed_dir.is_dir():
                    feed_name = feed_dir.name
                    cache_files = list(feed_dir.glob('*.json'))
                    result['feeds'][feed_name] = {
                        'cached_files': len(cache_files),
                        'latest': max([f.stat().st_mtime for f in cache_files]) if cache_files else None
                    }
        
        return result
    
    def run_validation(self) -> Dict:
        """Run complete validation."""
        print("=" * 80)
        print("RansomEye Comprehensive Module Validation")
        print("=" * 80)
        print()
        
        # Validate all modules
        print("Validating AI/ML/LLM Modules...")
        print("-" * 80)
        for module_name, module_def in MODULE_DEFINITIONS.items():
            print(f"Checking {module_name}...")
            result = self.validate_module(module_name, module_def)
            self.results['modules'][module_name] = result
            
            status_icon = {
                'complete': '✓',
                'partial': '⚠',
                'missing': '✗'
            }.get(result['status'], '?')
            
            print(f"  {status_icon} {module_name}: {result['status']}")
            for model_name, model_info in result['models'].items():
                if model_info['exists']:
                    print(f"    ✓ {model_name} ({model_info['info']['size_mb']} MB)")
                else:
                    print(f"    ✗ {model_name} (missing)")
        
        print()
        
        # Check systemd services
        print("Checking Systemd Services...")
        print("-" * 80)
        for service_name in SYSTEMD_SERVICES:
            status = self.check_systemd_service(service_name)
            self.results['systemd'][service_name] = status
            
            if status.get('exists'):
                icon = '✓' if status.get('active') else '○'
                enabled_icon = '✓' if status.get('enabled') else '○'
                print(f"  {icon} {service_name}: {status.get('status', 'unknown')} (enabled: {enabled_icon})")
            else:
                print(f"  ✗ {service_name}: not found")
        
        print()
        
        # Check database
        print("Checking Database...")
        print("-" * 80)
        db_status = self.check_database()
        self.results['database'] = db_status
        if db_status.get('connected'):
            print(f"  ✓ Database connected: {db_status.get('version', 'unknown')}")
        else:
            print(f"  ✗ Database not connected: {db_status.get('error', 'unknown error')}")
        
        print()
        
        # Check threat intel feeds
        print("Checking Threat Intelligence Feeds...")
        print("-" * 80)
        feeds_status = self.check_threat_intel_feeds()
        self.results['feeds'] = feeds_status
        if feeds_status.get('cache_dir_exists'):
            for feed_name, feed_info in feeds_status.get('feeds', {}).items():
                if feed_info.get('cached_files', 0) > 0:
                    print(f"  ✓ {feed_name}: {feed_info['cached_files']} cached files")
                else:
                    print(f"  ○ {feed_name}: no cached files")
        else:
            print("  ✗ Cache directory does not exist")
        
        print()
        
        # Generate summary
        self.generate_summary()
        
        return self.results
    
    def generate_summary(self):
        """Generate validation summary."""
        total_modules = len(MODULE_DEFINITIONS)
        complete_modules = sum(1 for m in self.results['modules'].values() if m['status'] == 'complete')
        partial_modules = sum(1 for m in self.results['modules'].values() if m['status'] == 'partial')
        missing_modules = sum(1 for m in self.results['modules'].values() if m['status'] == 'missing')
        
        total_models = sum(len(m['models']) for m in MODULE_DEFINITIONS.values())
        existing_models = sum(
            sum(1 for model_info in module_result['models'].values() if model_info['exists'])
            for module_result in self.results['modules'].values()
        )
        
        active_services = sum(1 for s in self.results['systemd'].values() if s.get('active'))
        enabled_services = sum(1 for s in self.results['systemd'].values() if s.get('enabled'))
        
        self.results['summary'] = {
            'modules': {
                'total': total_modules,
                'complete': complete_modules,
                'partial': partial_modules,
                'missing': missing_modules,
                'completion_percent': round((complete_modules / total_modules * 100) if total_modules > 0 else 0, 1)
            },
            'models': {
                'total': total_models,
                'existing': existing_models,
                'missing': total_models - existing_models,
                'completion_percent': round((existing_models / total_models * 100) if total_models > 0 else 0, 1)
            },
            'systemd': {
                'total': len(SYSTEMD_SERVICES),
                'active': active_services,
                'enabled': enabled_services
            },
            'database': {
                'connected': self.results['database'].get('connected', False)
            },
            'feeds': {
                'cache_exists': self.results['feeds'].get('cache_dir_exists', False),
                'feed_count': len(self.results['feeds'].get('feeds', {}))
            }
        }
        
        print("=" * 80)
        print("VALIDATION SUMMARY")
        print("=" * 80)
        print(f"Modules: {complete_modules}/{total_modules} complete ({self.results['summary']['modules']['completion_percent']}%)")
        print(f"Models: {existing_models}/{total_models} exist ({self.results['summary']['models']['completion_percent']}%)")
        print(f"Systemd Services: {active_services}/{len(SYSTEMD_SERVICES)} active, {enabled_services} enabled")
        print(f"Database: {'✓ Connected' if self.results['database'].get('connected') else '✗ Not connected'}")
        print(f"Threat Intel Feeds: {self.results['summary']['feeds']['feed_count']} feeds cached")
        print("=" * 80)
    
    def save_report(self, output_path: Path = None):
        """Save validation report to JSON file."""
        if output_path is None:
            output_path = PROJECT_ROOT / 'logs' / 'validation_report.json'
        
        try:
            # Create parent directory if it doesn't exist
            output_path.parent.mkdir(parents=True, exist_ok=True)
            
            with open(output_path, 'w') as f:
                json.dump(self.results, f, indent=2, default=str)
            
            print(f"\nValidation report saved to: {output_path}")
        except PermissionError as e:
            print(f"\n⚠ Warning: Permission denied saving to {output_path}")
            print(f"   Error: {e}")
            print(f"   Falling back to default location...")
            # Fallback to default location
            default_path = PROJECT_ROOT / 'logs' / 'validation_report.json'
            try:
                default_path.parent.mkdir(parents=True, exist_ok=True)
                with open(default_path, 'w') as f:
                    json.dump(self.results, f, indent=2, default=str)
                print(f"   Report saved to: {default_path}")
            except Exception as e2:
                print(f"   ✗ Failed to save report: {e2}")
        except FileNotFoundError as e:
            print(f"\n⚠ Warning: Invalid path: {output_path}")
            print(f"   Error: {e}")
            print(f"   Falling back to default location...")
            # Fallback to default location
            default_path = PROJECT_ROOT / 'logs' / 'validation_report.json'
            try:
                default_path.parent.mkdir(parents=True, exist_ok=True)
                with open(default_path, 'w') as f:
                    json.dump(self.results, f, indent=2, default=str)
                print(f"   Report saved to: {default_path}")
            except Exception as e2:
                print(f"   ✗ Failed to save report: {e2}")
        except Exception as e:
            print(f"\n✗ Error saving report to {output_path}: {e}")
            print(f"   Report data is available in memory but not saved to disk")


def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description='RansomEye Comprehensive Module Validation')
    parser.add_argument('--json', action='store_true',
                       help='Output results as JSON only')
    parser.add_argument('--save', type=str, default=None,
                       help='Save report to file')
    
    args = parser.parse_args()
    
    validator = ModuleValidator()
    results = validator.run_validation()
    
    if args.save:
        validator.save_report(Path(args.save))
    elif not args.json:
        validator.save_report()
    
    if args.json:
        print(json.dumps(results, indent=2, default=str))
    
    # Exit with error code if validation failed
    if results['summary']['modules']['completion_percent'] < 100:
        sys.exit(1)
    else:
        sys.exit(0)


if __name__ == '__main__':
    main()

