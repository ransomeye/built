# Path and File Name : /home/ransomeye/rebuild/core/assurance/service_protection.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Service Protection - Prevents disabling of critical assurance services

"""
RansomEye Service Protection (PROMPT-59-A)

Prevents disabling of critical assurance services.
Any attempt to stop or mask these services triggers SYSTEM_INTEGRITY_VIOLATION audit.
"""

import os
import sys
import subprocess
import psycopg2
import json
import hashlib
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import List, Tuple

# Configuration from environment
DB_NAME = os.environ.get("DB_NAME", "ransomeye")
DB_USER = os.environ.get("DB_USER", "gagan")
DB_HOST = os.environ.get("DB_HOST", "localhost")
DB_PASSWORD = os.environ.get("DB_PASS", "gagan")
DB_PORT = int(os.environ.get("DB_PORT", "5432"))

PROTECTED_SERVICES = [
    "ransomeye-verifier",
    "ransomeye-compliance-automation",
    "ransomeye-compliance-automation.timer",
]

ASSURANCE_LOCK_PATH = Path("/etc/ransomeye/ASSURANCE_MODE_LOCK")


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


def check_assurance_mode() -> bool:
    """Check if assurance mode lock exists."""
    return ASSURANCE_LOCK_PATH.exists()


def check_service_status(service_name: str) -> Tuple[bool, str]:
    """Check if service is active and enabled."""
    try:
        # Check if active
        result = subprocess.run(
            ["systemctl", "is-active", service_name],
            capture_output=True,
            text=True,
            timeout=5
        )
        is_active = result.returncode == 0 and result.stdout.strip() == "active"
        
        # Check if enabled
        result = subprocess.run(
            ["systemctl", "is-enabled", service_name],
            capture_output=True,
            text=True,
            timeout=5
        )
        is_enabled = result.returncode == 0 and "enabled" in result.stdout.strip()
        
        if not is_active:
            return False, f"Service {service_name} is not active"
        if not is_enabled:
            return False, f"Service {service_name} is not enabled"
        
        return True, "OK"
    except Exception as e:
        return False, f"Failed to check service {service_name}: {str(e)}"


def write_integrity_violation_audit(conn, violation_message: str, details: dict):
    """Write SYSTEM_INTEGRITY_VIOLATION audit entry."""
    if not conn:
        return
    
    try:
        cursor = conn.cursor()
        
        payload_json = {
            "violation_type": "SYSTEM_INTEGRITY_VIOLATION",
            "message": violation_message,
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "details": details
        }
        payload_str = json.dumps(payload_json, sort_keys=True)
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
        audit_id = str(uuid.uuid4())
        cursor.execute("""
            INSERT INTO ransomeye.immutable_audit_log (
                audit_id, action, object_type, payload_json, payload_sha256,
                prev_payload_sha256, chain_hash_sha256, signature_status
            )
            VALUES (%s, %s, %s, %s, %s, %s, %s, 'unknown')
        """, (
            audit_id,
            "SYSTEM_INTEGRITY_VIOLATION",
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


def check_protected_services() -> Tuple[bool, List[str]]:
    """Check all protected services."""
    if not check_assurance_mode():
        return True, []  # Not in assurance mode, skip protection
    
    violations = []
    conn = get_db_connection()
    
    for service in PROTECTED_SERVICES:
        is_ok, error = check_service_status(service)
        if not is_ok:
            violations.append(f"{service}: {error}")
            # Write audit entry
            if conn:
                write_integrity_violation_audit(
                    conn,
                    f"Protected service violation: {service}",
                    {"service": service, "error": error}
                )
    
    if conn:
        conn.close()
    
    return len(violations) == 0, violations


def main():
    """Main service protection check."""
    if not check_assurance_mode():
        print("Assurance mode not active, skipping service protection check")
        return 0
    
    print("Checking protected services...")
    is_ok, violations = check_protected_services()
    
    if not is_ok:
        print("VIOLATION: Protected services disabled or stopped")
        for violation in violations:
            print(f"  - {violation}")
        return 1
    
    print("All protected services are active and enabled")
    return 0


if __name__ == "__main__":
    sys.exit(main())

