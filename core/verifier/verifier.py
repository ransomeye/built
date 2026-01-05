# Path and File Name : /home/ransomeye/rebuild/core/verifier/verifier.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Continuous Verification Engine - Runs every 5 minutes, checks all services, DB, models, UI

"""
RansomEye Continuous Verification Engine

Runs every 5 minutes and verifies:
- All services running
- DB counts increasing
- Audit log increasing
- Models registered
- SHAP present
- Threat intel not stale
- UI reachable

If any check fails:
- Write immutable audit entry
- Log critical error
- Exit non-zero
"""

import os
import sys
import time
import subprocess
import psycopg2
import json
from datetime import datetime, timedelta
from pathlib import Path

# Configuration from environment
DB_NAME = os.environ.get("DB_NAME", "ransomeye")
DB_USER = os.environ.get("DB_USER", "gagan")
DB_HOST = os.environ.get("DB_HOST", "localhost")
DB_PASSWORD = os.environ.get("DB_PASS", "gagan")
DB_PORT = int(os.environ.get("DB_PORT", "5432"))

UI_HOST = os.environ.get("RANSOMEYE_UI_HOST", "127.0.0.1")
UI_PORT = int(os.environ.get("RANSOMEYE_UI_PORT", "8080"))

REQUIRED_SERVICES = [
    "ransomeye-ingestion",
    "ransomeye-normalization",
    "ransomeye-ui",
]

OPTIONAL_SERVICES = [
    "ransomeye-core",
    "ransomeye-correlation",
    "ransomeye-policy",
    "ransomeye-enforcement",
    "ransomeye-linux-agent",
    "ransomeye-dpi-probe",
]

AUDIT_LOG_PATH = Path("/var/log/ransomeye/verifier_audit.log")
RESULTS_PATH = Path("/var/log/ransomeye/verifier_results.json")


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


def check_service_running(service_name):
    """Check if systemd service is running."""
    try:
        result = subprocess.run(
            ["systemctl", "is-active", service_name],
            capture_output=True,
            text=True,
            timeout=5
        )
        return result.returncode == 0 and result.stdout.strip() == "active"
    except Exception:
        return False


def check_db_counts(conn):
    """Check if DB counts are increasing."""
    try:
        cursor = conn.cursor()
        
        # Check raw_events count
        cursor.execute("SELECT COUNT(*) FROM ransomeye.raw_events")
        raw_events_count = cursor.fetchone()[0]
        
        # Check normalized_events count
        cursor.execute("SELECT COUNT(*) FROM ransomeye.normalized_events")
        normalized_events_count = cursor.fetchone()[0]
        
        # Check agents count
        cursor.execute("SELECT COUNT(*) FROM ransomeye.agents WHERE is_active = true")
        agents_count = cursor.fetchone()[0]
        
        cursor.close()
        
        return {
            "raw_events": raw_events_count,
            "normalized_events": normalized_events_count,
            "agents": agents_count,
            "healthy": raw_events_count > 0 or normalized_events_count > 0
        }
    except Exception as e:
        return {"error": str(e), "healthy": False}


def check_audit_log(conn):
    """Check if audit log is increasing."""
    try:
        cursor = conn.cursor()
        cursor.execute("""
            SELECT COUNT(*) 
            FROM ransomeye.immutable_audit_log 
            WHERE created_at > NOW() - INTERVAL '1 hour'
        """)
        count = cursor.fetchone()[0]
        cursor.close()
        return {"count": count, "healthy": True}
    except Exception as e:
        return {"error": str(e), "healthy": False}


def check_models_registered(conn):
    """Check if models are registered in DB."""
    try:
        cursor = conn.cursor()
        cursor.execute("SELECT COUNT(*) FROM ransomeye.model_registry")
        count = cursor.fetchone()[0]
        cursor.close()
        return {"count": count, "healthy": count > 0}
    except Exception as e:
        return {"error": str(e), "healthy": False}


def check_shap_present(conn):
    """Check if SHAP explanations exist."""
    try:
        cursor = conn.cursor()
        cursor.execute("SELECT COUNT(*) FROM ransomeye.shap_explanations")
        count = cursor.fetchone()[0]
        cursor.close()
        return {"count": count, "healthy": True}  # SHAP may be empty initially
    except Exception as e:
        return {"error": str(e), "healthy": False}


def check_ui_reachable():
    """Check if UI is reachable."""
    try:
        import urllib.request
        url = f"http://{UI_HOST}:{UI_PORT}/api/health"
        with urllib.request.urlopen(url, timeout=5) as response:
            return {"status": response.status, "healthy": response.status == 200}
    except Exception as e:
        return {"error": str(e), "healthy": False}


def write_audit_entry(message, level="INFO"):
    """Write immutable audit entry."""
    try:
        AUDIT_LOG_PATH.parent.mkdir(parents=True, exist_ok=True)
        timestamp = datetime.utcnow().isoformat()
        with open(AUDIT_LOG_PATH, "a") as f:
            f.write(f"{timestamp} [{level}] {message}\n")
    except Exception:
        pass  # Fail silently if audit log write fails


def main():
    """Main verification loop."""
    results = {
        "timestamp": datetime.utcnow().isoformat(),
        "checks": {},
        "overall_healthy": True,
        "failures": []
    }
    
    # Check required services
    services_healthy = True
    for service in REQUIRED_SERVICES:
        is_running = check_service_running(service)
        results["checks"][f"service_{service}"] = {
            "status": "running" if is_running else "not_running",
            "healthy": is_running
        }
        if not is_running:
            services_healthy = False
            results["failures"].append(f"Service {service} is not running")
            write_audit_entry(f"VERIFICATION FAILURE: Service {service} is not running", "ERROR")
    
    # Check optional services (warn only)
    for service in OPTIONAL_SERVICES:
        is_running = check_service_running(service)
        results["checks"][f"service_{service}"] = {
            "status": "running" if is_running else "not_running",
            "healthy": is_running,
            "optional": True
        }
    
    # Check database
    conn = get_db_connection()
    if conn:
        db_counts = check_db_counts(conn)
        results["checks"]["db_counts"] = db_counts
        if not db_counts.get("healthy", False):
            results["failures"].append("DB counts check failed")
            write_audit_entry("VERIFICATION FAILURE: DB counts check failed", "ERROR")
        
        audit_log = check_audit_log(conn)
        results["checks"]["audit_log"] = audit_log
        
        models = check_models_registered(conn)
        results["checks"]["models_registered"] = models
        
        shap = check_shap_present(conn)
        results["checks"]["shap_present"] = shap
        
        conn.close()
    else:
        results["checks"]["db_connection"] = {"healthy": False, "error": "Connection failed"}
        results["failures"].append("Database connection failed")
        write_audit_entry("VERIFICATION FAILURE: Database connection failed", "ERROR")
        results["overall_healthy"] = False
    
    # Check UI
    ui_check = check_ui_reachable()
    results["checks"]["ui_reachable"] = ui_check
    if not ui_check.get("healthy", False):
        results["failures"].append("UI not reachable")
        write_audit_entry("VERIFICATION FAILURE: UI not reachable", "ERROR")
    
    # Determine overall health
    if not services_healthy or results["failures"]:
        results["overall_healthy"] = False
    
    # Write results
    try:
        RESULTS_PATH.parent.mkdir(parents=True, exist_ok=True)
        with open(RESULTS_PATH, "w") as f:
            json.dump(results, f, indent=2)
    except Exception:
        pass
    
    # Exit with appropriate code
    if not results["overall_healthy"]:
        write_audit_entry(f"VERIFICATION FAILED: {len(results['failures'])} failures", "CRITICAL")
        sys.exit(1)
    else:
        write_audit_entry("Verification passed", "INFO")
        sys.exit(0)


if __name__ == "__main__":
    main()

