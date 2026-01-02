# Path and File Name : /home/ransomeye/rebuild/ransomeye_installer/system/disk_check.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Validates disk space availability and enforces 80% watermark logic

"""
Disk Check: Validates disk space and enforces 80% watermark.
Warns if disk usage is high and enforces retention limits.
"""

import os
import shutil
import sys
from pathlib import Path
from typing import Dict, Tuple


class DiskCheck:
    """Validates disk space availability."""
    
    MIN_FREE_GB = 10  # Minimum 10GB free space required
    WARNING_THRESHOLD_PERCENT = 80  # Warn if usage >= 80%
    CRITICAL_THRESHOLD_PERCENT = 90  # Fail if usage >= 90%
    
    def __init__(self, check_path: str = "/home/ransomeye/rebuild"):
        self.check_path = Path(check_path)
        self.check_path.mkdir(parents=True, exist_ok=True)
    
    def get_disk_usage(self) -> Dict:
        """
        Get disk usage statistics.
        
        Returns:
            Dictionary with disk usage information
        """
        stat = shutil.disk_usage(self.check_path)
        
        total_gb = stat.total / (1024 ** 3)
        used_gb = stat.used / (1024 ** 3)
        free_gb = stat.free / (1024 ** 3)
        usage_percent = (stat.used / stat.total) * 100
        
        return {
            'total_gb': total_gb,
            'used_gb': used_gb,
            'free_gb': free_gb,
            'usage_percent': usage_percent,
            'path': str(self.check_path)
        }
    
    def check_availability(self) -> Tuple[bool, str, Dict]:
        """
        Check if disk space is available.
        
        Returns:
            Tuple of (is_available: bool, message: str, usage_info: Dict)
        """
        usage = self.get_disk_usage()
        
        # Check minimum free space
        if usage['free_gb'] < self.MIN_FREE_GB:
            return False, f"Insufficient disk space: {usage['free_gb']:.2f}GB free (minimum {self.MIN_FREE_GB}GB required)", usage
        
        # Check critical threshold
        if usage['usage_percent'] >= self.CRITICAL_THRESHOLD_PERCENT:
            return False, f"Disk usage critical: {usage['usage_percent']:.1f}% (>= {self.CRITICAL_THRESHOLD_PERCENT}%)", usage
        
        # Check warning threshold
        if usage['usage_percent'] >= self.WARNING_THRESHOLD_PERCENT:
            return True, f"WARNING: Disk usage high: {usage['usage_percent']:.1f}% (>= {self.WARNING_THRESHOLD_PERCENT}%). Retention enforcement will be active.", usage
        
        return True, f"Disk space available: {usage['free_gb']:.2f}GB free ({usage['usage_percent']:.1f}% used)", usage
    
    def is_at_watermark(self) -> bool:
        """
        Check if disk is at 80% watermark.
        
        Returns:
            True if usage >= 80%
        """
        usage = self.get_disk_usage()
        return usage['usage_percent'] >= self.WARNING_THRESHOLD_PERCENT


def main():
    """CLI entry point for disk check."""
    checker = DiskCheck()
    is_available, message, usage = checker.check_availability()
    
    print(f"Disk Usage: {usage['usage_percent']:.1f}%")
    print(f"Free Space: {usage['free_gb']:.2f}GB / {usage['total_gb']:.2f}GB")
    print(f"Status: {message}")
    
    if not is_available:
        sys.exit(1)
    
    sys.exit(0)


if __name__ == '__main__':
    main()

