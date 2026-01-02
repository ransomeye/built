# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/service_main.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Runtime service entrypoint for ransomeye_intelligence - initializes baseline intelligence and registers DB health heartbeat

"""
RansomEye Intelligence Service Entrypoint

Hard requirements (PROMPT-29C):
- Load configuration from environment (no build-time path assumptions)
- Connect to PostgreSQL (fail-closed)
- Register health in DB and remain running until shutdown
"""

from __future__ import annotations

import os
import signal
import subprocess
import sys
import time
from pathlib import Path
from typing import Dict, Optional, Tuple

from .intelligence_controller import IntelligenceController


def _require_env(keys: Tuple[str, ...]) -> Dict[str, str]:
    missing = [k for k in keys if not os.environ.get(k)]
    if missing:
        raise RuntimeError(f"FAIL-CLOSED: Missing required environment variables: {', '.join(missing)}")
    return {k: os.environ[k] for k in keys}


def _psql_cmd(db_env: Dict[str, str], sql: str) -> subprocess.CompletedProcess:
    if shutil_which("psql") is None:
        raise RuntimeError("FAIL-CLOSED: psql binary not found in PATH (required for DB integration)")

    env = os.environ.copy()
    # psql uses PGPASSWORD for non-interactive auth; DB_PASS is our canonical secret env var.
    env["PGPASSWORD"] = db_env["DB_PASS"]

    args = [
        "psql",
        "-h",
        db_env["DB_HOST"],
        "-p",
        str(db_env["DB_PORT"]),
        "-U",
        db_env["DB_USER"],
        "-d",
        db_env["DB_NAME"],
        "-v",
        "ON_ERROR_STOP=1",
        "-q",
        "-c",
        sql,
    ]
    return subprocess.run(args, capture_output=True, text=True, env=env)


def shutil_which(name: str) -> Optional[str]:
    # Avoid importing shutil at top-level to keep this entrypoint minimal and deterministic.
    import shutil

    return shutil.which(name)


def _db_health_bootstrap(db_env: Dict[str, str]) -> None:
    # Connectivity proof (fail-closed)
    r = _psql_cmd(db_env, "SELECT 1;")
    if r.returncode != 0:
        raise RuntimeError(f"FAIL-CLOSED: DB connectivity check failed: {r.stderr.strip() or r.stdout.strip()}")

    # Upsert component anchor (uses component_type enum; intelligence isn't explicitly enumerated, so use 'other').
    upsert_sql = """
    INSERT INTO ransomeye.components (
      component_type, component_name, instance_id, build_hash, version, started_at, last_heartbeat_at
    )
    VALUES ('other'::ransomeye.component_type, 'ransomeye_intelligence', NULL, NULL, NULL, NOW(), NOW())
    ON CONFLICT (component_type, component_name, (COALESCE(instance_id, '')))
    DO UPDATE SET last_heartbeat_at = NOW()
    RETURNING component_id;
    """
    r = _psql_cmd(db_env, upsert_sql)
    if r.returncode != 0:
        raise RuntimeError(f"FAIL-CLOSED: Failed to upsert components row: {r.stderr.strip() or r.stdout.strip()}")

    # Record initial health (best-effort; fail-closed if write fails)
    health_sql = """
    INSERT INTO ransomeye.component_health (component_id, observed_at, status, status_details, metrics_json)
    SELECT component_id, NOW(), 'healthy', 'startup', jsonb_build_object('state','STARTING')
    FROM ransomeye.components
    WHERE component_type='other'::ransomeye.component_type AND component_name='ransomeye_intelligence'
    ORDER BY updated_at DESC
    LIMIT 1;
    """
    r = _psql_cmd(db_env, health_sql)
    if r.returncode != 0:
        raise RuntimeError(f"FAIL-CLOSED: Failed to write component_health startup row: {r.stderr.strip() or r.stdout.strip()}")


def main() -> int:
    # Enforce runtime root (no build-time paths)
    runtime_root = Path(os.environ.get("RANSOMEYE_ROOT", "/opt/ransomeye")).resolve()
    if not runtime_root.exists():
        print(f"FAIL-CLOSED: RANSOMEYE_ROOT does not exist: {runtime_root}", file=sys.stderr)
        return 1

    # Mandatory DB configuration (provided by systemd EnvironmentFile=/etc/ransomeye/db.env)
    db_env = _require_env(("DB_HOST", "DB_PORT", "DB_NAME", "DB_USER", "DB_PASS"))
    _db_health_bootstrap(db_env)

    # Initialize baseline intelligence pack (fail-closed)
    controller = IntelligenceController()
    if not controller.initialize():
        print("FAIL-CLOSED: IntelligenceController initialization failed", file=sys.stderr)
        return 1

    # Long-running heartbeat loop
    stop = {"flag": False}

    def _handle(_sig, _frame):
        stop["flag"] = True

    signal.signal(signal.SIGTERM, _handle)
    signal.signal(signal.SIGINT, _handle)

    interval_sec = int(os.environ.get("RANSOMEYE_INTELLIGENCE_HEARTBEAT_SEC", "30"))
    if interval_sec < 5:
        print("FAIL-CLOSED: RANSOMEYE_INTELLIGENCE_HEARTBEAT_SEC must be >= 5", file=sys.stderr)
        return 1

    while not stop["flag"]:
        hb_sql = """
        UPDATE ransomeye.components
        SET last_heartbeat_at = NOW(), updated_at = NOW()
        WHERE component_type='other'::ransomeye.component_type AND component_name='ransomeye_intelligence';
        INSERT INTO ransomeye.component_health (component_id, observed_at, status, status_details, metrics_json)
        SELECT component_id, NOW(), 'healthy', 'running', jsonb_build_object('state','RUNNING')
        FROM ransomeye.components
        WHERE component_type='other'::ransomeye.component_type AND component_name='ransomeye_intelligence'
        ORDER BY updated_at DESC
        LIMIT 1;
        """
        r = _psql_cmd(db_env, hb_sql)
        if r.returncode != 0:
            print(f"FAIL-CLOSED: DB heartbeat write failed: {r.stderr.strip() or r.stdout.strip()}", file=sys.stderr)
            return 1
        time.sleep(interval_sec)

    return 0


