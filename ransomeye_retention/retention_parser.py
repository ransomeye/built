# Path and File Name : /home/ransomeye/rebuild/ransomeye_retention/retention_parser.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Parses retention.txt configuration and provides retention policy interface

"""
Retention Parser: Parses retention.txt and provides retention policy interface.
Handles defaults and validation.
"""

import os
from pathlib import Path
from typing import Dict, Optional
from datetime import datetime, timedelta


class RetentionParser:
    """Parses and validates retention configuration."""
    
    REQUIRED_VARS = [
        'TELEMETRY_RETENTION_MONTHS',
        'FORENSIC_RETENTION_DAYS',
        'DISK_MAX_USAGE_PERCENT'
    ]
    
    DEFAULT_VALUES = {
        'TELEMETRY_RETENTION_MONTHS': 6,
        'FORENSIC_RETENTION_DAYS': 10,
        'DISK_MAX_USAGE_PERCENT': 80
    }
    
    MIN_AI_RETENTION_YEARS = 2
    
    def __init__(self, config_path: str = "/home/ransomeye/rebuild/config/retention.txt"):
        self.config_path = Path(config_path)
        self.config: Dict = {}
        self._parse()
    
    def _parse(self) -> None:
        """Parse retention configuration file."""
        if not self.config_path.exists():
            # Use defaults
            self.config = self.DEFAULT_VALUES.copy()
            return
        
        try:
            with open(self.config_path, 'r') as f:
                for line in f:
                    line = line.strip()
                    if not line or line.startswith('#'):
                        continue
                    
                    if '=' in line:
                        key, value = line.split('=', 1)
                        key = key.strip()
                        value = value.strip()
                        
                        if key in self.REQUIRED_VARS:
                            try:
                                if 'PERCENT' in key:
                                    self.config[key] = int(value)
                                elif 'MONTHS' in key:
                                    self.config[key] = int(value)
                                elif 'DAYS' in key:
                                    self.config[key] = int(value)
                                else:
                                    self.config[key] = value
                            except ValueError:
                                # Invalid value, use default
                                self.config[key] = self.DEFAULT_VALUES.get(key, 0)
        except Exception:
            # On error, use defaults
            self.config = self.DEFAULT_VALUES.copy()
        
        # Ensure all required vars are present
        for var in self.REQUIRED_VARS:
            if var not in self.config:
                self.config[var] = self.DEFAULT_VALUES[var]
    
    def get_telemetry_retention_months(self) -> int:
        """Get telemetry retention period in months."""
        return self.config.get('TELEMETRY_RETENTION_MONTHS', 6)
    
    def get_forensic_retention_days(self) -> int:
        """Get forensic retention period in days."""
        return self.config.get('FORENSIC_RETENTION_DAYS', 10)
    
    def get_disk_max_usage_percent(self) -> int:
        """Get maximum disk usage percentage threshold."""
        return self.config.get('DISK_MAX_USAGE_PERCENT', 80)
    
    def get_ai_retention_years(self) -> int:
        """Get AI artifact retention period in years (minimum enforced)."""
        return max(self.MIN_AI_RETENTION_YEARS, 2)
    
    def get_telemetry_cutoff_date(self) -> datetime:
        """Get cutoff date for telemetry retention."""
        months = self.get_telemetry_retention_months()
        return datetime.utcnow() - timedelta(days=months * 30)
    
    def get_forensic_cutoff_date(self) -> datetime:
        """Get cutoff date for forensic retention."""
        days = self.get_forensic_retention_days()
        return datetime.utcnow() - timedelta(days=days)
    
    def get_ai_cutoff_date(self) -> datetime:
        """Get cutoff date for AI artifact retention."""
        years = self.get_ai_retention_years()
        return datetime.utcnow() - timedelta(days=years * 365)
    
    def is_valid(self) -> bool:
        """Check if configuration is valid."""
        # Validate ranges
        if not (0 <= self.get_telemetry_retention_months() <= 84):  # Max 7 years
            return False
        if not (0 <= self.get_forensic_retention_days() <= 3650):  # Max 10 years
            return False
        if not (50 <= self.get_disk_max_usage_percent() <= 100):
            return False
        return True
    
    def get_config(self) -> Dict:
        """Get full configuration dictionary."""
        return self.config.copy()

