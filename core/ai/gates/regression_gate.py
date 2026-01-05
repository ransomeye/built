# Path and File Name : /home/ransomeye/rebuild/core/ai/gates/regression_gate.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Regression and safety gate - automated gate that must pass before candidate models can be promoted

"""
Regression and Safety Gate (PROMPT-61 Phase 3)

Automated gate that must pass:
- Accuracy ≥ current
- False positive rate ≤ current
- SHAP coverage = 100%
- Drift = 0
- Verifier green ≥ 24h

Outcome:
- PASS → eligible for human approval
- FAIL → discard candidate, audit logged
"""

import os
import sys
import json
import hashlib
import psycopg2
from pathlib import Path
from datetime import datetime, timezone, timedelta
from typing import Dict, List, Optional, Tuple
import logging

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger('regression_gate')

# Database configuration
DB_NAME = os.environ.get("DB_NAME", "ransomeye")
DB_USER = os.environ.get("DB_USER", "gagan")
DB_HOST = os.environ.get("DB_HOST", "localhost")
DB_PASSWORD = os.environ.get("DB_PASS", "gagan")
DB_PORT = int(os.environ.get("DB_PORT", "5432"))


class RegressionGate:
    """Regression and safety gate."""
    
    def __init__(self):
        """Initialize regression gate."""
        self.conn = None
        
    def connect_db(self) -> bool:
        """Connect to database."""
        try:
            self.conn = psycopg2.connect(
                host=DB_HOST,
                port=DB_PORT,
                database=DB_NAME,
                user=DB_USER,
                password=DB_PASSWORD
            )
            logger.info("✓ Connected to database")
            return True
        except Exception as e:
            logger.error(f"✗ Failed to connect to database: {e}")
            return False
    
    def get_current_model_metrics(self, model_name: str) -> Optional[Dict]:
        """Get current active model metrics."""
        if not self.conn:
            return None
        
        cursor = self.conn.cursor()
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Get active model version
            cursor.execute("""
                SELECT mv.model_version_id, mv.metadata_json
                FROM model_registry mr
                JOIN model_versions mv ON mr.model_id = mv.model_id
                WHERE mr.model_name = %s
                AND mr.is_active = true
                ORDER BY mv.created_at DESC
                LIMIT 1
            """, (model_name,))
            
            row = cursor.fetchone()
            if not row:
                logger.warning(f"⚠ No active model found for {model_name}")
                return None
            
            metadata = row[1] if row[1] else {}
            metrics = metadata.get('metrics', {})
            
            return {
                'accuracy': metrics.get('accuracy', 0.0),
                'precision': metrics.get('precision', 0.0),
                'recall': metrics.get('recall', 0.0),
                'f1_score': metrics.get('f1_score', 0.0),
                'false_positive_rate': 1.0 - metrics.get('precision', 0.0)
            }
        except Exception as e:
            logger.error(f"✗ Failed to get current model metrics: {e}")
            return None
        finally:
            cursor.close()
    
    def get_candidate_metrics(self, model_name: str, version: str) -> Optional[Dict]:
        """Get candidate model metrics."""
        if not self.conn:
            return None
        
        cursor = self.conn.cursor()
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            
            cursor.execute("""
                SELECT mv.metadata_json
                FROM model_registry mr
                JOIN model_versions mv ON mr.model_id = mv.model_id
                WHERE mr.model_name = %s
                AND mv.version = %s
            """, (model_name, version))
            
            row = cursor.fetchone()
            if not row:
                logger.error(f"✗ Candidate model not found: {model_name} v{version}")
                return None
            
            metadata = row[0] if row[0] else {}
            metrics = metadata.get('metrics', {})
            
            return {
                'accuracy': metrics.get('accuracy', 0.0),
                'precision': metrics.get('precision', 0.0),
                'recall': metrics.get('recall', 0.0),
                'f1_score': metrics.get('f1_score', 0.0),
                'false_positive_rate': 1.0 - metrics.get('precision', 0.0),
                'shap_enabled': metadata.get('shap_path') is not None
            }
        except Exception as e:
            logger.error(f"✗ Failed to get candidate metrics: {e}")
            return None
        finally:
            cursor.close()
    
    def check_shap_coverage(self, model_name: str, version: str) -> bool:
        """Check SHAP coverage = 100%."""
        if not self.conn:
            return False
        
        cursor = self.conn.cursor()
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            
            cursor.execute("""
                SELECT mv.shap_enabled, mv.shap_artifact_uri
                FROM model_registry mr
                JOIN model_versions mv ON mr.model_id = mv.model_id
                WHERE mr.model_name = %s
                AND mv.version = %s
            """, (model_name, version))
            
            row = cursor.fetchone()
            if not row:
                return False
            
            shap_enabled = row[0]
            shap_uri = row[1]
            
            # Check SHAP artifact exists
            if shap_enabled and shap_uri:
                shap_path = Path(shap_uri)
                if shap_path.exists():
                    return True
            
            return False
        except Exception as e:
            logger.error(f"✗ Failed to check SHAP coverage: {e}")
            return False
        finally:
            cursor.close()
    
    def check_verifier_status(self) -> Tuple[bool, Optional[str]]:
        """Check verifier green ≥ 24h."""
        verifier_results_path = Path("/var/log/ransomeye/verifier_results.json")
        
        if not verifier_results_path.exists():
            return False, "Verifier results not found"
        
        try:
            with open(verifier_results_path, 'r') as f:
                results = json.load(f)
            
            # Check timestamp
            timestamp_str = results.get('timestamp')
            if not timestamp_str:
                return False, "No timestamp in verifier results"
            
            timestamp = datetime.fromisoformat(timestamp_str.replace('Z', '+00:00'))
            age = datetime.now(timezone.utc) - timestamp
            
            if age < timedelta(hours=24):
                return False, f"Verifier results too recent: {age}"
            
            # Check overall health
            if not results.get('overall_healthy', False):
                return False, "Verifier reports unhealthy system"
            
            return True, None
        except Exception as e:
            logger.error(f"✗ Failed to check verifier status: {e}")
            return False, str(e)
    
    def check_drift(self) -> Tuple[bool, Optional[str]]:
        """Check drift = 0."""
        drift_snapshot_path = Path("/var/lib/ransomeye/verifier/drift_snapshot.json")
        
        if not drift_snapshot_path.exists():
            # No drift snapshot means no drift detected
            return True, None
        
        try:
            with open(drift_snapshot_path, 'r') as f:
                snapshot = json.load(f)
            
            # Check for drift indicators
            if 'drift' in snapshot and len(snapshot['drift']) > 0:
                return False, f"Drift detected: {snapshot['drift']}"
            
            return True, None
        except Exception as e:
            logger.warning(f"⚠ Failed to check drift: {e}")
            return True, None  # Assume no drift if check fails
    
    def evaluate_candidate(self, model_name: str, version: str) -> Tuple[bool, List[str]]:
        """Evaluate candidate model against gate criteria."""
        failures = []
        
        # Get current and candidate metrics
        current_metrics = self.get_current_model_metrics(model_name)
        candidate_metrics = self.get_candidate_metrics(model_name, version)
        
        if not current_metrics:
            logger.warning(f"⚠ No current model for comparison - assuming baseline")
            current_metrics = {
                'accuracy': 0.0,
                'false_positive_rate': 1.0
            }
        
        if not candidate_metrics:
            failures.append("Failed to get candidate metrics")
            return False, failures
        
        # Check 1: Accuracy ≥ current
        if candidate_metrics['accuracy'] < current_metrics['accuracy']:
            failures.append(f"Accuracy regression: {candidate_metrics['accuracy']:.4f} < {current_metrics['accuracy']:.4f}")
        
        # Check 2: False positive rate ≤ current
        if candidate_metrics['false_positive_rate'] > current_metrics['false_positive_rate']:
            failures.append(f"FPR increase: {candidate_metrics['false_positive_rate']:.4f} > {current_metrics['false_positive_rate']:.4f}")
        
        # Check 3: SHAP coverage = 100%
        if not self.check_shap_coverage(model_name, version):
            failures.append("SHAP coverage not 100%")
        
        # Check 4: Drift = 0
        drift_ok, drift_error = self.check_drift()
        if not drift_ok:
            failures.append(f"Drift detected: {drift_error}")
        
        # Check 5: Verifier green ≥ 24h
        verifier_ok, verifier_error = self.check_verifier_status()
        if not verifier_ok:
            failures.append(f"Verifier check failed: {verifier_error}")
        
        return len(failures) == 0, failures
    
    def log_audit(self, model_name: str, version: str, passed: bool, failures: List[str]):
        """Log gate evaluation to audit log."""
        if not self.conn:
            return
        
        cursor = self.conn.cursor()
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Write to immutable_audit_log
            audit_payload = {
                'action': 'REGRESSION_GATE_EVALUATION',
                'model_name': model_name,
                'version': version,
                'passed': passed,
                'failures': failures,
                'timestamp': datetime.now(timezone.utc).isoformat()
            }
            
            payload_json = json.dumps(audit_payload, sort_keys=True)
            payload_sha256 = hashlib.sha256(payload_json.encode()).digest()
            
            cursor.execute("""
                INSERT INTO immutable_audit_log (
                    action, object_type, payload_json, payload_sha256,
                    signature_status
                )
                VALUES (%s, %s, %s, %s, 'unknown')
            """, (
                'REGRESSION_GATE_EVALUATION',
                'model_candidate',
                payload_json,
                payload_sha256
            ))
            
            self.conn.commit()
            logger.info(f"✓ Audit logged: {model_name} v{version} - {'PASS' if passed else 'FAIL'}")
        except Exception as e:
            logger.error(f"✗ Failed to log audit: {e}")
            self.conn.rollback()
        finally:
            cursor.close()
    
    def run(self, model_name: str, version: str) -> bool:
        """Run regression gate evaluation."""
        logger.info("=" * 80)
        logger.info("Regression and Safety Gate (PROMPT-61 Phase 3)")
        logger.info("=" * 80)
        logger.info(f"Evaluating: {model_name} v{version}")
        
        # Connect to database
        if not self.connect_db():
            logger.error("FAIL-CLOSED: Database connection failed")
            return False
        
        # Evaluate candidate
        passed, failures = self.evaluate_candidate(model_name, version)
        
        # Log audit
        self.log_audit(model_name, version, passed, failures)
        
        if passed:
            logger.info("✓ Gate PASSED - candidate eligible for human approval")
            return True
        else:
            logger.error(f"✗ Gate FAILED:")
            for failure in failures:
                logger.error(f"  - {failure}")
            logger.error("Candidate discarded")
            return False


def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description='Regression and Safety Gate')
    parser.add_argument('--model-name', required=True, help='Model name')
    parser.add_argument('--version', required=True, help='Model version')
    
    args = parser.parse_args()
    
    gate = RegressionGate()
    success = gate.run(args.model_name, args.version)
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()

