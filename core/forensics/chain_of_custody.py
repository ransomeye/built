# Path and File Name : /home/ransomeye/rebuild/core/forensics/chain_of_custody.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Forensic chain of custody - implements case ID generation, evidence sealing, custody transfer log, and read-only export bundles

"""
Forensic Chain of Custody (PROMPT-62 Phase 3)

Implements:
- Case ID generation
- Evidence sealing (hash + timestamp)
- Custody transfer log
- Read-only export bundles

Rules:
- Append-only
- Verifiable offline
- Tamper-evident
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
import tarfile
import tempfile
import uuid

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger('chain_of_custody')

# Database configuration
DB_NAME = os.environ.get("DB_NAME", "ransomeye")
DB_USER = os.environ.get("DB_USER", "gagan")
DB_HOST = os.environ.get("DB_HOST", "localhost")
DB_PASSWORD = os.environ.get("DB_PASS", "gagan")
DB_PORT = int(os.environ.get("DB_PORT", "5432"))


class ChainOfCustody:
    """Forensic chain of custody manager."""
    
    def __init__(self):
        """Initialize chain of custody."""
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
    
    def ensure_custody_tables(self) -> bool:
        """Ensure chain of custody tables exist."""
        if not self.conn:
            return False
        
        cursor = self.conn.cursor()
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Cases table
            cursor.execute("""
                CREATE TABLE IF NOT EXISTS forensic_cases (
                    case_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
                    case_number text NOT NULL UNIQUE,
                    case_name text,
                    created_at timestamptz NOT NULL DEFAULT now(),
                    created_by text,
                    status text NOT NULL DEFAULT 'open',
                    description text
                );
            """)
            
            # Evidence items table
            cursor.execute("""
                CREATE TABLE IF NOT EXISTS forensic_evidence (
                    evidence_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
                    case_id uuid NOT NULL REFERENCES forensic_cases(case_id) ON DELETE RESTRICT,
                    evidence_type text NOT NULL,
                    evidence_hash bytea NOT NULL,
                    evidence_data jsonb,
                    sealed_at timestamptz NOT NULL DEFAULT now(),
                    sealed_by text,
                    CONSTRAINT forensic_evidence_hash_len_chk CHECK (octet_length(evidence_hash) = 32)
                );
            """)
            
            # Custody transfer log (append-only)
            cursor.execute("""
                CREATE TABLE IF NOT EXISTS custody_transfer_log (
                    transfer_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
                    case_id uuid NOT NULL REFERENCES forensic_cases(case_id) ON DELETE RESTRICT,
                    evidence_id uuid REFERENCES forensic_evidence(evidence_id) ON DELETE RESTRICT,
                    transfer_type text NOT NULL,
                    from_custodian text,
                    to_custodian text,
                    transfer_reason text,
                    transfer_hash bytea NOT NULL,
                    created_at timestamptz NOT NULL DEFAULT now(),
                    CONSTRAINT custody_transfer_hash_len_chk CHECK (octet_length(transfer_hash) = 32)
                );
            """)
            
            # Create indexes
            cursor.execute("""
                CREATE INDEX IF NOT EXISTS idx_forensic_cases_case_number 
                ON forensic_cases (case_number);
            """)
            
            cursor.execute("""
                CREATE INDEX IF NOT EXISTS idx_forensic_evidence_case_id 
                ON forensic_evidence (case_id);
            """)
            
            cursor.execute("""
                CREATE INDEX IF NOT EXISTS idx_custody_transfer_case_id 
                ON custody_transfer_log (case_id);
            """)
            
            self.conn.commit()
            logger.info("✓ Chain of custody tables verified/created")
            return True
        except Exception as e:
            logger.error(f"✗ Failed to create custody tables: {e}")
            self.conn.rollback()
            return False
        finally:
            cursor.close()
    
    def generate_case_id(self, case_name: Optional[str] = None) -> Tuple[str, str]:
        """Generate case ID and case number."""
        case_id = str(uuid.uuid4())
        case_number = f"CASE-{datetime.now(timezone.utc).strftime('%Y%m%d')}-{uuid.uuid4().hex[:8].upper()}"
        
        return case_id, case_number
    
    def create_case(self, case_name: str, created_by: str, description: Optional[str] = None) -> Optional[str]:
        """Create new forensic case."""
        if not self.conn:
            return None
        
        case_id, case_number = self.generate_case_id(case_name)
        
        cursor = self.conn.cursor()
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            cursor.execute("""
                INSERT INTO forensic_cases (
                    case_id, case_number, case_name, created_by, description
                )
                VALUES (%s, %s, %s, %s, %s)
            """, (case_id, case_number, case_name, created_by, description))
            
            self.conn.commit()
            logger.info(f"✓ Case created: {case_number}")
            return case_number
        except Exception as e:
            logger.error(f"✗ Failed to create case: {e}")
            self.conn.rollback()
            return None
        finally:
            cursor.close()
    
    def seal_evidence(self, case_number: str, evidence_type: str, evidence_data: Dict, sealed_by: str) -> Optional[str]:
        """Seal evidence (hash + timestamp)."""
        if not self.conn:
            return None
        
        cursor = self.conn.cursor()
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Get case ID
            cursor.execute("SELECT case_id FROM forensic_cases WHERE case_number = %s", (case_number,))
            case_row = cursor.fetchone()
            if not case_row:
                logger.error(f"✗ Case not found: {case_number}")
                return None
            
            case_id = case_row[0]
            
            # Compute evidence hash
            evidence_json = json.dumps(evidence_data, sort_keys=True)
            evidence_hash = hashlib.sha256(evidence_json.encode()).digest()
            
            # Insert evidence
            cursor.execute("""
                INSERT INTO forensic_evidence (
                    case_id, evidence_type, evidence_hash, evidence_data, sealed_by
                )
                VALUES (%s, %s, %s, %s::jsonb, %s)
                RETURNING evidence_id
            """, (case_id, evidence_type, evidence_hash, evidence_json, sealed_by))
            
            evidence_id = cursor.fetchone()[0]
            
            self.conn.commit()
            logger.info(f"✓ Evidence sealed: {evidence_id}")
            return str(evidence_id)
        except Exception as e:
            logger.error(f"✗ Failed to seal evidence: {e}")
            self.conn.rollback()
            return None
        finally:
            cursor.close()
    
    def log_custody_transfer(self, case_number: str, evidence_id: Optional[str], 
                            transfer_type: str, from_custodian: str, to_custodian: str,
                            transfer_reason: str) -> bool:
        """Log custody transfer (append-only)."""
        if not self.conn:
            return False
        
        cursor = self.conn.cursor()
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Get case ID
            cursor.execute("SELECT case_id FROM forensic_cases WHERE case_number = %s", (case_number,))
            case_row = cursor.fetchone()
            if not case_row:
                logger.error(f"✗ Case not found: {case_number}")
                return False
            
            case_id = case_row[0]
            
            # Compute transfer hash
            transfer_data = {
                'case_id': str(case_id),
                'evidence_id': evidence_id,
                'transfer_type': transfer_type,
                'from_custodian': from_custodian,
                'to_custodian': to_custodian,
                'transfer_reason': transfer_reason,
                'timestamp': datetime.now(timezone.utc).isoformat()
            }
            transfer_json = json.dumps(transfer_data, sort_keys=True)
            transfer_hash = hashlib.sha256(transfer_json.encode()).digest()
            
            # Insert transfer log
            cursor.execute("""
                INSERT INTO custody_transfer_log (
                    case_id, evidence_id, transfer_type,
                    from_custodian, to_custodian, transfer_reason, transfer_hash
                )
                VALUES (%s, %s, %s, %s, %s, %s, %s)
            """, (
                case_id,
                uuid.UUID(evidence_id) if evidence_id else None,
                transfer_type,
                from_custodian,
                to_custodian,
                transfer_reason,
                transfer_hash
            ))
            
            self.conn.commit()
            logger.info(f"✓ Custody transfer logged: {transfer_type}")
            return True
        except Exception as e:
            logger.error(f"✗ Failed to log custody transfer: {e}")
            self.conn.rollback()
            return False
        finally:
            cursor.close()
    
    def export_case_bundle(self, case_number: str, output_path: Path) -> bool:
        """Export read-only case bundle."""
        if not self.conn:
            return False
        
        cursor = self.conn.cursor()
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Get case
            cursor.execute("""
                SELECT case_id, case_name, created_at, created_by, status, description
                FROM forensic_cases
                WHERE case_number = %s
            """, (case_number,))
            case_row = cursor.fetchone()
            if not case_row:
                logger.error(f"✗ Case not found: {case_number}")
                return False
            
            case_id = case_row[0]
            case_data = {
                'case_number': case_number,
                'case_name': case_row[1],
                'created_at': case_row[2].isoformat() if case_row[2] else None,
                'created_by': case_row[3],
                'status': case_row[4],
                'description': case_row[5]
            }
            
            # Get evidence
            cursor.execute("""
                SELECT evidence_id, evidence_type, evidence_hash, evidence_data, sealed_at, sealed_by
                FROM forensic_evidence
                WHERE case_id = %s
                ORDER BY sealed_at
            """, (case_id,))
            
            evidence_list = []
            for row in cursor.fetchall():
                evidence_list.append({
                    'evidence_id': str(row[0]),
                    'evidence_type': row[1],
                    'evidence_hash': row[2].hex() if row[2] else None,
                    'evidence_data': row[3],
                    'sealed_at': row[4].isoformat() if row[4] else None,
                    'sealed_by': row[5]
                })
            
            # Get custody transfers
            cursor.execute("""
                SELECT transfer_id, evidence_id, transfer_type, from_custodian, to_custodian,
                       transfer_reason, transfer_hash, created_at
                FROM custody_transfer_log
                WHERE case_id = %s
                ORDER BY created_at
            """, (case_id,))
            
            transfers = []
            for row in cursor.fetchall():
                transfers.append({
                    'transfer_id': str(row[0]),
                    'evidence_id': str(row[1]) if row[1] else None,
                    'transfer_type': row[2],
                    'from_custodian': row[3],
                    'to_custodian': row[4],
                    'transfer_reason': row[5],
                    'transfer_hash': row[6].hex() if row[6] else None,
                    'created_at': row[7].isoformat() if row[7] else None
                })
            
            # Create bundle
            bundle = {
                'case': case_data,
                'evidence': evidence_list,
                'custody_transfers': transfers,
                'exported_at': datetime.now(timezone.utc).isoformat(),
                'read_only': True
            }
            
            # Compute bundle hash
            bundle_json = json.dumps(bundle, sort_keys=True)
            bundle_hash = hashlib.sha256(bundle_json.encode()).hexdigest()
            bundle['bundle_hash'] = bundle_hash
            
            # Save bundle
            with tempfile.TemporaryDirectory() as tmpdir:
                tmp_path = Path(tmpdir)
                bundle_file = tmp_path / "case_bundle.json"
                with open(bundle_file, 'w') as f:
                    json.dump(bundle, f, indent=2)
                
                # Create archive (read-only)
                with tarfile.open(output_path, 'w:gz') as tar:
                    tar.add(bundle_file, arcname='case_bundle.json')
                
                # Make archive read-only
                output_path.chmod(0o444)
            
            logger.info(f"✓ Case bundle exported: {output_path}")
            logger.info(f"  Bundle hash: {bundle_hash[:16]}...")
            return True
        except Exception as e:
            logger.error(f"✗ Failed to export case bundle: {e}")
            return False
        finally:
            cursor.close()
    
    def run(self, case_name: str, created_by: str, output_path: Optional[Path] = None) -> bool:
        """Run chain of custody setup."""
        logger.info("=" * 80)
        logger.info("Forensic Chain of Custody (PROMPT-62 Phase 3)")
        logger.info("=" * 80)
        
        # Connect to database
        if not self.connect_db():
            logger.error("FAIL-CLOSED: Database connection failed")
            return False
        
        # Ensure tables exist
        if not self.ensure_custody_tables():
            logger.error("FAIL-CLOSED: Failed to create custody tables")
            return False
        
        # Create case
        case_number = self.create_case(case_name, created_by)
        if not case_number:
            logger.error("FAIL-CLOSED: Failed to create case")
            return False
        
        logger.info(f"✓ Case created: {case_number}")
        
        # Export bundle if output path specified
        if output_path:
            if not self.export_case_bundle(case_number, output_path):
                logger.error("FAIL-CLOSED: Failed to export case bundle")
                return False
        
        return True


def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description='Forensic Chain of Custody')
    parser.add_argument('--case-name', required=True, help='Case name')
    parser.add_argument('--created-by', required=True, help='Created by (custodian)')
    parser.add_argument('--output', type=Path, help='Output path for case bundle')
    
    args = parser.parse_args()
    
    custody = ChainOfCustody()
    success = custody.run(args.case_name, args.created_by, args.output)
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()

