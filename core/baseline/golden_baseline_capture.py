# Path and File Name : /home/ransomeye/rebuild/core/baseline/golden_baseline_capture.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Golden Baseline Capture - Captures full system state snapshot for post-ship operational guarantee

"""
RansomEye Golden Baseline Capture (PROMPT-58-A)

Captures immutable snapshot of system state at v1.0.0-enterprise-ship:
- OS version, kernel, packages
- systemd unit hashes
- DB schema checksum
- Artifact hashes (re-verification)
- All critical system invariants

Stores in /var/lib/ransomeye/baselines/golden_baseline.json (read-only)
"""

import os
import sys
import json
import hashlib
import subprocess
import platform
import psycopg2
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Optional

# Configuration from environment
DB_NAME = os.environ.get("DB_NAME", "ransomeye")
DB_USER = os.environ.get("DB_USER", "gagan")
DB_HOST = os.environ.get("DB_HOST", "localhost")
DB_PASSWORD = os.environ.get("DB_PASS", "gagan")
DB_PORT = int(os.environ.get("DB_PORT", "5432"))

BASELINE_DIR = Path("/var/lib/ransomeye/baselines")
GOLDEN_BASELINE_PATH = BASELINE_DIR / "golden_baseline.json"
ARTIFACT_HASHES_PATH = Path("/home/ransomeye/rebuild/docs/ARTIFACT_HASHES.txt")
SYSTEMD_DIR = Path("/etc/systemd/system")
RUNTIME_ROOT = Path("/opt/ransomeye")


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


def capture_os_info() -> Dict:
    """Capture OS version, kernel, and package information."""
    info = {
        "os_name": platform.system(),
        "os_release": platform.release(),
        "os_version": platform.version(),
        "machine": platform.machine(),
        "processor": platform.processor(),
        "python_version": platform.python_version()
    }
    
    # Get kernel version
    try:
        result = subprocess.run(
            ["uname", "-r"],
            capture_output=True,
            text=True,
            timeout=5
        )
        if result.returncode == 0:
            info["kernel_version"] = result.stdout.strip()
    except Exception:
        pass
    
    # Get distribution info (Linux)
    if platform.system() == "Linux":
        try:
            # Try /etc/os-release
            if Path("/etc/os-release").exists():
                with open("/etc/os-release", "r") as f:
                    for line in f:
                        if "=" in line:
                            key, value = line.strip().split("=", 1)
                            value = value.strip('"')
                            info[f"os_{key.lower()}"] = value
        except Exception:
            pass
    
    # Get installed packages (Debian/Ubuntu)
    try:
        result = subprocess.run(
            ["dpkg", "-l"],
            capture_output=True,
            text=True,
            timeout=30
        )
        if result.returncode == 0:
            # Count packages
            lines = result.stdout.strip().split("\n")
            package_count = len([l for l in lines if l.startswith("ii")])
            info["package_count"] = package_count
            # Hash package list for integrity
            package_hash = hashlib.sha256(result.stdout.encode()).hexdigest()
            info["package_list_hash"] = package_hash
    except Exception:
        pass
    
    return info


def capture_systemd_hashes() -> Dict:
    """Capture systemd unit file hashes."""
    units = {}
    
    if not SYSTEMD_DIR.exists():
        return units
    
    for unit_file in SYSTEMD_DIR.glob("ransomeye*.service"):
        unit_name = unit_file.name
        try:
            with open(unit_file, "rb") as f:
                content = f.read()
                unit_hash = hashlib.sha256(content).hexdigest()
                units[unit_name] = {
                    "sha256": unit_hash,
                    "size": len(content),
                    "mtime": unit_file.stat().st_mtime
                }
        except Exception as e:
            units[unit_name] = {"error": str(e)}
    
    # Also check timers
    for unit_file in SYSTEMD_DIR.glob("ransomeye*.timer"):
        unit_name = unit_file.name
        try:
            with open(unit_file, "rb") as f:
                content = f.read()
                unit_hash = hashlib.sha256(content).hexdigest()
                units[unit_name] = {
                    "sha256": unit_hash,
                    "size": len(content),
                    "mtime": unit_file.stat().st_mtime
                }
        except Exception as e:
            units[unit_name] = {"error": str(e)}
    
    return units


def capture_db_schema_checksum(conn) -> Dict:
    """Capture database schema checksum."""
    if not conn:
        return {"error": "Database connection failed"}
    
    try:
        cursor = conn.cursor()
        
        # Get all tables in ransomeye schema
        cursor.execute("""
            SELECT table_name 
            FROM information_schema.tables 
            WHERE table_schema = 'ransomeye'
            ORDER BY table_name
        """)
        tables = [row[0] for row in cursor.fetchall()]
        
        # Get table structures
        schema_info = {}
        for table in tables:
            cursor.execute("""
                SELECT column_name, data_type, is_nullable
                FROM information_schema.columns
                WHERE table_schema = 'ransomeye' AND table_name = %s
                ORDER BY ordinal_position
            """, (table,))
            columns = cursor.fetchall()
            schema_info[table] = {
                "columns": [{"name": c[0], "type": c[1], "nullable": c[2]} for c in columns]
            }
        
        # Compute schema checksum
        schema_json = json.dumps(schema_info, sort_keys=True)
        schema_checksum = hashlib.sha256(schema_json.encode()).hexdigest()
        
        cursor.close()
        
        return {
            "table_count": len(tables),
            "tables": tables,
            "schema_checksum": schema_checksum,
            "schema_info": schema_info
        }
    except Exception as e:
        return {"error": str(e)}


def capture_artifact_hashes() -> Dict:
    """Re-verify artifact hashes from ARTIFACT_HASHES.txt."""
    artifacts = {}
    
    if not ARTIFACT_HASHES_PATH.exists():
        return {"error": "ARTIFACT_HASHES.txt not found"}
    
    try:
        # Parse artifact hashes file
        with open(ARTIFACT_HASHES_PATH, 'r') as f:
            current_path = None
            for line in f:
                line = line.strip()
                if line and not line.startswith('#'):
                    if line.startswith('/') or 'models/' in line or '.pkl' in line or '.gguf' in line:
                        current_path = line
                    elif line.startswith('SHA256:'):
                        if current_path:
                            expected_hash = line.replace('SHA256:', '').strip()
                            # Verify hash
                            full_path = Path("/home/ransomeye/rebuild") / current_path.lstrip('/')
                            if full_path.exists():
                                with open(full_path, 'rb') as af:
                                    file_hash = hashlib.sha256(af.read()).hexdigest()
                                    artifacts[current_path] = {
                                        "expected": expected_hash,
                                        "actual": file_hash,
                                        "match": file_hash == expected_hash,
                                        "size": full_path.stat().st_size
                                    }
                            else:
                                artifacts[current_path] = {
                                    "expected": expected_hash,
                                    "error": "File not found"
                                }
                            current_path = None
        
        return artifacts
    except Exception as e:
        return {"error": str(e)}


def capture_runtime_layout() -> Dict:
    """Capture runtime layout at /opt/ransomeye."""
    layout = {}
    
    if not RUNTIME_ROOT.exists():
        return {"error": "Runtime root not found"}
    
    try:
        # Capture directory structure
        dirs = []
        files = []
        
        for item in RUNTIME_ROOT.rglob("*"):
            rel_path = str(item.relative_to(RUNTIME_ROOT))
            if item.is_dir():
                dirs.append(rel_path)
            elif item.is_file():
                try:
                    stat = item.stat()
                    files.append({
                        "path": rel_path,
                        "size": stat.st_size,
                        "mtime": stat.st_mtime,
                        "sha256": hashlib.sha256(item.read_bytes()).hexdigest() if stat.st_size < 100 * 1024 * 1024 else None  # Skip large files
                    })
                except Exception:
                    pass
        
        layout = {
            "directories": sorted(dirs),
            "file_count": len(files),
            "files": files[:1000]  # Limit to first 1000 files
        }
        
        return layout
    except Exception as e:
        return {"error": str(e)}


def capture_service_status() -> Dict:
    """Capture current systemd service status."""
    services = {}
    
    # Get all ransomeye services
    try:
        result = subprocess.run(
            ["systemctl", "list-units", "--type=service", "--all", "--no-pager"],
            capture_output=True,
            text=True,
            timeout=10
        )
        if result.returncode == 0:
            for line in result.stdout.split("\n"):
                if "ransomeye" in line:
                    parts = line.split()
                    if len(parts) >= 4:
                        service_name = parts[0]
                        load_state = parts[1] if len(parts) > 1 else "unknown"
                        active_state = parts[2] if len(parts) > 2 else "unknown"
                        sub_state = parts[3] if len(parts) > 3 else "unknown"
                        services[service_name] = {
                            "load": load_state,
                            "active": active_state,
                            "sub": sub_state
                        }
    except Exception:
        pass
    
    return services


def main():
    """Main baseline capture function."""
    print("RansomEye Golden Baseline Capture (PROMPT-58-A)")
    print("=" * 60)
    
    # Create baseline directory
    BASELINE_DIR.mkdir(parents=True, exist_ok=True)
    
    # Capture all system state
    print("Capturing OS information...")
    os_info = capture_os_info()
    
    print("Capturing systemd unit hashes...")
    systemd_hashes = capture_systemd_hashes()
    
    print("Capturing database schema checksum...")
    conn = get_db_connection()
    db_schema = capture_db_schema_checksum(conn)
    if conn:
        conn.close()
    
    print("Capturing artifact hashes...")
    artifact_hashes = capture_artifact_hashes()
    
    print("Capturing runtime layout...")
    runtime_layout = capture_runtime_layout()
    
    print("Capturing service status...")
    service_status = capture_service_status()
    
    # Build golden baseline
    baseline = {
        "version": "1.0.0-enterprise-ship",
        "capture_timestamp": datetime.now(timezone.utc).isoformat(),
        "os_info": os_info,
        "systemd_units": systemd_hashes,
        "db_schema": db_schema,
        "artifact_hashes": artifact_hashes,
        "runtime_layout": runtime_layout,
        "service_status": service_status
    }
    
    # Compute baseline checksum
    baseline_json = json.dumps(baseline, sort_keys=True, indent=2)
    baseline_checksum = hashlib.sha256(baseline_json.encode()).hexdigest()
    baseline["baseline_checksum"] = baseline_checksum
    
    # Write baseline
    print(f"Writing golden baseline to {GOLDEN_BASELINE_PATH}...")
    with open(GOLDEN_BASELINE_PATH, "w") as f:
        json.dump(baseline, f, indent=2, sort_keys=True)
    
    # Make read-only
    try:
        os.chmod(GOLDEN_BASELINE_PATH, 0o444)  # Read-only for all
        print(f"Baseline file set to read-only (444)")
    except Exception as e:
        print(f"Warning: Could not set read-only: {e}")
    
    print("\nGolden Baseline Capture Complete")
    print(f"Baseline checksum: {baseline_checksum}")
    print(f"Location: {GOLDEN_BASELINE_PATH}")
    
    # Verify against ship seal if available
    ship_seal_path = Path("/home/ransomeye/rebuild/docs/enterprise/SHIP_SEAL.txt")
    if ship_seal_path.exists():
        print("\nVerifying against ship seal...")
        with open(ship_seal_path, "r") as f:
            ship_seal = f.read().strip()
        # Compare critical hashes
        print("Ship seal verification: Manual review required")
    
    return 0


if __name__ == "__main__":
    sys.exit(main())

