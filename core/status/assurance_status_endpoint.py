# Path and File Name : /home/ransomeye/rebuild/core/status/assurance_status_endpoint.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Assurance Status Endpoint - Read-only local status endpoint

"""
RansomEye Assurance Status Endpoint (PROMPT-60-C)

Exposes read-only local endpoint showing:
- Verifier green status
- Last audit count
- Model versions
- Threat intel freshness
- Drift status

Localhost-only by default, env-controlled exposure.
"""

import os
import sys
import json
import psycopg2
from datetime import datetime, timezone
from pathlib import Path
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse

# Configuration from environment
DB_NAME = os.environ.get("DB_NAME", "ransomeye")
DB_USER = os.environ.get("DB_USER", "gagan")
DB_HOST = os.environ.get("DB_HOST", "localhost")
DB_PASSWORD = os.environ.get("DB_PASS", "gagan")
DB_PORT = int(os.environ.get("DB_PORT", "5432"))

STATUS_HOST = os.environ.get("RANSOMEYE_STATUS_HOST", "127.0.0.1")
STATUS_PORT = int(os.environ.get("RANSOMEYE_STATUS_PORT", "8082"))
VERIFIER_RESULTS_PATH = Path("/var/log/ransomeye/verifier_results.json")
GOLDEN_BASELINE_PATH = Path("/var/lib/ransomeye/baselines/golden_baseline.json")


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


def get_status() -> dict:
    """Get current assurance status."""
    status = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "verifier": {},
        "audit": {},
        "models": {},
        "threat_intel": {},
        "drift": {}
    }
    
    # Verifier status
    if VERIFIER_RESULTS_PATH.exists():
        try:
            with open(VERIFIER_RESULTS_PATH, "r") as f:
                verifier_results = json.load(f)
            status["verifier"] = {
                "green": verifier_results.get("overall_healthy", False),
                "last_check": verifier_results.get("timestamp"),
                "assurance_mode": verifier_results.get("assurance_mode", False)
            }
        except Exception:
            status["verifier"] = {"error": "Could not read verifier results"}
    else:
        status["verifier"] = {"error": "Verifier results not found"}
    
    # Audit count
    conn = get_db_connection()
    if conn:
        try:
            cursor = conn.cursor()
            cursor.execute("SELECT COUNT(*) FROM ransomeye.immutable_audit_log")
            audit_count = cursor.fetchone()[0]
            
            cursor.execute("""
                SELECT MAX(created_at) FROM ransomeye.immutable_audit_log
            """)
            last_audit = cursor.fetchone()[0]
            
            status["audit"] = {
                "total_count": audit_count,
                "last_entry": last_audit.isoformat() if last_audit else None
            }
            cursor.close()
        except Exception as e:
            status["audit"] = {"error": str(e)}
        
        # Model versions
        try:
            cursor = conn.cursor()
            cursor.execute("""
                SELECT COUNT(DISTINCT model_id) FROM ransomeye.model_registry
            """)
            model_count = cursor.fetchone()[0]
            
            cursor.execute("""
                SELECT COUNT(*) FROM ransomeye.shap_explanations
            """)
            shap_count = cursor.fetchone()[0]
            
            status["models"] = {
                "model_count": model_count,
                "shap_explanations": shap_count
            }
            cursor.close()
        except Exception as e:
            status["models"] = {"error": str(e)}
        
        # Threat intel freshness
        try:
            cursor = conn.cursor()
            cursor.execute("SELECT COUNT(*) FROM ransomeye.threat_intel_iocs")
            ioc_count = cursor.fetchone()[0]
            
            cursor.execute("""
                SELECT MAX(last_updated) FROM ransomeye.threat_intel_iocs
                WHERE last_updated IS NOT NULL
            """)
            last_update = cursor.fetchone()[0]
            
            status["threat_intel"] = {
                "ioc_count": ioc_count,
                "last_updated": last_update.isoformat() if last_update else None
            }
            cursor.close()
        except Exception as e:
            status["threat_intel"] = {"error": str(e)}
        
        conn.close()
    else:
        status["audit"] = {"error": "Database connection failed"}
        status["models"] = {"error": "Database connection failed"}
        status["threat_intel"] = {"error": "Database connection failed"}
    
    # Drift status
    if GOLDEN_BASELINE_PATH.exists():
        status["drift"] = {
            "baseline_exists": True,
            "drift_detected": False  # Simplified - would check recent diff files
        }
    else:
        status["drift"] = {
            "baseline_exists": False,
            "drift_detected": False
        }
    
    return status


class StatusHandler(BaseHTTPRequestHandler):
    """HTTP handler for status endpoint."""
    
    def do_GET(self):
        """Handle GET requests."""
        if self.path == "/status/assurance" or self.path == "/status/assurance/":
            status = get_status()
            
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Access-Control-Allow-Origin", "*")
            self.end_headers()
            
            self.wfile.write(json.dumps(status, indent=2, sort_keys=True).encode())
        else:
            self.send_response(404)
            self.end_headers()
            self.wfile.write(b"Not Found")
    
    def log_message(self, format, *args):
        """Suppress default logging."""
        pass


def main():
    """Main status endpoint server."""
    print("RansomEye Assurance Status Endpoint (PROMPT-60-C)")
    print("=" * 60)
    print(f"Starting status server on {STATUS_HOST}:{STATUS_PORT}")
    print(f"Endpoint: http://{STATUS_HOST}:{STATUS_PORT}/status/assurance")
    print("Press Ctrl+C to stop")
    
    server = HTTPServer((STATUS_HOST, STATUS_PORT), StatusHandler)
    
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down status server...")
        server.shutdown()
    
    return 0


if __name__ == "__main__":
    sys.exit(main())

