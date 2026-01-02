# Path and File Name: /home/ransomeye/rebuild/ransomeye_db_core/db_gate.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Central DB enablement gate - single source of truth for DB feature flag

"""
Database Enablement Gate: Central authority for DB feature availability.

CRITICAL RULES:
- Database is DISABLED by default
- ONLY enabled when RANSOMEYE_ENABLE_DB=1
- Malformed values cause FAIL-CLOSED abort
- Core installation succeeds WITHOUT DB
"""

import os
import sys
from typing import Tuple, Optional


class DatabaseGate:
    """
    Central database enablement gate.
    
    Single source of truth for whether database features are enabled.
    """
    
    # Environment variable name (canonical)
    ENV_VAR = "RANSOMEYE_ENABLE_DB"
    
    # Valid enable values
    VALID_ENABLE_VALUES = ["1", "true", "TRUE", "True", "yes", "YES", "Yes"]
    
    # Valid disable values
    VALID_DISABLE_VALUES = ["0", "false", "FALSE", "False", "no", "NO", "No", ""]
    
    @classmethod
    def is_enabled(cls, fail_closed: bool = False) -> bool:
        """
        Check if database features are enabled.
        
        Args:
            fail_closed: If True, malformed values cause SystemExit.
                        If False, malformed values default to disabled.
        
        Returns:
            True if DB enabled, False if disabled
        
        Raises:
            SystemExit: If fail_closed=True and value is malformed
        """
        value = os.environ.get(cls.ENV_VAR, "0")
        
        # Check if enabled
        if value in cls.VALID_ENABLE_VALUES:
            return True
        
        # Check if disabled
        if value in cls.VALID_DISABLE_VALUES:
            return False
        
        # Malformed value detected
        if fail_closed:
            print("", file=sys.stderr)
            print("=" * 80, file=sys.stderr)
            print("FATAL ERROR: Malformed database enablement flag", file=sys.stderr)
            print("=" * 80, file=sys.stderr)
            print("", file=sys.stderr)
            print(f"Environment variable: {cls.ENV_VAR}", file=sys.stderr)
            print(f"Current value: '{value}'", file=sys.stderr)
            print("", file=sys.stderr)
            print("Valid values to ENABLE database:", file=sys.stderr)
            print(f"  {', '.join(cls.VALID_ENABLE_VALUES)}", file=sys.stderr)
            print("", file=sys.stderr)
            print("Valid values to DISABLE database:", file=sys.stderr)
            print(f"  {', '.join(cls.VALID_DISABLE_VALUES)}", file=sys.stderr)
            print("", file=sys.stderr)
            print("Installation aborted (fail-closed).", file=sys.stderr)
            print("=" * 80, file=sys.stderr)
            sys.exit(1)
        else:
            # Non-fail-closed mode: default to disabled
            return False
    
    @classmethod
    def get_status_message(cls) -> str:
        """
        Get human-readable status message.
        
        Returns:
            Status message string
        """
        if cls.is_enabled():
            return "Database features: ENABLED (RANSOMEYE_ENABLE_DB=1)"
        else:
            return "Database features: DISABLED (default - set RANSOMEYE_ENABLE_DB=1 to enable)"
    
    @classmethod
    def require_enabled(cls, context: str = "operation") -> None:
        """
        Require database to be enabled, abort if not.
        
        Args:
            context: Description of operation requiring DB
        
        Raises:
            SystemExit: If database not enabled
        """
        if not cls.is_enabled(fail_closed=True):
            print("", file=sys.stderr)
            print(f"ERROR: {context} requires database to be enabled", file=sys.stderr)
            print(f"Set {cls.ENV_VAR}=1 to enable database features", file=sys.stderr)
            sys.exit(1)
    
    @classmethod
    def validate_credentials_present(cls) -> Tuple[bool, Optional[str]]:
        """
        Validate all required DB credentials are present.
        
        Returns:
            Tuple of (all_present, missing_var_name)
            If all present: (True, None)
            If any missing: (False, first_missing_var)
        """
        required_vars = [
            "RANSOMEYE_DB_HOST",
            "RANSOMEYE_DB_PORT",
            "RANSOMEYE_DB_NAME",
            "RANSOMEYE_DB_USER",
            "RANSOMEYE_DB_PASSWORD",
        ]
        
        for var in required_vars:
            if not os.environ.get(var):
                return False, var
        
        return True, None
    
    @classmethod
    def get_db_config(cls) -> Optional[dict]:
        """
        Get database configuration from environment.
        
        Returns:
            Dict with DB config if enabled and credentials present, None otherwise
        """
        if not cls.is_enabled():
            return None
        
        all_present, missing_var = cls.validate_credentials_present()
        if not all_present:
            return None
        
        try:
            port = int(os.environ.get("RANSOMEYE_DB_PORT", "5432"))
        except ValueError:
            return None
        
        return {
            "host": os.environ.get("RANSOMEYE_DB_HOST"),
            "port": port,
            "database": os.environ.get("RANSOMEYE_DB_NAME"),
            "user": os.environ.get("RANSOMEYE_DB_USER"),
            "password": os.environ.get("RANSOMEYE_DB_PASSWORD"),
        }


def is_db_enabled() -> bool:
    """
    Convenience function: Check if database is enabled.
    
    Returns:
        True if enabled, False otherwise
    """
    return DatabaseGate.is_enabled()


def require_db_enabled(context: str = "operation") -> None:
    """
    Convenience function: Require database enabled or abort.
    
    Args:
        context: Description of operation
    
    Raises:
        SystemExit: If DB not enabled
    """
    DatabaseGate.require_enabled(context)

