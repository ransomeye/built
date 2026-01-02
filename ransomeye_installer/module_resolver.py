# Path and File Name : /home/ransomeye/rebuild/ransomeye_installer/module_resolver.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Canonical module resolver - scans disk and validates only existing modules

"""
Canonical Module Resolver: Enumerates actual modules on disk and maps them to canonical names.
Rejects anything not present physically. Fail-closed on phantom modules.
"""

import sys
import os
from pathlib import Path
from typing import List, Dict, Optional, Set
import yaml
import json


class ModuleResolver:
    """Resolves and validates modules based on actual disk presence."""
    
    PROJECT_ROOT = Path("/home/ransomeye/rebuild")
    MODULE_PHASE_MAP = PROJECT_ROOT / "MODULE_PHASE_MAP.yaml"
    GUARDRAILS_SPEC = PROJECT_ROOT / "core/guardrails/guardrails.yaml"
    
    # Standalone agents - must NOT be installed by main installer
    STANDALONE_AGENTS = {
        'ransomeye_linux_agent',
        'ransomeye_windows_agent',
        'ransomeye_dpi_probe',
    }
    
    # Modules that are tools/libraries and don't need services
    TOOL_MODULES = {
        'ransomeye_guardrails',
        'ransomeye_installer',
        'ransomeye_architecture',
        'ransomeye_retention',
        'ransomeye_trust',
        'ransomeye_governance',
        'ransomeye_validation',
    }
    
    def __init__(self):
        self.project_root = self.PROJECT_ROOT
        self.phantom_modules: Set[str] = set()
        self.existing_modules: Dict[str, Path] = {}
        self.service_modules: List[str] = []
        self.standalone_modules: List[str] = []
        self.tool_modules: List[str] = []
        
        # Load module phase map for reference
        self.module_map = self._load_module_map()
        
        # Scan disk for actual modules
        self._scan_disk_modules()
        
        # Validate against guardrails (if available)
        self._validate_against_guardrails()
    
    def _load_module_map(self) -> Dict:
        """Load MODULE_PHASE_MAP.yaml for reference."""
        if not self.MODULE_PHASE_MAP.exists():
            return {}
        
        try:
            with open(self.MODULE_PHASE_MAP, 'r') as f:
                return yaml.safe_load(f) or {}
        except Exception as e:
            print(f"Warning: Failed to load MODULE_PHASE_MAP.yaml: {e}", file=sys.stderr)
            return {}
    
    def _scan_disk_modules(self) -> None:
        """Scan project root for actual module directories."""
        if not self.project_root.exists():
            raise SystemExit(f"Project root does not exist: {self.project_root}")
        
        # Scan for ransomeye_* and core/* directories
        for item in self.project_root.iterdir():
            if not item.is_dir():
                continue
            
            name = item.name
            
            # Check for ransomeye_* modules
            if name.startswith('ransomeye_'):
                self.existing_modules[name] = item
                
                # Categorize
                if name in self.STANDALONE_AGENTS:
                    self.standalone_modules.append(name)
                elif name in self.TOOL_MODULES:
                    self.tool_modules.append(name)
                else:
                    # Assume service module if not categorized
                    self.service_modules.append(name)
            
            # Check for core/* subdirectories (Rust modules)
            elif name == 'core' and item.is_dir():
                for core_item in item.iterdir():
                    if core_item.is_dir():
                        core_name = f"core/{core_item.name}"
                        self.existing_modules[core_name] = core_item
                        # Core modules are typically libraries, not services
                        if core_name not in self.tool_modules:
                            self.tool_modules.append(core_name)
    
    def _validate_against_guardrails(self) -> None:
        """Validate modules against guardrails specification if available."""
        if not self.GUARDRAILS_SPEC.exists():
            # Guardrails not signed yet - skip validation
            return
        
        try:
            with open(self.GUARDRAILS_SPEC, 'r') as f:
                guardrails = yaml.safe_load(f)
            
            if not guardrails:
                return
            
            # Get allowed modules from guardrails
            allowed_modules = guardrails.get('allowed_modules', [])
            
            # Check for phantom modules (referenced but don't exist)
            for module_name in allowed_modules:
                if module_name not in self.existing_modules:
                    # Check if it's a core module (core/*)
                    if module_name.startswith('core/'):
                        core_path = self.project_root / module_name
                        if not core_path.exists():
                            self.phantom_modules.add(module_name)
                    else:
                        module_path = self.project_root / module_name
                        if not module_path.exists():
                            self.phantom_modules.add(module_name)
        except Exception as e:
            print(f"Warning: Failed to validate against guardrails: {e}", file=sys.stderr)
    
    def get_service_modules(self) -> List[str]:
        """
        Get list of service modules that exist on disk.
        
        Returns:
            List of module names that require systemd services
        """
        # Filter to only modules that exist and are not standalone/tools
        valid_services = []
        for module in self.service_modules:
            if module in self.existing_modules:
                valid_services.append(module)
        
        return sorted(valid_services)
    
    def get_standalone_modules(self) -> List[str]:
        """
        Get list of standalone agent modules.
        
        Returns:
            List of standalone module names
        """
        return sorted(self.standalone_modules)
    
    def validate_module_exists(self, module_name: str) -> bool:
        """
        Validate that a module exists on disk.
        
        Args:
            module_name: Module name to validate
            
        Returns:
            True if module exists, False otherwise
        """
        return module_name in self.existing_modules
    
    def reject_phantom_module(self, module_name: str) -> None:
        """
        Fail-closed: Reject a phantom module.
        
        Args:
            module_name: Module name to reject
            
        Raises:
            SystemExit: Always raises (fail-closed)
        """
        error_msg = (
            f"PHANTOM MODULE DETECTED: {module_name}\n"
            f"  This module is referenced but does not exist on disk.\n"
            f"  Installation aborted (fail-closed).\n"
            f"  Check MODULE_PHASE_MAP.yaml for canonical module mappings."
        )
        print(f"ERROR: {error_msg}", file=sys.stderr)
        sys.exit(1)
    
    def get_module_path(self, module_name: str) -> Optional[Path]:
        """
        Get the filesystem path for a module.
        
        Args:
            module_name: Module name
            
        Returns:
            Path if module exists, None otherwise
        """
        return self.existing_modules.get(module_name)
    
    def get_all_modules(self) -> Dict[str, Dict]:
        """
        Get all modules with metadata.
        
        Returns:
            Dictionary mapping module names to metadata
        """
        result = {}
        
        for module_name, module_path in self.existing_modules.items():
            module_type = 'unknown'
            if module_name in self.STANDALONE_AGENTS:
                module_type = 'standalone'
            elif module_name in self.TOOL_MODULES or module_name.startswith('core/'):
                module_type = 'tool'
            elif module_name in self.service_modules:
                module_type = 'service'
            
            result[module_name] = {
                'path': str(module_path),
                'type': module_type,
                'exists': True,
            }
        
        return result
    
    def generate_module_manifest(self) -> Dict:
        """
        Generate install manifest data for all modules.
        
        Returns:
            Dictionary with module manifest data
        """
        import hashlib
        from datetime import datetime
        
        manifest = {
            'install_timestamp': datetime.utcnow().isoformat() + 'Z',
            'project_root': str(self.project_root),
            'modules': {},
        }
        
        for module_name, module_path in self.existing_modules.items():
            # Compute module hash (simple directory hash)
            try:
                # Get all files in module (first level only for speed)
                files = []
                for item in module_path.iterdir():
                    if item.is_file():
                        files.append(item.name)
                
                # Create simple hash from file list
                module_hash = hashlib.sha256(
                    json.dumps(sorted(files), sort_keys=True).encode()
                ).hexdigest()[:16]
            except Exception:
                module_hash = 'unknown'
            
            manifest['modules'][module_name] = {
                'path': str(module_path),
                'hash': module_hash,
                'type': 'standalone' if module_name in self.STANDALONE_AGENTS
                       else 'tool' if module_name in self.TOOL_MODULES or module_name.startswith('core/')
                       else 'service',
            }
        
        return manifest


def main():
    """CLI entry point for module resolver."""
    resolver = ModuleResolver()
    
    print("Canonical Module Resolver")
    print("=" * 80)
    print()
    
    print(f"Service Modules ({len(resolver.get_service_modules())}):")
    for module in resolver.get_service_modules():
        print(f"  ✓ {module}")
    
    print()
    print(f"Standalone Modules ({len(resolver.get_standalone_modules())}):")
    for module in resolver.get_standalone_modules():
        print(f"  ⚠ {module} (use dedicated installer)")
    
    print()
    print(f"Tool/Library Modules ({len(resolver.tool_modules)}):")
    for module in sorted(resolver.tool_modules):
        print(f"  ℹ {module}")
    
    if resolver.phantom_modules:
        print()
        print(f"⚠ Phantom Modules Detected ({len(resolver.phantom_modules)}):")
        for module in sorted(resolver.phantom_modules):
            print(f"  ✗ {module}")
        print()
        print("ERROR: Phantom modules detected. Installation cannot proceed.")
        sys.exit(1)
    
    print()
    print("✓ All modules validated - no phantoms detected")


if __name__ == '__main__':
    main()

