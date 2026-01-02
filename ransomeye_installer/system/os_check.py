# Path and File Name : /home/ransomeye/rebuild/ransomeye_installer/system/os_check.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Validates OS compatibility - Ubuntu >=22.04 OR RHEL >=8

"""
OS Check: Validates operating system compatibility.
Supports Ubuntu >=22.04 and RHEL >=8 only.
"""

import os
import platform
import subprocess
import sys
from typing import Dict, Optional, Tuple


class OSCheck:
    """Validates OS compatibility."""
    
    SUPPORTED_DISTROS = {
        'ubuntu': {'min_major': 22, 'min_minor': 4},
        'rhel': {'min_major': 8, 'min_minor': 0},
        'centos': {'min_major': 8, 'min_minor': 0},  # CentOS 8 is RHEL-compatible
    }
    
    def __init__(self):
        self.distro_info: Optional[Dict] = None
        self._detect_distro()
    
    def _detect_distro(self) -> None:
        """Detect Linux distribution."""
        try:
            # Try /etc/os-release (most reliable)
            if os.path.exists('/etc/os-release'):
                with open('/etc/os-release', 'r') as f:
                    os_release = {}
                    for line in f:
                        if '=' in line:
                            key, value = line.strip().split('=', 1)
                            os_release[key] = value.strip('"')
                    
                    distro_id = os_release.get('ID', '').lower()
                    version_id = os_release.get('VERSION_ID', '')
                    
                    # Parse version
                    version_parts = version_id.split('.')
                    major = int(version_parts[0]) if version_parts else 0
                    minor = int(version_parts[1]) if len(version_parts) > 1 else 0
                    
                    self.distro_info = {
                        'id': distro_id,
                        'name': os_release.get('NAME', ''),
                        'version_id': version_id,
                        'major': major,
                        'minor': minor
                    }
                    return
            
            # Fallback: try lsb_release
            result = subprocess.run(['lsb_release', '-a'], capture_output=True, text=True, timeout=5)
            if result.returncode == 0:
                lsb_info = {}
                for line in result.stdout.split('\n'):
                    if ':' in line:
                        key, value = line.split(':', 1)
                        lsb_info[key.strip()] = value.strip()
                
                distro_id = lsb_info.get('Distributor ID', '').lower()
                release = lsb_info.get('Release', '')
                
                version_parts = release.split('.')
                major = int(version_parts[0]) if version_parts else 0
                minor = int(version_parts[1]) if len(version_parts) > 1 else 0
                
                self.distro_info = {
                    'id': distro_id,
                    'name': lsb_info.get('Description', ''),
                    'version_id': release,
                    'major': major,
                    'minor': minor
                }
                return
        except Exception:
            pass
        
        # Default: unknown
        self.distro_info = {
            'id': 'unknown',
            'name': platform.system(),
            'version_id': platform.release(),
            'major': 0,
            'minor': 0
        }
    
    def is_supported(self) -> Tuple[bool, str]:
        """
        Check if OS is supported.
        
        Returns:
            Tuple of (is_supported: bool, reason: str)
        """
        if self.distro_info is None:
            return False, "Could not detect operating system"
        
        distro_id = self.distro_info['id']
        
        # Normalize distro ID
        if 'ubuntu' in distro_id:
            distro_key = 'ubuntu'
        elif 'rhel' in distro_id or 'redhat' in distro_id:
            distro_key = 'rhel'
        elif 'centos' in distro_id:
            distro_key = 'centos'
        else:
            return False, f"Unsupported distribution: {self.distro_info['name']}"
        
        if distro_key not in self.SUPPORTED_DISTROS:
            return False, f"Distribution not in supported list: {distro_id}"
        
        # Check version
        requirements = self.SUPPORTED_DISTROS[distro_key]
        major = self.distro_info['major']
        minor = self.distro_info['minor']
        
        if major < requirements['min_major']:
            return False, f"{self.distro_info['name']} {major}.{minor} is below minimum {requirements['min_major']}.{requirements['min_minor']}"
        
        if major == requirements['min_major'] and minor < requirements['min_minor']:
            return False, f"{self.distro_info['name']} {major}.{minor} is below minimum {requirements['min_major']}.{requirements['min_minor']}"
        
        return True, f"Supported: {self.distro_info['name']} {major}.{minor}"
    
    def get_distro_info(self) -> Dict:
        """Get detected distribution information."""
        return self.distro_info.copy() if self.distro_info else {}


def main():
    """CLI entry point for OS check."""
    checker = OSCheck()
    is_supported, reason = checker.is_supported()
    
    if is_supported:
        print(f"✓ {reason}")
        sys.exit(0)
    else:
        print(f"✗ {reason}", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()

