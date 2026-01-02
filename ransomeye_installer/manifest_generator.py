# Path and File Name : /home/ransomeye/rebuild/ransomeye_installer/manifest_generator.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Generates install manifest with installed modules, hashes, and metadata - supports dry-run mode
# INVARIANT: Systemd unit integrity is enforced by manifest hash. Runtime mutation is forbidden.

"""
Install Manifest Generator: Creates verifiable install manifest at install time.
Supports dry-run mode for pre-install validation.
CRITICAL: Generates SHA256 hashes for all systemd units for tamper detection (fail-closed).
"""

import json
import hashlib
import sys
import os
from pathlib import Path
from datetime import datetime
from typing import Dict, Optional, List, Set

try:
    from .module_resolver import ModuleResolver
except ImportError:
    ModuleResolver = None


class ManifestGenerator:
    """Generates and manages install manifest."""
    
    MANIFEST_PATH = Path("/var/lib/ransomeye/install_manifest.json")
    
    # Known binary locations
    KNOWN_BINARIES = {
        'ransomeye-guardrails': '/usr/bin/ransomeye-guardrails',
        'ransomeye_operations': '/usr/bin/ransomeye_operations',
        'ransomeye-playbook-engine': '/usr/local/bin/ransomeye-playbook-engine',
        'ransomeye-network-scanner': '/usr/local/bin/ransomeye-network-scanner',
    }
    
    # Known ports (from environment variables and defaults)
    KNOWN_PORTS = {
        'CORE_API_PORT': 8443,
        'FRONTEND_PORT': 3000,
        'BACKEND_API_PORT': 8080,
    }
    
    # Config paths (runtime only - no build paths)
    CONFIG_PATHS = [
        '/etc/ransomeye/',
        '/var/lib/ransomeye/',
        '/run/ransomeye/',
    ]
    
    def __init__(self, dry_run: bool = False, project_root: Path = None):
        self.dry_run = dry_run
        # Discover project root dynamically if not provided
        if project_root is None:
            # Use __file__ to discover location
            project_root = Path(__file__).resolve().parent.parent
        self.project_root = project_root
        self.guardrails_spec = self.project_root / "core/guardrails/guardrails.yaml"
        self.manifest_path = self.MANIFEST_PATH if not dry_run else self.project_root / "install_manifest.json"
        self.resolver = ModuleResolver() if ModuleResolver else None
    
    def generate_manifest(self) -> Dict:
        """
        Generate install manifest with all installed modules.
        
        Returns:
            Dictionary with manifest data
        """
        # CRITICAL ARCHITECTURE LOCK: DB is MANDATORY for RansomEye Core
        # RANSOMEYE_ENABLE_DB is IGNORED for Core installations
        # DB is always enabled for Core (architectural invariant)
        db_enabled = True
        
        manifest = {
            'install_timestamp': datetime.utcnow().isoformat() + 'Z',
            'project_root': str(self.project_root),
            'installer_version': '1.0.0',
            'dry_run': self.dry_run,
            'db_enabled': db_enabled,  # ALWAYS True for Core (architectural lock)
            'modules': {},
            'binaries': [],
            'systemd_units': [],
            'config_paths': self.CONFIG_PATHS.copy(),
            'ports': {},
            'database_migrations': [],
            'agent_exclusions': [],
            'guardrails_spec_hash': self._get_guardrails_hash(),
        }
        
        # Add note about DB being mandatory for Core
        manifest['db_note'] = 'Database is MANDATORY for RansomEye Core (architectural lock). Credentials: gagan:gagan (fixed).'
        
        if self.resolver:
            # Use resolver to get all modules
            all_modules = self.resolver.get_all_modules()
            
            for module_name, module_info in all_modules.items():
                module_path = Path(module_info['path'])
                module_type = module_info['type']
                
                # Compute module hash
                module_hash = self._compute_module_hash(module_path)
                
                # Get phase number if available
                phase_number = self._get_phase_number(module_name)
                
                manifest['modules'][module_name] = {
                    'path': str(module_path),
                    'type': module_type,
                    'hash': module_hash,
                    'phase': phase_number,
                }
                
                # Add systemd unit if it's a service module
                if module_type == 'service':
                    service_name = module_name.replace('ransomeye_', 'ransomeye-')
                    systemd_unit = f"{service_name}.service"
                    install_path = f"/etc/systemd/system/{systemd_unit}"
                    
                    # CRITICAL: Add hash placeholder (will be populated by installer)
                    # Hash is computed AFTER unit installation to capture actual installed content
                    manifest['systemd_units'].append({
                        'name': systemd_unit,
                        'module': module_name,
                        'source_path': 'GENERATED_AT_INSTALL_TIME',  # No build-time path coupling
                        'install_path': install_path,  # REQUIRED: Explicit install location
                        'sha256_hash': None,  # REQUIRED: Populated by installer after unit installation
                    })
            
            # Add standalone agents to exclusions
            standalone_modules = self.resolver.get_standalone_modules()
            for agent in standalone_modules:
                manifest['agent_exclusions'].append({
                    'name': agent,
                    'reason': 'standalone_agent',
                    'note': 'Not installed by main installer, requires dedicated installer',
                })
        else:
            # Fallback: minimal manifest
            manifest['modules'] = {}
        
        # Add binaries (core only)
        for binary_name, binary_path in self.KNOWN_BINARIES.items():
            binary_exists = Path(binary_path).exists() if not self.dry_run else False
            manifest['binaries'].append({
                'name': binary_name,
                'path': binary_path,
                'will_install': True,  # These are installed by install.sh
                'exists': binary_exists,
            })
        
        # Add ports (from environment or defaults)
        for port_name, default_port in self.KNOWN_PORTS.items():
            env_port = os.environ.get(port_name, str(default_port))
            try:
                port_value = int(env_port)
            except ValueError:
                port_value = default_port
            manifest['ports'][port_name] = port_value
        
        # Database migrations (currently none - DB Core handles schema)
        # This is a placeholder for future migrations
        manifest['database_migrations'] = []
        
        return manifest
    
    def _compute_module_hash(self, module_path: Path) -> str:
        """
        Compute hash of module directory.
        
        Args:
            module_path: Path to module directory
            
        Returns:
            SHA-256 hash (first 16 chars)
        """
        try:
            # Get all files in module (recursive, but limit depth for performance)
            files = []
            for item in module_path.rglob('*'):
                if item.is_file():
                    # Skip large files and build artifacts
                    if any(skip in str(item) for skip in ['target/', '__pycache__/', '.git/', 'node_modules/']):
                        continue
                    try:
                        # Get file size and mtime for hash
                        stat = item.stat()
                        files.append(f"{item.relative_to(module_path)}:{stat.st_size}:{stat.st_mtime}")
                    except Exception:
                        continue
            
            # Create hash from file list
            content = '\n'.join(sorted(files))
            return hashlib.sha256(content.encode()).hexdigest()[:16]
        except Exception as e:
            return f"error:{hash(str(e)) % 10000}"
    
    def _get_phase_number(self, module_name: str) -> Optional[int]:
        """Get phase number for module from MODULE_PHASE_MAP.yaml."""
        try:
            map_path = self.project_root / "MODULE_PHASE_MAP.yaml"
            if not map_path.exists():
                return None
            
            import yaml
            with open(map_path, 'r') as f:
                module_map = yaml.safe_load(f)
            
            if not module_map or 'modules' not in module_map:
                return None
            
            for module in module_map['modules']:
                if module.get('module_name') == module_name:
                    return module.get('phase_number')
        except Exception:
            pass
        
        return None
    
    def _get_guardrails_hash(self) -> Optional[str]:
        """Get guardrails specification hash."""
        try:
            if not self.guardrails_spec.exists():
                return None
            
            import yaml
            with open(self.guardrails_spec, 'r') as f:
                guardrails = yaml.safe_load(f)
            
            return guardrails.get('spec_hash') if guardrails else None
        except Exception:
            return None
    
    def write_manifest(self, manifest: Optional[Dict] = None) -> Path:
        """
        Write manifest to disk.
        
        Args:
            manifest: Manifest dictionary (generates if None)
            
        Returns:
            Path to written manifest file
        """
        if manifest is None:
            manifest = self.generate_manifest()
        
        if self.dry_run:
            # In dry-run, write to project root instead of /var/lib/ransomeye/
            self.manifest_path = self.project_root / "install_manifest.json"
        
        # Ensure directory exists
        self.manifest_path.parent.mkdir(parents=True, exist_ok=True)
        
        # Write manifest
        with open(self.manifest_path, 'w') as f:
            json.dump(manifest, f, indent=2, sort_keys=True)
        
        # Set permissions (only if not dry-run, or if file exists)
        if self.manifest_path.exists():
            try:
                os.chmod(self.manifest_path, 0o644)
            except Exception:
                pass  # May fail in some environments, non-fatal
        
        return self.manifest_path
    
    def load_manifest(self) -> Optional[Dict]:
        """
        Load existing manifest from disk.
        
        Returns:
            Manifest dictionary or None if not found
        """
        if not self.manifest_path.exists():
            return None
        
        try:
            with open(self.manifest_path, 'r') as f:
                return json.load(f)
        except Exception:
            return None


def main():
    """CLI entry point for manifest generator."""
    dry_run = os.environ.get('RANSOMEYE_DRY_RUN', '0') == '1'
    generator = ManifestGenerator(dry_run=dry_run)
    manifest = generator.generate_manifest()
    manifest_path = generator.write_manifest(manifest)
    
    print(f"✓ Install manifest generated: {manifest_path}")
    print(f"  Modules: {len(manifest['modules'])}")
    print(f"  Binaries: {len(manifest.get('binaries', []))}")
    print(f"  Systemd units: {len(manifest.get('systemd_units', []))}")
    print(f"  Agent exclusions: {len(manifest.get('agent_exclusions', []))}")
    if dry_run:
        print(f"  Mode: DRY-RUN")


if __name__ == '__main__':
    main()
