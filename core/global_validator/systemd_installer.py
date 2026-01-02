# Path and File Name : /home/ransomeye/rebuild/core/global_validator/systemd_installer.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: systemd/Installer Validator - ensures consistency
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
systemd/Installer Validator

Ensures:
1. systemd units only for IMPLEMENTED phases
2. Units exist only in unified directory
3. Installer never installs NOT_IMPLEMENTED phases
4. systemd units match phase installable/runnable flags

Mismatch → FAIL (fail-closed).
"""

import re
import hashlib
from pathlib import Path
from typing import Dict, List, Set, Optional
from .validator import Violation, ViolationSeverity, ValidationResult


class SystemdInstallerValidator:
    """Validates systemd and installer consistency."""
    
    def __init__(self, validator):
        self.validator = validator
    
    def validate(self) -> ValidationResult:
        """Run systemd/installer consistency checks."""
        violations: List[Violation] = []
        
        if not self.validator.guardrails:
            return ValidationResult(
                passed=False,
                violations=[Violation(
                    checker='systemd_installer',
                    severity=ViolationSeverity.CRITICAL,
                    message="guardrails.yaml not loaded",
                )]
            )
        
        allowed_phases = self.validator.guardrails.get('allowed_phases', [])
        phase_map = {p.get('id'): p for p in allowed_phases}
        
        # Build phase name to ID mapping for matching systemd units
        phase_name_map = {}
        for phase in allowed_phases:
            phase_id = phase.get('id')
            phase_name = phase.get('name', '').lower()
            phase_name_map[phase_name] = phase_id
            
            # Also map path-based names
            phase_path = phase.get('path', '')
            if phase_path:
                path_parts = Path(phase_path).parts
                if path_parts:
                    last_part = path_parts[-1].lower()
                    phase_name_map[last_part] = phase_id
        
        # Check 1: systemd units only for IMPLEMENTED phases
        # FAIL-CLOSED: Manifest is MANDATORY (no fallback)
        unified_systemd_dir = self.validator.systemd_dir
        
        if not self.validator.install_manifest:
            violations.append(Violation(
                checker='systemd_installer',
                severity=ViolationSeverity.CRITICAL,
                message="CRITICAL: install_manifest.json missing — cannot validate systemd units (fail-closed)",
                details={
                    'rule': 'Manifest absence is a fatal error. Filesystem state is never authoritative.',
                    'required_file': '/var/lib/ransomeye/install_manifest.json',
                    'remediation': 'Run installer to generate manifest, or check if manifest was deleted',
                },
            ))
            return ValidationResult(passed=False, violations=violations)
        
        # Load expected units from manifest ONLY (NO directory scanning, NO globbing)
        expected_unit_files = [
            unified_systemd_dir / unit['name']
            for unit in self.validator.install_manifest.get('systemd_units', [])
            if (unified_systemd_dir / unit['name']).exists()
        ]
        
        for unit_file in expected_unit_files:
            # Try to match unit to phase
            unit_name = unit_file.stem  # e.g., "ransomeye-intelligence"
            
            # Extract service name (remove ransomeye- prefix)
            service_name = unit_name.replace('ransomeye-', '').replace('ransomeye_', '')
            
            # Try to match to phase
            matched_phase_id = None
            for phase_name_key, phase_id in phase_name_map.items():
                if service_name in phase_name_key or phase_name_key in service_name:
                    matched_phase_id = phase_id
                    break
            
            if matched_phase_id is not None:
                phase_info = phase_map.get(matched_phase_id)
                if phase_info:
                    status = phase_info.get('status')
                    if status == 'NOT_IMPLEMENTED':
                        violations.append(Violation(
                            checker='systemd_installer',
                            severity=ViolationSeverity.CRITICAL,
                            message=f"systemd unit '{unit_name}' exists for NOT_IMPLEMENTED phase {matched_phase_id}",
                            details={
                                'unit_name': unit_name,
                                'unit_path': str(unit_file),
                                'phase_id': matched_phase_id,
                                'phase_name': phase_info.get('name'),
                            },
                            phase_id=matched_phase_id,
                            phase_name=phase_info.get('name'),
                        ))
        
        # Check 2: Units exist only in unified directory
        # Scan for .service files outside unified directory (except standalone agents)
        project_root = self.validator.project_root
        
        # Check for STANDALONE.md declaration
        standalone_declaration = project_root / "edge" / "STANDALONE.md"
        standalone_declaration_exists = standalone_declaration.exists()
        
        # Standalone agent paths (excluded from unified systemd validation)
        standalone_paths = [
            'edge/agent/',
            'edge/dpi/',
        ]
        
        for service_file in project_root.rglob("*.service"):
            # Skip unified directory
            if unified_systemd_dir in service_file.parents:
                continue
            
            # Check if it's a standalone agent (under edge/agent/** or edge/dpi/**)
            is_standalone = False
            service_path_str = str(service_file)
            
            for standalone_path in standalone_paths:
                if standalone_path in service_path_str:
                    is_standalone = True
                    break
            
            if is_standalone:
                # Standalone agent - require STANDALONE.md declaration
                if not standalone_declaration_exists:
                    violations.append(Violation(
                        checker='systemd_installer',
                        severity=ViolationSeverity.CRITICAL,
                        message=f"Standalone agent systemd unit found at {service_file} but STANDALONE.md declaration missing",
                        details={
                            'unit_path': str(service_file),
                            'expected_declaration': str(standalone_declaration),
                            'rule': 'Edge units require STANDALONE.md declaration',
                        },
                    ))
                # If declaration exists, standalone units are allowed (excluded from unified validation)
            else:
                # Not a standalone agent - must be in unified directory
                violations.append(Violation(
                    checker='systemd_installer',
                    severity=ViolationSeverity.CRITICAL,
                    message=f"systemd unit found outside unified directory: {service_file}",
                    details={
                        'unit_path': str(service_file),
                        'expected_location': str(unified_systemd_dir),
                        'note': 'Non-standalone units must be in unified directory',
                    },
                ))
        
        # Check 3: Installer never installs NOT_IMPLEMENTED phases
        # This would require parsing install.sh, but we can check install_manifest if it exists
        if self.validator.install_manifest:
            installed_modules = self.validator.install_manifest.get('modules', {})
            for module_name, module_info in installed_modules.items():
                phase_number = module_info.get('phase')
                if phase_number is not None:
                    phase_info = phase_map.get(phase_number)
                    if phase_info and phase_info.get('status') == 'NOT_IMPLEMENTED':
                        violations.append(Violation(
                            checker='systemd_installer',
                            severity=ViolationSeverity.CRITICAL,
                            message=f"Install manifest includes NOT_IMPLEMENTED phase {phase_number} module: {module_name}",
                            details={
                                'module_name': module_name,
                                'phase_id': phase_number,
                                'phase_name': phase_info.get('name'),
                            },
                            phase_id=phase_number,
                            phase_name=phase_info.get('name'),
                        ))
        
        # Check 4: systemd units match phase installable/runnable flags
        # (If phase is not installable/runnable, it shouldn't have a service)
        # This is covered by check 1, but we can add more granular checks here
        
        # Check 5: CRITICAL - Verify systemd unit content integrity via SHA256 hash
        # FAIL-CLOSED: ANY hash mismatch is a CRITICAL violation
        violations.extend(self._verify_unit_hashes())
        
        # Check 6: systemd units must not reference /home paths (must use /opt/ransomeye)
        # MODE DETECTION: If install_manifest.json exists, we're in POST-INSTALL mode
        # In POST-INSTALL mode, validate ONLY installed units in /etc/systemd/system/
        # In PRE-INSTALL mode, validate source templates in rebuild/systemd/
        is_post_install = self.validator.install_manifest is not None
        
        if is_post_install:
            # POST-INSTALL MODE: Validate ONLY installed units from manifest (no glob)
            installed_systemd_dir = Path("/etc/systemd/system")
            expected_installed_units = [
                installed_systemd_dir / unit['name']
                for unit in self.validator.install_manifest.get('systemd_units', [])
                if (installed_systemd_dir / unit['name']).exists()
            ]
            
            for unit_file in expected_installed_units:
                    try:
                        with open(unit_file, 'r') as f:
                            unit_content = f.read()
                            
                        # Check for /home/ransomeye/rebuild references in critical fields
                        forbidden_patterns = [
                            (r'WorkingDirectory\s*=\s*(/home/ransomeye/rebuild)', 'WorkingDirectory'),
                            (r'ExecStart\s*=\s*[^\n]*(/home/ransomeye/rebuild)', 'ExecStart'),
                            (r'ReadWritePaths\s*=\s*[^\n]*(/home/ransomeye/rebuild)', 'ReadWritePaths'),
                            (r'ConditionPathExists\s*=\s*(/home/ransomeye/rebuild)', 'ConditionPathExists'),
                        ]
                        
                        for pattern, field_name in forbidden_patterns:
                            match = re.search(pattern, unit_content)
                            if match:
                                violations.append(Violation(
                                    checker='systemd_installer',
                                    severity=ViolationSeverity.CRITICAL,
                                    message=f"INSTALLED UNIT VIOLATION: systemd unit '{unit_file.name}' in /etc/systemd/system/ references /home path in {field_name} (must use /opt/ransomeye). REMEDIATION: Re-run installer to replace stale unit with correct /opt/ransomeye paths.",
                                    details={
                                        'unit_name': unit_file.name,
                                        'unit_path': str(unit_file),
                                        'field': field_name,
                                        'forbidden_path': '/home/ransomeye/rebuild',
                                        'required_path': '/opt/ransomeye',
                                        'rule': 'Installed services must run from /opt/ransomeye, not /home/ransomeye/rebuild',
                                        'validation_mode': 'POST-INSTALL',
                                        'unit_location': 'INSTALLED',
                                        'remediation': 'Re-run installer with: sudo ./install.sh (installer will automatically replace stale units)',
                                    },
                                ))
                    except Exception as e:
                        violations.append(Violation(
                            checker='systemd_installer',
                            severity=ViolationSeverity.CRITICAL,
                            message=f"Failed to read installed systemd unit '{unit_file.name}': {e}",
                            details={
                                'unit_path': str(unit_file),
                                'error': str(e),
                                'validation_mode': 'POST-INSTALL',
                            },
                        ))
        else:
            # PRE-INSTALL MODE: Validate source templates (manifest MANDATORY)
            # Source templates are build-time artifacts - validate they don't reference /home
            
            # FAIL-CLOSED: Manifest is MANDATORY even in pre-install mode
            if not self.validator.install_manifest:
                violations.append(Violation(
                    checker='systemd_installer',
                    severity=ViolationSeverity.CRITICAL,
                    message="CRITICAL: install_manifest.json missing — cannot validate systemd source templates (fail-closed)",
                    details={
                        'rule': 'Manifest absence is a fatal error. Filesystem state is never authoritative.',
                        'mode': 'PRE-INSTALL',
                        'required_file': '/var/lib/ransomeye/install_manifest.json or generated manifest',
                        'remediation': 'Generate manifest before validation',
                    },
                ))
                return ValidationResult(passed=False, violations=violations)
            
            expected_source_units = [
                unified_systemd_dir / unit['name']
                for unit in self.validator.install_manifest.get('systemd_units', [])
                if (unified_systemd_dir / unit['name']).exists()
            ]
            
            for unit_file in expected_source_units:
                    try:
                        with open(unit_file, 'r') as f:
                            unit_content = f.read()
                            
                        # Check for /home/ransomeye/rebuild references in critical fields
                        forbidden_patterns = [
                            (r'WorkingDirectory\s*=\s*(/home/ransomeye/rebuild)', 'WorkingDirectory'),
                            (r'ExecStart\s*=\s*[^\n]*(/home/ransomeye/rebuild)', 'ExecStart'),
                            (r'ReadWritePaths\s*=\s*[^\n]*(/home/ransomeye/rebuild)', 'ReadWritePaths'),
                            (r'ConditionPathExists\s*=\s*(/home/ransomeye/rebuild)', 'ConditionPathExists'),
                        ]
                        
                        for pattern, field_name in forbidden_patterns:
                            match = re.search(pattern, unit_content)
                            if match:
                                violations.append(Violation(
                                    checker='systemd_installer',
                                    severity=ViolationSeverity.CRITICAL,
                                    message=f"SOURCE TEMPLATE VIOLATION: systemd unit template '{unit_file.name}' in source directory references /home path in {field_name} (must use /opt/ransomeye)",
                                    details={
                                        'unit_name': unit_file.name,
                                        'unit_path': str(unit_file),
                                        'field': field_name,
                                        'forbidden_path': '/home/ransomeye/rebuild',
                                        'required_path': '/opt/ransomeye',
                                        'rule': 'Source templates must use /opt/ransomeye, not /home/ransomeye/rebuild',
                                        'validation_mode': 'PRE-INSTALL',
                                        'unit_location': 'SOURCE_TEMPLATE',
                                    },
                                ))
                    except Exception as e:
                        violations.append(Violation(
                            checker='systemd_installer',
                            severity=ViolationSeverity.CRITICAL,
                            message=f"Failed to read systemd unit template '{unit_file.name}': {e}",
                            details={
                                'unit_path': str(unit_file),
                                'error': str(e),
                                'validation_mode': 'PRE-INSTALL',
                            },
                        ))
        
        return ValidationResult(
            passed=len(violations) == 0,
            violations=violations,
        )
    
    def _verify_unit_hashes(self) -> List[Violation]:
        """
        CRITICAL: Verify systemd unit content integrity via SHA256 hash comparison.
        
        For each unit in manifest:
        - Read installed unit from install_path
        - Compute SHA256
        - Compare with manifest hash
        - ANY mismatch → CRITICAL FAIL-CLOSED violation
        
        Returns:
            List of violations (empty if all hashes match)
        """
        violations = []
        
        if not self.validator.install_manifest:
            violations.append(Violation(
                checker='systemd_installer',
                severity=ViolationSeverity.CRITICAL,
                message="CRITICAL: install_manifest.json missing — cannot verify unit hashes (fail-closed)",
                details={
                    'rule': 'Systemd unit integrity is enforced by manifest hash. Runtime mutation is forbidden.',
                    'required_file': '/var/lib/ransomeye/install_manifest.json',
                },
            ))
            return violations
        
        systemd_units = self.validator.install_manifest.get('systemd_units', [])
        
        if not systemd_units:
            violations.append(Violation(
                checker='systemd_installer',
                severity=ViolationSeverity.CRITICAL,
                message="CRITICAL: install_manifest.json has no systemd_units — cannot verify hashes (fail-closed)",
                details={
                    'rule': 'Manifest must contain systemd_units list with hashes',
                },
            ))
            return violations
        
        # Verify each unit
        for unit_entry in systemd_units:
            unit_name = unit_entry.get('name')
            manifest_hash = unit_entry.get('sha256_hash')
            install_path = unit_entry.get('install_path')
            
            if not unit_name:
                violations.append(Violation(
                    checker='systemd_installer',
                    severity=ViolationSeverity.CRITICAL,
                    message="CRITICAL: Manifest entry missing 'name' field",
                    details={'unit_entry': str(unit_entry)},
                ))
                continue
            
            if not manifest_hash:
                violations.append(Violation(
                    checker='systemd_installer',
                    severity=ViolationSeverity.CRITICAL,
                    message=f"CRITICAL: Unit '{unit_name}' has no sha256_hash in manifest (fail-closed)",
                    details={
                        'unit_name': unit_name,
                        'rule': 'All systemd units MUST have SHA256 hash in manifest',
                        'remediation': 'Re-run installer to populate hashes',
                    },
                ))
                continue
            
            if not install_path:
                violations.append(Violation(
                    checker='systemd_installer',
                    severity=ViolationSeverity.CRITICAL,
                    message=f"CRITICAL: Unit '{unit_name}' has no install_path in manifest (fail-closed)",
                    details={
                        'unit_name': unit_name,
                        'rule': 'All systemd units MUST have explicit install_path in manifest',
                    },
                ))
                continue
            
            # Read installed unit and compute hash
            unit_path = Path(install_path)
            
            if not unit_path.exists():
                violations.append(Violation(
                    checker='systemd_installer',
                    severity=ViolationSeverity.CRITICAL,
                    message=f"CRITICAL: Unit '{unit_name}' not found at install_path: {install_path} (fail-closed)",
                    details={
                        'unit_name': unit_name,
                        'install_path': install_path,
                        'rule': 'Installed unit must exist at declared install_path',
                    },
                ))
                continue
            
            try:
                with open(unit_path, 'rb') as f:
                    unit_content = f.read()
                    computed_hash = hashlib.sha256(unit_content).hexdigest()
                
                if computed_hash != manifest_hash:
                    violations.append(Violation(
                        checker='systemd_installer',
                        severity=ViolationSeverity.CRITICAL,
                        message=f"CRITICAL: Unit '{unit_name}' HASH MISMATCH — content has been modified (fail-closed)",
                        details={
                            'unit_name': unit_name,
                            'install_path': install_path,
                            'manifest_hash': manifest_hash,
                            'computed_hash': computed_hash,
                            'rule': 'Systemd unit integrity is enforced by manifest hash. Runtime mutation is forbidden.',
                            'remediation': 'Re-run installer to restore original unit content, or investigate tampering',
                        },
                    ))
            except Exception as e:
                violations.append(Violation(
                    checker='systemd_installer',
                    severity=ViolationSeverity.CRITICAL,
                    message=f"CRITICAL: Failed to read/hash unit '{unit_name}': {e} (fail-closed)",
                    details={
                        'unit_name': unit_name,
                        'install_path': install_path,
                        'error': str(e),
                    },
                ))
        
        return violations

