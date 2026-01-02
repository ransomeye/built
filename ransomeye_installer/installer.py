# Path and File Name : /home/ransomeye/rebuild/ransomeye_installer/installer.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Main installer orchestrator - validates prerequisites, enforces EULA, configures retention, generates identity, creates systemd units

"""
RansomEye Unified Installer: Main orchestrator for installation.
Validates prerequisites, enforces EULA, configures retention, generates identity.
"""

import sys
import os
from pathlib import Path
from typing import Optional, Tuple, List

from .state_manager import StateManager
from .system.os_check import OSCheck
from .system.disk_check import DiskCheck
from .system.swap_check import SwapCheck
from .system.clock_check import ClockCheck
from .retention.retention_writer import RetentionWriter
from .retention.retention_validator import RetentionValidator
from .crypto.identity_generator import IdentityGenerator
from .services.systemd_writer import SystemdWriter
from .module_resolver import ModuleResolver
from .manifest_generator import ManifestGenerator
from .runtime.runtime_deployer import RuntimeDeployer


class RansomEyeInstaller:
    """Main installer orchestrator."""
    
    VERSION = "1.0.0"
    EULA_PATH = Path("/home/ransomeye/rebuild/ransomeye_installer/eula/EULA.txt")
    
    def __init__(self):
        self.state_manager = StateManager()
        self.os_check = OSCheck()
        self.disk_check = DiskCheck()
        self.swap_check = SwapCheck()
        self.clock_check = ClockCheck()
        self.retention_writer = RetentionWriter()
        
        # ====================================================================
        # DATABASE CONFIGURATION (DB MANDATORY, HA OPTIONAL)
        # ====================================================================
        # CRITICAL ARCHITECTURE RULE: DB IS MANDATORY FOR RANSOMEYE CORE
        # - Database is ALWAYS installed and enabled
        # - Only DB_MODE is configurable: standalone (default) or ha
        # - No DB disable option exists
        #
        # FAIL-CLOSED RULES:
        # - DB_MODE must be "standalone" or "ha"
        # - If DB_MODE not provided, default to "standalone"
        # - No validation bypass, no auto-correction
        # ====================================================================
        
        # Read DB mode from environment (default to standalone if not set)
        self.db_mode = os.environ.get('RANSOMEYE_DB_MODE', 'standalone').strip()
        
        # Validate DB_MODE value
        if self.db_mode not in ['standalone', 'ha']:
            print("", file=sys.stderr)
            print("="*80, file=sys.stderr)
            print(f"FATAL: Invalid DB_MODE='{self.db_mode}'", file=sys.stderr)
            print("="*80, file=sys.stderr)
            print("", file=sys.stderr)
            print("DB_MODE must be 'standalone' or 'ha'", file=sys.stderr)
            print(f"Received: '{self.db_mode}'", file=sys.stderr)
            print("", file=sys.stderr)
            print("Installation aborted (fail-closed).", file=sys.stderr)
            print("="*80, file=sys.stderr)
            sys.exit(1)
        
        print(f"✓ Database: MANDATORY (mode={self.db_mode})")
        
        # Initialize retention validator (ALWAYS - DB mandatory)
        self.retention_validator = RetentionValidator()
        self.identity_generator = IdentityGenerator()
        
        # Initialize module resolver (validates modules exist on disk)
        self.module_resolver = ModuleResolver()
        
        # Check for phantom modules (fail-closed)
        if self.module_resolver.phantom_modules:
            print("ERROR: Phantom modules detected:", file=sys.stderr)
            for phantom in sorted(self.module_resolver.phantom_modules):
                print(f"  ✗ {phantom}", file=sys.stderr)
            print("Installation aborted (fail-closed).", file=sys.stderr)
            sys.exit(1)
        
        # Initialize systemd writer (uses module resolver)
        # CRITICAL: Pass temp output directory to prevent legacy path pollution
        import tempfile
        self._systemd_temp_dir = Path(tempfile.mkdtemp(prefix="ransomeye_systemd_"))
        self.systemd_writer = SystemdWriter(output_dir=self._systemd_temp_dir)
        self._generated_units: List[Path] = []  # Track generated units for installation
        
        # Initialize manifest generator
        self.manifest_generator = ManifestGenerator()
        
        # Initialize runtime deployer
        self.runtime_deployer = RuntimeDeployer()
    
    def _validate_prerequisites(self) -> None:
        """
        Validate all system prerequisites.
        
        Raises:
            RuntimeError: If any prerequisite check fails
        """
        # Check OS
        is_supported, os_reason = self.os_check.is_supported()
        if not is_supported:
            raise RuntimeError(f"OS check failed: {os_reason}")
        print(f"✓ {os_reason}")
        
        # Check disk
        is_available, disk_message, disk_usage = self.disk_check.check_availability()
        if not is_available:
            raise RuntimeError(f"Disk check failed: {disk_message}")
        print(f"✓ {disk_message}")
        
        # Check swap
        swap_ok, swap_message, swap_info = self.swap_check.check_swap()
        if not swap_ok:
            raise RuntimeError(f"Swap check failed: {swap_message}")
        print(f"✓ {swap_message}")
        
        # Check clock (warn only)
        clock_ok, clock_message, clock_info = self.clock_check.check_sync()
        print(f"✓ {clock_message}")
    
    def _verify_eula_acceptance(self) -> None:
        """
        Verify EULA was accepted via install.sh.
        
        CRITICAL: This method MUST NOT prompt for EULA.
        EULA acceptance is ONLY handled by install.sh.
        This method ONLY verifies the acceptance marker exists.
        
        Raises:
            RuntimeError: If EULA acceptance marker not found or invalid
        """
        eula_marker = Path("/var/lib/ransomeye/eula.accepted")
        
        # FAIL-CLOSED: Marker MUST exist
        if not eula_marker.exists():
            print("\n" + "="*80, file=sys.stderr)
            print("FATAL ERROR: EULA NOT ACCEPTED", file=sys.stderr)
            print("="*80, file=sys.stderr)
            print("", file=sys.stderr)
            print("The EULA acceptance marker is missing.", file=sys.stderr)
            print("", file=sys.stderr)
            print("EULA acceptance MUST be performed via install.sh.", file=sys.stderr)
            print("This Python installer CANNOT prompt for EULA.", file=sys.stderr)
            print("", file=sys.stderr)
            print("Expected marker: /var/lib/ransomeye/eula.accepted", file=sys.stderr)
            print("", file=sys.stderr)
            print("Installation aborted (fail-closed).", file=sys.stderr)
            print("="*80, file=sys.stderr)
            raise RuntimeError("EULA not accepted via install.sh — installation aborted")
        
        # Verify marker is readable
        try:
            import json
            with open(eula_marker, 'r') as f:
                marker_data = json.load(f)
            
            # Verify acceptance flag
            if not marker_data.get('accepted', False):
                raise RuntimeError("EULA acceptance marker exists but accepted=false")
            
            # Display verification info
            print(f"✓ EULA acceptance verified (accepted at {marker_data.get('timestamp', 'UNKNOWN')})")
            
        except json.JSONDecodeError as e:
            raise RuntimeError(f"EULA acceptance marker is corrupted: {e}")
        except Exception as e:
            raise RuntimeError(f"Failed to verify EULA acceptance marker: {e}")
    
    def _configure_retention(self) -> None:
        """
        PROMPT-24 (LOCKED): Install-time, user-configurable data retention configuration.

        Requirements enforced here:
        - Prompt operator for retention period in days
        - Default is 30 days ONLY if operator explicitly accepts default
        - Validate 1..2555 days (schema constraint)
        - Persist as one row per retention-eligible table in ransomeye.retention_policies
        - Write an immutable audit log entry proving the decision was captured
        - Fail-closed if configuration is skipped / non-interactive / invalid
        
        Raises:
            RuntimeError: If retention configuration fails
        """
        import math
        import json
        from datetime import datetime, timezone

        DEFAULT_RETENTION_DAYS = 30
        MIN_RETENTION_DAYS = 1
        MAX_RETENTION_DAYS = 2555  # 7 years (schema constraint)

        # FAIL-CLOSED: retention configuration requires an interactive operator
        if not sys.stdin.isatty():
            raise RuntimeError(
                "FAIL-CLOSED: Retention configuration is mandatory and requires an interactive TTY. "
                "Re-run the installer interactively to explicitly configure retention days."
            )

        print("\n" + "=" * 80)
        print("INSTALL-TIME DATA RETENTION CONFIGURATION (MANDATORY)")
        print("=" * 80)
        print("RansomEye will enforce data retention by purging/archive logic at runtime (NOT implemented yet).")
        print("This step configures the retention policy at install-time and persists it to the database.")
        print("")
        print(f"Default retention requirement: {DEFAULT_RETENTION_DAYS} days")
        print(f"Valid range: {MIN_RETENTION_DAYS}..{MAX_RETENTION_DAYS} days")
        print("")
        print("This configuration is mandatory and explicit:")
        print(f"  - To accept the DEFAULT ({DEFAULT_RETENTION_DAYS} days), you MUST type: DEFAULT")
        print("  - To set a custom value, enter an integer number of days")
        print("")

        retention_days: int
        while True:
            raw = input("Enter retention period (days) or type DEFAULT to accept default: ").strip()
            if not raw:
                print("✗ Retention configuration is mandatory. Empty input is not allowed.")
                continue

            if raw.upper() == "DEFAULT":
                retention_days = DEFAULT_RETENTION_DAYS
                break

            try:
                retention_days = int(raw)
            except ValueError:
                print("✗ Invalid input. Enter an integer number of days, or type DEFAULT.")
                continue

            if retention_days < MIN_RETENTION_DAYS or retention_days > MAX_RETENTION_DAYS:
                print(f"✗ Invalid retention_days={retention_days}. Must be between {MIN_RETENTION_DAYS} and {MAX_RETENTION_DAYS}.")
                continue

            break

        print("")
        print("You selected:")
        print(f"  RETENTION_DAYS = {retention_days}")
        print("")
        print("Impact summary:")
        print("  - Runtime enforcement will purge and/or archive data older than this window (later phase).")
        print("  - This will be applied system-wide across all retention-eligible tables.")
        print("")

        confirm = input(f"Type APPLY to persist retention={retention_days} days into the database now: ").strip()
        if confirm != "APPLY":
            raise RuntimeError(
                "FAIL-CLOSED: Retention configuration was not explicitly confirmed. "
                "Installation aborted. Re-run and type APPLY to confirm."
            )

        # Persist the retention to ransomeye.retention_policies and immutable_audit_log
        # (DB is mandatory for Core; fail-closed on any DB error)
        sql_upsert = r"""
WITH append_only_tables AS (
  SELECT DISTINCT n.nspname AS table_schema, c.relname AS table_name
  FROM pg_trigger t
  JOIN pg_class c ON c.oid = t.tgrelid
  JOIN pg_namespace n ON n.oid = c.relnamespace
  JOIN pg_proc p ON p.oid = t.tgfoid
  WHERE NOT t.tgisinternal
    AND p.proname = 'prevent_update_delete'
),
eligible_tables AS (
  SELECT t.table_schema, t.table_name
  FROM information_schema.tables t
  WHERE t.table_type = 'BASE TABLE'
    AND t.table_schema IN ('ransomeye', 'public')
    AND NOT (t.table_schema = 'ransomeye' AND t.table_name = 'retention_policies')
    AND NOT EXISTS (
      SELECT 1
      FROM append_only_tables a
      WHERE a.table_schema = t.table_schema AND a.table_name = t.table_name
    )
)
INSERT INTO ransomeye.retention_policies (table_name, retention_days, retention_enabled)
SELECT (eligible_tables.table_schema || '.' || eligible_tables.table_name) AS table_name,
       %(retention_days)s::int AS retention_days,
       TRUE AS retention_enabled
FROM eligible_tables
ON CONFLICT (table_name) DO UPDATE SET
  retention_days = EXCLUDED.retention_days,
  retention_enabled = TRUE,
  updated_at = now()
RETURNING table_name;
"""

        sql_audit_insert = r"""
WITH last AS (
  SELECT audit_id, chain_hash_sha256
  FROM ransomeye.immutable_audit_log
  ORDER BY created_at DESC
  LIMIT 1
),
payload AS (
  SELECT
    jsonb_build_object(
      'event', 'install_time_retention_configuration',
      'retention_days', %(retention_days)s::int,
      'applies_to', 'all_retention_eligible_tables',
      'note', 'install-time configuration only; runtime purge/archive is NOT implemented yet',
      'timestamp_utc', %(ts_utc)s::text
    ) AS payload_json
),
hashes AS (
  SELECT
    payload.payload_json,
    digest(payload.payload_json::text, 'sha256') AS payload_sha256,
    COALESCE((SELECT chain_hash_sha256 FROM last), decode(repeat('00', 32), 'hex')) AS prev_chain_hash
  FROM payload
)
INSERT INTO ransomeye.immutable_audit_log (
  actor_component_id,
  actor_agent_id,
  action,
  object_type,
  object_id,
  event_time,
  payload_json,
  payload_sha256,
  prev_audit_id,
  prev_payload_sha256,
  chain_hash_sha256,
  signature_status
)
SELECT
  NULL,
  NULL,
  'install_time_retention_configuration',
  'other'::ransomeye.trust_object_type,
  NULL,
  now(),
  hashes.payload_json,
  hashes.payload_sha256,
  (SELECT audit_id FROM last),
  NULL,
  digest(hashes.prev_chain_hash || hashes.payload_sha256, 'sha256'),
  'unknown'::ransomeye.signature_status
FROM hashes
RETURNING audit_id, chain_hash_sha256;
"""

        print("SQL to populate ransomeye.retention_policies (executed):")
        print(sql_upsert.strip())
        print("")

        # Load DB config (prefer /etc/ransomeye/db.env written by install.sh)
        db_env_path = Path("/etc/ransomeye/db.env")
        if not db_env_path.exists():
            raise RuntimeError("FAIL-CLOSED: /etc/ransomeye/db.env not found (DB is mandatory for Core).")

        db_cfg = {}
        for line in db_env_path.read_text().splitlines():
            line = line.strip()
            if not line or line.startswith("#") or "=" not in line:
                continue
            k, v = line.split("=", 1)
            db_cfg[k.strip()] = v.strip()

        required = ["DB_HOST", "DB_PORT", "DB_NAME", "DB_USER", "DB_PASS"]
        missing = [k for k in required if not db_cfg.get(k)]
        if missing:
            raise RuntimeError(f"FAIL-CLOSED: Missing required DB config keys in /etc/ransomeye/db.env: {', '.join(missing)}")

        try:
            import psycopg2
        except ImportError as e:
            raise RuntimeError(
                "FAIL-CLOSED: psycopg2 is required to persist retention configuration into the database. "
                "Ensure dependencies are installed via the unified installer."
            ) from e

        inserted_tables: list[str] = []
        audit_row: tuple[str, bytes] | None = None

        conn = psycopg2.connect(
            host=db_cfg["DB_HOST"],
            port=int(db_cfg["DB_PORT"]),
            dbname=db_cfg["DB_NAME"],
            user=db_cfg["DB_USER"],
            password=db_cfg["DB_PASS"],
            connect_timeout=10,
        )
        try:
            conn.autocommit = False
            with conn.cursor() as cur:
                cur.execute(sql_upsert, {"retention_days": retention_days})
                inserted_tables = [r[0] for r in cur.fetchall()]

                ts_utc = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
                cur.execute(sql_audit_insert, {"retention_days": retention_days, "ts_utc": ts_utc})
                audit_row = cur.fetchone()

            conn.commit()
        except Exception:
            conn.rollback()
            raise
        finally:
            conn.close()

        # Fail-closed: must configure at least one table
        if not inserted_tables:
            raise RuntimeError(
                "FAIL-CLOSED: No retention-eligible tables were discovered; retention_policies population produced zero rows."
            )

        print(f"✓ retention_policies populated: {len(inserted_tables)} row(s) upserted (retention_enabled=true)")
        print("  Example table_names:")
        for t in inserted_tables[:10]:
            print(f"    - {t}")
        if len(inserted_tables) > 10:
            print(f"    ... (+{len(inserted_tables) - 10} more)")
        print("")

        if not audit_row:
            raise RuntimeError("FAIL-CLOSED: immutable_audit_log insert did not return an audit row.")

        audit_id = audit_row[0]
        chain_hash = audit_row[1]
        chain_hash_hex = chain_hash.hex() if isinstance(chain_hash, (bytes, bytearray)) else str(chain_hash)
        print("Audit proof (immutable_audit_log):")
        print("  SQL executed (audit insert):")
        print(sql_audit_insert.strip())
        print("")
        print(f"  ✓ audit_id: {audit_id}")
        print(f"  ✓ chain_hash_sha256: {chain_hash_hex}")
        print("")

        # Maintain legacy retention.txt for existing validators (explicitly derived from retention_days)
        telemetry_months = max(1, min(84, int(math.ceil(retention_days / 30.0))))
        forensic_days = max(1, min(3650, retention_days))
        disk_percent = 80  # unchanged legacy default; runtime purge logic is not implemented in this phase

        self.retention_writer.write_retention(
            telemetry_months=telemetry_months,
            forensic_days=forensic_days,
            disk_max_percent=disk_percent,
        )

        is_valid, message = self.retention_validator.validate()
        if not is_valid:
            raise RuntimeError(f"Legacy retention.txt validation failed after write: {message}")

        cfg = self.retention_validator.get_config()
        print(f"✓ {message} (legacy retention.txt maintained for validators)")
        print(f"  TELEMETRY_RETENTION_MONTHS: {cfg.get('TELEMETRY_RETENTION_MONTHS')}")
        print(f"  FORENSIC_RETENTION_DAYS: {cfg.get('FORENSIC_RETENTION_DAYS')}")
        print(f"  DISK_MAX_USAGE_PERCENT: {cfg.get('DISK_MAX_USAGE_PERCENT')}%")
    
    def _generate_identity(self) -> None:
        """
        Generate cryptographic identity.
        
        Raises:
            RuntimeError: If identity generation fails
        """
        if self.identity_generator.identity_exists():
            print("✓ Identity already exists")
            print(f"  Identity Hash: {self.identity_generator.get_identity_hash()}")
        else:
            metadata = self.identity_generator.generate_identity()
            print("✓ Identity generated")
            print(f"  Identity Hash: {metadata['identity_hash']}")
    
    def _create_runtime_root(self) -> None:
        """
        Create /opt/ransomeye runtime root directory structure.
        
        This MUST be called BEFORE systemd unit installation to prevent CHDIR failures.
        Creates the canonical runtime layout with proper permissions and ownership.
        
        UNCONDITIONAL: This method MUST execute and MUST succeed for installation to proceed.
        Fail-closed: Any failure raises RuntimeError which aborts installation immediately.
        
        Raises:
            RuntimeError: If runtime root creation fails or validation fails
        """
        # Check root privileges (required for /opt deployment)
        if os.geteuid() != 0:
            raise RuntimeError("Runtime root creation requires root privileges. Cannot proceed without runtime root.")
        
        # EXPLICIT LOGGING: Log BEFORE runtime root creation
        print("[INSTALL] ENTERING runtime root creation")
        
        # Create runtime layout using deployer
        # This ensures /opt/ransomeye exists even if deployment fails later
        print("[INSTALL] Creating runtime root at /opt/ransomeye...")
        self.runtime_deployer.create_runtime_layout()
        
        # HARD VALIDATION: Immediately assert /opt/ransomeye exists
        # This is a fail-closed check - if this fails, installation MUST abort
        runtime_root = self.runtime_deployer.RUNTIME_ROOT
        if not runtime_root.exists():
            raise RuntimeError(f"Runtime root creation failed: {runtime_root} does not exist after creation attempt")
        
        # HARD VALIDATION: Assert bin directory exists (required for launchers)
        bin_dir = self.runtime_deployer.RUNTIME_DIRS.get('bin')
        if not bin_dir or not bin_dir.exists():
            raise RuntimeError(f"Runtime bin directory creation failed: {bin_dir} does not exist after creation attempt")
        
        # Validate that required subdirectories exist
        required_dirs = ['bin', 'modules', 'config', 'logs']
        for dir_name in required_dirs:
            dir_path = self.runtime_deployer.RUNTIME_DIRS.get(dir_name)
            if dir_path and not dir_path.exists():
                raise RuntimeError(f"Runtime directory creation failed: {dir_path} does not exist after creation attempt")
        
        # EXPLICIT LOGGING: Log AFTER runtime root creation (only if it actually exists)
        if runtime_root.exists() and bin_dir.exists():
            print(f"[INSTALL] Runtime root created at {runtime_root}")
            print(f"✓ Runtime root created: {runtime_root}")
            for dir_name, dir_path in self.runtime_deployer.RUNTIME_DIRS.items():
                if dir_path.exists():
                    print(f"  ✓ {dir_name}/ -> {dir_path}")
    
    def _validate_runtime_root_exists(self) -> bool:
        """
        Validate that /opt/ransomeye exists before systemd unit operations.
        
        Fail-closed: Returns False if runtime root is missing.
        
        Returns:
            True if runtime root exists, False otherwise
        """
        if not self.runtime_deployer.RUNTIME_ROOT.exists():
            print(f"✗ CRITICAL: Runtime root does not exist: {self.runtime_deployer.RUNTIME_ROOT}", file=sys.stderr)
            print("  This will cause systemd CHDIR failures. Aborting.", file=sys.stderr)
            return False
        
        # Check that bin directory exists (required for launcher scripts)
        if not self.runtime_deployer.RUNTIME_DIRS['bin'].exists():
            print(f"✗ CRITICAL: Runtime bin directory does not exist: {self.runtime_deployer.RUNTIME_DIRS['bin']}", file=sys.stderr)
            print("  This will cause systemd ExecStart failures. Aborting.", file=sys.stderr)
            return False
        
        return True
    
    def get_generated_units(self) -> List[Path]:
        """
        Get list of generated systemd units from last run.
        
        Returns:
            List of generated unit file paths
        """
        return self._generated_units
    
    def _deploy_runtime(self) -> None:
        """
        Deploy runtime files to /opt/ransomeye.
        
        Raises:
            RuntimeError: If runtime deployment fails
        """
        # Check root privileges (required for /opt deployment)
        if os.geteuid() != 0:
            raise RuntimeError("Runtime deployment requires root privileges")
        
        # PRECONDITION: /opt/ransomeye MUST exist (validated in step 5)
        if not self.runtime_deployer.RUNTIME_ROOT.exists():
            raise RuntimeError(f"Runtime root missing before deployment: {self.runtime_deployer.RUNTIME_ROOT}")
        
        # Get service modules to deploy
        service_modules = self.module_resolver.get_service_modules()
        
        if not service_modules:
            print("⚠ No service modules found to deploy")
            return  # Not a failure, just nothing to deploy
        
        # Deploy all modules
        deployed = self.runtime_deployer.deploy_all_modules(service_modules)
        print(f"✓ Deployed {len(deployed)} modules to /opt/ransomeye")
        
        # Validate deployment
        if not self.runtime_deployer.validate_runtime_layout():
            raise RuntimeError("Runtime layout validation failed after deployment")
        
        print("✓ Runtime layout validated")
    
    def _create_systemd_units(self) -> None:
        """
        Create systemd unit files.
        
        CRITICAL: This method assumes /opt/ransomeye already exists.
        Call _create_runtime_root() BEFORE this method.
        
        DEFENSIVE GUARD: If /opt/ransomeye does not exist, raises RuntimeError.
        
        Raises:
            RuntimeError: If runtime root does not exist or unit creation fails
        """
        # DEFENSIVE GUARD: Hard validation that runtime root exists
        # This is a fail-closed check - if /opt/ransomeye is missing, abort immediately
        runtime_root = self.runtime_deployer.RUNTIME_ROOT
        if not runtime_root.exists():
            raise RuntimeError(f"Cannot create systemd units: runtime root does not exist: {runtime_root}. This will cause systemd CHDIR failures.")
        
        # Additional validation using existing method
        if not self._validate_runtime_root_exists():
            raise RuntimeError("Cannot create systemd units: runtime root validation failed")
        
        # Generate units and store list for installation
        self._generated_units = self.systemd_writer.write_service_units()
        print(f"✓ Generated {len(self._generated_units)} systemd service units")
        print("  Note: Services are disabled by default and will not auto-start")
        print("  Runtime: Services use /opt/ransomeye (not /home/ransomeye/rebuild)")
    
    def run(self) -> None:
        """
        SINGLE AUTHORITATIVE INSTALLER ENTRYPOINT.
        
        Executes ALL installation steps sequentially and unconditionally.
        NO early returns, NO sys.exit(0), NO exception swallowing.
        All failures raise RuntimeError which propagates to top-level.
        
        Raises:
            RuntimeError: If any installation step fails
        """
        print("="*80)
        print("RANSOMEYE INSTALLER")
        print("="*80)
        print(f"Version: {self.VERSION}\n")
        
        # Step 1: Validate prerequisites
        print("[1/10] Validating prerequisites...")
        self._validate_prerequisites()
        
        # Step 2: Verify EULA acceptance (MUST be accepted via install.sh)
        print("\n[2/10] Verifying EULA acceptance...")
        self._verify_eula_acceptance()
        
        # Step 3: Configure retention
        print("\n[3/10] Configuring retention...")
        self._configure_retention()
        
        # Step 4: Generate identity
        print("\n[4/10] Generating cryptographic identity...")
        self._generate_identity()
        
        # Step 5: Create runtime root directory structure
        # CRITICAL: This MUST happen before systemd unit creation to prevent CHDIR failures
        # UNCONDITIONAL: This step MUST execute and MUST succeed
        print("\n[5/10] Creating runtime root directory structure...")
        self._create_runtime_root()
        
        # HARD VALIDATION: If runtime root does not exist immediately after step 5, ABORT
        if not self.runtime_deployer.RUNTIME_ROOT.exists():
            raise RuntimeError(f"FATAL: Runtime root does not exist after creation step: {self.runtime_deployer.RUNTIME_ROOT}. Installation aborted.")
        
        # Step 6: Deploy runtime files to /opt/ransomeye
        # PRECONDITION: /opt/ransomeye MUST exist (validated in step 5)
        print("\n[6/10] Deploying runtime files to /opt/ransomeye...")
        self._deploy_runtime()
        
        # Step 7: Create systemd units
        # Validation ensures /opt/ransomeye exists before unit creation
        # NOTE: Units are generated but NOT installed here
        # Installation happens later in install.sh after manifest generation
        print("\n[7/10] Creating systemd units...")
        self._create_systemd_units()
        
        # Step 8: Save install state (manifest generated by install.sh)
        print("\n[8/10] Saving installation state...")
        state = self.state_manager.create_state(
            version=self.VERSION,
            eula_accepted=True,
            retention_configured=True,
            identity_generated=True,
            state='INSTALLED'
        )
        self.state_manager.save_state(state)
        print("✓ Installation state saved and signed")
        
        # Step 9: Summary
        print("\n[9/9] Python installer complete!")
        print("\n" + "="*80)
        print("PYTHON INSTALLER SUMMARY")
        print("="*80)
        print("✓ Prerequisites validated")
        print("✓ EULA acceptance verified (accepted via install.sh)")
        print("✓ Retention configured")
        print("✓ Cryptographic identity generated")
        print("✓ Runtime root created at /opt/ransomeye")
        print("✓ Runtime files deployed to /opt/ransomeye")
        print("✓ Systemd units generated (installation by install.sh)")
        print("✓ Installation state saved")
        print("\nNote: Systemd unit installation and manifest generation continue in install.sh")
        
        # Display installed modules
        service_modules = self.module_resolver.get_service_modules()
        standalone_modules = self.module_resolver.get_standalone_modules()
        
        print(f"\nInstalled modules:")
        print(f"  Service modules: {len(service_modules)}")
        for module in service_modules:
            print(f"    ✓ {module}")
        
        if standalone_modules:
            print(f"\n  Standalone agents (not installed by main installer): {len(standalone_modules)}")
            for module in standalone_modules:
                print(f"    ⚠ {module} (use dedicated installer)")
        
        print("\nNext steps:")
        print("  1. Enable services: sudo systemctl enable ransomeye-*")
        print("  2. Start services: sudo systemctl start ransomeye-*")
        print("  3. Check status: sudo systemctl status ransomeye-*")
        print("="*80)
    
    def install(self) -> bool:
        """
        Legacy entrypoint for backward compatibility.
        Delegates to run() and converts exceptions to return values.
        
        Returns:
            True if installation successful, False otherwise
        """
        try:
            self.run()
            return True
        except RuntimeError as e:
            print(f"✗ {e}", file=sys.stderr)
            return False
        except Exception as e:
            print(f"✗ Fatal error during installation: {e}", file=sys.stderr)
            return False


def main():
    """
    CLI entry point for installer.
    
    Uses single authoritative run() method. All failures raise RuntimeError
    which propagates to top-level and triggers rollback via install.sh trap.
    """
    installer = RansomEyeInstaller()
    
    try:
        # Use single authoritative run() method - NO early returns, NO sys.exit(0)
        installer.run()
        # Only exit 0 if run() completes without raising exception
        sys.exit(0)
    except KeyboardInterrupt:
        print("\n\nInstallation cancelled by user.", file=sys.stderr)
        sys.exit(1)
    except RuntimeError as e:
        # RuntimeError indicates installation failure - propagate to trigger rollback
        print(f"\n✗ Installation failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        # Any other exception is a fatal error
        print(f"\n✗ Fatal error during installation: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == '__main__':
    main()

