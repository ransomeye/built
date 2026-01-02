# Path and File Name : /home/ransomeye/rebuild/ransomeye_installer/retention/retention_validator.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Validates retention.txt configuration using retention enforcer

"""
Retention Validator: Validates retention.txt configuration.
Uses retention enforcer to ensure configuration is valid.

Note: This validator does NOT require psycopg2 or database connectivity.
It only validates the retention.txt configuration file format and values.
"""

import sys
from pathlib import Path

# Lazy import RetentionParser (does not import DB code)
try:
    from ransomeye_retention.retention_parser import RetentionParser
except ImportError as e:
    # If retention_parser can't be imported, this is a critical error
    # (not related to psycopg2, but to missing module)
    RetentionParser = None
    _PARSER_IMPORT_ERROR = e


class RetentionValidator:
    """Validates retention configuration."""
    
    def __init__(self, config_path: str = "/home/ransomeye/rebuild/config/retention.txt"):
        self.config_path = Path(config_path)
        if RetentionParser is None:
            raise ImportError(
                f"Failed to import RetentionParser: {_PARSER_IMPORT_ERROR}. "
                "This indicates a missing module, not a database dependency issue."
            )
        self.parser = RetentionParser(str(self.config_path))
    
    def validate(self) -> tuple[bool, str]:
        """
        Validate retention configuration.
        
        Returns:
            Tuple of (is_valid: bool, error_message: str)
        """
        if not self.config_path.exists():
            return False, f"Retention configuration file not found: {self.config_path}"
        
        if not self.parser.is_valid():
            return False, "Retention configuration is invalid"
        
        # Validate values are reasonable
        telemetry_months = self.parser.get_telemetry_retention_months()
        forensic_days = self.parser.get_forensic_retention_days()
        disk_percent = self.parser.get_disk_max_usage_percent()
        
        if telemetry_months < 1 or telemetry_months > 84:
            return False, f"TELEMETRY_RETENTION_MONTHS must be between 1 and 84, got {telemetry_months}"
        
        if forensic_days < 1 or forensic_days > 3650:
            return False, f"FORENSIC_RETENTION_DAYS must be between 1 and 3650, got {forensic_days}"
        
        if disk_percent < 50 or disk_percent > 100:
            return False, f"DISK_MAX_USAGE_PERCENT must be between 50 and 100, got {disk_percent}"
        
        return True, "Retention configuration is valid"
    
    def get_config(self) -> dict:
        """Get parsed retention configuration."""
        return self.parser.get_config()


def main():
    """CLI entry point for retention validator."""
    validator = RetentionValidator()
    is_valid, message = validator.validate()
    
    if is_valid:
        print(f"✓ {message}")
        config = validator.get_config()
        print(f"  TELEMETRY_RETENTION_MONTHS: {config.get('TELEMETRY_RETENTION_MONTHS')}")
        print(f"  FORENSIC_RETENTION_DAYS: {config.get('FORENSIC_RETENTION_DAYS')}")
        print(f"  DISK_MAX_USAGE_PERCENT: {config.get('DISK_MAX_USAGE_PERCENT')}%")
        sys.exit(0)
    else:
        print(f"✗ {message}", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()

