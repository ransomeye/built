# Path and File Name : /home/ransomeye/rebuild/core/ai/registry/promote_candidate.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Controlled promotion - promotes candidate models to ACTIVE state with human approval, generates promotion audit and attestation

"""
Controlled Promotion (PROMPT-61 Phase 4)

If approved:
- Promote candidate → ACTIVE
- Update model_registry + versions
- Generate promotion audit + attestation

If not approved:
- Candidate expires automatically (30 days)
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
logger = logging.getLogger('promote_candidate')

# Database configuration
DB_NAME = os.environ.get("DB_NAME", "ransomeye")
DB_USER = os.environ.get("DB_USER", "gagan")
DB_HOST = os.environ.get("DB_HOST", "localhost")
DB_PASSWORD = os.environ.get("DB_PASS", "gagan")
DB_PORT = int(os.environ.get("DB_PORT", "5432"))


class CandidatePromoter:
    """Controlled candidate promotion engine."""
    
    def __init__(self):
        """Initialize candidate promoter."""
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
    
    def check_candidate_expired(self, model_name: str, version: str) -> bool:
        """Check if candidate has expired (30 days)."""
        if not self.conn:
            return True
        
        cursor = self.conn.cursor()
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            
            cursor.execute("""
                SELECT mv.created_at
                FROM model_registry mr
                JOIN model_versions mv ON mr.model_id = mv.model_id
                WHERE mr.model_name = %s
                AND mv.version = %s
            """, (model_name, version))
            
            row = cursor.fetchone()
            if not row:
                return True
            
            created_at = row[0]
            age = datetime.now(timezone.utc) - created_at.replace(tzinfo=timezone.utc)
            
            if age > timedelta(days=30):
                logger.warning(f"⚠ Candidate expired: {age.days} days old")
                return True
            
            return False
        except Exception as e:
            logger.error(f"✗ Failed to check expiration: {e}")
            return True
        finally:
            cursor.close()
    
    def check_gate_passed(self, model_name: str, version: str) -> bool:
        """Check if regression gate passed."""
        if not self.conn:
            return False
        
        cursor = self.conn.cursor()
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Check audit log for gate evaluation
            cursor.execute("""
                SELECT payload_json
                FROM immutable_audit_log
                WHERE action = 'REGRESSION_GATE_EVALUATION'
                AND payload_json->>'model_name' = %s
                AND payload_json->>'version' = %s
                ORDER BY created_at DESC
                LIMIT 1
            """, (model_name, version))
            
            row = cursor.fetchone()
            if not row:
                logger.warning(f"⚠ No gate evaluation found for {model_name} v{version}")
                return False
            
            payload = json.loads(row[0])
            passed = payload.get('passed', False)
            
            return passed
        except Exception as e:
            logger.error(f"✗ Failed to check gate status: {e}")
            return False
        finally:
            cursor.close()
    
    def deactivate_current_model(self, model_id: str) -> bool:
        """Deactivate current active model version."""
        if not self.conn:
            return False
        
        cursor = self.conn.cursor()
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Deactivate model in registry
            cursor.execute("""
                UPDATE model_registry
                SET is_active = false, updated_at = now()
                WHERE model_id = %s
            """, (model_id,))
            
            self.conn.commit()
            logger.info(f"✓ Deactivated current model: {model_id}")
            return True
        except Exception as e:
            logger.error(f"✗ Failed to deactivate current model: {e}")
            self.conn.rollback()
            return False
        finally:
            cursor.close()
    
    def promote_candidate(self, model_name: str, version: str, approver: str) -> bool:
        """Promote candidate to ACTIVE."""
        if not self.conn:
            return False
        
        cursor = self.conn.cursor()
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Get model and version IDs
            cursor.execute("""
                SELECT mr.model_id, mv.model_version_id
                FROM model_registry mr
                JOIN model_versions mv ON mr.model_id = mv.model_id
                WHERE mr.model_name = %s
                AND mv.version = %s
            """, (model_name, version))
            
            row = cursor.fetchone()
            if not row:
                logger.error(f"✗ Candidate not found: {model_name} v{version}")
                return False
            
            model_id, version_id = row
            
            # Deactivate current model
            self.deactivate_current_model(model_id)
            
            # Activate candidate
            cursor.execute("""
                UPDATE model_registry
                SET is_active = true, updated_at = now()
                WHERE model_id = %s
            """, (model_id,))
            
            self.conn.commit()
            logger.info(f"✓ Promoted candidate: {model_name} v{version}")
            
            # Generate promotion audit
            self.generate_promotion_audit(model_name, version, approver)
            
            return True
        except Exception as e:
            logger.error(f"✗ Failed to promote candidate: {e}")
            self.conn.rollback()
            return False
        finally:
            cursor.close()
    
    def generate_promotion_audit(self, model_name: str, version: str, approver: str):
        """Generate promotion audit and attestation."""
        if not self.conn:
            return
        
        cursor = self.conn.cursor()
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Create attestation
            attestation = {
                'action': 'MODEL_PROMOTION',
                'model_name': model_name,
                'version': version,
                'approver': approver,
                'timestamp': datetime.now(timezone.utc).isoformat(),
                'promotion_type': 'CANDIDATE_TO_ACTIVE'
            }
            
            payload_json = json.dumps(attestation, sort_keys=True)
            payload_sha256 = hashlib.sha256(payload_json.encode()).digest()
            
            # Get previous audit entry for chain
            cursor.execute("""
                SELECT audit_id, chain_hash_sha256, payload_sha256
                FROM immutable_audit_log
                ORDER BY created_at DESC
                LIMIT 1
            """)
            prev_row = cursor.fetchone()
            
            if prev_row:
                prev_chain_hash = prev_row[1] if prev_row[1] else bytes(32)
            else:
                prev_chain_hash = bytes(32)
            
            # Compute chain hash
            chain_input = prev_chain_hash + payload_sha256
            chain_hash_sha256 = hashlib.sha256(chain_input).digest()
            
            # Insert audit entry
            cursor.execute("""
                INSERT INTO immutable_audit_log (
                    action, object_type, payload_json, payload_sha256,
                    prev_payload_sha256, chain_hash_sha256, signature_status
                )
                VALUES (%s, %s, %s, %s, %s, %s, 'unknown')
            """, (
                'MODEL_PROMOTION',
                'model_version',
                payload_json,
                payload_sha256,
                prev_row[2] if prev_row else None,
                chain_hash_sha256
            ))
            
            self.conn.commit()
            
            # Write attestation file
            attestation_dir = Path("/var/lib/ransomeye/attestations")
            attestation_dir.mkdir(parents=True, exist_ok=True)
            
            attestation_file = attestation_dir / f"promotion_{model_name}_{version}_{datetime.now(timezone.utc).strftime('%Y%m%d%H%M%S')}.json"
            with open(attestation_file, 'w') as f:
                json.dump(attestation, f, indent=2)
            
            logger.info(f"✓ Promotion audit and attestation generated: {attestation_file}")
        except Exception as e:
            logger.error(f"✗ Failed to generate promotion audit: {e}")
            self.conn.rollback()
        finally:
            cursor.close()
    
    def expire_candidates(self):
        """Expire candidates older than 30 days."""
        if not self.conn:
            return
        
        cursor = self.conn.cursor()
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Find expired candidates
            cursor.execute("""
                SELECT mr.model_name, mv.version, mv.created_at
                FROM model_registry mr
                JOIN model_versions mv ON mr.model_id = mv.model_id
                WHERE mr.is_active = false
                AND mv.version LIKE 'candidate-%'
                AND mv.created_at < NOW() - INTERVAL '30 days'
            """)
            
            expired = cursor.fetchall()
            for model_name, version, created_at in expired:
                logger.info(f"Expiring candidate: {model_name} v{version} (created: {created_at})")
                
                # Log expiration
                expiration_payload = {
                    'action': 'CANDIDATE_EXPIRATION',
                    'model_name': model_name,
                    'version': version,
                    'created_at': created_at.isoformat() if created_at else None,
                    'expired_at': datetime.now(timezone.utc).isoformat()
                }
                
                payload_json = json.dumps(expiration_payload, sort_keys=True)
                payload_sha256 = hashlib.sha256(payload_json.encode()).digest()
                
                cursor.execute("""
                    INSERT INTO immutable_audit_log (
                        action, object_type, payload_json, payload_sha256,
                        signature_status
                    )
                    VALUES (%s, %s, %s, %s, 'unknown')
                """, (
                    'CANDIDATE_EXPIRATION',
                    'model_version',
                    payload_json,
                    payload_sha256
                ))
            
            self.conn.commit()
            logger.info(f"✓ Expired {len(expired)} candidates")
        except Exception as e:
            logger.error(f"✗ Failed to expire candidates: {e}")
            self.conn.rollback()
        finally:
            cursor.close()
    
    def run(self, model_name: str, version: str, approver: str, force: bool = False) -> bool:
        """Run controlled promotion."""
        logger.info("=" * 80)
        logger.info("Controlled Promotion (PROMPT-61 Phase 4)")
        logger.info("=" * 80)
        logger.info(f"Promoting: {model_name} v{version}")
        logger.info(f"Approver: {approver}")
        
        # Connect to database
        if not self.connect_db():
            logger.error("FAIL-CLOSED: Database connection failed")
            return False
        
        # Check expiration
        if not force and self.check_candidate_expired(model_name, version):
            logger.error("✗ Candidate expired (30 days)")
            return False
        
        # Check gate passed
        if not force and not self.check_gate_passed(model_name, version):
            logger.error("✗ Regression gate not passed")
            return False
        
        # Promote candidate
        if not self.promote_candidate(model_name, version, approver):
            logger.error("FAIL-CLOSED: Failed to promote candidate")
            return False
        
        # Expire old candidates
        self.expire_candidates()
        
        logger.info("✓ Controlled promotion complete")
        return True


def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description='Controlled Candidate Promotion')
    parser.add_argument('--model-name', required=True, help='Model name')
    parser.add_argument('--version', required=True, help='Model version')
    parser.add_argument('--approver', required=True, help='Approver identifier')
    parser.add_argument('--force', action='store_true', help='Force promotion (skip checks)')
    
    args = parser.parse_args()
    
    promoter = CandidatePromoter()
    success = promoter.run(args.model_name, args.version, args.approver, args.force)
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()

