# Path and File Name : /home/ransomeye/rebuild/ransomeye_installer/system/clock_check.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Validates system clock synchronization for accurate timestamps

"""
Clock Check: Validates system clock synchronization.
Ensures accurate timestamps for logs and cryptographic operations.
"""

import os
import subprocess
import sys
from datetime import datetime
from typing import Dict, Tuple, Optional


class ClockCheck:
    """Validates system clock synchronization."""
    
    MAX_DRIFT_SECONDS = 60  # Maximum acceptable clock drift (60 seconds)
    
    def __init__(self):
        self.clock_info: Optional[Dict] = None
        self._check_clock()
    
    def _check_clock(self) -> None:
        """Check system clock status."""
        try:
            # Get system time
            system_time = datetime.utcnow()
            
            # Try to get NTP status
            ntp_synced = False
            ntp_status = "unknown"
            
            # Check systemd-timesyncd
            if os.path.exists('/usr/bin/timedatectl'):
                try:
                    result = subprocess.run(
                        ['timedatectl', 'status'],
                        capture_output=True,
                        text=True,
                        timeout=5
                    )
                    if result.returncode == 0:
                        output = result.stdout
                        if 'NTP synchronized: yes' in output:
                            ntp_synced = True
                            ntp_status = "synchronized"
                        elif 'NTP synchronized: no' in output:
                            ntp_status = "not synchronized"
                except Exception:
                    pass
            
            # Check chronyd
            if not ntp_synced and os.path.exists('/usr/bin/chronyc'):
                try:
                    result = subprocess.run(
                        ['chronyc', 'tracking'],
                        capture_output=True,
                        text=True,
                        timeout=5
                    )
                    if result.returncode == 0:
                        ntp_synced = True
                        ntp_status = "synchronized (chronyd)"
                except Exception:
                    pass
            
            # Check ntpd
            if not ntp_synced and os.path.exists('/usr/sbin/ntpq'):
                try:
                    result = subprocess.run(
                        ['ntpq', '-p'],
                        capture_output=True,
                        text=True,
                        timeout=5
                    )
                    if result.returncode == 0 and '*' in result.stdout:
                        ntp_synced = True
                        ntp_status = "synchronized (ntpd)"
                except Exception:
                    pass
            
            self.clock_info = {
                'system_time': system_time.isoformat(),
                'ntp_synced': ntp_synced,
                'ntp_status': ntp_status
            }
        except Exception:
            self.clock_info = {
                'system_time': datetime.utcnow().isoformat(),
                'ntp_synced': False,
                'ntp_status': 'unknown'
            }
    
    def check_sync(self) -> Tuple[bool, str, Dict]:
        """
        Check if clock is synchronized.
        
        Returns:
            Tuple of (is_synced: bool, message: str, info: Dict)
        """
        if self.clock_info is None:
            return False, "Could not check clock status", {}
        
        if self.clock_info['ntp_synced']:
            return True, f"Clock synchronized: {self.clock_info['ntp_status']}", self.clock_info
        
        # Warn but don't fail (clock may sync later)
        return True, f"WARNING: Clock may not be synchronized ({self.clock_info['ntp_status']}). Ensure NTP is configured for accurate timestamps.", self.clock_info
    
    def get_clock_info(self) -> Dict:
        """Get clock information."""
        return self.clock_info.copy() if self.clock_info else {}


def main():
    """CLI entry point for clock check."""
    checker = ClockCheck()
    is_synced, message, info = checker.check_sync()
    
    print(f"System Time: {info.get('system_time', 'unknown')}")
    print(f"NTP Status: {info.get('ntp_status', 'unknown')}")
    print(f"Status: {message}")
    
    if not is_synced:
        sys.exit(1)
    
    sys.exit(0)


if __name__ == '__main__':
    main()

