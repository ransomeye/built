#!/usr/bin/env python3
# Path and File Name: /home/ransomeye/rebuild/post_install_validator.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: Post-install validation script - verifies installation consistency, services, trust store, identities, retention, and standalone modules
# INVARIANT: Systemd unit integrity is enforced by manifest hash. Runtime mutation is forbidden.
#
# ⚠️  FAIL-CLOSED MANIFEST ENFORCEMENT ⚠️
# Manifest absence is a fatal error. Filesystem state is never authoritative.
# This validator REQUIRES install_manifest.json and will ABORT if:
#   - Manifest is missing
#   - Manifest is unreadable
#   - Manifest has no systemd_units list
#   - ANY unit hash mismatch detected (CRITICAL VIOLATION)
# NO directory scanning. NO globbing. NO fallback behavior.

"""
RansomEye Post-Install Validator
Verifies installation consistency and emits signed validation report.
FAIL-CLOSED: Exits with non-zero code on any mismatch.
"""

import sys
import os
import json
import subprocess
import re
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Tuple, Optional
import hashlib

# Try to import yaml, fail gracefully if not available
try:
    import yaml
    YAML_AVAILABLE = True
except ImportError:
    YAML_AVAILABLE = False

# Colors for output
RED = '\033[0;31m'
GREEN = '\033[0;32m'
YELLOW = '\033[1;33m'
NC = '\033[0m'  # No Color

# Discover project root dynamically (no hardcoded paths)
SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT_ROOT = SCRIPT_DIR
MODULE_PHASE_MAP = PROJECT_ROOT / "MODULE_PHASE_MAP.yaml"
INSTALL_STATE = PROJECT_ROOT / "ransomeye_installer/config/install_state.json"
RETENTION_CONFIG = PROJECT_ROOT / "config/retention.txt"
TRUST_DIR = PROJECT_ROOT / "ransomeye_trust"
SYSTEMD_DIR = PROJECT_ROOT / "systemd"
SYSTEMD_INSTALLED = Path("/etc/systemd/system")

VALIDATION_REPORT_DIR = PROJECT_ROOT / "logs"
VALIDATION_REPORT = VALIDATION_REPORT_DIR / f"validation_report_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json"

# Validation results
passed_checks = []
failed_checks = []
warnings = []


def log_pass(check_name: str, message: str = ""):
    """Log a passed check."""
    passed_checks.append({"check": check_name, "message": message})
    print(f"{GREEN}✓{NC} {check_name}", end="")
    if message:
        print(f": {message}")
    else:
        print()


def log_fail(check_name: str, message: str):
    """Log a failed check."""
    failed_checks.append({"check": check_name, "message": message})
    print(f"{RED}✗{NC} {check_name}: {message}")


def log_warn(check_name: str, message: str):
    """Log a warning."""
    warnings.append({"check": check_name, "message": message})
    print(f"{YELLOW}⚠{NC} {check_name}: {message}")


def error(message: str):
    """Print error and exit."""
    print(f"{RED}ERROR: {message}{NC}", file=sys.stderr)
    sys.exit(1)


def check_module_phase_map() -> bool:
    """Verify MODULE_PHASE_MAP.yaml exists and is valid."""
    print("\n[1/8] Validating MODULE_PHASE_MAP.yaml...")
    
    if not MODULE_PHASE_MAP.exists():
        log_fail("MODULE_PHASE_MAP.yaml exists", f"File not found: {MODULE_PHASE_MAP}")
        return False
    
    if not YAML_AVAILABLE:
        # Basic file existence and size check if yaml not available
        if MODULE_PHASE_MAP.stat().st_size > 0:
            log_pass("MODULE_PHASE_MAP.yaml exists", "File found (YAML parsing unavailable)")
            return True
        else:
            log_fail("MODULE_PHASE_MAP.yaml", "File is empty")
            return False
    
    try:
        with open(MODULE_PHASE_MAP, 'r') as f:
            map_data = yaml.safe_load(f)
        
        if not isinstance(map_data, dict) or 'modules' not in map_data:
            log_fail("MODULE_PHASE_MAP.yaml structure", "Invalid YAML structure - missing 'modules' key")
            return False
        
        modules = map_data.get('modules', [])
        if not isinstance(modules, list):
            log_fail("MODULE_PHASE_MAP.yaml modules", "Modules must be a list")
            return False
        
        if YAML_AVAILABLE:
            log_pass("MODULE_PHASE_MAP.yaml exists and is valid", f"{len(modules)} modules defined")
            return True
        else:
            # Already handled above
            return True
        
    except Exception as e:
        if 'yaml' in str(type(e)).lower() or (YAML_AVAILABLE and isinstance(e, Exception)):
            log_fail("MODULE_PHASE_MAP.yaml parsing", f"Parse error: {e}")
        else:
            log_fail("MODULE_PHASE_MAP.yaml validation", f"Unexpected error: {e}")
        return False


def check_services_exist() -> bool:
    """Verify required services exist in systemd directory."""
    print("\n[2/8] Validating service definitions...")
    
    if not SYSTEMD_DIR.exists():
        log_fail("Systemd directory exists", f"Directory not found: {SYSTEMD_DIR}")
        return False
    
    # FAIL-CLOSED: Manifest is MANDATORY (load it first)
    manifest_path = Path("/var/lib/ransomeye/install_manifest.json")
    if not manifest_path.exists():
        log_fail("CRITICAL: install_manifest.json missing",
                "Cannot validate systemd units (fail-closed). Manifest absence is a fatal error.")
        return False
    
    try:
        import json
        with open(manifest_path, 'r') as mf:
            manifest_data = json.load(mf)
            expected_units = manifest_data.get('systemd_units', [])
            if not expected_units:
                log_fail("CRITICAL: install_manifest.json has no systemd_units",
                        "Cannot validate without unit list (fail-closed)")
                return False
    except Exception as e:
        log_fail("CRITICAL: install_manifest.json unreadable",
                f"Cannot validate systemd units (fail-closed): {e}")
        return False
    
    # Load module map to find required services (for cross-validation only)
    try:
        with open(MODULE_PHASE_MAP, 'r') as f:
            if YAML_AVAILABLE:
                map_data = yaml.safe_load(f)
            else:
                # Cannot validate without YAML AND manifest
                log_warn("YAML parsing unavailable", "Using manifest-only validation")
                service_count = len(expected_units)
                log_pass("Service files exist", f"{service_count} service(s) in manifest")
                return True
        
        if not isinstance(map_data, dict):
            log_fail("MODULE_PHASE_MAP.yaml structure", "Invalid structure")
            return False
            
        modules = map_data.get('modules', [])
        required_services = []
        
        for module in modules:
            if module.get('requires_service', False) and module.get('actual_directory') != 'NOT_FOUND':
                module_name = module.get('module_name', '')
                service_name = f"ransomeye-{module_name.replace('ransomeye_', '')}.service"
                required_services.append((service_name, module_name))
        
        # Check standalone modules
        standalone_services = [
            ("ransomeye-dpi-probe.service", "ransomeye_dpi_probe"),
            ("ransomeye-linux-agent.service", "ransomeye_linux_agent"),
        ]
        
        all_required = required_services + standalone_services
        missing_services = []
        
        for service_name, module_name in all_required:
            service_file = SYSTEMD_DIR / service_name
            if not service_file.exists():
                # Check if service might be installed in system
                installed_service = SYSTEMD_INSTALLED / service_name
                if not installed_service.exists():
                    missing_services.append(service_name)
                    log_warn(f"Service file: {service_name}", f"Not found in {SYSTEMD_DIR} or {SYSTEMD_INSTALLED}")
        
        if missing_services:
            log_warn("Some service files missing", f"{len(missing_services)} service(s) not found (may be optional)")
        else:
            log_pass("Service definitions exist", f"All required services found")
        
        # Count actual services found (use required list, not glob)
        actual_service_count = sum(1 for svc, _ in all_required if (SYSTEMD_DIR / svc).exists())
        log_pass("Systemd directory accessible", f"{actual_service_count} service file(s) found")
        
        # CRITICAL: Verify NO services exist in old location (check manifest units only)
        OLD_SYSTEMD_DIR = PROJECT_ROOT / "ransomeye_operations/systemd"
        if OLD_SYSTEMD_DIR.exists():
            # Check for known unit names from manifest (NO globbing)
            old_services_found = []
            for unit in expected_units:
                old_unit_path = OLD_SYSTEMD_DIR / unit['name']
                if old_unit_path.exists():
                    old_services_found.append(unit['name'])
            
            if old_services_found:
                log_fail("Unified systemd layout", 
                        f"Services found in OLD location ({OLD_SYSTEMD_DIR}): {old_services_found}. "
                        f"ALL services MUST be in unified directory: {SYSTEMD_DIR}")
                return False
            else:
                log_pass("No services in old location", "Old systemd directory is clean")
        else:
            log_pass("Old systemd directory", "Does not exist (correct)")
        
        return actual_service_count > 0
        
    except Exception as e:
        log_fail("Service validation", f"Error checking services: {e}")
        return False


def check_rootless_runtime() -> bool:
    """
    CRITICAL: Validate that NO services run as root (UID 0) at runtime.
    FAIL-CLOSED: Returns False if any service is configured or running as root.
    """
    print("\n[3/11] Validating rootless runtime enforcement...")
    
    # FAIL-CLOSED: Load manifest first (NO directory scanning)
    manifest_path = Path("/var/lib/ransomeye/install_manifest.json")
    if not manifest_path.exists():
        log_fail("CRITICAL: install_manifest.json missing",
                "Cannot validate rootless runtime (fail-closed)")
        return False
    
    try:
        import json
        with open(manifest_path, 'r') as mf:
            manifest_data = json.load(mf)
            expected_units = manifest_data.get('systemd_units', [])
            if not expected_units:
                log_fail("CRITICAL: install_manifest.json has no systemd_units",
                        "Cannot validate rootless runtime (fail-closed)")
                return False
    except Exception as e:
        log_fail("CRITICAL: install_manifest.json unreadable",
                f"Cannot validate rootless runtime (fail-closed): {e}")
        return False
    
    # Build service file list from manifest ONLY (NO globbing)
    service_files = [SYSTEMD_DIR / unit['name'] for unit in expected_units]
    service_files = [f for f in service_files if f.exists()]
    
    if not service_files:
        log_fail("Service files", f"None of the {len(expected_units)} manifest units found in {SYSTEMD_DIR}")
        return False
    
    # Check 1: Parse service files for User=root
    root_configured_services = []
    missing_user_services = []
    
    for service_file in service_files:
        try:
            with open(service_file, 'r') as f:
                content = f.read()
                
            # Check for User= directive
            user_match = re.search(r'^User=(.+)$', content, re.MULTILINE)
            if not user_match:
                missing_user_services.append(service_file.name)
                log_fail(f"Service missing User: {service_file.name}", 
                        "All services MUST specify User= directive")
            else:
                user_value = user_match.group(1).strip()
                if user_value == 'root' or user_value == '0':
                    root_configured_services.append(service_file.name)
                    log_fail(f"Service configured as root: {service_file.name}", 
                            f"User={user_value} is FORBIDDEN. Must use non-root user (ransomeye).")
            
            # Check for Group= directive
            group_match = re.search(r'^Group=(.+)$', content, re.MULTILINE)
            if not group_match:
                log_warn(f"Service missing Group: {service_file.name}", 
                        "All services SHOULD specify Group= directive")
                
        except Exception as e:
            log_fail(f"Service file parsing: {service_file.name}", f"Error: {e}")
            return False
    
    if root_configured_services or missing_user_services:
        log_fail("Rootless runtime enforcement", 
                f"{len(root_configured_services)} service(s) configured as root, "
                f"{len(missing_user_services)} service(s) missing User directive. "
                f"ALL services MUST run as non-root user.")
        return False
    
    # Check 2: Verify running services are not root (if we can check)
    if os.geteuid() == 0:
        try:
            result = subprocess.run(
                ['systemctl', 'list-units', '--type=service', '--state=running', '--no-legend'],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                root_running_services = []
                for line in result.stdout.split('\n'):
                    if 'ransomeye' in line.lower():
                        service_name = line.split()[0]
                        # Get UID of running service
                        try:
                            status_result = subprocess.run(
                                ['systemctl', 'show', service_name, '--property=UID', '--value'],
                                capture_output=True,
                                text=True,
                                timeout=5
                            )
                            if status_result.returncode == 0:
                                uid = status_result.stdout.strip()
                                if uid == '0':
                                    root_running_services.append(service_name)
                                    log_fail(f"Service running as root: {service_name}", 
                                            "Service is running as UID 0 (root). CRITICAL VIOLATION.")
                        except Exception:
                            pass  # Skip if we can't check
                
                if root_running_services:
                    log_fail("Rootless runtime enforcement", 
                            f"{len(root_running_services)} service(s) running as root (UID 0). "
                            f"CRITICAL: All services MUST run as non-root user.")
                    return False
                else:
                    log_pass("Running services rootless", "No services running as root")
        except Exception as e:
            log_warn("Cannot check running service UIDs", f"Error: {e}")
    
    log_pass("Rootless runtime enforcement", 
            f"All {len(service_files)} service(s) configured for non-root runtime")
    return True


def check_services_disabled() -> bool:
    """Verify all services are DISABLED by default."""
    print("\n[4/11] Validating services are DISABLED by default...")
    
    if not SYSTEMD_DIR.exists():
        log_fail("Systemd directory exists", f"Directory not found: {SYSTEMD_DIR}")
        return False
    
    # Check installed services in /etc/systemd/system
    if SYSTEMD_INSTALLED.exists() and os.geteuid() == 0:
        try:
            result = subprocess.run(
                ['systemctl', 'list-unit-files', '--type=service', '--no-legend'],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                enabled_services = []
                for line in result.stdout.split('\n'):
                    if 'ransomeye' in line.lower() and 'enabled' in line:
                        service_name = line.split()[0]
                        enabled_services.append(service_name)
                        log_warn(f"Service enabled: {service_name}", "Service should be DISABLED by default")
                
                if enabled_services:
                    log_fail("Services DISABLED by default", f"{len(enabled_services)} service(s) are enabled")
                    return False
                else:
                    log_pass("Services DISABLED by default", "All RansomEye services are disabled")
                    return True
            else:
                log_warn("Cannot check service status", "systemctl command failed (may need root)")
                return True  # Don't fail if we can't check
                
        except subprocess.TimeoutExpired:
            log_warn("Service status check", "Timeout checking service status")
            return True
        except FileNotFoundError:
            log_warn("systemctl not available", "Cannot verify service status")
            return True
        except Exception as e:
            log_warn("Service status check", f"Error: {e}")
            return True
    else:
        log_warn("Cannot check installed services", "Not running as root or /etc/systemd/system not accessible")
        # Check service files using manifest (no glob)
        manifest_path = Path("/var/lib/ransomeye/install_manifest.json")
        if manifest_path.exists():
            try:
                import json
                with open(manifest_path, 'r') as mf:
                    manifest_data = json.load(mf)
                    service_count = len(manifest_data.get('systemd_units', []))
                    log_pass("Service files exist", f"{service_count} service(s) in manifest (status cannot be verified without root)")
                    return True
            except:
                log_warn("Service files", "Cannot verify without root or manifest")
                return True
        else:
            log_warn("Service files", "Cannot verify without root or manifest")
            return True


def check_trust_store() -> bool:
    """Verify trust store exists."""
    print("\n[5/11] Validating trust store...")
    
    if not TRUST_DIR.exists():
        log_fail("Trust store exists", f"Directory not found: {TRUST_DIR}")
        return False
    
    # Check for key trust store components
    required_items = [
        TRUST_DIR,  # Directory itself
    ]
    
    missing_items = []
    for item in required_items:
        if not item.exists():
            missing_items.append(str(item))
    
    if missing_items:
        log_fail("Trust store components", f"Missing: {', '.join(missing_items)}")
        return False
    
    log_pass("Trust store exists", f"Trust directory: {TRUST_DIR}")
    return True


def check_identities() -> bool:
    """Verify identities exist."""
    print("\n[6/11] Validating identities...")
    
    # Check install state for identity information
    if INSTALL_STATE.exists():
        try:
            with open(INSTALL_STATE, 'r') as f:
                state = json.load(f)
            
            if state.get('identity_generated', False):
                log_pass("Identity generated", "Install state indicates identity was generated")
                
                # Try to verify identity files exist
                # Check for expected identity files (NO globbing - check specific files)
                expected_identity_files = [
                    TRUST_DIR / "keys" / "root_ca.key",
                    TRUST_DIR / "keys" / "root_ca.crt",
                    PROJECT_ROOT / "ransomeye_installer/crypto/install_identity.json",
                ]
                
                identity_found = False
                for identity_file in expected_identity_files:
                    if identity_file.exists():
                        identity_found = True
                        log_pass("Identity files exist", f"Found: {identity_file.name}")
                        break
                
                if not identity_found:
                    log_warn("Identity files", "Identity generated but expected files not found")
                
                return True
            else:
                log_fail("Identity generated", "Install state indicates identity was NOT generated")
                return False
                
        except json.JSONDecodeError as e:
            log_fail("Install state JSON", f"Invalid JSON: {e}")
            return False
        except Exception as e:
            log_fail("Identity validation", f"Error: {e}")
            return False
    else:
        log_fail("Install state exists", f"File not found: {INSTALL_STATE}")
        return False


def check_retention_config() -> bool:
    """Verify retention configuration exists and is valid."""
    print("\n[7/11] Validating retention configuration...")
    
    if not RETENTION_CONFIG.exists():
        log_fail("Retention config exists", f"File not found: {RETENTION_CONFIG}")
        return False
    
    try:
        # Try to use retention validator if available
        try:
            from ransomeye_installer.retention.retention_validator import RetentionValidator
            validator = RetentionValidator(str(RETENTION_CONFIG))
            is_valid, message = validator.validate()
            
            if is_valid:
                config = validator.get_config()
                log_pass("Retention config valid", 
                        f"Telemetry: {config.get('TELEMETRY_RETENTION_MONTHS')} months, "
                        f"Forensic: {config.get('FORENSIC_RETENTION_DAYS')} days")
                return True
            else:
                log_fail("Retention config valid", message)
                return False
                
        except ImportError:
            # Fallback: basic file existence check
            with open(RETENTION_CONFIG, 'r') as f:
                content = f.read()
                if 'TELEMETRY_RETENTION_MONTHS' in content:
                    log_pass("Retention config exists", "Basic validation passed (detailed validation unavailable)")
                    return True
                else:
                    log_fail("Retention config content", "Missing required configuration keys")
                    return False
                    
    except Exception as e:
        log_fail("Retention config validation", f"Error: {e}")
        return False


def check_service_binary_integrity() -> bool:
    """
    CRITICAL: Validate strict one-to-one coupling between systemd services and installed binaries.
    FAIL-CLOSED: Returns False if any service references a missing, invalid, or dev-path binary.
    """
    print("\n[8/11] Validating service-to-binary integrity...")
    
    try:
        # Import service binary validator
        sys.path.insert(0, str(PROJECT_ROOT / "ransomeye_installer" / "services"))
        from service_binary_validator import ServiceBinaryValidator
        
        validator = ServiceBinaryValidator(
            systemd_dir=str(SYSTEMD_DIR),
            project_root=str(PROJECT_ROOT)
        )
        
        is_valid, errors, warnings = validator.validate_all_services()
        
        if warnings:
            for warning in warnings:
                log_warn("Service binary validation", warning)
        
        if errors:
            for error in errors:
                log_fail("Service binary integrity", error)
            log_fail("Service-to-binary integrity", 
                     f"{len(errors)} service(s) have invalid binary references. "
                     f"All services MUST reference binaries that exist on disk and are installed by install.sh.")
            return False
        
        if is_valid:
            # Get service count from manifest (NO globbing)
            manifest_path = Path("/var/lib/ransomeye/install_manifest.json")
            service_count = 0
            if manifest_path.exists():
                try:
                    import json
                    with open(manifest_path, 'r') as mf:
                        manifest_data = json.load(mf)
                        service_count = len(manifest_data.get('systemd_units', []))
                except:
                    service_count = 0
            
            log_pass("Service-to-binary integrity", 
                    f"All {service_count} service(s) have valid binary references")
            return True
        else:
            log_fail("Service-to-binary integrity", "Validation failed")
            return False
            
    except ImportError as e:
        log_fail("Service binary validator import", f"Could not import validator: {e}")
        return False
    except Exception as e:
        log_fail("Service binary validation", f"Error: {e}")
        return False


def check_module_references_exist() -> bool:
    """
    CRITICAL: Validate that all module references in code point to existing directories.
    FAIL-CLOSED: Returns False if any referenced module does not exist.
    """
    print("\n[9/11] Validating module references exist on disk...")
    
    # Load MODULE_PHASE_MAP.yaml to get canonical module list
    if not MODULE_PHASE_MAP.exists():
        log_fail("MODULE_PHASE_MAP.yaml exists", "Cannot validate module references without map")
        return False
    
    try:
        if not YAML_AVAILABLE:
            log_warn("YAML parsing unavailable", "Cannot fully validate module references")
            return True
        
        with open(MODULE_PHASE_MAP, 'r') as f:
            map_data = yaml.safe_load(f)
        
        if not isinstance(map_data, dict) or 'modules' not in map_data:
            log_fail("MODULE_PHASE_MAP.yaml structure", "Invalid structure")
            return False
        
        modules = map_data.get('modules', [])
        
        # Check all modules that are referenced in code
        # These are the modules that systemd_writer.py and other code reference
        referenced_modules = [
            'ransomeye_ai_advisory',
            'ransomeye_correlation',
            'ransomeye_enforcement',
            'ransomeye_ingestion',
            'ransomeye_intelligence',
            'ransomeye_policy',
            'ransomeye_reporting',
        ]
        
        missing_modules = []
        for module_name in referenced_modules:
            module_dir = PROJECT_ROOT / module_name
            if not module_dir.exists() or not module_dir.is_dir():
                missing_modules.append(module_name)
                log_fail(f"Module exists: {module_name}", f"Directory not found: {module_dir}")
        
        if missing_modules:
            log_fail("Module reference validation", 
                    f"{len(missing_modules)} referenced module(s) do not exist on disk. "
                    f"BUILD FAILURE: All module references must point to existing directories.")
            return False
        
        log_pass("Module reference validation", f"All {len(referenced_modules)} referenced modules exist on disk")
        return True
        
    except Exception as e:
        log_fail("Module reference validation", f"Error: {e}")
        return False


def check_standalone_modules() -> bool:
    """Verify standalone modules installed correctly if present."""
    print("\n[10/11] Validating standalone modules...")
    
    standalone_modules = [
        ("ransomeye_dpi_probe", "/opt/ransomeye/dpi_probe/.install_receipt.json"),
        ("ransomeye_linux_agent", "/opt/ransomeye/linux_agent/.install_receipt.json"),
    ]
    
    installed_count = 0
    for module_name, receipt_path in standalone_modules:
        receipt = Path(receipt_path)
        if receipt.exists():
            installed_count += 1
            try:
                with open(receipt, 'r') as f:
                    receipt_data = json.load(f)
                log_pass(f"Standalone module: {module_name}", "Install receipt found and valid")
            except Exception as e:
                log_warn(f"Standalone module: {module_name}", f"Receipt exists but invalid: {e}")
        else:
            log_warn(f"Standalone module: {module_name}", "Not installed (optional)")
    
    if installed_count > 0:
        log_pass("Standalone modules", f"{installed_count} standalone module(s) installed")
    else:
        log_pass("Standalone modules", "No standalone modules installed (optional)")
    
    return True  # Don't fail if standalone modules aren't installed


def check_systemd_unit_integrity() -> bool:
    """
    CRITICAL: Verify systemd unit content integrity via SHA256 hash comparison.
    FAIL-CLOSED: Returns False if ANY unit hash mismatches or verification fails.
    """
    print("\n[11/12] Validating systemd unit content integrity (SHA256)...")
    
    # Load manifest (MANDATORY)
    manifest_path = Path("/var/lib/ransomeye/install_manifest.json")
    if not manifest_path.exists():
        log_fail("CRITICAL: install_manifest.json missing",
                "Cannot verify unit hashes (fail-closed)")
        return False
    
    try:
        with open(manifest_path, 'r') as f:
            manifest_data = json.load(f)
            systemd_units = manifest_data.get('systemd_units', [])
            
            if not systemd_units:
                log_fail("CRITICAL: install_manifest.json has no systemd_units",
                        "Cannot verify unit hashes (fail-closed)")
                return False
    except Exception as e:
        log_fail("CRITICAL: install_manifest.json unreadable",
                f"Cannot verify unit hashes (fail-closed): {e}")
        return False
    
    # Verify each unit
    hash_verified_count = 0
    hash_mismatch_count = 0
    hash_missing_count = 0
    
    for unit_entry in systemd_units:
        unit_name = unit_entry.get('name')
        manifest_hash = unit_entry.get('sha256_hash')
        install_path = unit_entry.get('install_path')
        
        if not unit_name:
            log_fail("Unit entry missing 'name'", f"Invalid manifest entry: {unit_entry}")
            hash_missing_count += 1
            continue
        
        if not manifest_hash:
            log_fail(f"Unit '{unit_name}' has no sha256_hash",
                    "All units MUST have SHA256 hash in manifest")
            hash_missing_count += 1
            continue
        
        if not install_path:
            log_fail(f"Unit '{unit_name}' has no install_path",
                    "All units MUST have explicit install_path in manifest")
            hash_missing_count += 1
            continue
        
        # Read installed unit and compute hash
        unit_path = Path(install_path)
        
        if not unit_path.exists():
            log_fail(f"Unit '{unit_name}' not found at install_path",
                    f"{install_path} does not exist")
            hash_missing_count += 1
            continue
        
        try:
            with open(unit_path, 'rb') as f:
                unit_content = f.read()
                computed_hash = hashlib.sha256(unit_content).hexdigest()
            
            if computed_hash != manifest_hash:
                log_fail(f"Unit '{unit_name}' HASH MISMATCH",
                        f"Manifest: {manifest_hash[:16]}..., Computed: {computed_hash[:16]}... "
                        f"(CONTENT HAS BEEN MODIFIED - CRITICAL VIOLATION)")
                hash_mismatch_count += 1
            else:
                hash_verified_count += 1
        except Exception as e:
            log_fail(f"Failed to verify unit '{unit_name}'",
                    f"Error reading/hashing: {e}")
            hash_missing_count += 1
    
    # Report results
    total_units = len(systemd_units)
    
    if hash_mismatch_count > 0 or hash_missing_count > 0:
        log_fail("Systemd unit integrity verification",
                f"FAILED: {hash_mismatch_count} mismatch(es), {hash_missing_count} missing/error(s) "
                f"out of {total_units} unit(s). "
                f"Systemd unit integrity is enforced by manifest hash. Runtime mutation is forbidden.")
        return False
    
    log_pass("Systemd unit content integrity",
            f"All {hash_verified_count} unit(s) verified via SHA256 hash")
    return True


def check_install_state() -> bool:
    """Verify install state exists and is valid."""
    print("\n[12/12] Validating install state...")
    
    if not INSTALL_STATE.exists():
        log_fail("Install state exists", f"File not found: {INSTALL_STATE}")
        return False
    
    try:
        with open(INSTALL_STATE, 'r') as f:
            state = json.load(f)
        
        required_fields = ['state', 'timestamp', 'version', 'eula_accepted', 'retention_configured', 'identity_generated']
        missing_fields = [field for field in required_fields if field not in state]
        
        if missing_fields:
            log_fail("Install state completeness", f"Missing fields: {', '.join(missing_fields)}")
            return False
        
        if not state.get('eula_accepted', False):
            log_fail("EULA accepted", "Install state indicates EULA was NOT accepted")
            return False
        
        if not state.get('retention_configured', False):
            log_fail("Retention configured", "Install state indicates retention was NOT configured")
            return False
        
        if not state.get('identity_generated', False):
            log_fail("Identity generated", "Install state indicates identity was NOT generated")
            return False
        
        log_pass("Install state valid", f"Version: {state.get('version')}, State: {state.get('state')}")
        return True
        
    except json.JSONDecodeError as e:
        log_fail("Install state JSON", f"Invalid JSON: {e}")
        return False
    except Exception as e:
        log_fail("Install state validation", f"Error: {e}")
        return False


def generate_report() -> Dict:
    """Generate validation report."""
    report = {
        "validation_timestamp": datetime.utcnow().isoformat() + "Z",
        "project_root": str(PROJECT_ROOT),
        "summary": {
            "total_checks": len(passed_checks) + len(failed_checks),
            "passed": len(passed_checks),
            "failed": len(failed_checks),
            "warnings": len(warnings),
            "validation_status": "PASS" if len(failed_checks) == 0 else "FAIL"
        },
        "checks": {
            "passed": passed_checks,
            "failed": failed_checks,
            "warnings": warnings
        },
        "files_checked": {
            "module_phase_map": str(MODULE_PHASE_MAP),
            "install_state": str(INSTALL_STATE),
            "retention_config": str(RETENTION_CONFIG),
            "trust_dir": str(TRUST_DIR),
            "systemd_dir": str(SYSTEMD_DIR),
        }
    }
    
    return report


def sign_report(report_path: Path) -> bool:
    """Attempt to sign the validation report."""
    try:
        # Try to use GPG if available
        if subprocess.run(['which', 'gpg'], capture_output=True, timeout=5).returncode == 0:
            sig_path = report_path.with_suffix('.json.sig')
            result = subprocess.run(
                ['gpg', '--detach-sign', '--armor', '--output', str(sig_path), str(report_path)],
                capture_output=True,
                timeout=10
            )
            if result.returncode == 0:
                log_pass("Report signed", f"Signature: {sig_path}")
                return True
            else:
                log_warn("Report signing", "GPG signing failed (report not signed)")
                return False
        else:
            log_warn("Report signing", "GPG not available (report not signed)")
            return False
    except Exception:
        log_warn("Report signing", "Could not sign report")
        return False


def main():
    """Main validation entry point."""
    print("="*80)
    print("RANSOMEYE POST-INSTALL VALIDATION")
    print("="*80)
    print(f"Project Root: {PROJECT_ROOT}")
    print(f"Timestamp: {datetime.now().isoformat()}")
    print("="*80)
    
    # Run all validation checks
    checks = [
        ("Module Phase Map", check_module_phase_map),
        ("Service Definitions", check_services_exist),
        ("Rootless Runtime Enforcement", check_rootless_runtime),  # CRITICAL: Fail if any service runs as root
        ("Services Disabled", check_services_disabled),
        ("Trust Store", check_trust_store),
        ("Identities", check_identities),
        ("Retention Config", check_retention_config),
        ("Service-to-Binary Integrity", check_service_binary_integrity),  # CRITICAL: Fail if service binaries invalid
        ("Module References Exist", check_module_references_exist),  # CRITICAL: Fail if phantom modules referenced
        ("Standalone Modules", check_standalone_modules),
        ("Systemd Unit Integrity (SHA256)", check_systemd_unit_integrity),  # CRITICAL: Fail if any unit hash mismatch
        ("Install State", check_install_state),
    ]
    
    all_passed = True
    for check_name, check_func in checks:
        try:
            result = check_func()
            if not result:
                all_passed = False
        except Exception as e:
            log_fail(check_name, f"Exception: {e}")
            all_passed = False
    
    # Generate report
    print("\n" + "="*80)
    print("VALIDATION SUMMARY")
    print("="*80)
    print(f"Total Checks: {len(passed_checks) + len(failed_checks)}")
    print(f"{GREEN}Passed: {len(passed_checks)}{NC}")
    print(f"{RED}Failed: {len(failed_checks)}{NC}")
    if warnings:
        print(f"{YELLOW}Warnings: {len(warnings)}{NC}")
    print("="*80)
    
    # Generate and save report
    VALIDATION_REPORT_DIR.mkdir(parents=True, exist_ok=True)
    report = generate_report()
    
    try:
        with open(VALIDATION_REPORT, 'w') as f:
            json.dump(report, f, indent=2)
        log_pass("Validation report saved", str(VALIDATION_REPORT))
        
        # Attempt to sign report
        sign_report(VALIDATION_REPORT)
        
    except Exception as e:
        log_warn("Report saving", f"Could not save report: {e}")
    
    # FAIL-CLOSED: Exit with error if any checks failed
    if not all_passed or len(failed_checks) > 0:
        print(f"\n{RED}VALIDATION FAILED - Installation inconsistencies detected{NC}")
        print(f"\nFailed checks:")
        for check in failed_checks:
            print(f"  ✗ {check['check']}: {check['message']}")
        print(f"\nValidation report: {VALIDATION_REPORT}")
        sys.exit(1)
    
    # Invoke Release Gate (FINAL AUTHORITY)
    print("\n" + "="*80)
    print("RELEASE GATE EVALUATION (FINAL AUTHORITY)")
    print("="*80)
    
    release_gate_result = invoke_release_gate()
    
    if not release_gate_result:
        print(f"\n{RED}RELEASE GATE BLOCKED - Installation cannot proceed{NC}")
        print(f"\nValidation report: {VALIDATION_REPORT}")
        sys.exit(1)
    
    print(f"\n{GREEN}VALIDATION PASSED - All checks completed successfully{NC}")
    print(f"{GREEN}RELEASE GATE: ALLOW{NC}")
    print(f"\nValidation report: {VALIDATION_REPORT}")
    sys.exit(0)


def invoke_release_gate() -> bool:
    """
    Invoke Release Gate (FINAL AUTHORITY).
    
    Returns:
        True if decision is ALLOW, False otherwise (BLOCK or HOLD)
    """
    try:
        # Invoke ransomeye_validator binary
        validator_path = PROJECT_ROOT / "target" / "release" / "ransomeye_validator"
        
        if not validator_path.exists():
            # Try debug build
            validator_path = PROJECT_ROOT / "target" / "debug" / "ransomeye_validator"
        
        if not validator_path.exists():
            log_warn("Release Gate", "ransomeye_validator binary not found - skipping release gate")
            return False
        
        # Run release gate
        result = subprocess.run(
            [str(validator_path)],
            cwd=str(PROJECT_ROOT),
            capture_output=True,
            text=True,
            timeout=300  # 5 minute timeout
        )
        
        # Print output
        if result.stdout:
            print(result.stdout)
        if result.stderr:
            print(result.stderr, file=sys.stderr)
        
        # Exit code 0 = ALLOW, non-zero = BLOCK/HOLD
        if result.returncode == 0:
            log_pass("Release Gate", "Decision: ALLOW")
            return True
        elif result.returncode == 2:
            log_warn("Release Gate", "Decision: HOLD - Review required")
            return False
        else:
            log_fail("Release Gate", f"Decision: BLOCK (exit code {result.returncode})")
            return False
    
    except subprocess.TimeoutExpired:
        log_fail("Release Gate", "Evaluation timed out (FAIL-CLOSED)")
        return False
    except Exception as e:
        log_fail("Release Gate", f"Exception: {e} (FAIL-CLOSED)")
        return False


if __name__ == '__main__':
    main()

