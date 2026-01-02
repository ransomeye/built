# Path and File Name : /home/ransomeye/rebuild/ransomeye_retention/ai_retention_guard.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: AI artifact protection - blocks deletion of AI artifacts unless explicitly authorized

"""
AI Retention Guard: Protects AI training artifacts from deletion.
Enforces minimum 2-year retention and requires explicit operator approval for deletion.
"""

import os
import json
from pathlib import Path
from typing import List, Dict, Optional, Set
from datetime import datetime, timedelta

from .retention_parser import RetentionParser
import sys
from pathlib import Path
# Add project root to path for trust imports
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))
try:
    from ransomeye_trust.sign_tool import SignTool
except ImportError:
    # Fallback for development
    SignTool = None


class AIRetentionGuard:
    """Protects AI artifacts from unauthorized deletion."""
    
    MIN_RETENTION_YEARS = 2
    AI_ARTIFACT_PATTERNS = [
        '*.pkl', '*.h5', '*.pb', '*.onnx', '*.pt', '*.pth', '*.ckpt', '*.gguf',
        '*_model.*', '*_weights.*', '*_checkpoint.*',
        '*_training_data.*', '*_dataset.*',
        '*_shap.*', '*_explainer.*',
        '*_metadata.json'
    ]
    
    AI_DIRECTORIES = [
        'models', 'checkpoints', 'training_data', 'datasets', 'shap_outputs'
    ]
    
    def __init__(self, retention_parser: RetentionParser, ai_dir: str = "/home/ransomeye/rebuild/ransomeye_ai_core", trust_dir: str = "/home/ransomeye/rebuild/ransomeye_trust"):
        self.retention_parser = retention_parser
        self.ai_dir = Path(ai_dir)
        self.trust_dir = Path(trust_dir)
        self.sign_tool = SignTool(str(self.trust_dir))
        self.approvals_dir = Path("/home/ransomeye/rebuild/logs/ai_deletion_approvals")
        self.approvals_dir.mkdir(parents=True, exist_ok=True)
        self.ledger_path = Path("/home/ransomeye/rebuild/logs/retention_ledger.jsonl")
        self.ledger_path.parent.mkdir(parents=True, exist_ok=True)
    
    def _is_ai_artifact(self, file_path: Path) -> bool:
        """Check if file is an AI artifact."""
        # Check filename patterns
        filename = file_path.name.lower()
        for pattern in self.AI_ARTIFACT_PATTERNS:
            if self._match_pattern(pattern, filename):
                return True
        
        # Check directory
        rel_path = str(file_path.relative_to(self.ai_dir)) if self.ai_dir in file_path.parents else str(file_path)
        for ai_dir in self.AI_DIRECTORIES:
            if ai_dir in rel_path:
                return True
        
        return False
    
    def _match_pattern(self, pattern: str, filename: str) -> bool:
        """Simple glob pattern matching."""
        import fnmatch
        return fnmatch.fnmatch(filename, pattern)
    
    def _get_ai_artifacts(self, directory: Optional[Path] = None) -> List[Dict]:
        """Get list of all AI artifacts."""
        if directory is None:
            directory = self.ai_dir
        
        if not directory.exists():
            return []
        
        artifacts = []
        
        for root, dirs, files in os.walk(directory):
            for file in files:
                file_path = Path(root) / file
                if self._is_ai_artifact(file_path):
                    try:
                        artifacts.append({
                            'path': file_path,
                            'size': file_path.stat().st_size,
                            'mtime': datetime.fromtimestamp(file_path.stat().st_mtime).isoformat(),
                            'age_days': (datetime.now() - datetime.fromtimestamp(file_path.stat().st_mtime)).days
                        })
                    except Exception:
                        continue
        
        return artifacts
    
    def _check_retention_period(self, artifact: Dict) -> bool:
        """Check if artifact is within minimum retention period."""
        age_days = artifact.get('age_days', 0)
        min_days = self.MIN_RETENTION_YEARS * 365
        return age_days < min_days
    
    def _has_approval(self, artifact_path: Path) -> bool:
        """Check if artifact has explicit deletion approval."""
        # Look for approval file
        approval_pattern = f"approval_{artifact_path.stem}_*.json"
        
        for approval_file in self.approvals_dir.glob(approval_pattern):
            try:
                with open(approval_file, 'r') as f:
                    approval = json.load(f)
                    if approval.get('approved') and approval.get('artifact_path') == str(artifact_path):
                        # Verify approval signature
                        # (In production, verify signature)
                        return True
            except Exception:
                continue
        
        return False
    
    def can_delete(self, artifact_path: Path, reason: str = "retention") -> Dict:
        """
        Check if AI artifact can be deleted.
        
        Args:
            artifact_path: Path to artifact
            reason: Reason for deletion
        
        Returns:
            Dictionary with 'allowed', 'reason', and 'requirements'
        """
        if not self._is_ai_artifact(artifact_path):
            return {
                'allowed': True,
                'reason': 'Not an AI artifact'
            }
        
        # Get artifact info
        try:
            artifact = {
                'path': artifact_path,
                'age_days': (datetime.now() - datetime.fromtimestamp(artifact_path.stat().st_mtime)).days
            }
        except Exception:
            return {
                'allowed': False,
                'reason': 'Cannot access artifact'
            }
        
        # Check retention period
        if self._check_retention_period(artifact):
            return {
                'allowed': False,
                'reason': f'Artifact is within minimum {self.MIN_RETENTION_YEARS}-year retention period',
                'age_days': artifact['age_days'],
                'min_days': self.MIN_RETENTION_YEARS * 365
            }
        
        # Check for explicit approval
        if not self._has_approval(artifact_path):
            return {
                'allowed': False,
                'reason': 'No explicit deletion approval found',
                'requirements': [
                    'Artifact must be older than 2 years',
                    'Explicit operator approval required',
                    'Signed approval certificate required'
                ]
            }
        
        return {
            'allowed': True,
            'reason': 'Approved for deletion'
        }
    
    def block_deletion(self, artifact_path: Path, reason: str) -> Dict:
        """
        Block deletion of AI artifact (called by retention enforcers).
        
        Returns:
            Dictionary with blocking result
        """
        check_result = self.can_delete(artifact_path, reason)
        
        if not check_result.get('allowed'):
            return {
                'blocked': True,
                'artifact_path': str(artifact_path),
                'reason': check_result.get('reason'),
                'message': f'AI artifact deletion blocked: {check_result.get("reason")}'
            }
        
        return {
            'blocked': False,
            'artifact_path': str(artifact_path)
        }
    
    def create_approval(self, artifact_path: Path, operator: str, reason: str) -> Path:
        """
        Create signed approval for AI artifact deletion.
        
        Args:
            artifact_path: Path to artifact
            operator: Operator name/ID
            reason: Reason for deletion
        
        Returns:
            Path to approval certificate
        """
        approval = {
            'timestamp': datetime.utcnow().isoformat(),
            'artifact_path': str(artifact_path),
            'operator': operator,
            'reason': reason,
            'approved': True,
            'min_retention_years': self.MIN_RETENTION_YEARS
        }
        
        # Save approval
        approval_filename = f"approval_{artifact_path.stem}_{datetime.utcnow().strftime('%Y%m%d_%H%M%S')}.json"
        approval_path = self.approvals_dir / approval_filename
        
        with open(approval_path, 'w') as f:
            json.dump(approval, f, indent=2)
        
        # Sign approval
        metadata = {
            'timestamp': datetime.utcnow().isoformat(),
            'version': '1.0.0',
            'type': 'ai_deletion_approval'
        }
        self.sign_tool.create_and_sign_manifest(approval_path, metadata, 'reports')
        
        return approval_path
    
    def get_protected_artifacts(self) -> List[Dict]:
        """Get list of all protected AI artifacts."""
        artifacts = self._get_ai_artifacts()
        protected = []
        
        for artifact in artifacts:
            if self._check_retention_period(artifact):
                protected.append(artifact)
        
        return protected


def main():
    """CLI entry point for AI retention guard."""
    import argparse
    
    parser = argparse.ArgumentParser(description='RansomEye AI Retention Guard')
    parser.add_argument('--check', type=Path, help='Check if artifact can be deleted')
    parser.add_argument('--list-protected', action='store_true', help='List all protected artifacts')
    parser.add_argument('--create-approval', type=Path, help='Create deletion approval for artifact')
    parser.add_argument('--operator', default='system', help='Operator name for approval')
    parser.add_argument('--reason', default='operator_request', help='Reason for deletion')
    
    args = parser.parse_args()
    
    from .retention_parser import RetentionParser
    retention_parser = RetentionParser()
    guard = AIRetentionGuard(retention_parser)
    
    if args.check:
        result = guard.can_delete(args.check)
        print(f"Can delete: {result.get('allowed')}")
        print(f"Reason: {result.get('reason')}")
    elif args.list_protected:
        protected = guard.get_protected_artifacts()
        print(f"Protected AI artifacts: {len(protected)}")
        for artifact in protected:
            print(f"  {artifact['path']} (age: {artifact['age_days']} days)")
    elif args.create_approval:
        approval_path = guard.create_approval(args.create_approval, args.operator, args.reason)
        print(f"Approval created: {approval_path}")


if __name__ == '__main__':
    main()

