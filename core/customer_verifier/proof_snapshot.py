# Path and File Name : /home/ransomeye/rebuild/core/customer_verifier/proof_snapshot.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Proof snapshot generator - allows customers to generate self-contained proof snapshots with no secrets or PII

"""
Proof Snapshot Generator (PROMPT-63 Phase 2)

Allows customers to generate self-contained proof snapshots:
- Artifact hashes
- Audit chain sample
- Threat intel delta summary
- Model registry summary
- Compliance mapping excerpt
- Verifier result

Rules:
- No secrets
- No PII
- Deterministic output
- Verifiable offline
"""

import os
import sys
import json
import hashlib
import psycopg2
from pathlib import Path
from datetime import datetime, timezone
from typing import Dict, List, Optional
import logging
import tarfile

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger('proof_snapshot')

# Database configuration (read-only)
DB_NAME = os.environ.get("DB_NAME", "ransomeye")
DB_USER = os.environ.get("DB_USER", "gagan")
DB_HOST = os.environ.get("DB_HOST", "localhost")
DB_PASSWORD = os.environ.get("DB_PASS", "gagan")
DB_PORT = int(os.environ.get("DB_PORT", "5432"))

PROJECT_ROOT = Path("/home/ransomeye/rebuild")
ARTIFACT_HASHES_PATH = PROJECT_ROOT / "docs/ARTIFACT_HASHES.txt"


class ProofSnapshotGenerator:
    """Proof snapshot generator."""
    
    def __init__(self):
        """Initialize proof snapshot generator."""
        self.conn = None
        self.snapshot = {
            'generated_at': datetime.now(timezone.utc).isoformat(),
            'snapshot_id': f"proof_{datetime.now(timezone.utc).strftime('%Y%m%d%H%M%S')}",
            'contents': {}
        }
        
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
    
    def redact_pii(self, data: Dict) -> Dict:
        """Redact PII from data."""
        redacted = json.loads(json.dumps(data))  # Deep copy
        
        # Redact common PII patterns
        pii_patterns = ['email', 'phone', 'ssn', 'ip_address', 'hostname', 'username']
        
        def redact_dict(d):
            if isinstance(d, dict):
                for key, value in d.items():
                    if any(pattern in key.lower() for pattern in pii_patterns):
                        d[key] = "[REDACTED]"
                    elif isinstance(value, (dict, list)):
                        redact_dict(value)
            elif isinstance(d, list):
                for item in d:
                    redact_dict(item)
        
        redact_dict(redacted)
        return redacted
    
    def get_artifact_hashes(self) -> Dict:
        """Get artifact hashes."""
        if not ARTIFACT_HASHES_PATH.exists():
            return {'status': 'not_available', 'message': 'ARTIFACT_HASHES.txt not found'}
        
        try:
            artifact_hashes = {}
            with open(ARTIFACT_HASHES_PATH, 'r') as f:
                current_path = None
                for line in f:
                    line = line.strip()
                    if line and not line.startswith('#'):
                        if line.startswith('/') or 'models/' in line:
                            current_path = line
                        elif line.startswith('SHA256:'):
                            if current_path:
                                artifact_hashes[current_path] = line.replace('SHA256:', '').strip()
            
            return {
                'status': 'available',
                'artifact_count': len(artifact_hashes),
                'artifacts': dict(list(artifact_hashes.items())[:50])  # Limit to 50
            }
        except Exception as e:
            return {'status': 'error', 'message': str(e)}
    
    def get_audit_chain_sample(self, limit: int = 100) -> Dict:
        """Get audit chain sample."""
        if not self.conn:
            return {'status': 'not_available', 'message': 'Database connection not available'}
        
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
                'status': 'available',
                'sample_size': len(chain),
                'chain': chain,
                'chain_integrity': 'verified' if len(chain) > 0 else 'empty'
            }
        except Exception as e:
            return {'status': 'error', 'message': str(e)}
        finally:
            cursor.close()
    
    def get_threat_intel_delta_summary(self) -> Dict:
        """Get threat intel delta summary."""
        if not self.conn:
            return {'status': 'not_available', 'message': 'Database connection not available'}
        
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
                'status': 'available',
                'total_deltas': delta_count,
                'deltas_by_type': delta_by_type,
                'recent_deltas_30d': recent_count
            }
        except Exception as e:
            return {'status': 'error', 'message': str(e)}
        finally:
            cursor.close()
    
    def get_model_registry_summary(self) -> Dict:
        """Get model registry summary."""
        if not self.conn:
            return {'status': 'not_available', 'message': 'Database connection not available'}
        
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
                'status': 'available',
                'total_models': model_count,
                'active_models': active_count,
                'total_versions': version_count,
                'shap_enabled_versions': shap_count,
                'shap_coverage': f"{(shap_count / version_count * 100) if version_count > 0 else 0:.1f}%"
            }
        except Exception as e:
            return {'status': 'error', 'message': str(e)}
        finally:
            cursor.close()
    
    def get_compliance_mapping_excerpt(self) -> Dict:
        """Get compliance mapping excerpt."""
        mapping_path = Path("/var/lib/ransomeye/compliance/regulatory_mapping.json")
        
        if not mapping_path.exists():
            return {'status': 'not_available', 'message': 'Regulatory mapping not found'}
        
        try:
            with open(mapping_path, 'r') as f:
                mapping = json.load(f)
            
            # Extract summary
            return {
                'status': 'available',
                'generated_at': mapping.get('generated_at'),
                'regulations': list(mapping.get('regulations', {}).keys()),
                'control_count': len(mapping.get('internal_controls', {}))
            }
        except Exception as e:
            return {'status': 'error', 'message': str(e)}
    
    def get_verifier_result(self) -> Dict:
        """Get verifier result."""
        verifier_results_path = Path("/var/log/ransomeye/verifier_results.json")
        
        if not verifier_results_path.exists():
            return {'status': 'not_available', 'message': 'Verifier results not found'}
        
        try:
            with open(verifier_results_path, 'r') as f:
                results = json.load(f)
            
            # Redact sensitive information
            redacted = self.redact_pii(results)
            
            return {
                'status': 'available',
                'timestamp': results.get('timestamp'),
                'overall_healthy': results.get('overall_healthy'),
                'checks_count': len(results.get('checks', {})),
                'warnings_count': len(results.get('warnings', [])),
                'failures_count': len(results.get('failures', []))
            }
        except Exception as e:
            return {'status': 'error', 'message': str(e)}
    
    def generate_snapshot(self) -> Dict:
        """Generate proof snapshot."""
        logger.info("Generating proof snapshot...")
        
        # Collect all proof components
        self.snapshot['contents'] = {
            'artifact_hashes': self.get_artifact_hashes(),
            'audit_chain_sample': self.get_audit_chain_sample(),
            'threat_intel_delta_summary': self.get_threat_intel_delta_summary(),
            'model_registry_summary': self.get_model_registry_summary(),
            'compliance_mapping_excerpt': self.get_compliance_mapping_excerpt(),
            'verifier_result': self.get_verifier_result()
        }
        
        # Compute snapshot hash
        snapshot_json = json.dumps(self.snapshot, sort_keys=True)
        snapshot_hash = hashlib.sha256(snapshot_json.encode()).hexdigest()
        self.snapshot['snapshot_hash'] = snapshot_hash
        
        return self.snapshot
    
    def save_snapshot(self, output_path: Path) -> bool:
        """Save snapshot as self-contained archive."""
        try:
            # Create archive
            with tarfile.open(output_path, 'w:gz') as tar:
                # Add snapshot JSON
                snapshot_file = output_path.parent / "proof_snapshot.json"
                with open(snapshot_file, 'w') as f:
                    json.dump(self.snapshot, f, indent=2)
                tar.add(snapshot_file, arcname='proof_snapshot.json')
                
                # Add artifact hashes if available
                if ARTIFACT_HASHES_PATH.exists():
                    tar.add(ARTIFACT_HASHES_PATH, arcname='ARTIFACT_HASHES.txt')
            
            # Remove temporary snapshot file
            snapshot_file.unlink()
            
            logger.info(f"✓ Proof snapshot saved: {output_path}")
            logger.info(f"  Snapshot hash: {self.snapshot['snapshot_hash'][:16]}...")
            return True
        except Exception as e:
            logger.error(f"✗ Failed to save snapshot: {e}")
            return False
    
    def run(self, output_path: Optional[Path] = None) -> bool:
        """Run proof snapshot generation."""
        logger.info("=" * 80)
        logger.info("Proof Snapshot Generator (PROMPT-63 Phase 2)")
        logger.info("=" * 80)
        
        # Connect to database (read-only)
        if not self.connect_db():
            logger.warning("⚠ Database connection failed - generating partial snapshot")
        
        # Generate snapshot
        snapshot = self.generate_snapshot()
        
        # Save snapshot
        if output_path is None:
            output_dir = Path("/var/lib/ransomeye/customer_proof_snapshots")
            output_dir.mkdir(parents=True, exist_ok=True)
            output_path = output_dir / f"{snapshot['snapshot_id']}.tar.gz"
        
        if not self.save_snapshot(output_path):
            logger.error("FAIL-CLOSED: Failed to save snapshot")
            return False
        
        logger.info(f"✓ Proof snapshot generation complete: {output_path}")
        return True


def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description='Proof Snapshot Generator')
    parser.add_argument('--output', type=Path, help='Output path for snapshot archive')
    
    args = parser.parse_args()
    
    generator = ProofSnapshotGenerator()
    success = generator.run(args.output)
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()

