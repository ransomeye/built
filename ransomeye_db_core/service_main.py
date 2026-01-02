# Path and File Name : /home/ransomeye/rebuild/ransomeye_db_core/service_main.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Runtime service entrypoint for ransomeye_db_core - enforces DB reachability and schema presence with periodic heartbeats

"""
RansomEye DB Core Service Entrypoint

Hard requirements:
- DB is mandatory for Core runtime
- Fail-closed if DB not reachable
- Remain running and publish health facts into the DB

Implementation note:
- Uses system `psql` client to avoid requiring Python DB driver packages in the base runtime.
"""

from __future__ import annotations

import os
import signal
import subprocess
import sys
import time
from typing import Dict, Optional, Tuple


def _which(name: str) -> Optional[str]:
    import shutil

    return shutil.which(name)


def _require_env(keys: Tuple[str, ...]) -> Dict[str, str]:
    missing = [k for k in keys if not os.environ.get(k)]
    if missing:
        raise RuntimeError(f"FAIL-CLOSED: Missing required environment variables: {', '.join(missing)}")
    return {k: os.environ[k] for k in keys}


def _psql(db_env: Dict[str, str], sql: str) -> subprocess.CompletedProcess:
    if _which("psql") is None:
        raise RuntimeError("FAIL-CLOSED: psql binary not found in PATH (required for DB core service)")

    env = os.environ.copy()
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


def main() -> int:
    db_env = _require_env(("DB_HOST", "DB_PORT", "DB_NAME", "DB_USER", "DB_PASS"))

    # Fail-closed DB reachability check
    r = _psql(db_env, "SELECT 1;")
    if r.returncode != 0:
        print(f"FAIL-CLOSED: DB connectivity check failed: {r.stderr.strip() or r.stdout.strip()}", file=sys.stderr)
        return 1

    # Ensure the mandatory retention_policies relation is reachable (schema alignment proof).
    r = _psql(db_env, "SELECT 1 FROM ransomeye.retention_policies LIMIT 1;")
    if r.returncode != 0:
        print(
            "FAIL-CLOSED: Mandatory schema table missing or inaccessible: ransomeye.retention_policies",
            file=sys.stderr,
        )
        print(r.stderr.strip() or r.stdout.strip(), file=sys.stderr)
        return 1

    # Upsert component and start heartbeat
    upsert_sql = """
    INSERT INTO ransomeye.components (
      component_type, component_name, instance_id, build_hash, version, started_at, last_heartbeat_at
    )
    VALUES ('db_core'::ransomeye.component_type, 'ransomeye_db_core', NULL, NULL, NULL, NOW(), NOW())
    ON CONFLICT (component_type, component_name, (COALESCE(instance_id, '')))
    DO UPDATE SET last_heartbeat_at = NOW()
    RETURNING component_id;
    """
    r = _psql(db_env, upsert_sql)
    if r.returncode != 0:
        print(f"FAIL-CLOSED: Failed to upsert components row: {r.stderr.strip() or r.stdout.strip()}", file=sys.stderr)
        return 1

    stop = {"flag": False}

    def _handle(_sig, _frame):
        stop["flag"] = True

    signal.signal(signal.SIGTERM, _handle)
    signal.signal(signal.SIGINT, _handle)

    interval_sec = int(os.environ.get("RANSOMEYE_DB_CORE_HEARTBEAT_SEC", "30"))
    if interval_sec < 5:
        print("FAIL-CLOSED: RANSOMEYE_DB_CORE_HEARTBEAT_SEC must be >= 5", file=sys.stderr)
        return 1

    while not stop["flag"]:
        hb_sql = """
        UPDATE ransomeye.components
        SET last_heartbeat_at = NOW(), updated_at = NOW()
        WHERE component_type='db_core'::ransomeye.component_type AND component_name='ransomeye_db_core';
        INSERT INTO ransomeye.component_health (component_id, observed_at, status, status_details, metrics_json)
        SELECT component_id, NOW(), 'healthy', 'running', jsonb_build_object('state','RUNNING')
        FROM ransomeye.components
        WHERE component_type='db_core'::ransomeye.component_type AND component_name='ransomeye_db_core'
        ORDER BY updated_at DESC
        LIMIT 1;
        """
        r = _psql(db_env, hb_sql)
        if r.returncode != 0:
            print(f"FAIL-CLOSED: DB core heartbeat write failed: {r.stderr.strip() or r.stdout.strip()}", file=sys.stderr)
            return 1
        time.sleep(interval_sec)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())


