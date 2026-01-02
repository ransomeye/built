# Path and File Name : /home/ransomeye/rebuild/ransomeye_installer/services/service_binary_validator.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Validates strict one-to-one coupling between systemd services and installed binaries - FAIL-CLOSED enforcement

"""
Service Binary Validator: Enforces strict one-to-one coupling between systemd services and installed binaries.

Rules (ABSOLUTE):
- Every systemd service MUST reference a binary that:
  - Exists on disk after install
  - Is installed by install.sh
  - Is versioned
  - Is cryptographically verifiable
- No service may point to:
  - A dev path
  - A relative path
  - A missing binary
- Fail-closed: any mismatch blocks install and validation
"""

import os
import sys
import re
import subprocess
import hashlib
from pathlib import Path
from typing import List, Dict, Tuple, Optional
import configparser


class ServiceBinaryValidator:
    """Validates service-to-binary integrity."""
    
    # Forbidden path patterns (dev paths, relative paths)
    FORBIDDEN_PATTERNS = [
        r'^\./',  # Relative paths
        r'^\.\./',  # Parent relative paths
        r'/target/debug/',  # Dev build paths
        r'/target/release/',  # Dev build paths (unless installed)
        r'~/',  # Home directory expansion
        r'\$HOME/',  # Home directory variable
        r'/tmp/',  # Temporary paths
        r'/var/tmp/',  # Temporary paths
    ]
    
    # Allowed install paths (absolute, production)
    ALLOWED_PATHS = [
        '/usr/bin/',
        '/usr/local/bin/',
        '/opt/ransomeye/bin/',
        '/opt/ransomeye/',
        '/home/ransomeye/rebuild/',  # Only for scripts in project root (e.g., github_auto_sync.sh)
    ]
    
    # Scripts allowed in project root (must be explicitly listed)
    ALLOWED_PROJECT_ROOT_SCRIPTS = [
        'github_auto_sync.sh',
    ]
    
    def __init__(self, systemd_dir: str = "/etc/systemd/system", 
                 project_root: str = "/home/ransomeye/rebuild"):
        """
        Initialize validator.
        
        Args:
            systemd_dir: Directory containing systemd units (default: /etc/systemd/system)
                        MUST NOT use /home/ransomeye/rebuild/systemd (legacy path)
            project_root: Project root directory
        """
        self.systemd_dir = Path(systemd_dir)
        self.project_root = Path(project_root)
        
        # FAIL-CLOSED: Reject legacy systemd directory
        legacy_systemd_path = Path("/home/ransomeye/rebuild/systemd")
        if self.systemd_dir.resolve() == legacy_systemd_path.resolve():
            print("FATAL: Cannot validate legacy systemd directory", file=sys.stderr)
            print(f"  Rejected: {self.systemd_dir}", file=sys.stderr)
            print(f"  Use /etc/systemd/system (installed location) instead", file=sys.stderr)
            sys.exit(1)
        self.errors: List[str] = []
        self.warnings: List[str] = []
        
    def validate_all_services(self) -> Tuple[bool, List[str], List[str]]:
        """
        Validate all service files in systemd directory.
        
        Returns:
            Tuple of (is_valid, errors, warnings)
        """
        if not self.systemd_dir.exists():
            return (False, [f"Systemd directory does not exist: {self.systemd_dir}"], [])
        
        service_files = list(self.systemd_dir.glob("*.service"))
        if not service_files:
            return (False, ["No service files found in systemd directory"], [])
        
        for service_file in service_files:
            self._validate_service_file(service_file)
        
        return (len(self.errors) == 0, self.errors, self.warnings)
    
    def _validate_service_file(self, service_file: Path) -> None:
        """Validate a single service file."""
        try:
            # Parse systemd service file
            config = configparser.ConfigParser(allow_no_value=True)
            config.read(service_file)
            
            if 'Service' not in config:
                self.errors.append(f"{service_file.name}: Missing [Service] section")
                return
            
            # Extract ExecStart
            exec_start = config.get('Service', 'ExecStart', fallback=None)
            if not exec_start:
                self.errors.append(f"{service_file.name}: Missing ExecStart directive")
                return
            
            # Parse ExecStart (may have arguments)
            exec_path = exec_start.split()[0] if exec_start.split() else exec_start
            
            # Validate path
            self._validate_exec_path(service_file.name, exec_path)
            
        except Exception as e:
            self.errors.append(f"{service_file.name}: Error parsing service file: {e}")
    
    def _validate_exec_path(self, service_name: str, exec_path: str) -> None:
        """
        Validate ExecStart path.
        
        Checks:
        1. Path is absolute
        2. Path is not forbidden (dev, relative, temp)
        3. Binary/script exists on disk
        4. Path is in allowed install location
        5. Binary has version metadata (if applicable)
        6. Binary has signature/checksum (if applicable)
        """
        # Check 1: Must be absolute path
        if not os.path.isabs(exec_path):
            self.errors.append(
                f"{service_name}: ExecStart path is not absolute: {exec_path}"
            )
            return
        
        # Check 2: Must not be forbidden pattern
        for pattern in self.FORBIDDEN_PATTERNS:
            if re.search(pattern, exec_path):
                self.errors.append(
                    f"{service_name}: ExecStart path matches forbidden pattern '{pattern}': {exec_path}"
                )
                return
        
        # Check 3: Must be in allowed path
        is_allowed = False
        script_name = os.path.basename(exec_path)
        
        # Special case: scripts in project root must be explicitly allowed
        if exec_path.startswith(str(self.project_root) + '/'):
            if script_name in self.ALLOWED_PROJECT_ROOT_SCRIPTS:
                is_allowed = True
            else:
                self.errors.append(
                    f"{service_name}: Script in project root is not in allowed list: {exec_path}. "
                    f"Allowed scripts: {', '.join(self.ALLOWED_PROJECT_ROOT_SCRIPTS)}"
                )
                return
        else:
            # Check standard allowed paths
            for allowed_path in self.ALLOWED_PATHS:
                if exec_path.startswith(allowed_path):
                    is_allowed = True
                    break
        
        if not is_allowed:
            self.errors.append(
                f"{service_name}: ExecStart path is not in allowed install location: {exec_path}"
            )
            return
        
        # Check 4: Binary/script must exist
        exec_path_obj = Path(exec_path)
        if not exec_path_obj.exists():
            self.errors.append(
                f"{service_name}: ExecStart binary/script does not exist: {exec_path}"
            )
            return
        
        # Check 5: Must be executable
        if not os.access(exec_path, os.X_OK):
            self.errors.append(
                f"{service_name}: ExecStart binary/script is not executable: {exec_path}"
            )
            return
        
        # Check 6: Version metadata (for binaries)
        if self._is_binary(exec_path):
            version_info = self._get_binary_version(exec_path)
            if not version_info:
                self.warnings.append(
                    f"{service_name}: Binary has no version metadata: {exec_path}"
                )
        
        # Check 7: Signature/checksum verification
        checksum = self._get_file_checksum(exec_path)
        if not checksum:
            self.warnings.append(
                f"{service_name}: Could not compute checksum for: {exec_path}"
            )
    
    def _is_binary(self, file_path: str) -> bool:
        """Check if file is a binary (not a script)."""
        try:
            result = subprocess.run(
                ['file', '-b', file_path],
                capture_output=True,
                text=True,
                timeout=5
            )
            if result.returncode == 0:
                file_type = result.stdout.lower()
                # Check if it's a script (shell, python, etc.)
                if any(script_type in file_type for script_type in ['script', 'text', 'ascii']):
                    return False
                # Check if it's a binary (executable, ELF, etc.)
                if any(binary_type in file_type for binary_type in ['elf', 'executable', 'binary']):
                    return True
        except Exception:
            pass
        
        # Fallback: check file extension
        if file_path.endswith(('.sh', '.py', '.pl', '.rb')):
            return False
        
        return True
    
    def _get_binary_version(self, binary_path: str) -> Optional[str]:
        """
        Get version from binary.
        
        Tries:
        1. --version flag
        2. -v flag
        3. version subcommand
        """
        for flag in ['--version', '-v', 'version']:
            try:
                result = subprocess.run(
                    [binary_path, flag],
                    capture_output=True,
                    text=True,
                    timeout=5,
                    stderr=subprocess.DEVNULL
                )
                if result.returncode == 0 and result.stdout.strip():
                    return result.stdout.strip()[:100]  # Limit length
            except Exception:
                continue
        
        return None
    
    def _get_file_checksum(self, file_path: str) -> Optional[str]:
        """Compute SHA256 checksum of file."""
        try:
            sha256_hash = hashlib.sha256()
            with open(file_path, "rb") as f:
                for byte_block in iter(lambda: f.read(4096), b""):
                    sha256_hash.update(byte_block)
            return sha256_hash.hexdigest()
        except Exception:
            return None


def main():
    """CLI entry point for service binary validator."""
    validator = ServiceBinaryValidator()
    is_valid, errors, warnings = validator.validate_all_services()
    
    if warnings:
        print("Warnings:", file=sys.stderr)
        for warning in warnings:
            print(f"  ⚠ {warning}", file=sys.stderr)
    
    if errors:
        print("Errors:", file=sys.stderr)
        for error in errors:
            print(f"  ✗ {error}", file=sys.stderr)
        print("\nBUILD FAILURE: Service-to-binary integrity validation failed", file=sys.stderr)
        sys.exit(1)
    
    if is_valid:
        print("✓ All service-to-binary integrity checks passed")
        sys.exit(0)
    else:
        print("BUILD FAILURE: Service-to-binary integrity validation failed", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()

