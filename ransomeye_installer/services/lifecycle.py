# Path and File Name : /home/ransomeye/rebuild/ransomeye_installer/services/lifecycle.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Service lifecycle management - start/stop/restart orchestration with state validation

"""
Service Lifecycle: Manages service start/stop/restart.
All operations require valid installer state.
"""

import subprocess
import sys
from pathlib import Path
from typing import List, Tuple

from ..state_manager import StateManager


class ServiceLifecycle:
    """Manages service lifecycle operations."""
    
    # CRITICAL: Use installed systemd location (NOT legacy build directory)
    # Units are installed to /etc/systemd/system during installation
    # Legacy path /home/ransomeye/rebuild/systemd is FORBIDDEN
    SYSTEMD_DIR = Path("/etc/systemd/system")
    
    def __init__(self):
        self.state_manager = StateManager()
    
    def _validate_state(self) -> bool:
        """Validate installer state before operations."""
        return self.state_manager.is_state_valid()
    
    def _get_service_names(self) -> List[str]:
        """Get list of service names."""
        services = []
        for unit_file in self.SYSTEMD_DIR.glob("*.service"):
            services.append(unit_file.stem)
        return services
    
    def start_services(self) -> Tuple[bool, List[str]]:
        """
        Start all RansomEye services.
        
        Returns:
            Tuple of (success: bool, messages: List[str])
        """
        if not self._validate_state():
            return False, ["Installer state is not valid. Services cannot start."]
        
        services = self._get_service_names()
        messages = []
        success = True
        
        for service in services:
            try:
                result = subprocess.run(
                    ['systemctl', 'start', service],
                    capture_output=True,
                    text=True,
                    timeout=30
                )
                if result.returncode == 0:
                    messages.append(f"✓ Started {service}")
                else:
                    messages.append(f"✗ Failed to start {service}: {result.stderr}")
                    success = False
            except Exception as e:
                messages.append(f"✗ Error starting {service}: {str(e)}")
                success = False
        
        return success, messages
    
    def stop_services(self) -> Tuple[bool, List[str]]:
        """
        Stop all RansomEye services.
        
        Returns:
            Tuple of (success: bool, messages: List[str])
        """
        services = self._get_service_names()
        messages = []
        success = True
        
        for service in services:
            try:
                result = subprocess.run(
                    ['systemctl', 'stop', service],
                    capture_output=True,
                    text=True,
                    timeout=30
                )
                if result.returncode == 0:
                    messages.append(f"✓ Stopped {service}")
                else:
                    messages.append(f"✗ Failed to stop {service}: {result.stderr}")
                    success = False
            except Exception as e:
                messages.append(f"✗ Error stopping {service}: {str(e)}")
                success = False
        
        return success, messages
    
    def restart_services(self) -> Tuple[bool, List[str]]:
        """
        Restart all RansomEye services.
        
        Returns:
            Tuple of (success: bool, messages: List[str])
        """
        if not self._validate_state():
            return False, ["Installer state is not valid. Services cannot restart."]
        
        services = self._get_service_names()
        messages = []
        success = True
        
        for service in services:
            try:
                result = subprocess.run(
                    ['systemctl', 'restart', service],
                    capture_output=True,
                    text=True,
                    timeout=30
                )
                if result.returncode == 0:
                    messages.append(f"✓ Restarted {service}")
                else:
                    messages.append(f"✗ Failed to restart {service}: {result.stderr}")
                    success = False
            except Exception as e:
                messages.append(f"✗ Error restarting {service}: {str(e)}")
                success = False
        
        return success, messages
    
    def enable_services(self) -> Tuple[bool, List[str]]:
        """
        Enable services to start on boot.
        
        Returns:
            Tuple of (success: bool, messages: List[str])
        """
        if not self._validate_state():
            return False, ["Installer state is not valid. Services cannot be enabled."]
        
        services = self._get_service_names()
        messages = []
        success = True
        
        for service in services:
            try:
                result = subprocess.run(
                    ['systemctl', 'enable', service],
                    capture_output=True,
                    text=True,
                    timeout=30
                )
                if result.returncode == 0:
                    messages.append(f"✓ Enabled {service}")
                else:
                    messages.append(f"✗ Failed to enable {service}: {result.stderr}")
                    success = False
            except Exception as e:
                messages.append(f"✗ Error enabling {service}: {str(e)}")
                success = False
        
        return success, messages
    
    def disable_services(self) -> Tuple[bool, List[str]]:
        """
        Disable services from starting on boot.
        
        Returns:
            Tuple of (success: bool, messages: List[str])
        """
        services = self._get_service_names()
        messages = []
        success = True
        
        for service in services:
            try:
                result = subprocess.run(
                    ['systemctl', 'disable', service],
                    capture_output=True,
                    text=True,
                    timeout=30
                )
                if result.returncode == 0:
                    messages.append(f"✓ Disabled {service}")
                else:
                    messages.append(f"✗ Failed to disable {service}: {result.stderr}")
                    success = False
            except Exception as e:
                messages.append(f"✗ Error disabling {service}: {str(e)}")
                success = False
        
        return success, messages


def main():
    """CLI entry point for service lifecycle."""
    import argparse
    
    parser = argparse.ArgumentParser(description='RansomEye Service Lifecycle')
    parser.add_argument('action', choices=['start', 'stop', 'restart', 'enable', 'disable'],
                       help='Action to perform')
    
    args = parser.parse_args()
    
    lifecycle = ServiceLifecycle()
    
    if args.action == 'start':
        success, messages = lifecycle.start_services()
    elif args.action == 'stop':
        success, messages = lifecycle.stop_services()
    elif args.action == 'restart':
        success, messages = lifecycle.restart_services()
    elif args.action == 'enable':
        success, messages = lifecycle.enable_services()
    elif args.action == 'disable':
        success, messages = lifecycle.disable_services()
    
    for message in messages:
        print(message)
    
    sys.exit(0 if success else 1)


if __name__ == '__main__':
    main()

