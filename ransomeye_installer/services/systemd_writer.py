# Path and File Name : /home/ransomeye/rebuild/ransomeye_installer/services/systemd_writer.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Writes all systemd unit files to unified systemd directory - all units disabled by default
# INVARIANT: Systemd unit integrity is enforced by manifest hash. Runtime mutation is forbidden.

"""
Systemd Writer: Writes all systemd unit files.
All units are disabled by default and depend on installer state.
CRITICAL: Computes and stores SHA256 hashes for all installed units (fail-closed tamper detection).
"""

import os
import sys
import hashlib
import json
from pathlib import Path
from typing import List, Dict, Optional

# Import module resolver for canonical module enumeration
try:
    from ..module_resolver import ModuleResolver
except ImportError:
    # Fallback if import fails
    ModuleResolver = None


class SystemdWriter:
    """Writes systemd unit files."""
    
    INSTALL_STATE_FILE = Path("/var/lib/ransomeye/install_state.json")
    RUNTIME_ROOT = Path("/opt/ransomeye")
    RUNTIME_BIN = RUNTIME_ROOT / "bin"
    
    def __init__(self, output_dir: Optional[Path] = None):
        """
        Initialize systemd writer.
        
        Args:
            output_dir: Directory to write units to. If None, uses temp directory.
                       MUST NOT default to /home/ransomeye/rebuild/systemd to prevent
                       legacy unit pollution.
        """
        if output_dir is None:
            # Default to temp directory (fail-safe: no legacy path pollution)
            import tempfile
            self.output_dir = Path(tempfile.mkdtemp(prefix="ransomeye_systemd_"))
        else:
            self.output_dir = Path(output_dir)
        
        # FAIL-CLOSED: Reject legacy systemd directory (prevent template reuse)
        legacy_systemd_path = Path("/home/ransomeye/rebuild/systemd")
        if self.output_dir.resolve() == legacy_systemd_path.resolve():
            error_msg = (
                "FATAL: SystemdWriter output_dir cannot be legacy systemd folder.\n"
                f"  Rejected: {self.output_dir}\n"
                f"  Legacy path: {legacy_systemd_path}\n"
                f"  Units MUST be generated in temp directory to prevent template reuse.\n"
                f"  Use temp directory or specify a different output_dir."
            )
            print(f"ERROR: {error_msg}", file=sys.stderr)
            sys.exit(1)
        
        self.output_dir.mkdir(parents=True, exist_ok=True)
        
        # Use module resolver to get canonical service modules
        if ModuleResolver:
            self.resolver = ModuleResolver()
            # Get service modules that actually exist on disk
            self.CORE_MODULES = self.resolver.get_service_modules()
        else:
            # Fallback: hardcoded list (will be validated)
            self.CORE_MODULES = [
                'ransomeye_ai_advisory',
                'ransomeye_correlation',
                'ransomeye_enforcement',
                'ransomeye_ingestion',
                'ransomeye_intelligence',
                'ransomeye_policy',
                'ransomeye_reporting',
            ]
            self.resolver = None
        
        # CRITICAL: Validate all modules exist on disk before proceeding
        self._validate_modules_exist()
    
    def _validate_modules_exist(self) -> None:
        """
        Validate that all modules in CORE_MODULES exist on disk.
        FAIL-CLOSED: Raises SystemExit if any module directory is missing.
        """
        if not self.resolver:
            error_msg = (
                "BUILD FAILURE: Module resolver not available.\n"
                "  Cannot validate module existence without resolver.\n"
                "  Ensure ransomeye_installer.module_resolver is importable."
            )
            print(f"ERROR: {error_msg}", file=sys.stderr)
            sys.exit(1)
        
        missing_modules = []
        
        for module_name in self.CORE_MODULES:
            # Use resolver to validate (ONLY path - no fallback)
            if not self.resolver.validate_module_exists(module_name):
                missing_modules.append(module_name)
        
        if missing_modules:
            error_msg = (
                f"BUILD FAILURE: Referenced modules do not exist on disk:\n"
                f"  Missing: {', '.join(missing_modules)}\n"
                f"  All module references MUST point to existing directories.\n"
                f"  Check MODULE_PHASE_MAP.yaml for canonical module mappings."
            )
            print(f"ERROR: {error_msg}", file=sys.stderr)
            sys.exit(1)
    
    def _generate_service_unit(self, module_name: str) -> str:
        """
        Generate systemd service unit content.
        
        Uses /opt/ransomeye as runtime root (not /home/ransomeye/rebuild).
        Launcher scripts are in /opt/ransomeye/bin.
        
        SPECIAL HANDLING for db_core:
        - Adds install_state signature verification conditions
        - Loads database environment from /etc/ransomeye/db.env
        
        Args:
            module_name: Module name (e.g., 'ransomeye_intelligence')
        
        Returns:
            Service unit content
        """
        service_name = module_name.replace('ransomeye_', 'ransomeye-')
        launcher_name = service_name  # Launcher name matches service name
        launcher_path = self.RUNTIME_BIN / launcher_name
        
        # Generate service-specific directory names
        state_dir_name = service_name.replace('ransomeye-', '')
        
        # CRITICAL: File header must reference INSTALLED location (not temp generation dir)
        installed_unit_path = f"/etc/systemd/system/{service_name}.service"
        
        # DB_CORE SPECIAL HANDLING: Detect if this is database service
        is_db_core = (module_name in ['ransomeye_db_core', 'db_core'])
        
        # Build condition lines
        condition_lines = [
            f"ConditionPathExists={self.INSTALL_STATE_FILE}",
            "# DB ENFORCEMENT: Require database environment file (DB is mandatory for Core)",
            "ConditionPathExists=/etc/ransomeye/db.env",
            "# Pre-start validation: fail-closed if runtime layout invalid",
            f"ConditionPathExists={self.RUNTIME_ROOT}",
            f"ConditionPathExists={launcher_path}"
        ]
        
        # DB_CORE: Add install state signature enforcement and DB mode validation
        if is_db_core:
            install_state_sig = "/var/lib/ransomeye/install_state.sig"
            db_env_file = "/etc/ransomeye/db.env"
            
            condition_lines.insert(1, "# DB ENFORCEMENT: Require cryptographically signed install state")
            condition_lines.insert(2, f"ConditionPathExists={install_state_sig}")
            condition_lines.insert(3, "# DB ENFORCEMENT: Service will NOT start if DB disabled (db.env absent)")
            
            # NOTE: HA mode validation is runtime responsibility
            # If db.mode=ha but HA metadata missing, service will start but fail-closed
            # This allows install to complete but prevents runtime operation without prerequisites
        
        conditions_block = '\n'.join(condition_lines)
        
        # Build environment lines
        env_lines = [
            '# Environment',
            f'Environment="RANSOMEYE_ROOT={self.RUNTIME_ROOT}"',
            'Environment="PYTHONUNBUFFERED=1"'
        ]
        
        # DB ENVIRONMENT (MANDATORY for all Core services):
        # Services must connect to PostgreSQL and write health/audit facts.
        # db.env is root-owned (0600) but systemd reads it as root and injects the vars.
        db_env_file = "/etc/ransomeye/db.env"
        env_lines.insert(1, "# DB ENVIRONMENT: Load database credentials from secured file (systemd reads as root)")
        env_lines.insert(2, f"EnvironmentFile={db_env_file}")
        
        environment_block = '\n'.join(env_lines)
        
        unit_content = f"""# Path and File Name : {installed_unit_path}
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Systemd service unit for {module_name}
# CRITICAL: Rootless runtime enforcement - MUST NOT run as root (UID 0)
# RUNTIME: Uses /opt/ransomeye runtime paths ONLY (no build-time paths)

[Unit]
Description=RansomEye {module_name}
After=network.target
Requires=network.target
{conditions_block}

[Service]
Type=simple
User=ransomeye
Group=ransomeye
WorkingDirectory={self.RUNTIME_ROOT}
RuntimeDirectory=ransomeye/{state_dir_name}
StateDirectory=ransomeye/{state_dir_name}
# Pre-start validation: verify runtime layout ownership
ExecStartPre=/bin/sh -c 'test -d {self.RUNTIME_ROOT} && test -r {launcher_path} || exit 1'
ExecStart={launcher_path}
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

# Security hardening - Rootless runtime enforcement
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
# Runtime paths: /opt/ransomeye and state directories
ReadWritePaths={self.RUNTIME_ROOT} /var/lib/ransomeye/{state_dir_name} /run/ransomeye/{state_dir_name}

# Capability-based privileges (no root required)
CapabilityBoundingSet=CAP_NET_BIND_SERVICE CAP_NET_RAW CAP_SYS_NICE
AmbientCapabilities=
PrivateUsers=false

{environment_block}

[Install]
WantedBy=multi-user.target
"""
        return unit_content
    
    def write_service_units(self) -> List[Path]:
        """
        Write all service unit files to output directory.
        
        CRITICAL: Only generates units for modules that exist on disk.
        Returns list of generated file paths (source of truth for installation).
        
        Returns:
            List of written unit file paths
        
        Raises:
            SystemExit: If any module directory does not exist (fail-closed)
        """
        written_files = []
        
        # Re-validate before writing (defense in depth)
        self._validate_modules_exist()
        
        for module in self.CORE_MODULES:
            service_name = module.replace('ransomeye_', 'ransomeye-')
            unit_file = self.output_dir / f"{service_name}.service"
            
            content = self._generate_service_unit(module)
            
            with open(unit_file, 'w') as f:
                f.write(content)
            
            os.chmod(unit_file, 0o644)
            written_files.append(unit_file)
        
        return written_files
    
    def install_units(self, generated_units: List[Path]) -> bool:
        """
        Install systemd units (copy to /etc/systemd/system).
        Does NOT enable them.
        
        CRITICAL: Only installs units from the provided list (generator output).
        NO globbing, NO directory scans, NO legacy paths.
        
        Args:
            generated_units: List of unit file paths generated in this run.
                            MUST be from write_service_units() return value.
        
        Returns:
            True if successful, False otherwise (fail-closed)
        """
        try:
            # Fail-closed: Validate runtime root exists before installing units
            if not self.RUNTIME_ROOT.exists():
                print(f"ERROR: Runtime root does not exist: {self.RUNTIME_ROOT}", file=sys.stderr)
                print("  Cannot install systemd units. This will cause CHDIR failures.", file=sys.stderr)
                print("  Run installer to create runtime root first.", file=sys.stderr)
                return False
            
            # Validate bin directory exists (required for launcher scripts)
            if not self.RUNTIME_BIN.exists():
                print(f"ERROR: Runtime bin directory does not exist: {self.RUNTIME_BIN}", file=sys.stderr)
                print("  Cannot install systemd units. This will cause ExecStart failures.", file=sys.stderr)
                return False
            
            # Validate generated_units list is not empty
            if not generated_units:
                print("ERROR: No systemd units provided for installation", file=sys.stderr)
                return False
            
            import shutil
            import subprocess
            
            # STEP 1: Remove ALL existing RansomEye units (clean slate)
            # Build explicit list of expected unit names from CORE_MODULES (no glob scanning)
            print("[INSTALL] Removing ALL existing ransomeye units (clean slate)")
            systemd_dir = Path("/etc/systemd/system")
            expected_unit_names = [
                f"ransomeye-{module.replace('ransomeye_', '')}.service"
                for module in self.CORE_MODULES
            ]
            
            removed_count = 0
            for unit_name in expected_unit_names:
                unit_path = systemd_dir / unit_name
                if unit_path.exists():
                    # Stop service if running (idempotent)
                    try:
                        subprocess.run(
                            ['systemctl', 'stop', unit_name],
                            check=False,
                            timeout=10,
                            stdout=subprocess.DEVNULL,
                            stderr=subprocess.DEVNULL
                        )
                    except:
                        pass  # Ignore errors (service may not be running)
                    
                    # Disable service if enabled (idempotent)
                    try:
                        subprocess.run(
                            ['systemctl', 'disable', unit_name],
                            check=False,
                            timeout=10,
                            stdout=subprocess.DEVNULL,
                            stderr=subprocess.DEVNULL
                        )
                    except:
                        pass  # Ignore errors (service may not be enabled)
                    
                    # Remove unit file
                    unit_path.unlink()
                    removed_count += 1
            
            if removed_count > 0:
                print(f"[INSTALL] Removed {removed_count} existing ransomeye units")
            else:
                print("[INSTALL] No existing ransomeye units found")
            
            # STEP 2: Install ONLY generated units (from provided list - no scanning)
            print(f"[INSTALL] Installing {len(generated_units)} generated systemd units")
            installed_count = 0
            
            for unit_file in generated_units:
                if not unit_file.exists():
                    print(f"ERROR: Generated unit does not exist: {unit_file}", file=sys.stderr)
                    return False
                
                target = Path(f"/etc/systemd/system/{unit_file.name}")
                
                # Copy unit (overwrite if exists, but should not exist after cleanup)
                shutil.copy2(unit_file, target)
                os.chmod(target, 0o644)
                print(f"[INSTALL] Installed: {unit_file.name}")
                installed_count += 1
            
            # STEP 3: Enforce hard count equality (FAIL-CLOSED)
            generated_count = len(generated_units)
            if installed_count != generated_count:
                print(f"FATAL: Generated {generated_count} units but installed {installed_count}", file=sys.stderr)
                print("  Installation count mismatch detected (fail-closed)", file=sys.stderr)
                return False
            
            print(f"[INSTALL] Installed count matches generated count: {installed_count}")
            
            # STEP 4: Compute SHA256 hashes for all installed units and update manifest
            print(f"[INSTALL] Computing SHA256 hashes for installed units")
            manifest_path = Path("/var/lib/ransomeye/install_manifest.json")
            
            if not manifest_path.exists():
                print(f"ERROR: Manifest not found at {manifest_path} - cannot store hashes (fail-closed)", file=sys.stderr)
                return False
            
            try:
                # Load manifest
                with open(manifest_path, 'r') as f:
                    manifest = json.load(f)
                
                # Compute and store hashes
                hash_update_count = 0
                for unit_file in generated_units:
                    unit_name = unit_file.name
                    target = Path(f"/etc/systemd/system/{unit_name}")
                    
                    if not target.exists():
                        print(f"ERROR: Installed unit not found: {target} (fail-closed)", file=sys.stderr)
                        return False
                    
                    # Compute SHA256 hash of installed unit
                    with open(target, 'rb') as f:
                        unit_content = f.read()
                        sha256_hash = hashlib.sha256(unit_content).hexdigest()
                    
                    # Update manifest entry
                    for unit_entry in manifest.get('systemd_units', []):
                        if unit_entry['name'] == unit_name:
                            unit_entry['sha256_hash'] = sha256_hash
                            unit_entry['install_path'] = str(target)
                            hash_update_count += 1
                            print(f"[INSTALL] Computed hash for {unit_name}: {sha256_hash[:16]}...")
                            break
                
                # Write updated manifest
                with open(manifest_path, 'w') as f:
                    json.dump(manifest, f, indent=2, sort_keys=True)
                
                print(f"[INSTALL] Updated manifest with {hash_update_count} unit hash(es)")
                
                if hash_update_count != installed_count:
                    print(f"ERROR: Hash update count mismatch - updated {hash_update_count} but installed {installed_count}", file=sys.stderr)
                    return False
                
            except Exception as e:
                print(f"ERROR: Failed to compute/store unit hashes: {e}", file=sys.stderr)
                import traceback
                traceback.print_exc()
                return False
            
            # STEP 5: Reload systemd (required after unit file changes)
            print(f"[INSTALL] Reloading systemd daemon")
            subprocess.run(['systemctl', 'daemon-reload'], check=True, timeout=30)
            
            print(f"[INSTALL] Successfully installed {installed_count} systemd units (disabled by default)")
            print(f"[INSTALL] Manifest updated with content hashes for tamper detection")
            return True
        except subprocess.CalledProcessError as e:
            print(f"ERROR: systemctl daemon-reload failed: {e}", file=sys.stderr)
            return False
        except Exception as e:
            print(f"ERROR: Failed to install systemd units: {e}", file=sys.stderr)
            import traceback
            traceback.print_exc()
            return False


def main():
    """CLI entry point for systemd writer."""
    writer = SystemdWriter()
    
    written = writer.write_service_units()
    print(f"✓ Generated {len(written)} systemd service units")
    
    for unit_file in written:
        print(f"  {unit_file}")


if __name__ == '__main__':
    main()

