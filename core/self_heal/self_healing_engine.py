# Path and File Name : /home/ransomeye/rebuild/core/self_heal/self_healing_engine.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Self-Healing Engine - Controlled self-healing with escalation levels and hard stops

"""
RansomEye Self-Healing Engine (PROMPT-59-D)

Controlled self-healing with escalation levels:
- Level 1: Auto-restart (transient failures only)
- Level 2: Quarantine service
- Level 3: Full system lock + audit

Hard stops for:
- Integrity violations
- Drift
- Audit failures
"""

import os
import sys
import json
import subprocess
import psycopg2
import time
from datetime import datetime, timezone, timedelta
from pathlib import Path
from typing import Dict, List, Tuple

# Configuration from environment
DB_NAME = os.environ.get("DB_NAME", "ransomeye")
DB_USER = os.environ.get("DB_USER", "gagan")
DB_HOST = os.environ.get("DB_HOST", "localhost")
DB_PASSWORD = os.environ.get("DB_PASS", "gagan")
DB_PORT = int(os.environ.get("DB_PORT", "5432"))

ESCALATION_STATE_PATH = Path("/var/lib/ransomeye/self_heal/escalation_state.json")
MAX_RESTART_ATTEMPTS = 3
RESTART_WINDOW_SECONDS = 300  # 5 minutes


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


def load_escalation_state() -> dict:
    """Load escalation state."""
    if not ESCALATION_STATE_PATH.exists():
        return {
            "services": {},
            "system_lock": False,
            "lock_timestamp": None
        }
    
    try:
        with open(ESCALATION_STATE_PATH, "r") as f:
            return json.load(f)
    except Exception:
        return {
            "services": {},
            "system_lock": False,
            "lock_timestamp": None
        }


def save_escalation_state(state: dict):
    """Save escalation state."""
    ESCALATION_STATE_PATH.parent.mkdir(parents=True, exist_ok=True)
    with open(ESCALATION_STATE_PATH, "w") as f:
        json.dump(state, f, indent=2, sort_keys=True)


def check_integrity_violation(conn) -> bool:
    """Check for integrity violations (hard stop)."""
    if not conn:
        return False
    
    try:
        cursor = conn.cursor()
        # Check for recent integrity violations
        cursor.execute("""
            SELECT COUNT(*) 
            FROM ransomeye.immutable_audit_log
            WHERE action = 'SYSTEM_INTEGRITY_VIOLATION'
            AND created_at > NOW() - INTERVAL '1 hour'
        """)
        count = cursor.fetchone()[0]
        cursor.close()
        return count > 0
    except Exception:
        return False


def check_drift() -> bool:
    """Check for drift (hard stop)."""
    verifier_results_path = Path("/var/log/ransomeye/verifier_results.json")
    if not verifier_results_path.exists():
        return False
    
    try:
        with open(verifier_results_path, "r") as f:
            results = json.load(f)
        # Check if drift detected
        drift_check = results.get("checks", {}).get("drift", {})
        return not drift_check.get("healthy", True)
    except Exception:
        return False


def check_audit_failure(conn) -> bool:
    """Check for audit failures (hard stop)."""
    if not conn:
        return False
    
    try:
        cursor = conn.cursor()
        # Check for broken audit chain
        cursor.execute("""
            SELECT COUNT(*) 
            FROM ransomeye.immutable_audit_log
            WHERE chain_hash_sha256 IS NULL
            AND created_at > (SELECT MIN(created_at) FROM ransomeye.immutable_audit_log)
        """)
        count = cursor.fetchone()[0]
        cursor.close()
        return count > 0
    except Exception:
        return False


def level_1_restart(service_name: str, state: dict) -> Tuple[bool, str]:
    """Level 1: Auto-restart service."""
    service_state = state["services"].get(service_name, {
        "restart_count": 0,
        "last_restart": None,
        "level": 1
    })
    
    # Check restart count in window
    now = datetime.now(timezone.utc)
    if service_state["last_restart"]:
        last_restart = datetime.fromisoformat(service_state["last_restart"])
        if (now - last_restart).total_seconds() < RESTART_WINDOW_SECONDS:
            service_state["restart_count"] += 1
        else:
            service_state["restart_count"] = 1
    else:
        service_state["restart_count"] = 1
    
    service_state["last_restart"] = now.isoformat()
    
    # If too many restarts, escalate
    if service_state["restart_count"] > MAX_RESTART_ATTEMPTS:
        return False, f"Too many restarts ({service_state['restart_count']}), escalating"
    
    # Attempt restart
    try:
        subprocess.run(
            ["systemctl", "restart", service_name],
            capture_output=True,
            timeout=30
        )
        state["services"][service_name] = service_state
        save_escalation_state(state)
        return True, f"Service {service_name} restarted (attempt {service_state['restart_count']})"
    except Exception as e:
        return False, f"Failed to restart {service_name}: {str(e)}"


def level_2_quarantine(service_name: str, state: dict) -> Tuple[bool, str]:
    """Level 2: Quarantine service."""
    service_state = state["services"].get(service_name, {})
    service_state["level"] = 2
    service_state["quarantined_at"] = datetime.now(timezone.utc).isoformat()
    
    # Stop service
    try:
        subprocess.run(
            ["systemctl", "stop", service_name],
            capture_output=True,
            timeout=30
        )
        state["services"][service_name] = service_state
        save_escalation_state(state)
        return True, f"Service {service_name} quarantined"
    except Exception as e:
        return False, f"Failed to quarantine {service_name}: {str(e)}"


def level_3_system_lock(state: dict, reason: str) -> Tuple[bool, str]:
    """Level 3: Full system lock."""
    state["system_lock"] = True
    state["lock_timestamp"] = datetime.now(timezone.utc).isoformat()
    state["lock_reason"] = reason
    
    # Stop all RansomEye services
    try:
        result = subprocess.run(
            ["systemctl", "list-units", "--type=service", "--no-pager"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        services_to_stop = []
        for line in result.stdout.split("\n"):
            if "ransomeye" in line and "active" in line:
                service_name = line.split()[0]
                services_to_stop.append(service_name)
        
        for service in services_to_stop:
            try:
                subprocess.run(
                    ["systemctl", "stop", service],
                    capture_output=True,
                    timeout=30
                )
            except Exception:
                pass
        
        save_escalation_state(state)
        
        # Write audit entry
        conn = get_db_connection()
        if conn:
            write_system_lock_audit(conn, reason)
            conn.close()
        
        return True, f"System locked: {reason}"
    except Exception as e:
        return False, f"Failed to lock system: {str(e)}"


def write_system_lock_audit(conn, reason: str):
    """Write system lock audit entry."""
    try:
        cursor = conn.cursor()
        import hashlib
        import uuid
        
        payload_json = {
            "violation_type": "SYSTEM_LOCK",
            "reason": reason,
            "timestamp": datetime.now(timezone.utc).isoformat()
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
            "SYSTEM_LOCK",
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


def handle_service_failure(service_name: str) -> Tuple[bool, str]:
    """Handle service failure with escalation."""
    state = load_escalation_state()
    
    # Check for hard stops
    conn = get_db_connection()
    if check_integrity_violation(conn):
        return level_3_system_lock(state, "Integrity violation detected")
    
    if check_drift():
        return level_3_system_lock(state, "Drift detected")
    
    if check_audit_failure(conn):
        return level_3_system_lock(state, "Audit failure detected")
    
    if conn:
        conn.close()
    
    # Check if system is locked
    if state.get("system_lock", False):
        return False, "System is locked, cannot heal"
    
    # Check service escalation level
    service_state = state["services"].get(service_name, {"level": 1})
    level = service_state.get("level", 1)
    
    if level == 1:
        # Try restart
        success, message = level_1_restart(service_name, state)
        if not success:
            # Escalate to level 2
            return level_2_quarantine(service_name, state)
        return success, message
    
    elif level == 2:
        # Already quarantined, escalate to level 3
        return level_3_system_lock(state, f"Service {service_name} failed after quarantine")
    
    else:
        # Already at level 3
        return False, "System already locked"


def main():
    """Main self-healing engine."""
    if len(sys.argv) < 2:
        print("Usage: self_healing_engine.py <service_name>")
        return 1
    
    service_name = sys.argv[1]
    
    print(f"Self-healing engine: Handling failure for {service_name}")
    success, message = handle_service_failure(service_name)
    
    if success:
        print(f"SUCCESS: {message}")
        return 0
    else:
        print(f"FAILED: {message}")
        return 1


if __name__ == "__main__":
    sys.exit(main())

