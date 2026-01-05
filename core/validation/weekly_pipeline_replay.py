# Path and File Name : /home/ransomeye/rebuild/core/validation/weekly_pipeline_replay.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Weekly Pipeline Sanity Replay - Synthetic end-to-end pipeline validation

"""
RansomEye Weekly Pipeline Sanity Replay (PROMPT-60-A)

Synthetic end-to-end pipeline validation:
- Generate synthetic events
- Replay through full pipeline
- Verify all stages complete
- Validate audit trail integrity
- Check export capabilities
"""

import os
import sys
import json
import psycopg2
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict

# Configuration from environment
DB_NAME = os.environ.get("DB_NAME", "ransomeye")
DB_USER = os.environ.get("DB_USER", "gagan")
DB_HOST = os.environ.get("DB_HOST", "localhost")
DB_PASSWORD = os.environ.get("DB_PASS", "gagan")
DB_PORT = int(os.environ.get("DB_PORT", "5432"))

PIPELINE_REPLAY_DIR = Path("/var/lib/ransomeye/pipeline_replays")


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


def generate_synthetic_event() -> Dict:
    """Generate synthetic event for pipeline replay."""
    return {
        "event_id": str(uuid.uuid4()),
        "source_type": "synthetic_replay",
        "observed_at": datetime.now(timezone.utc).isoformat(),
        "payload_json": {
            "test": True,
            "replay_id": f"weekly_{datetime.now(timezone.utc).strftime('%Y%m%d')}",
            "event_type": "SYNTHETIC_PIPELINE_REPLAY"
        }
    }


def inject_synthetic_event(conn) -> bool:
    """Inject synthetic event into raw_events table."""
    if not conn:
        return False
    
    try:
        event = generate_synthetic_event()
        cursor = conn.cursor()
        
        cursor.execute("""
            INSERT INTO ransomeye.raw_events (
                event_id, source_type, observed_at, payload_json
            )
            VALUES (%s, %s, %s, %s)
        """, (
            event["event_id"],
            event["source_type"],
            event["observed_at"],
            json.dumps(event["payload_json"])
        ))
        
        conn.commit()
        cursor.close()
        return True
    except Exception as e:
        print(f"ERROR: Failed to inject synthetic event: {e}")
        return False


def verify_pipeline_stages(conn, event_id: str) -> Dict:
    """Verify event progressed through all pipeline stages."""
    verification = {
        "raw_event": False,
        "normalized_event": False,
        "audit_log": False,
        "export_capability": False
    }
    
    if not conn:
        return verification
    
    try:
        cursor = conn.cursor()
        
        # Check raw event
        cursor.execute("""
            SELECT COUNT(*) FROM ransomeye.raw_events WHERE event_id = %s
        """, (event_id,))
        if cursor.fetchone()[0] > 0:
            verification["raw_event"] = True
        
        # Check normalized event (may not exist immediately)
        cursor.execute("""
            SELECT COUNT(*) FROM ransomeye.normalized_events WHERE event_id = %s
        """, (event_id,))
        if cursor.fetchone()[0] > 0:
            verification["normalized_event"] = True
        
        # Check audit log
        cursor.execute("""
            SELECT COUNT(*) FROM ransomeye.immutable_audit_log
            WHERE payload_json::text LIKE %s
        """, (f"%{event_id}%",))
        if cursor.fetchone()[0] > 0:
            verification["audit_log"] = True
        
        cursor.close()
        
        # Check export capability (file system check)
        export_dirs = [
            Path("/var/lib/ransomeye/reports"),
            Path("/home/ransomeye/rebuild/logs"),
        ]
        
        for export_dir in export_dirs:
            if export_dir.exists():
                for ext in ["csv", "html", "pdf"]:
                    if list(export_dir.rglob(f"*.{ext}")):
                        verification["export_capability"] = True
                        break
        
        return verification
    except Exception as e:
        print(f"ERROR: Pipeline verification failed: {e}")
        return verification


def main():
    """Main pipeline replay function."""
    print("RansomEye Weekly Pipeline Sanity Replay (PROMPT-60-A)")
    print("=" * 60)
    
    conn = get_db_connection()
    if not conn:
        print("ERROR: Database connection failed")
        return 1
    
    # Generate and inject synthetic event
    print("Generating synthetic event...")
    event = generate_synthetic_event()
    event_id = event["event_id"]
    
    print(f"Injecting synthetic event: {event_id}")
    if not inject_synthetic_event(conn):
        print("ERROR: Failed to inject synthetic event")
        conn.close()
        return 1
    
    # Wait a moment for pipeline processing
    import time
    time.sleep(2)
    
    # Verify pipeline stages
    print("Verifying pipeline stages...")
    verification = verify_pipeline_stages(conn, event_id)
    
    # Generate replay report
    PIPELINE_REPLAY_DIR.mkdir(parents=True, exist_ok=True)
    replay_file = PIPELINE_REPLAY_DIR / f"weekly_{datetime.now(timezone.utc).strftime('%Y%m%d')}.json"
    
    replay_report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "event_id": event_id,
        "verification": verification,
        "all_stages_passed": all(verification.values())
    }
    
    with open(replay_file, "w") as f:
        json.dump(replay_report, f, indent=2, sort_keys=True)
    
    print(f"Replay report written to {replay_file}")
    
    # Check results
    if replay_report["all_stages_passed"]:
        print("SUCCESS: All pipeline stages verified")
        conn.close()
        return 0
    else:
        print("WARNING: Some pipeline stages not verified")
        for stage, passed in verification.items():
            status = "✓" if passed else "✗"
            print(f"  {status} {stage}")
        conn.close()
        return 1


if __name__ == "__main__":
    sys.exit(main())

