# Path: /home/ransomeye/rebuild/ransomeye_governance/tooling/ai_baseline_enforcer.py
# Author: RansomEye Core Team
# Purpose: Enforces AI Day-1 readiness - fails if baseline AI artifacts are missing or invalid

"""
AI Baseline Enforcer: Enforces AI Day-1 readiness.

Rules:
- AI MUST NOT start without baseline
- Baseline pack MUST be present
- Baseline pack MUST be signed
- Baseline pack MUST be valid
"""

import os
import sys
import json
from pathlib import Path
from typing import List, Dict, Optional

# Expected baseline pack location
DEFAULT_BASELINE_PACK_PATH = "/home/ransomeye/rebuild/ransomeye_ai_core/baseline_pack"

# Required baseline components
REQUIRED_BASELINE_COMPONENTS = [
    'models/ransomware_behavior.pkl',
    'models/anomaly_detection.pkl',
    'shap/baseline_distributions.json',
    'shap/explainers.pkl',
    'llm/rag_index',
    'intelligence/ioc_database.json',
    'metadata.json',
    'signature.sig',
]


class AIBaselineEnforcer:
    """Enforces AI Day-1 readiness."""
    
    def __init__(self, baseline_pack_path: Optional[Path] = None):
        if baseline_pack_path is None:
            baseline_pack_path = Path(os.environ.get('AI_BASELINE_PACK_PATH', DEFAULT_BASELINE_PACK_PATH))
        
        self.baseline_pack_path = Path(baseline_pack_path)
        self.violations: List[Dict] = []
    
    def check_baseline_presence(self) -> bool:
        """Check if baseline pack exists."""
        if not self.baseline_pack_path.exists():
            self.violations.append({
                'type': 'missing_baseline',
                'path': str(self.baseline_pack_path),
                'message': f'Baseline pack not found at: {self.baseline_pack_path}'
            })
            return False
        
        return True
    
    def check_baseline_components(self) -> bool:
        """Check if all required baseline components exist."""
        if not self.baseline_pack_path.exists():
            return False
        
        missing_components = []
        
        for component in REQUIRED_BASELINE_COMPONENTS:
            component_path = self.baseline_pack_path / component
            if not component_path.exists():
                missing_components.append(component)
        
        if missing_components:
            for component in missing_components:
                self.violations.append({
                    'type': 'missing_component',
                    'component': component,
                    'message': f'Required baseline component missing: {component}'
                })
            return False
        
        return True
    
    def check_baseline_signature(self) -> bool:
        """Check if baseline pack is signed."""
        signature_path = self.baseline_pack_path / 'signature.sig'
        
        if not signature_path.exists():
            self.violations.append({
                'type': 'missing_signature',
                'message': 'Baseline pack signature not found'
            })
            return False
        
        # In real implementation, would verify signature
        # For now, just check presence
        return True
    
    def check_baseline_metadata(self) -> bool:
        """Check if baseline metadata is valid."""
        metadata_path = self.baseline_pack_path / 'metadata.json'
        
        if not metadata_path.exists():
            self.violations.append({
                'type': 'missing_metadata',
                'message': 'Baseline pack metadata not found'
            })
            return False
        
        try:
            with open(metadata_path, 'r') as f:
                metadata = json.load(f)
            
            # Validate required metadata fields
            required_fields = ['version', 'hash', 'trained_on', 'signature']
            missing_fields = [field for field in required_fields if field not in metadata]
            
            if missing_fields:
                self.violations.append({
                    'type': 'invalid_metadata',
                    'missing_fields': missing_fields,
                    'message': f'Baseline metadata missing fields: {missing_fields}'
                })
                return False
        
        except Exception as e:
            self.violations.append({
                'type': 'metadata_error',
                'error': str(e),
                'message': f'Error reading baseline metadata: {e}'
            })
            return False
        
        return True
    
    def enforce(self) -> bool:
        """Run full AI baseline enforcement."""
        print("=" * 80)
        print("RansomEye AI Baseline Enforcer")
        print("=" * 80)
        print()
        
        presence_ok = self.check_baseline_presence()
        if not presence_ok:
            return False
        
        components_ok = self.check_baseline_components()
        signature_ok = self.check_baseline_signature()
        metadata_ok = self.check_baseline_metadata()
        
        if not (components_ok and signature_ok and metadata_ok):
            print(f"\n{'='*80}")
            print(f"AI BASELINE ENFORCEMENT FAILED: {len(self.violations)} violation(s)")
            print(f"{'='*80}\n")
            
            for violation in self.violations:
                print(f"  {violation['message']}")
            
            return False
        
        print("\n✓ AI baseline pack valid and ready.")
        return True
    
    def fail_closed(self, message: str):
        """Fail-closed: exit with error."""
        print(f"\n{'='*80}")
        print("AI BASELINE ENFORCEMENT FAILED")
        print(f"{'='*80}")
        print(message)
        print("\nAI subsystem blocked - baseline pack missing or invalid.")
        print("AI MUST NOT start without valid baseline.")
        sys.exit(1)


def main():
    """CLI entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description='RansomEye AI Baseline Enforcer')
    parser.add_argument('--baseline-path', default=None,
                       help='Path to baseline pack')
    
    args = parser.parse_args()
    
    baseline_path = Path(args.baseline_path) if args.baseline_path else None
    
    enforcer = AIBaselineEnforcer(baseline_path)
    is_valid = enforcer.enforce()
    
    if not is_valid:
        enforcer.fail_closed(f"Found {len(enforcer.violations)} AI baseline violation(s).")
    
    print("\n✓ AI baseline enforcement passed.")
    sys.exit(0)


if __name__ == '__main__':
    main()

