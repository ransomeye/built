# Path and File Name : /home/ransomeye/rebuild/ransomeye_installer/runtime/runtime_deployer.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Deploys RansomEye runtime files to /opt/ransomeye with proper layout and permissions

"""
Runtime Deployer: Deploys RansomEye modules and launchers to /opt/ransomeye.

Creates canonical runtime layout:
/opt/ransomeye/
  bin/          - Service launcher scripts
  lib/          - Shared libraries (if any)
  modules/      - Python modules
  config/       - Runtime configuration
  logs/         - Runtime logs (symlink to /var/log/ransomeye)
"""

import os
import sys
import shutil
from pathlib import Path
from typing import List, Set
import stat


class RuntimeDeployer:
    """Deploys RansomEye runtime to /opt/ransomeye."""
    
    RUNTIME_ROOT = Path("/opt/ransomeye")
    DEV_ROOT = Path("/home/ransomeye/rebuild")
    
    # Runtime directory structure
    RUNTIME_DIRS = {
        'bin': RUNTIME_ROOT / "bin",
        'lib': RUNTIME_ROOT / "lib",
        'modules': RUNTIME_ROOT / "modules",
        'config': RUNTIME_ROOT / "config",
        'logs': RUNTIME_ROOT / "logs",
    }
    
    def __init__(self):
        self.deployed_modules: Set[str] = set()
    
    def create_runtime_layout(self) -> None:
        """
        Create /opt/ransomeye directory structure.
        Sets ownership to ransomeye:ransomeye and permissions 750 (dirs).
        
        Fail-closed: Raises RuntimeError if any operation fails.
        Idempotent: Safe to call multiple times.
        
        UNCONDITIONAL: This method MUST create /opt/ransomeye and all subdirectories.
        Any failure raises RuntimeError which aborts installation.
        
        Raises:
            RuntimeError: If directory creation, ownership, or permissions fail
        """
        # Create runtime root first - this MUST succeed
        try:
            self.RUNTIME_ROOT.mkdir(parents=True, exist_ok=True)
        except OSError as e:
            raise RuntimeError(f"FATAL: Failed to create runtime root {self.RUNTIME_ROOT}: {e}")
        
        # HARD VALIDATION: Immediately verify runtime root was created
        if not self.RUNTIME_ROOT.exists():
            raise RuntimeError(f"FATAL: Runtime root {self.RUNTIME_ROOT} does not exist after creation attempt")
        
        # Create all runtime subdirectories
        for dir_name, dir_path in self.RUNTIME_DIRS.items():
            try:
                dir_path.mkdir(parents=True, exist_ok=True)
            except OSError as e:
                raise RuntimeError(f"FATAL: Failed to create runtime directory {dir_path}: {e}")
            
            # HARD VALIDATION: Verify directory was created
            if not dir_path.exists():
                raise RuntimeError(f"FATAL: Runtime directory {dir_path} does not exist after creation attempt")
            
            # Set ownership (must be run as root)
            if os.geteuid() == 0:
                import pwd
                import grp
                try:
                    ransomeye_uid = pwd.getpwnam('ransomeye').pw_uid
                    ransomeye_gid = grp.getgrnam('ransomeye').gr_gid
                    os.chown(dir_path, ransomeye_uid, ransomeye_gid)
                    # Also set ownership on runtime root
                    os.chown(self.RUNTIME_ROOT, ransomeye_uid, ransomeye_gid)
                except KeyError as e:
                    raise RuntimeError(f"FATAL: ransomeye user/group not found. User must be created before runtime layout creation: {e}")
                except OSError as e:
                    raise RuntimeError(f"FATAL: Failed to set ownership on {dir_path}: {e}")
            
            # Set permissions: 755 (rwxr-xr-x) for systemd WorkingDirectory compatibility
            # systemd needs read+execute for User=ransomeye to chdir
            try:
                os.chmod(dir_path, 0o755)
            except OSError as e:
                raise RuntimeError(f"FATAL: Failed to set permissions on {dir_path}: {e}")
        
        # Set permissions on runtime root: 755 (systemd WorkingDirectory requirement)
        try:
            os.chmod(self.RUNTIME_ROOT, 0o755)
        except OSError as e:
            raise RuntimeError(f"FATAL: Failed to set permissions on {self.RUNTIME_ROOT}: {e}")
        
        # FINAL VALIDATION: Verify all directories exist before returning
        if not self.RUNTIME_ROOT.exists():
            raise RuntimeError(f"FATAL: Runtime root {self.RUNTIME_ROOT} missing after layout creation")
        
        for dir_name, dir_path in self.RUNTIME_DIRS.items():
            if not dir_path.exists():
                raise RuntimeError(f"FATAL: Runtime directory {dir_path} missing after layout creation")

        # Deploy authoritative DB schema into runtime config (single source of truth).
        # This file is referenced by RANSOMEYE_SCHEMA_SQL_PATH and must exist at runtime.
        self._deploy_authoritative_db_schema()
        self._deploy_retention_config()

    def _deploy_authoritative_db_schema(self) -> None:
        """
        Deploy the certified authoritative DB schema SQL file into /usr/share/ransomeye/schema.

        Single source of truth:
        - Source: /home/ransomeye/rebuild/ransomeye_db_core/schema/schema.sql
        - Runtime: /usr/share/ransomeye/schema/ransomeye_schema.sql

        FAIL-CLOSED: Raises RuntimeError if copy/permissions fail.
        """
        src = self.DEV_ROOT / "ransomeye_db_core" / "schema" / "schema.sql"
        schema_dir = Path("/usr/share/ransomeye/schema")
        dst = schema_dir / "ransomeye_schema.sql"

        if not src.exists():
            raise RuntimeError(f"FATAL: Authoritative schema source missing: {src}")

        # Create schema directory if it doesn't exist (fail-closed)
        try:
            schema_dir.mkdir(parents=True, exist_ok=True)
        except Exception as e:
            raise RuntimeError(f"FATAL: Failed to create schema directory {schema_dir}: {e}")

        try:
            shutil.copy2(src, dst)
        except Exception as e:
            raise RuntimeError(f"FATAL: Failed to copy authoritative schema to runtime: {e}")

        # Ownership and permissions (non-secret, readable by runtime user)
        if os.geteuid() == 0:
            import pwd
            import grp

            try:
                ransomeye_uid = pwd.getpwnam("ransomeye").pw_uid
                ransomeye_gid = grp.getgrnam("ransomeye").gr_gid
                os.chown(schema_dir, ransomeye_uid, ransomeye_gid)
                os.chown(dst, ransomeye_uid, ransomeye_gid)
            except (KeyError, OSError) as e:
                raise RuntimeError(f"FATAL: Failed to set ownership on {dst}: {e}")

        try:
            os.chmod(schema_dir, 0o755)
            os.chmod(dst, 0o644)
        except OSError as e:
            raise RuntimeError(f"FATAL: Failed to set permissions on {dst}: {e}")

        if not dst.exists() or dst.stat().st_size < 1024:
            raise RuntimeError(f"FATAL: Runtime authoritative schema missing or unexpectedly small: {dst}")

    def _deploy_retention_config(self) -> None:
        """
        Deploy the runtime retention configuration into /opt/ransomeye/config.

        Source of truth:
        - Source: /home/ransomeye/rebuild/config/retention.txt
        - Runtime: /opt/ransomeye/config/retention.txt
        """
        src = self.DEV_ROOT / "config" / "retention.txt"
        dst = self.RUNTIME_DIRS["config"] / "retention.txt"

        if not src.exists():
            raise RuntimeError(f"FATAL: Retention config source missing: {src}")

        try:
            shutil.copy2(src, dst)
        except Exception as e:
            raise RuntimeError(f"FATAL: Failed to copy retention config to runtime: {e}")

        if os.geteuid() == 0:
            import pwd
            import grp

            try:
                ransomeye_uid = pwd.getpwnam("ransomeye").pw_uid
                ransomeye_gid = grp.getgrnam("ransomeye").gr_gid
                os.chown(dst, ransomeye_uid, ransomeye_gid)
            except (KeyError, OSError) as e:
                raise RuntimeError(f"FATAL: Failed to set ownership on {dst}: {e}")

        try:
            os.chmod(dst, 0o644)
        except OSError as e:
            raise RuntimeError(f"FATAL: Failed to set permissions on {dst}: {e}")

        if not dst.exists() or dst.stat().st_size < 10:
            raise RuntimeError(f"FATAL: Runtime retention config missing or unexpectedly small: {dst}")
    
    def deploy_module(self, module_name: str, dev_module_path: Path) -> Path:
        """
        Deploy a Python module to /opt/ransomeye/modules.
        
        Args:
            module_name: Module name (e.g., 'ransomeye_intelligence')
            dev_module_path: Path to module in dev tree
        
        Returns:
            Path to deployed module
        """
        if not dev_module_path.exists():
            raise FileNotFoundError(f"Module not found: {dev_module_path}")
        
        runtime_module_path = self.RUNTIME_DIRS['modules'] / module_name
        
        # Remove existing deployment if present
        if runtime_module_path.exists():
            shutil.rmtree(runtime_module_path)
        
        # Copy module directory
        shutil.copytree(dev_module_path, runtime_module_path, 
                       ignore=shutil.ignore_patterns('__pycache__', '*.pyc', '.git', '*.md', 'tests'))
        
        # Set ownership and permissions
        if os.geteuid() == 0:
            import pwd
            import grp
            try:
                ransomeye_uid = pwd.getpwnam('ransomeye').pw_uid
                ransomeye_gid = grp.getgrnam('ransomeye').gr_gid
                
                # Set ownership recursively
                for root, dirs, files in os.walk(runtime_module_path):
                    os.chown(root, ransomeye_uid, ransomeye_gid)
                    for d in dirs:
                        os.chown(Path(root) / d, ransomeye_uid, ransomeye_gid)
                    for f in files:
                        os.chown(Path(root) / f, ransomeye_uid, ransomeye_gid)
            except (KeyError, OSError) as e:
                raise RuntimeError(f"Failed to set ownership on {runtime_module_path}: {e}")
        
        # Set directory permissions: 750
        # Set file permissions: 640 (Python files), 550 (executables)
        for root, dirs, files in os.walk(runtime_module_path):
            os.chmod(Path(root), 0o750)
            for f in files:
                file_path = Path(root) / f
                # Check if file is executable (has shebang or .sh extension)
                if f.endswith('.sh') or (f.endswith('.py') and self._is_executable_python(file_path)):
                    os.chmod(file_path, 0o550)
                else:
                    os.chmod(file_path, 0o640)
        
        self.deployed_modules.add(module_name)
        return runtime_module_path
    
    def _is_executable_python(self, file_path: Path) -> bool:
        """Check if Python file has executable shebang."""
        try:
            with open(file_path, 'rb') as f:
                first_line = f.readline()
                return first_line.startswith(b'#!/usr/bin/env python') or first_line.startswith(b'#!/usr/bin/python')
        except:
            return False
    
    def create_launcher(self, module_name: str) -> Path:
        """
        Create launcher script in /opt/ransomeye/bin.
        
        Args:
            module_name: Module name (e.g., 'ransomeye_intelligence')
        
        Returns:
            Path to launcher script
        """
        launcher_name = module_name.replace('ransomeye_', 'ransomeye-')
        launcher_path = self.RUNTIME_DIRS['bin'] / launcher_name
        
        # Create launcher script
        launcher_content = f"""#!/usr/bin/env python3
# Path and File Name : /opt/ransomeye/bin/{launcher_name}
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Launcher script for {module_name} service

\"\"\"
RansomEye Service Launcher: {module_name}

This launcher ensures proper Python path and environment for runtime execution.
\"\"\"

import sys
import os
from pathlib import Path

# Add runtime modules to Python path
RUNTIME_ROOT = Path("/opt/ransomeye")
sys.path.insert(0, str(RUNTIME_ROOT / "modules"))

# Set runtime environment
os.environ['RANSOMEYE_ROOT'] = str(RUNTIME_ROOT)
os.environ['PYTHONUNBUFFERED'] = '1'

# Import and run module
try:
    from {module_name} import main
    if __name__ == '__main__':
        sys.exit(main() if callable(main) else 0)
except ImportError as e:
    print(f"ERROR: Failed to import {module_name}: {{e}}", file=sys.stderr)
    sys.exit(1)
except Exception as e:
    print(f"ERROR: Failed to start {module_name}: {{e}}", file=sys.stderr)
    sys.exit(1)
"""
        
        with open(launcher_path, 'w') as f:
            f.write(launcher_content)
        
        # Set ownership
        if os.geteuid() == 0:
            import pwd
            import grp
            try:
                ransomeye_uid = pwd.getpwnam('ransomeye').pw_uid
                ransomeye_gid = grp.getgrnam('ransomeye').gr_gid
                os.chown(launcher_path, ransomeye_uid, ransomeye_gid)
            except (KeyError, OSError) as e:
                raise RuntimeError(f"Failed to set ownership on {launcher_path}: {e}")
        
        # Set executable permissions: 550
        os.chmod(launcher_path, 0o550)
        
        return launcher_path
    
    def deploy_all_modules(self, module_names: List[str]) -> List[Path]:
        """
        Deploy all specified modules to runtime.
        
        Args:
            module_names: List of module names to deploy
        
        Returns:
            List of deployed module paths
        """
        # Create runtime layout first
        self.create_runtime_layout()
        
        deployed_paths = []

        # Deploy supporting runtime dependencies (non-service Python modules) that
        # service modules import at runtime. These MUST be present under /opt/ransomeye/modules
        # but MUST NOT create systemd launchers.
        self._deploy_supporting_modules()
        
        for module_name in module_names:
            dev_module_path = self.DEV_ROOT / module_name
            
            if not dev_module_path.exists():
                print(f"WARNING: Module {module_name} not found in dev tree, skipping", file=sys.stderr)
                continue
            
            # Deploy module
            runtime_path = self.deploy_module(module_name, dev_module_path)
            deployed_paths.append(runtime_path)
            
            # Create launcher
            launcher_path = self.create_launcher(module_name)
            print(f"✓ Deployed {module_name} -> {runtime_path}")
            print(f"  Launcher: {launcher_path}")
        
        return deployed_paths

    def _deploy_supporting_modules(self) -> None:
        """
        Deploy non-service dependency modules required by service modules.

        FAIL-CLOSED if a required dependency module is missing.
        """
        required = [
            "ransomeye_trust",
            "ransomeye_guardrails",
        ]
        for module_name in required:
            dev_path = self.DEV_ROOT / module_name
            if not dev_path.exists():
                raise RuntimeError(f"FATAL: Required dependency module missing from dev tree: {dev_path}")
            self.deploy_module(module_name, dev_path)
    
    def validate_runtime_layout(self) -> bool:
        """
        Validate that runtime layout exists and has correct permissions.
        Fail-closed: returns False if validation fails.
        
        Returns:
            True if valid, False otherwise
        """
        # Check runtime root exists
        if not self.RUNTIME_ROOT.exists():
            print(f"ERROR: Runtime root does not exist: {self.RUNTIME_ROOT}", file=sys.stderr)
            return False
        
        # Check all required directories exist
        for dir_name, dir_path in self.RUNTIME_DIRS.items():
            if not dir_path.exists():
                print(f"ERROR: Runtime directory missing: {dir_path}", file=sys.stderr)
                return False
            
            # Check permissions (should be 750)
            stat_info = os.stat(dir_path)
            mode = stat_info.st_mode & 0o777
            if mode != 0o750:
                print(f"WARNING: Runtime directory {dir_path} has incorrect permissions: {oct(mode)} (expected 750)", file=sys.stderr)
        
        return True


def main():
    """CLI entry point for runtime deployer."""
    import argparse
    
    parser = argparse.ArgumentParser(description='Deploy RansomEye runtime to /opt/ransomeye')
    parser.add_argument('--modules', nargs='+', help='Module names to deploy')
    parser.add_argument('--validate-only', action='store_true', help='Only validate runtime layout')
    
    args = parser.parse_args()
    
    deployer = RuntimeDeployer()
    
    if args.validate_only:
        if deployer.validate_runtime_layout():
            print("✓ Runtime layout validation passed")
            sys.exit(0)
        else:
            print("✗ Runtime layout validation failed", file=sys.stderr)
            sys.exit(1)
    
    if not args.modules:
        print("ERROR: --modules required", file=sys.stderr)
        sys.exit(1)
    
    # Check root privileges
    if os.geteuid() != 0:
        print("ERROR: Runtime deployment requires root privileges", file=sys.stderr)
        sys.exit(1)
    
    try:
        deployed = deployer.deploy_all_modules(args.modules)
        print(f"\n✓ Deployed {len(deployed)} modules to {deployer.RUNTIME_ROOT}")
        sys.exit(0)
    except Exception as e:
        print(f"ERROR: Deployment failed: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()

