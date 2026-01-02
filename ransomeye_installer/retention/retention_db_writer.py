# Path and File Name : /home/ransomeye/rebuild/ransomeye_installer/retention/retention_db_writer.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Non-interactive installer utility to materialize retention configuration into ransomeye.retention_policies (fail-closed)

"""
Retention DB Writer (PROMPT-29B support)

Purpose:
- Ensure ransomeye.retention_policies exists AND is populated from the install-time
  retention configuration file, without requiring an interactive TTY.

Hard requirements:
- Single source of truth for retention inputs: retention.txt (runtime copy under /opt/ransomeye/config)
- Idempotent: safe to run multiple times (UPSERT)
- Fail-closed on any missing prerequisite or DB write failure
"""

from __future__ import annotations

import os
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional, Tuple


def _which(name: str) -> Optional[str]:
    import shutil

    return shutil.which(name)


def _require_env(keys: Tuple[str, ...]) -> Dict[str, str]:
    missing = [k for k in keys if not os.environ.get(k)]
    if missing:
        raise RuntimeError(f"FAIL-CLOSED: Missing required environment variables: {', '.join(missing)}")
    return {k: os.environ[k] for k in keys}


def _read_kv_file(path: Path) -> Dict[str, str]:
    if not path.exists():
        raise RuntimeError(f"FAIL-CLOSED: retention config missing: {path}")
    out: Dict[str, str] = {}
    for raw in path.read_text(encoding="utf-8", errors="replace").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if "=" not in line:
            raise RuntimeError(f"FAIL-CLOSED: Invalid retention config line (missing '='): {raw!r}")
        k, v = line.split("=", 1)
        out[k.strip()] = v.strip()
    return out


def _psql(db_env: Dict[str, str], sql: str) -> subprocess.CompletedProcess:
    if _which("psql") is None:
        raise RuntimeError("FAIL-CLOSED: psql binary not found in PATH")

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


@dataclass(frozen=True)
class RetentionPlan:
    telemetry_retention_days: int
    forensic_retention_days: int


def _build_plan(cfg: Dict[str, str]) -> RetentionPlan:
    try:
        telemetry_months = int(cfg["TELEMETRY_RETENTION_MONTHS"])
        forensic_days = int(cfg["FORENSIC_RETENTION_DAYS"])
    except KeyError as e:
        raise RuntimeError(f"FAIL-CLOSED: retention.txt missing required key: {e}") from e
    except ValueError as e:
        raise RuntimeError(f"FAIL-CLOSED: retention.txt contains non-integer value: {e}") from e

    # Canonical conversion: months -> days for DB retention_policies (bounded to schema max 2555)
    telemetry_days = telemetry_months * 30
    if telemetry_days < 1 or telemetry_days > 2555:
        raise RuntimeError(f"FAIL-CLOSED: telemetry retention_days={telemetry_days} out of allowed range 1..2555")
    if forensic_days < 1 or forensic_days > 2555:
        raise RuntimeError(f"FAIL-CLOSED: forensic retention_days={forensic_days} out of allowed range 1..2555")

    return RetentionPlan(telemetry_retention_days=telemetry_days, forensic_retention_days=forensic_days)


def _eligible_tables() -> List[str]:
    # IMPORTANT: Never include immutable/protected tables.
    # The runtime retention engine will fail-closed if illegal tables are targeted.
    return [
        "linux_agent_telemetry",
        "windows_agent_telemetry",
        "dpi_probe_telemetry",
        "raw_events",
        "normalized_events",
        "correlation_graph",
        "detection_results",
        "confidence_scores",
        "model_registry",
        "model_versions",
        "inference_results",
        "shap_explanations",
        "feature_contributions",
        "llm_requests",
        "llm_responses",
        "policy_evaluations",
        "enforcement_decisions",
        "actions_taken",
        "component_health",
        "startup_events",
        "error_events",
    ]


def apply_retention_policies(db_env: Dict[str, str], plan: RetentionPlan) -> None:
    # Ensure table exists
    r = _psql(db_env, "SELECT 1 FROM information_schema.tables WHERE table_schema='ransomeye' AND table_name='retention_policies' LIMIT 1;")
    if r.returncode != 0:
        raise RuntimeError(f"FAIL-CLOSED: Cannot probe retention_policies existence: {r.stderr.strip() or r.stdout.strip()}")

    tables = _eligible_tables()
    # Upsert rows (idempotent)
    for t in tables:
        sql = f"""
        INSERT INTO ransomeye.retention_policies (table_name, retention_days, retention_enabled)
        VALUES ('ransomeye.{t}', {plan.telemetry_retention_days}, TRUE)
        ON CONFLICT (table_name) DO UPDATE
        SET retention_days = EXCLUDED.retention_days,
            retention_enabled = EXCLUDED.retention_enabled,
            updated_at = NOW();
        """
        r = _psql(db_env, sql)
        if r.returncode != 0:
            raise RuntimeError(f"FAIL-CLOSED: Failed to upsert retention policy for {t}: {r.stderr.strip() or r.stdout.strip()}")


def main() -> int:
    db_env = _require_env(("DB_HOST", "DB_PORT", "DB_NAME", "DB_USER", "DB_PASS"))
    retention_path = Path(os.environ.get("RANSOMEYE_RETENTION_CONFIG_PATH", "/opt/ransomeye/config/retention.txt"))
    cfg = _read_kv_file(retention_path)
    plan = _build_plan(cfg)
    apply_retention_policies(db_env, plan)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as e:
        print(str(e), file=sys.stderr)
        raise SystemExit(1)


