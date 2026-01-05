# Path and File Name : /home/ransomeye/rebuild/core/baseline/daily_integrity_diff.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Daily Integrity Snapshot Diff - Compares current state against golden baseline

"""
RansomEye Daily Integrity Snapshot Diff (PROMPT-60-A)

Compares current system state against golden baseline daily.
Generates diff report and alerts on any drift detected.
"""

import os
import sys
import json
import subprocess
import psycopg2
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List

# Configuration from environment
DB_NAME = os.environ.get("DB_NAME", "ransomeye")
DB_USER = os.environ.get("DB_USER", "gagan")
DB_HOST = os.environ.get("DB_HOST", "localhost")
DB_PASSWORD = os.environ.get("DB_PASS", "gagan")
DB_PORT = int(os.environ.get("DB_PORT", "5432"))

GOLDEN_BASELINE_PATH = Path("/var/lib/ransomeye/baselines/golden_baseline.json")
INTEGRITY_DIFF_DIR = Path("/var/lib/ransomeye/integrity_diffs")
SYSTEMD_DIR = Path("/etc/systemd/system")


def get_db_connection():
    """Get database connection."""
    try:
        return psycopg2.connect(
            host=DB_HOST,
            port=DB_PORT,
            database=DB_NAME,
            user=DB_USER,
            password=DB_PASSWORD
        )
    except Exception:
        return None


def capture_current_snapshot() -> Dict:
    """Capture current system state snapshot."""
    snapshot = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "systemd_units": {},
        "db_schema": {}
    }
    
    # Capture systemd unit hashes
    if SYSTEMD_DIR.exists():
        for unit_file in SYSTEMD_DIR.glob("ransomeye*.service"):
            try:
                import hashlib
                with open(unit_file, "rb") as f:
                    content = f.read()
                    unit_hash = hashlib.sha256(content).hexdigest()
                    snapshot["systemd_units"][unit_file.name] = {
                        "sha256": unit_hash,
                        "mtime": unit_file.stat().st_mtime
                    }
            except Exception:
                pass
    
    # Capture DB schema checksum
    conn = get_db_connection()
    if conn:
        try:
            cursor = conn.cursor()
            cursor.execute("""
                SELECT COUNT(*) 
                FROM information_schema.tables 
                WHERE table_schema = 'ransomeye'
            """)
            table_count = cursor.fetchone()[0]
            snapshot["db_schema"]["table_count"] = table_count
            cursor.close()
        except Exception:
            pass
        conn.close()
    
    return snapshot


def load_golden_baseline() -> Dict:
    """Load golden baseline."""
    if not GOLDEN_BASELINE_PATH.exists():
        return {}
    
    try:
        with open(GOLDEN_BASELINE_PATH, "r") as f:
            return json.load(f)
    except Exception:
        return {}


def compare_snapshots(current: Dict, baseline: Dict) -> Dict:
    """Compare current snapshot against baseline."""
    diff = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "drift_detected": False,
        "drifts": []
    }
    
    # Compare systemd units
    baseline_units = baseline.get("systemd_units", {})
    current_units = current.get("systemd_units", {})
    
    for unit_name, baseline_data in baseline_units.items():
        if unit_name not in current_units:
            diff["drift_detected"] = True
            diff["drifts"].append(f"Missing systemd unit: {unit_name}")
        else:
            current_data = current_units[unit_name]
            if baseline_data.get("sha256") != current_data.get("sha256"):
                diff["drift_detected"] = True
                diff["drifts"].append(f"Modified systemd unit: {unit_name}")
    
    # Check for new units
    for unit_name in current_units:
        if unit_name not in baseline_units:
            diff["drift_detected"] = True
            diff["drifts"].append(f"New systemd unit: {unit_name}")
    
    # Compare DB schema
    baseline_table_count = baseline.get("db_schema", {}).get("table_count", 0)
    current_table_count = current.get("db_schema", {}).get("table_count", 0)
    
    if baseline_table_count > 0 and current_table_count != baseline_table_count:
        diff["drift_detected"] = True
        diff["drifts"].append(
            f"DB schema changed: table count {baseline_table_count} -> {current_table_count}"
        )
    
    return diff


def write_integrity_violation_audit(conn, drift_details: List):
    """Write integrity violation audit entry for drift."""
    if not conn:
        return
    
    try:
        cursor = conn.cursor()
        import hashlib
        import uuid
        
        payload_json = {
            "violation_type": "DRIFT_DETECTED",
            "message": f"Daily integrity diff detected {len(drift_details)} drifts",
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "drifts": drift_details
        }
        payload_str = json.dumps(payload_json, sort_keys=True)
        payload_sha256 = hashlib.sha256(payload_str.encode()).digest()
        
        # Get previous audit entry
        cursor.execute("""
            SELECT audit_id, chain_hash_sha256, payload_sha256
            FROM ransomeye.immutable_audit_log
            ORDER BY created_at DESC
            LIMIT 1
        """)
        prev_row = cursor.fetchone()
        
        if prev_row:
            prev_chain_hash = prev_row[1] if prev_row[1] else bytes(32)
        else:
            prev_chain_hash = bytes(32)
        
        chain_input = prev_chain_hash + payload_sha256
        chain_hash_sha256 = hashlib.sha256(chain_input).digest()
        
        audit_id = str(uuid.uuid4())
        cursor.execute("""
            INSERT INTO ransomeye.immutable_audit_log (
                audit_id, action, object_type, payload_json, payload_sha256,
                prev_payload_sha256, chain_hash_sha256, signature_status
            )
            VALUES (%s, %s, %s, %s, %s, %s, %s, 'unknown')
        """, (
            audit_id,
            "DRIFT_DETECTED",
            "system",
            payload_str,
            payload_sha256,
            prev_row[2] if prev_row else None,
            chain_hash_sha256
        ))
        
        conn.commit()
        cursor.close()
    except Exception:
        pass


def main():
    """Main integrity diff function."""
    print("RansomEye Daily Integrity Snapshot Diff (PROMPT-60-A)")
    print("=" * 60)
    
    # Load golden baseline
    print("Loading golden baseline...")
    baseline = load_golden_baseline()
    
    if not baseline:
        print("ERROR: Golden baseline not found")
        return 1
    
    # Capture current snapshot
    print("Capturing current snapshot...")
    current = capture_current_snapshot()
    
    # Compare snapshots
    print("Comparing snapshots...")
    diff = compare_snapshots(current, baseline)
    
    # Save diff report
    INTEGRITY_DIFF_DIR.mkdir(parents=True, exist_ok=True)
    diff_file = INTEGRITY_DIFF_DIR / f"daily_{datetime.now(timezone.utc).strftime('%Y%m%d')}.json"
    
    diff_report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "baseline_version": baseline.get("version", "unknown"),
        "baseline_timestamp": baseline.get("capture_timestamp", "unknown"),
        "current_snapshot": current,
        "diff": diff
    }
    
    with open(diff_file, "w") as f:
        json.dump(diff_report, f, indent=2, sort_keys=True)
    
    print(f"Diff report written to {diff_file}")
    
    # Handle drift
    if diff["drift_detected"]:
        print(f"WARNING: Drift detected - {len(diff['drifts'])} changes")
        for drift in diff["drifts"]:
            print(f"  - {drift}")
        
        # Write audit entry
        conn = get_db_connection()
        write_integrity_violation_audit(conn, diff["drifts"])
        if conn:
            conn.close()
        
        return 1
    else:
        print("No drift detected - system integrity maintained")
        return 0


if __name__ == "__main__":
    sys.exit(main())

