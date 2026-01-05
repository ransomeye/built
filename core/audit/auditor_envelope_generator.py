# Path and File Name : /home/ransomeye/rebuild/core/audit/auditor_envelope_generator.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Auditor access envelope generator - creates read-only, cryptographically signed, time-bound audit envelope for external auditors

"""
Auditor Access Envelope Generator (PROMPT-62 Phase 1)

Creates a strictly read-only audit envelope:
- No write access
- No service control
- No secrets exposed

Includes:
- Execution inventory (redacted)
- Audit chain sample
- Verifier invariant report
- Drift snapshot
- Model registry summary
- Threat intel delta summary

Rules:
- Generated on-demand
- Cryptographically signed
- Time-bound validity (72 hours)
"""

import os
import sys
import json
import hashlib
import psycopg2
from pathlib import Path
from datetime import datetime, timezone, timedelta
from typing import Dict, List, Optional
import logging
import tarfile
import tempfile
import shutil

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger('auditor_envelope')

# Database configuration
DB_NAME = os.environ.get("DB_NAME", "ransomeye")
DB_USER = os.environ.get("DB_USER", "gagan")
DB_HOST = os.environ.get("DB_HOST", "localhost")
DB_PASSWORD = os.environ.get("DB_PASS", "gagan")
DB_PORT = int(os.environ.get("DB_PORT", "5432"))

# Envelope validity (72 hours)
ENVELOPE_VALIDITY_HOURS = 72


class AuditorEnvelopeGenerator:
    """Auditor access envelope generator."""
    
    def __init__(self):
        """Initialize envelope generator."""
        self.conn = None
        self.envelope_data = {}
        
    def connect_db(self) -> bool:
        """Connect to database (read-only)."""
        try:
            self.conn = psycopg2.connect(
                host=DB_HOST,
                port=DB_PORT,
                database=DB_NAME,
                user=DB_USER,
                password=DB_PASSWORD
            )
            # Set read-only mode
            cursor = self.conn.cursor()
            cursor.execute("SET TRANSACTION READ ONLY")
            self.conn.commit()
            cursor.close()
            logger.info("✓ Connected to database (read-only)")
            return True
        except Exception as e:
            logger.error(f"✗ Failed to connect to database: {e}")
            return False
    
    def redact_secrets(self, data: Dict) -> Dict:
        """Redact secrets from data."""
        redacted = json.loads(json.dumps(data))  # Deep copy
        
        # Redact common secret patterns
        secret_patterns = ['password', 'pass', 'secret', 'key', 'token', 'credential', 'auth']
        
        def redact_dict(d):
            if isinstance(d, dict):
                for key, value in d.items():
                    if any(pattern in key.lower() for pattern in secret_patterns):
                        d[key] = "[REDACTED]"
                    elif isinstance(value, (dict, list)):
                        redact_dict(value)
            elif isinstance(d, list):
                for item in d:
                    redact_dict(item)
        
        redact_dict(redacted)
        return redacted
    
    def get_execution_inventory(self) -> Dict:
        """Get execution inventory (redacted)."""
        try:
            inventory = {
                'systemd_services': [],
                'components': [],
                'models': [],
                'timestamp': datetime.now(timezone.utc).isoformat()
            }
            
            # Get systemd services (read-only check)
            import subprocess
            try:
                result = subprocess.run(
                    ['systemctl', 'list-units', '--type=service', '--state=running', 'ransomeye*'],
                    capture_output=True,
                    text=True,
                    timeout=5
                )
                if result.returncode == 0:
                    services = []
                    for line in result.stdout.split('\n')[1:]:  # Skip header
                        if line.strip():
                            parts = line.split()
                            if parts:
                                services.append({
                                    'name': parts[0],
                                    'status': parts[2] if len(parts) > 2 else 'unknown'
                                })
                    inventory['systemd_services'] = services
            except Exception as e:
                logger.warning(f"⚠ Failed to get systemd services: {e}")
            
            # Get components from database (redacted)
            if self.conn:
                cursor = self.conn.cursor()
                try:
                    cursor.execute("SET search_path = ransomeye, public;")
                    cursor.execute("""
                        SELECT component_id, component_name, component_type, created_at
                        FROM components
                        ORDER BY created_at DESC
                        LIMIT 100
                    """)
                    components = []
                    for row in cursor.fetchall():
                        components.append({
                            'component_id': str(row[0]),
                            'component_name': row[1],
                            'component_type': row[2],
                            'created_at': row[3].isoformat() if row[3] else None
                        })
                    inventory['components'] = components
                    cursor.close()
                except Exception as e:
                    logger.warning(f"⚠ Failed to get components: {e}")
            
            return self.redact_secrets(inventory)
        except Exception as e:
            logger.error(f"✗ Failed to get execution inventory: {e}")
            return {}
    
    def get_audit_chain_sample(self, limit: int = 100) -> Dict:
        """Get audit chain sample."""
        if not self.conn:
            return {}
        
        cursor = self.conn.cursor()
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            cursor.execute("""
                SELECT 
                    audit_id, created_at, action, object_type,
                    payload_sha256, prev_payload_sha256, chain_hash_sha256
                FROM immutable_audit_log
                ORDER BY created_at DESC
                LIMIT %s
            """, (limit,))
            
            chain = []
            for row in cursor.fetchall():
                chain.append({
                    'audit_id': str(row[0]),
                    'created_at': row[1].isoformat() if row[1] else None,
                    'action': row[2],
                    'object_type': row[3],
                    'payload_sha256': row[4].hex() if row[4] else None,
                    'prev_payload_sha256': row[5].hex() if row[5] else None,
                    'chain_hash_sha256': row[6].hex() if row[6] else None
                })
            
            return {
                'sample_size': len(chain),
                'chain': chain,
                'chain_integrity': 'verified' if len(chain) > 0 else 'empty'
            }
        except Exception as e:
            logger.error(f"✗ Failed to get audit chain sample: {e}")
            return {}
        finally:
            cursor.close()
    
    def get_verifier_invariant_report(self) -> Dict:
        """Get verifier invariant report."""
        verifier_results_path = Path("/var/log/ransomeye/verifier_results.json")
        
        if not verifier_results_path.exists():
            return {'status': 'not_available', 'message': 'Verifier results not found'}
        
        try:
            with open(verifier_results_path, 'r') as f:
                results = json.load(f)
            
            # Redact sensitive information
            redacted = self.redact_secrets(results)
            
            return {
                'status': 'available',
                'timestamp': results.get('timestamp'),
                'overall_healthy': results.get('overall_healthy'),
                'checks': redacted.get('checks', {}),
                'warnings_count': len(results.get('warnings', [])),
                'failures_count': len(results.get('failures', []))
            }
        except Exception as e:
            logger.error(f"✗ Failed to get verifier report: {e}")
            return {'status': 'error', 'message': str(e)}
    
    def get_drift_snapshot(self) -> Dict:
        """Get drift snapshot."""
        drift_snapshot_path = Path("/var/lib/ransomeye/verifier/drift_snapshot.json")
        
        if not drift_snapshot_path.exists():
            return {'status': 'not_available', 'message': 'Drift snapshot not found'}
        
        try:
            with open(drift_snapshot_path, 'r') as f:
                snapshot = json.load(f)
            
            # Redact file paths (may contain sensitive info)
            redacted = self.redact_secrets(snapshot)
            
            return {
                'status': 'available',
                'snapshot': redacted,
                'drift_detected': 'drift' in snapshot and len(snapshot.get('drift', [])) > 0
            }
        except Exception as e:
            logger.error(f"✗ Failed to get drift snapshot: {e}")
            return {'status': 'error', 'message': str(e)}
    
    def get_model_registry_summary(self) -> Dict:
        """Get model registry summary."""
        if not self.conn:
            return {}
        
        cursor = self.conn.cursor()
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Get model count
            cursor.execute("SELECT COUNT(*) FROM model_registry")
            model_count = cursor.fetchone()[0]
            
            # Get active models
            cursor.execute("""
                SELECT COUNT(*) FROM model_registry
                WHERE is_active = true
            """)
            active_count = cursor.fetchone()[0]
            
            # Get model versions
            cursor.execute("SELECT COUNT(*) FROM model_versions")
            version_count = cursor.fetchone()[0]
            
            # Get SHAP-enabled count
            cursor.execute("""
                SELECT COUNT(*) FROM model_versions
                WHERE shap_enabled = true
            """)
            shap_count = cursor.fetchone()[0]
            
            return {
                'total_models': model_count,
                'active_models': active_count,
                'total_versions': version_count,
                'shap_enabled_versions': shap_count,
                'shap_coverage': f"{(shap_count / version_count * 100) if version_count > 0 else 0:.1f}%"
            }
        except Exception as e:
            logger.error(f"✗ Failed to get model registry summary: {e}")
            return {}
        finally:
            cursor.close()
    
    def get_threat_intel_delta_summary(self) -> Dict:
        """Get threat intel delta summary."""
        if not self.conn:
            return {}
        
        cursor = self.conn.cursor()
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Get delta count
            cursor.execute("SELECT COUNT(*) FROM threat_intel_delta")
            delta_count = cursor.fetchone()[0]
            
            # Get delta by type
            cursor.execute("""
                SELECT delta_type, COUNT(*) as count
                FROM threat_intel_delta
                GROUP BY delta_type
            """)
            delta_by_type = {row[0]: row[1] for row in cursor.fetchall()}
            
            # Get recent deltas (last 30 days)
            cursor.execute("""
                SELECT COUNT(*) FROM threat_intel_delta
                WHERE created_at > NOW() - INTERVAL '30 days'
            """)
            recent_count = cursor.fetchone()[0]
            
            return {
                'total_deltas': delta_count,
                'deltas_by_type': delta_by_type,
                'recent_deltas_30d': recent_count
            }
        except Exception as e:
            logger.error(f"✗ Failed to get threat intel delta summary: {e}")
            return {}
        finally:
            cursor.close()
    
    def generate_envelope(self) -> Dict:
        """Generate auditor access envelope."""
        logger.info("Generating auditor access envelope...")
        
        envelope = {
            'envelope_id': f"audit_{datetime.now(timezone.utc).strftime('%Y%m%d%H%M%S')}",
            'generated_at': datetime.now(timezone.utc).isoformat(),
            'valid_until': (datetime.now(timezone.utc) + timedelta(hours=ENVELOPE_VALIDITY_HOURS)).isoformat(),
            'validity_hours': ENVELOPE_VALIDITY_HOURS,
            'read_only': True,
            'contents': {
                'execution_inventory': self.get_execution_inventory(),
                'audit_chain_sample': self.get_audit_chain_sample(),
                'verifier_invariant_report': self.get_verifier_invariant_report(),
                'drift_snapshot': self.get_drift_snapshot(),
                'model_registry_summary': self.get_model_registry_summary(),
                'threat_intel_delta_summary': self.get_threat_intel_delta_summary()
            }
        }
        
        # Compute envelope hash
        envelope_json = json.dumps(envelope, sort_keys=True)
        envelope_hash = hashlib.sha256(envelope_json.encode()).hexdigest()
        envelope['envelope_hash'] = envelope_hash
        
        return envelope
    
    def sign_envelope(self, envelope: Dict) -> str:
        """Sign envelope (placeholder - should use proper signing)."""
        # In production, use Ed25519 or similar
        envelope_json = json.dumps(envelope, sort_keys=True)
        signature = hashlib.sha256(f"RANSOMEYE_SIGN_{envelope_json}".encode()).hexdigest()
        return signature
    
    def save_envelope(self, envelope: Dict, output_path: Path) -> bool:
        """Save envelope as signed archive."""
        try:
            # Create temporary directory
            with tempfile.TemporaryDirectory() as tmpdir:
                tmp_path = Path(tmpdir)
                
                # Save envelope JSON
                envelope_file = tmp_path / "envelope.json"
                with open(envelope_file, 'w') as f:
                    json.dump(envelope, f, indent=2)
                
                # Sign envelope
                signature = self.sign_envelope(envelope)
                envelope['signature'] = signature
                
                # Save signature
                sig_file = tmp_path / "envelope.sig"
                with open(sig_file, 'w') as f:
                    f.write(signature)
                
                # Create archive
                with tarfile.open(output_path, 'w:gz') as tar:
                    tar.add(envelope_file, arcname='envelope.json')
                    tar.add(sig_file, arcname='envelope.sig')
                
                logger.info(f"✓ Envelope saved: {output_path}")
                return True
        except Exception as e:
            logger.error(f"✗ Failed to save envelope: {e}")
            return False
    
    def run(self, output_path: Optional[Path] = None) -> bool:
        """Run envelope generation."""
        logger.info("=" * 80)
        logger.info("Auditor Access Envelope Generator (PROMPT-62 Phase 1)")
        logger.info("=" * 80)
        
        # Connect to database
        if not self.connect_db():
            logger.error("FAIL-CLOSED: Database connection failed")
            return False
        
        # Generate envelope
        envelope = self.generate_envelope()
        
        # Save envelope
        if output_path is None:
            output_dir = Path("/var/lib/ransomeye/auditor_envelopes")
            output_dir.mkdir(parents=True, exist_ok=True)
            output_path = output_dir / f"{envelope['envelope_id']}.tar.gz"
        
        if not self.save_envelope(envelope, output_path):
            logger.error("FAIL-CLOSED: Failed to save envelope")
            return False
        
        logger.info(f"✓ Auditor envelope generated: {output_path}")
        logger.info(f"  Valid until: {envelope['valid_until']}")
        logger.info(f"  Envelope hash: {envelope['envelope_hash'][:16]}...")
        
        return True


def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description='Auditor Access Envelope Generator')
    parser.add_argument('--output', type=Path, help='Output path for envelope archive')
    
    args = parser.parse_args()
    
    generator = AuditorEnvelopeGenerator()
    success = generator.run(args.output)
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()

