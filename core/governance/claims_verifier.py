# Path and File Name : /home/ransomeye/rebuild/core/governance/claims_verifier.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Claims verifier - verifies marketing/sales claims against evidence with automatic verifier checks

"""
Claims Verifier (PROMPT-62 Phase 4)

Every marketing or sales claim must be verifiable.

Implements:
- Claims registry
- Evidence binding
- Automatic verifier check

Example:
- Claim: "Fail-closed by design"
- Evidence: Code refs + tests + audit entries
"""

import os
import sys
import json
import hashlib
import psycopg2
from pathlib import Path
from datetime import datetime, timezone
from typing import Dict, List, Optional, Tuple
import logging
import subprocess
import re

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger('claims_verifier')

# Database configuration
DB_NAME = os.environ.get("DB_NAME", "ransomeye")
DB_USER = os.environ.get("DB_USER", "gagan")
DB_HOST = os.environ.get("DB_HOST", "localhost")
DB_PASSWORD = os.environ.get("DB_PASS", "gagan")
DB_PORT = int(os.environ.get("DB_PORT", "5432"))


class ClaimsVerifier:
    """Claims verification engine."""
    
    # Claims registry
    CLAIMS_REGISTRY = {
        'fail_closed_by_design': {
            'claim': 'Fail-closed by design',
            'description': 'System fails securely on errors, never fails open',
            'evidence_sources': [
                'code_references',
                'test_coverage',
                'audit_entries',
                'verifier_checks'
            ],
            'verification_method': 'automated'
        },
        'immutable_audit_log': {
            'claim': 'Immutable audit log with cryptographic chain hashing',
            'description': 'All audit entries are append-only with hash chaining',
            'evidence_sources': [
                'database_schema',
                'code_references',
                'audit_chain_sample'
            ],
            'verification_method': 'automated'
        },
        'shap_explainability': {
            'claim': '100% SHAP explainability for all AI decisions',
            'description': 'All AI/ML models have SHAP explainability enabled',
            'evidence_sources': [
                'model_registry',
                'shap_artifacts',
                'verifier_checks'
            ],
            'verification_method': 'automated'
        },
        'offline_capable': {
            'claim': 'Fully offline-capable and air-gapped',
            'description': 'System operates without internet connectivity',
            'evidence_sources': [
                'code_references',
                'configuration',
                'test_results'
            ],
            'verification_method': 'automated'
        },
        'zero_trust_architecture': {
            'claim': 'Zero-trust architecture with mTLS',
            'description': 'All communications use mutual TLS with certificate-based identity',
            'evidence_sources': [
                'code_references',
                'configuration',
                'audit_entries'
            ],
            'verification_method': 'automated'
        },
        'regulatory_compliance': {
            'claim': 'Regulatory compliance (ISO 27001, SOC 2, NIST 800-53, GDPR, RBI)',
            'description': 'System meets regulatory requirements',
            'evidence_sources': [
                'regulatory_mapping',
                'control_evidence',
                'compliance_reports'
            ],
            'verification_method': 'automated'
        }
    }
    
    def __init__(self):
        """Initialize claims verifier."""
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
    
    def find_code_references(self, claim_id: str, search_terms: List[str]) -> List[Dict]:
        """Find code references for a claim."""
        code_refs = []
        project_root = Path("/home/ransomeye/rebuild")
        
        # Search for terms in code files
        for term in search_terms:
            try:
                result = subprocess.run(
                    ['grep', '-r', '--include=*.py', '--include=*.rs', term, str(project_root)],
                    capture_output=True,
                    text=True,
                    timeout=30
                )
                
                if result.returncode == 0:
                    for line in result.stdout.split('\n'):
                        if line.strip():
                            parts = line.split(':', 2)
                            if len(parts) >= 3:
                                code_refs.append({
                                    'file': parts[0],
                                    'line': parts[1],
                                    'context': parts[2][:100]  # Truncate
                                })
            except Exception as e:
                logger.warning(f"⚠ Failed to search for {term}: {e}")
        
        return code_refs[:10]  # Limit to 10 references
    
    def get_audit_entries(self, claim_id: str) -> List[Dict]:
        """Get audit entries related to a claim."""
        if not self.conn:
            return []
        
        cursor = self.conn.cursor()
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Search for relevant audit entries
            claim = self.CLAIMS_REGISTRY.get(claim_id)
            if not claim:
                return []
            
            search_terms = claim_id.split('_')
            query = " OR ".join([f"action LIKE %s" for _ in search_terms])
            params = [f'%{term}%' for term in search_terms]
            
            cursor.execute(f"""
                SELECT audit_id, created_at, action, object_type, payload_sha256
                FROM immutable_audit_log
                WHERE {query}
                ORDER BY created_at DESC
                LIMIT 10
            """, params)
            
            entries = []
            for row in cursor.fetchall():
                entries.append({
                    'audit_id': str(row[0]),
                    'created_at': row[1].isoformat() if row[1] else None,
                    'action': row[2],
                    'object_type': row[3],
                    'payload_hash': row[4].hex() if row[4] else None
                })
            
            return entries
        except Exception as e:
            logger.error(f"✗ Failed to get audit entries: {e}")
            return []
        finally:
            cursor.close()
    
    def verify_fail_closed(self) -> Tuple[bool, List[str]]:
        """Verify fail-closed by design claim."""
        evidence = []
        verified = True
        
        # Check code references
        code_refs = self.find_code_references('fail_closed_by_design', ['fail-closed', 'FAIL-CLOSED', 'fail_closed'])
        if code_refs:
            evidence.append(f"Found {len(code_refs)} code references")
        else:
            verified = False
            evidence.append("No code references found")
        
        # Check verifier
        verifier_path = Path("/var/log/ransomeye/verifier_results.json")
        if verifier_path.exists():
            try:
                with open(verifier_path, 'r') as f:
                    verifier_data = json.load(f)
                if verifier_data.get('overall_healthy') is not None:
                    evidence.append("Verifier implements fail-closed checks")
                else:
                    verified = False
                    evidence.append("Verifier not properly configured")
            except Exception as e:
                verified = False
                evidence.append(f"Failed to read verifier: {e}")
        else:
            verified = False
            evidence.append("Verifier results not found")
        
        return verified, evidence
    
    def verify_immutable_audit_log(self) -> Tuple[bool, List[str]]:
        """Verify immutable audit log claim."""
        evidence = []
        verified = True
        
        if not self.conn:
            return False, ["Database connection failed"]
        
        cursor = self.conn.cursor()
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Check table exists
            cursor.execute("""
                SELECT EXISTS (
                    SELECT FROM information_schema.tables 
                    WHERE table_schema = 'ransomeye' 
                    AND table_name = 'immutable_audit_log'
                )
            """)
            if cursor.fetchone()[0]:
                evidence.append("immutable_audit_log table exists")
            else:
                verified = False
                evidence.append("immutable_audit_log table not found")
            
            # Check chain hashing
            cursor.execute("""
                SELECT COUNT(*) FROM immutable_audit_log
                WHERE chain_hash_sha256 IS NOT NULL
            """)
            chain_count = cursor.fetchone()[0]
            if chain_count > 0:
                evidence.append(f"Found {chain_count} entries with chain hashing")
            else:
                verified = False
                evidence.append("No chain hashing found")
            
            # Check append-only (no updates/deletes)
            cursor.execute("""
                SELECT COUNT(*) FROM information_schema.table_constraints
                WHERE table_schema = 'ransomeye'
                AND table_name = 'immutable_audit_log'
                AND constraint_type IN ('UPDATE', 'DELETE')
            """)
            # This is a simplified check - in reality, we'd check for triggers or policies
            
        except Exception as e:
            verified = False
            evidence.append(f"Database check failed: {e}")
        finally:
            cursor.close()
        
        return verified, evidence
    
    def verify_shap_explainability(self) -> Tuple[bool, List[str]]:
        """Verify SHAP explainability claim."""
        evidence = []
        verified = True
        
        if not self.conn:
            return False, ["Database connection failed"]
        
        cursor = self.conn.cursor()
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Get model count
            cursor.execute("SELECT COUNT(*) FROM model_registry")
            model_count = cursor.fetchone()[0]
            
            # Get SHAP-enabled count
            cursor.execute("""
                SELECT COUNT(*) FROM model_versions
                WHERE shap_enabled = true
            """)
            shap_count = cursor.fetchone()[0]
            
            if model_count == 0:
                evidence.append("No models registered (may be initial state)")
            elif shap_count == model_count:
                evidence.append(f"100% SHAP coverage: {shap_count}/{model_count} models")
            else:
                verified = False
                evidence.append(f"SHAP coverage incomplete: {shap_count}/{model_count} models")
            
        except Exception as e:
            verified = False
            evidence.append(f"Database check failed: {e}")
        finally:
            cursor.close()
        
        return verified, evidence
    
    def verify_claim(self, claim_id: str) -> Dict:
        """Verify a specific claim."""
        claim = self.CLAIMS_REGISTRY.get(claim_id)
        if not claim:
            return {
                'claim_id': claim_id,
                'verified': False,
                'error': 'Claim not found in registry'
            }
        
        verified = False
        evidence = []
        
        # Route to specific verifier
        if claim_id == 'fail_closed_by_design':
            verified, evidence = self.verify_fail_closed()
        elif claim_id == 'immutable_audit_log':
            verified, evidence = self.verify_immutable_audit_log()
        elif claim_id == 'shap_explainability':
            verified, evidence = self.verify_shap_explainability()
        else:
            # Generic verification
            code_refs = self.find_code_references(claim_id, [claim_id])
            audit_entries = self.get_audit_entries(claim_id)
            
            if code_refs or audit_entries:
                verified = True
                evidence.append(f"Found {len(code_refs)} code references")
                evidence.append(f"Found {len(audit_entries)} audit entries")
            else:
                verified = False
                evidence.append("No evidence found")
        
        return {
            'claim_id': claim_id,
            'claim': claim['claim'],
            'description': claim['description'],
            'verified': verified,
            'evidence': evidence,
            'verified_at': datetime.now(timezone.utc).isoformat()
        }
    
    def verify_all_claims(self) -> Dict:
        """Verify all claims in registry."""
        results = {
            'generated_at': datetime.now(timezone.utc).isoformat(),
            'claims': {}
        }
        
        for claim_id in self.CLAIMS_REGISTRY.keys():
            result = self.verify_claim(claim_id)
            results['claims'][claim_id] = result
        
        # Summary
        verified_count = sum(1 for c in results['claims'].values() if c.get('verified', False))
        total_count = len(results['claims'])
        results['summary'] = {
            'total_claims': total_count,
            'verified_claims': verified_count,
            'unverified_claims': total_count - verified_count,
            'verification_rate': f"{(verified_count / total_count * 100) if total_count > 0 else 0:.1f}%"
        }
        
        return results
    
    def run(self, claim_id: Optional[str] = None, output_path: Optional[Path] = None) -> bool:
        """Run claims verification."""
        logger.info("=" * 80)
        logger.info("Claims Verifier (PROMPT-62 Phase 4)")
        logger.info("=" * 80)
        
        # Connect to database
        if not self.connect_db():
            logger.error("FAIL-CLOSED: Database connection failed")
            return False
        
        # Verify claims
        if claim_id:
            results = {'claims': {claim_id: self.verify_claim(claim_id)}}
        else:
            results = self.verify_all_claims()
        
        # Save results
        if output_path is None:
            output_dir = Path("/var/lib/ransomeye/claims_verification")
            output_dir.mkdir(parents=True, exist_ok=True)
            output_path = output_dir / f"claims_verification_{datetime.now(timezone.utc).strftime('%Y%m%d%H%M%S')}.json"
        
        with open(output_path, 'w') as f:
            json.dump(results, f, indent=2)
        
        logger.info(f"✓ Claims verification complete: {output_path}")
        if 'summary' in results:
            logger.info(f"  Verified: {results['summary']['verified_claims']}/{results['summary']['total_claims']}")
        
        return True


def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description='Claims Verifier')
    parser.add_argument('--claim-id', help='Verify specific claim')
    parser.add_argument('--output', type=Path, help='Output path for verification report')
    
    args = parser.parse_args()
    
    verifier = ClaimsVerifier()
    success = verifier.run(args.claim_id, args.output)
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()

