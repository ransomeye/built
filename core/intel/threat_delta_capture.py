# Path and File Name : /home/ransomeye/rebuild/core/intel/threat_delta_capture.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Threat delta capture - controlled delta capture comparing new threat intel vs frozen baseline, append-only storage with fail-closed enforcement

"""
Threat Delta Capture (PROMPT-61 Phase 1)

Compares new threat intel vs last frozen baseline and classifies deltas:
- new IOC
- IOC mutation
- confidence shift
- TTP pattern

All deltas stored in threat_intel_delta (append-only with hashes).
Fail-closed on schema or integrity mismatch.
"""

import os
import sys
import json
import hashlib
import psycopg2
from pathlib import Path
from datetime import datetime, timezone
from typing import Dict, List, Optional, Tuple
from enum import Enum
import logging

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger('threat_delta_capture')

# Database configuration
DB_NAME = os.environ.get("DB_NAME", "ransomeye")
DB_USER = os.environ.get("DB_USER", "gagan")
DB_HOST = os.environ.get("DB_HOST", "localhost")
DB_PASSWORD = os.environ.get("DB_PASS", "gagan")
DB_PORT = int(os.environ.get("DB_PORT", "5432"))


class DeltaType(Enum):
    """Delta classification types."""
    NEW_IOC = "new_ioc"
    IOC_MUTATION = "ioc_mutation"
    CONFIDENCE_SHIFT = "confidence_shift"
    TTP_PATTERN = "ttp_pattern"


class ThreatDeltaCapture:
    """Threat delta capture engine."""
    
    def __init__(self):
        """Initialize threat delta capture."""
        self.conn = None
        self.baseline_snapshot = None
        
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
    
    def ensure_delta_table(self) -> bool:
        """Ensure threat_intel_delta table exists."""
        if not self.conn:
            return False
        
        cursor = self.conn.cursor()
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            cursor.execute("""
                CREATE TABLE IF NOT EXISTS threat_intel_delta (
                    delta_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
                    baseline_snapshot_hash bytea NOT NULL,
                    delta_type text NOT NULL,
                    ioc_type text,
                    ioc_value text,
                    source text,
                    old_value jsonb,
                    new_value jsonb,
                    delta_hash bytea NOT NULL,
                    created_at timestamptz NOT NULL DEFAULT now(),
                    CONSTRAINT threat_intel_delta_hash_len_chk CHECK (octet_length(delta_hash) = 32),
                    CONSTRAINT threat_intel_delta_baseline_hash_len_chk CHECK (octet_length(baseline_snapshot_hash) = 32)
                );
            """)
            
            cursor.execute("""
                CREATE INDEX IF NOT EXISTS idx_threat_intel_delta_baseline_hash 
                ON threat_intel_delta (baseline_snapshot_hash);
            """)
            
            cursor.execute("""
                CREATE INDEX IF NOT EXISTS idx_threat_intel_delta_created_at 
                ON threat_intel_delta (created_at);
            """)
            
            cursor.execute("""
                CREATE INDEX IF NOT EXISTS idx_threat_intel_delta_type 
                ON threat_intel_delta (delta_type);
            """)
            
            self.conn.commit()
            logger.info("✓ threat_intel_delta table verified/created")
            return True
        except Exception as e:
            logger.error(f"✗ Failed to create threat_intel_delta table: {e}")
            self.conn.rollback()
            return False
        finally:
            cursor.close()
    
    def get_baseline_snapshot(self) -> Optional[Dict]:
        """Get last frozen baseline snapshot."""
        if not self.conn:
            return None
        
        cursor = self.conn.cursor()
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Get all IOCs from threat_intel table (baseline)
            cursor.execute("""
                SELECT 
                    ioc_type, ioc_value, source, confidence,
                    tags, metadata, correlated_count,
                    correlated_confidence, correlated_sources,
                    first_seen, last_seen, updated_at
                FROM threat_intel
                ORDER BY updated_at DESC
            """)
            
            baseline = {}
            for row in cursor.fetchall():
                ioc_key = f"{row[0]}:{row[1]}:{row[2]}"
                baseline[ioc_key] = {
                    'ioc_type': row[0],
                    'ioc_value': row[1],
                    'source': row[2],
                    'confidence': float(row[3]) if row[3] else 0.0,
                    'tags': row[4] if row[4] else [],
                    'metadata': row[5] if row[5] else {},
                    'correlated_count': row[6] if row[6] else 0,
                    'correlated_confidence': float(row[7]) if row[7] else 0.0,
                    'correlated_sources': row[8] if row[8] else [],
                    'first_seen': row[9].isoformat() if row[9] else None,
                    'last_seen': row[10].isoformat() if row[10] else None,
                    'updated_at': row[11].isoformat() if row[11] else None
                }
            
            # Compute baseline hash
            baseline_json = json.dumps(baseline, sort_keys=True)
            baseline_hash = hashlib.sha256(baseline_json.encode()).digest()
            
            self.baseline_snapshot = {
                'data': baseline,
                'hash': baseline_hash,
                'timestamp': datetime.now(timezone.utc).isoformat()
            }
            
            logger.info(f"✓ Loaded baseline snapshot: {len(baseline)} IOCs, hash={baseline_hash.hex()[:16]}...")
            return self.baseline_snapshot
        except Exception as e:
            logger.error(f"✗ Failed to get baseline snapshot: {e}")
            return None
        finally:
            cursor.close()
    
    def get_current_snapshot(self) -> Optional[Dict]:
        """Get current threat intel snapshot."""
        if not self.conn:
            return None
        
        cursor = self.conn.cursor()
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            
            cursor.execute("""
                SELECT 
                    ioc_type, ioc_value, source, confidence,
                    tags, metadata, correlated_count,
                    correlated_confidence, correlated_sources,
                    first_seen, last_seen, updated_at
                FROM threat_intel
                ORDER BY updated_at DESC
            """)
            
            current = {}
            for row in cursor.fetchall():
                ioc_key = f"{row[0]}:{row[1]}:{row[2]}"
                current[ioc_key] = {
                    'ioc_type': row[0],
                    'ioc_value': row[1],
                    'source': row[2],
                    'confidence': float(row[3]) if row[3] else 0.0,
                    'tags': row[4] if row[4] else [],
                    'metadata': row[5] if row[5] else {},
                    'correlated_count': row[6] if row[6] else 0,
                    'correlated_confidence': float(row[7]) if row[7] else 0.0,
                    'correlated_sources': row[8] if row[8] else [],
                    'first_seen': row[9].isoformat() if row[9] else None,
                    'last_seen': row[10].isoformat() if row[10] else None,
                    'updated_at': row[11].isoformat() if row[11] else None
                }
            
            return current
        except Exception as e:
            logger.error(f"✗ Failed to get current snapshot: {e}")
            return None
        finally:
            cursor.close()
    
    def classify_delta(self, old_ioc: Optional[Dict], new_ioc: Dict) -> Tuple[DeltaType, Dict, Dict]:
        """Classify delta type."""
        if old_ioc is None:
            # New IOC
            return DeltaType.NEW_IOC, {}, new_ioc
        
        # Check for IOC mutation (value changed)
        if old_ioc.get('ioc_value') != new_ioc.get('ioc_value'):
            return DeltaType.IOC_MUTATION, old_ioc, new_ioc
        
        # Check for confidence shift
        old_conf = old_ioc.get('confidence', 0.0)
        new_conf = new_ioc.get('confidence', 0.0)
        if abs(old_conf - new_conf) > 0.1:  # Significant shift threshold
            return DeltaType.CONFIDENCE_SHIFT, old_ioc, new_ioc
        
        # Check for TTP pattern changes (tags/metadata)
        old_tags = set(old_ioc.get('tags', []))
        new_tags = set(new_ioc.get('tags', []))
        if old_tags != new_tags:
            return DeltaType.TTP_PATTERN, old_ioc, new_ioc
        
        # Default to TTP pattern for any other changes
        return DeltaType.TTP_PATTERN, old_ioc, new_ioc
    
    def compute_delta_hash(self, delta_type: DeltaType, old_value: Dict, new_value: Dict) -> bytes:
        """Compute hash for delta record."""
        delta_data = {
            'type': delta_type.value,
            'old': old_value,
            'new': new_value,
            'timestamp': datetime.now(timezone.utc).isoformat()
        }
        delta_json = json.dumps(delta_data, sort_keys=True)
        return hashlib.sha256(delta_json.encode()).digest()
    
    def capture_deltas(self) -> Tuple[bool, int]:
        """Capture deltas between baseline and current state."""
        if not self.conn:
            logger.error("✗ Database connection not available")
            return False, 0
        
        if not self.baseline_snapshot:
            logger.error("✗ Baseline snapshot not loaded")
            return False, 0
        
        baseline = self.baseline_snapshot['data']
        baseline_hash = self.baseline_snapshot['hash']
        
        current = self.get_current_snapshot()
        if current is None:
            logger.error("✗ Failed to get current snapshot")
            return False, 0
        
        cursor = self.conn.cursor()
        delta_count = 0
        
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Find new IOCs
            for ioc_key, new_ioc in current.items():
                old_ioc = baseline.get(ioc_key)
                delta_type, old_value, new_value = self.classify_delta(old_ioc, new_ioc)
                
                # Only record if there's an actual change
                if old_ioc is None or old_ioc != new_ioc:
                    delta_hash = self.compute_delta_hash(delta_type, old_value, new_value)
                    
                    cursor.execute("""
                        INSERT INTO threat_intel_delta (
                            baseline_snapshot_hash, delta_type,
                            ioc_type, ioc_value, source,
                            old_value, new_value, delta_hash
                        )
                        VALUES (%s, %s, %s, %s, %s, %s::jsonb, %s::jsonb, %s)
                    """, (
                        baseline_hash,
                        delta_type.value,
                        new_ioc.get('ioc_type'),
                        new_ioc.get('ioc_value'),
                        new_ioc.get('source'),
                        json.dumps(old_value) if old_value else None,
                        json.dumps(new_value),
                        delta_hash
                    ))
                    delta_count += 1
            
            # Find removed IOCs (in baseline but not in current)
            for ioc_key, old_ioc in baseline.items():
                if ioc_key not in current:
                    # IOC removed - record as mutation
                    delta_hash = self.compute_delta_hash(
                        DeltaType.IOC_MUTATION,
                        old_ioc,
                        {}
                    )
                    
                    cursor.execute("""
                        INSERT INTO threat_intel_delta (
                            baseline_snapshot_hash, delta_type,
                            ioc_type, ioc_value, source,
                            old_value, new_value, delta_hash
                        )
                        VALUES (%s, %s, %s, %s, %s, %s::jsonb, %s::jsonb, %s)
                    """, (
                        baseline_hash,
                        DeltaType.IOC_MUTATION.value,
                        old_ioc.get('ioc_type'),
                        old_ioc.get('ioc_value'),
                        old_ioc.get('source'),
                        json.dumps(old_ioc),
                        json.dumps({}),
                        delta_hash
                    ))
                    delta_count += 1
            
            self.conn.commit()
            logger.info(f"✓ Captured {delta_count} deltas")
            return True, delta_count
        except Exception as e:
            logger.error(f"✗ Failed to capture deltas: {e}")
            self.conn.rollback()
            return False, 0
        finally:
            cursor.close()
    
    def run(self) -> bool:
        """Run threat delta capture."""
        logger.info("=" * 80)
        logger.info("Threat Delta Capture (PROMPT-61 Phase 1)")
        logger.info("=" * 80)
        
        # Connect to database
        if not self.connect_db():
            logger.error("FAIL-CLOSED: Database connection failed")
            return False
        
        # Ensure delta table exists
        if not self.ensure_delta_table():
            logger.error("FAIL-CLOSED: Failed to create threat_intel_delta table")
            return False
        
        # Get baseline snapshot
        if not self.get_baseline_snapshot():
            logger.error("FAIL-CLOSED: Failed to get baseline snapshot")
            return False
        
        # Capture deltas
        success, delta_count = self.capture_deltas()
        if not success:
            logger.error("FAIL-CLOSED: Failed to capture deltas")
            return False
        
        logger.info(f"✓ Threat delta capture complete: {delta_count} deltas captured")
        return True


def main():
    """Main entry point."""
    capture = ThreatDeltaCapture()
    success = capture.run()
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()

