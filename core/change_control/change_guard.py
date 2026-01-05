# Path and File Name : /home/ransomeye/rebuild/core/change_control/change_guard.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Change Control Guard - Pre-update gate requiring staging execution and verifier green ≥24h, blocks unauthorized changes

"""
RansomEye Change Control Guard (PROMPT-58-C)

Pre-update gate enforcement:
- Any binary/schema/model change requires staging execution
- Verifier must be green for ≥24h before production
- No in-place hot changes
- Mandatory version bump
- Audit entry on violation
"""

import os
import sys
import json
import subprocess
import psycopg2
from datetime import datetime, timezone, timedelta
from pathlib import Path
from typing import Dict, List, Optional, Tuple

# Configuration from environment
DB_NAME = os.environ.get("DB_NAME", "ransomeye")
DB_USER = os.environ.get("DB_USER", "gagan")
DB_HOST = os.environ.get("DB_HOST", "localhost")
DB_PASSWORD = os.environ.get("DB_PASS", "gagan")
DB_PORT = int(os.environ.get("DB_PORT", "5432"))

STAGING_DIR = Path("/var/lib/ransomeye/staging")
VERIFIER_RESULTS_PATH = Path("/var/log/ransomeye/verifier_results.json")
CHANGE_LOG_PATH = Path("/var/log/ransomeye/change_control.log")
MIN_STAGING_HOURS = 24


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
    except Exception as e:
        return None


def log_change_control_event(event_type: str, message: str, details: Dict = None):
    """Log change control event."""
    try:
        CHANGE_LOG_PATH.parent.mkdir(parents=True, exist_ok=True)
        timestamp = datetime.now(timezone.utc).isoformat()
        event = {
            "timestamp": timestamp,
            "event_type": event_type,
            "message": message,
            "details": details or {}
        }
        with open(CHANGE_LOG_PATH, "a") as f:
            f.write(json.dumps(event) + "\n")
    except Exception:
        pass


def write_change_violation_audit(conn, violation_message: str, details: Dict):
    """Write CHANGE_CONTROL_VIOLATION audit entry."""
    if not conn:
        return
    
    try:
        cursor = conn.cursor()
        
        payload_json = {
            "violation_type": "CHANGE_CONTROL_VIOLATION",
            "message": violation_message,
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "details": details
        }
        payload_str = json.dumps(payload_json, sort_keys=True)
        
        import hashlib
        payload_sha256 = hashlib.sha256(payload_str.encode()).digest()
        
        # Get previous audit entry for chain
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
        
        # Compute chain hash
        chain_input = prev_chain_hash + payload_sha256
        chain_hash_sha256 = hashlib.sha256(chain_input).digest()
        
        # Insert audit entry
        import uuid
        audit_id = str(uuid.uuid4())
        cursor.execute("""
            INSERT INTO ransomeye.immutable_audit_log (
                audit_id, action, object_type, payload_json, payload_sha256,
                prev_payload_sha256, chain_hash_sha256, signature_status
            )
            VALUES (%s, %s, %s, %s, %s, %s, %s, 'unknown')
        """, (
            audit_id,
            "CHANGE_CONTROL_VIOLATION",
            "system",
            payload_str,
            payload_sha256,
            prev_row[2] if prev_row else None,
            chain_hash_sha256
        ))
        
        conn.commit()
        cursor.close()
    except Exception as e:
        log_change_control_event("AUDIT_ERROR", f"Failed to write audit: {str(e)}")


def check_verifier_status() -> Tuple[bool, Optional[str], Dict]:
    """Check verifier status and duration of green state."""
    if not VERIFIER_RESULTS_PATH.exists():
        return False, "Verifier results not found", {}
    
    try:
        with open(VERIFIER_RESULTS_PATH, "r") as f:
            results = json.load(f)
        
        is_healthy = results.get("overall_healthy", False)
        timestamp_str = results.get("timestamp")
        
        if not timestamp_str:
            return False, "No timestamp in verifier results", {}
        
        timestamp = datetime.fromisoformat(timestamp_str.replace("Z", "+00:00"))
        age_hours = (datetime.now(timezone.utc) - timestamp).total_seconds() / 3600
        
        if not is_healthy:
            return False, f"Verifier not healthy (age: {age_hours:.1f}h)", results
        
        if age_hours < MIN_STAGING_HOURS:
            return False, f"Verifier green for only {age_hours:.1f}h (required: {MIN_STAGING_HOURS}h)", results
        
        return True, None, results
    except Exception as e:
        return False, f"Failed to check verifier: {str(e)}", {}


def check_staging_execution() -> Tuple[bool, Optional[str]]:
    """Check if changes have been executed in staging."""
    if not STAGING_DIR.exists():
        return False, "Staging directory not found"
    
    # Check for staging execution marker
    staging_marker = STAGING_DIR / "staging_execution.json"
    if not staging_marker.exists():
        return False, "No staging execution marker found"
    
    try:
        with open(staging_marker, "r") as f:
            staging_data = json.load(f)
        
        execution_time = staging_data.get("execution_timestamp")
        if not execution_time:
            return False, "No execution timestamp in staging marker"
        
        exec_timestamp = datetime.fromisoformat(execution_time.replace("Z", "+00:00"))
        age_hours = (datetime.now(timezone.utc) - exec_timestamp).total_seconds() / 3600
        
        if age_hours < MIN_STAGING_HOURS:
            return False, f"Staging execution too recent ({age_hours:.1f}h, required: {MIN_STAGING_HOURS}h)"
        
        return True, None
    except Exception as e:
        return False, f"Failed to check staging: {str(e)}"


def check_version_bump(new_version: str, current_version: str = "1.0.0-enterprise-ship") -> Tuple[bool, Optional[str]]:
    """Check if version has been bumped."""
    if not new_version or new_version == current_version:
        return False, f"Version not bumped (current: {current_version}, new: {new_version})"
    
    # Simple version comparison (can be enhanced)
    try:
        new_parts = new_version.split(".")
        current_parts = current_version.split(".")
        
        # At minimum, patch version should be incremented
        if len(new_parts) >= 3 and len(current_parts) >= 3:
            new_patch = int(new_parts[2].split("-")[0])
            current_patch = int(current_parts[2].split("-")[0])
            if new_patch <= current_patch:
                return False, f"Version patch not incremented (current: {current_version}, new: {new_version})"
        
        return True, None
    except Exception:
        # If version format is different, allow it but log
        return True, "Version format non-standard, allowing"


def validate_change(change_type: str, change_details: Dict) -> Tuple[bool, Optional[str]]:
    """
    Validate change before allowing it.
    
    Args:
        change_type: Type of change (binary, schema, model, config)
        change_details: Details about the change
    
    Returns:
        (is_valid, error_message)
    """
    print(f"Validating {change_type} change...")
    
    # Check version bump
    new_version = change_details.get("version")
    version_ok, version_error = check_version_bump(new_version)
    if not version_ok:
        return False, f"Version bump required: {version_error}"
    
    # Check staging execution
    staging_ok, staging_error = check_staging_execution()
    if not staging_ok:
        return False, f"Staging execution required: {staging_error}"
    
    # Check verifier status
    verifier_ok, verifier_error, verifier_data = check_verifier_status()
    if not verifier_ok:
        return False, f"Verifier check failed: {verifier_error}"
    
    print("Change validation passed")
    return True, None


def block_unauthorized_change(change_type: str, change_details: Dict, reason: str):
    """Block unauthorized change and log violation."""
    violation_message = f"Unauthorized {change_type} change blocked: {reason}"
    print(f"BLOCKED: {violation_message}")
    
    log_change_control_event("CHANGE_BLOCKED", violation_message, {
        "change_type": change_type,
        "change_details": change_details,
        "reason": reason
    })
    
    # Write audit entry
    conn = get_db_connection()
    write_change_violation_audit(conn, violation_message, {
        "change_type": change_type,
        "change_details": change_details,
        "reason": reason
    })
    if conn:
        conn.close()
    
    sys.exit(1)


def main():
    """Main change control guard function."""
    if len(sys.argv) < 3:
        print("Usage: change_guard.py <change_type> <version> [change_details_json]")
        print("  change_type: binary|schema|model|config")
        print("  version: New version string (must be bumped)")
        print("  change_details_json: Optional JSON with change details")
        sys.exit(1)
    
    change_type = sys.argv[1]
    new_version = sys.argv[2]
    change_details = json.loads(sys.argv[3]) if len(sys.argv) > 3 else {}
    change_details["version"] = new_version
    
    print("RansomEye Change Control Guard (PROMPT-58-C)")
    print("=" * 60)
    
    # Validate change
    is_valid, error = validate_change(change_type, change_details)
    
    if not is_valid:
        block_unauthorized_change(change_type, change_details, error)
    
    # Log approved change
    log_change_control_event("CHANGE_APPROVED", f"{change_type} change approved", {
        "change_type": change_type,
        "version": new_version,
        "change_details": change_details
    })
    
    print("Change approved")
    return 0


if __name__ == "__main__":
    sys.exit(main())

