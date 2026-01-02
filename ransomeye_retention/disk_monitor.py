# Path and File Name : /home/ransomeye/rebuild/ransomeye_retention/disk_monitor.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Monitors disk usage and triggers retention enforcement when threshold exceeded

"""
Disk Monitor: Monitors disk usage and triggers retention enforcement.
Enforces 80% disk usage rule (configurable).
"""

import os
import shutil
from pathlib import Path
from typing import Dict, Optional, Callable
from datetime import datetime


class DiskMonitor:
    """Monitors disk usage and triggers retention enforcement."""
    
    def __init__(self, retention_parser, monitor_path: str = "/home/ransomeye/rebuild"):
        self.retention_parser = retention_parser
        self.monitor_path = Path(monitor_path)
        self.callbacks: Dict[str, Callable] = {}
    
    def register_callback(self, event_type: str, callback: Callable) -> None:
        """Register callback for disk pressure events."""
        self.callbacks[event_type] = callback
    
    def get_disk_usage(self, path: Optional[Path] = None) -> Dict:
        """
        Get disk usage statistics.
        
        Returns:
            Dictionary with 'total', 'used', 'free', 'percent'
        """
        if path is None:
            path = self.monitor_path
        
        stat = shutil.disk_usage(path)
        
        return {
            'total': stat.total,
            'used': stat.used,
            'free': stat.free,
            'percent': (stat.used / stat.total) * 100,
            'path': str(path),
            'timestamp': datetime.utcnow().isoformat()
        }
    
    def is_disk_pressure(self, path: Optional[Path] = None) -> bool:
        """Check if disk usage exceeds threshold."""
        usage = self.get_disk_usage(path)
        threshold = self.retention_parser.get_disk_max_usage_percent()
        return usage['percent'] >= threshold
    
    def check_and_trigger(self, path: Optional[Path] = None) -> Dict:
        """
        Check disk usage and trigger callbacks if threshold exceeded.
        
        Returns:
            Dictionary with check results and triggered actions
        """
        usage = self.get_disk_usage(path)
        threshold = self.retention_parser.get_disk_max_usage_percent()
        is_pressure = usage['percent'] >= threshold
        
        result = {
            'usage': usage,
            'threshold': threshold,
            'is_pressure': is_pressure,
            'triggered': []
        }
        
        if is_pressure:
            # Trigger callbacks
            for event_type, callback in self.callbacks.items():
                try:
                    callback_result = callback(usage)
                    result['triggered'].append({
                        'event_type': event_type,
                        'result': callback_result
                    })
                except Exception as e:
                    result['triggered'].append({
                        'event_type': event_type,
                        'error': str(e)
                    })
        
        return result
    
    def get_cleanup_target_size(self, current_usage_percent: float, target_percent: float = 70) -> int:
        """
        Calculate target size to free up to reach target percentage.
        
        Args:
            current_usage_percent: Current disk usage percentage
            target_percent: Target usage percentage after cleanup
        
        Returns:
            Bytes to free
        """
        usage = self.get_disk_usage()
        total = usage['total']
        
        current_used = total * (current_usage_percent / 100)
        target_used = total * (target_percent / 100)
        
        return int(current_used - target_used)

