# Path and File Name : /home/ransomeye/rebuild/core/assurance/assurance_lock.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Assurance Mode Lock - Creates and manages permanent assurance mode lock file

"""
RansomEye Assurance Mode Lock (PROMPT-59-A)

Creates and manages the permanent assurance mode lock file.
Once created, this lock cannot be removed without triggering integrity violations.
"""

import os
import sys
import json
import hashlib
from datetime import datetime, timezone
from pathlib import Path

ASSURANCE_LOCK_PATH = Path("/etc/ransomeye/ASSURANCE_MODE_LOCK")
ASSURANCE_LOCK_DIR = ASSURANCE_LOCK_PATH.parent


def create_assurance_lock():
    """Create the assurance mode lock file."""
    try:
        # Create directory if needed
        ASSURANCE_LOCK_DIR.mkdir(parents=True, exist_ok=True)
        
        # Create lock file with metadata
        lock_data = {
            "version": "1.0.0-enterprise-ship",
            "created": datetime.now(timezone.utc).isoformat(),
            "mode": "PERMANENT_ASSURANCE",
            "irreversible": True,
            "description": "Permanent Assurance Mode Lock - Cannot be disabled without integrity violation",
            "protected_services": [
                "ransomeye-verifier",
                "ransomeye-compliance-automation",
                "ransomeye-change-control",
                "ransomeye-baseline-capture"
            ]
        }
        
        with open(ASSURANCE_LOCK_PATH, "w") as f:
            json.dump(lock_data, f, indent=2, sort_keys=True)
        
        # Make read-only (444)
        os.chmod(ASSURANCE_LOCK_PATH, 0o444)
        
        print(f"Assurance Mode Lock created: {ASSURANCE_LOCK_PATH}")
        print("Mode: PERMANENT_ASSURANCE")
        print("Status: IRREVERSIBLE")
        
        return True
    except Exception as e:
        print(f"ERROR: Failed to create assurance lock: {e}", file=sys.stderr)
        return False


def check_assurance_lock() -> bool:
    """Check if assurance lock exists."""
    return ASSURANCE_LOCK_PATH.exists()


def get_assurance_lock_data() -> dict:
    """Get assurance lock data."""
    if not check_assurance_lock():
        return {}
    
    try:
        with open(ASSURANCE_LOCK_PATH, "r") as f:
            return json.load(f)
    except Exception:
        return {}


def main():
    """Main function to create assurance lock."""
    if check_assurance_lock():
        print("Assurance Mode Lock already exists")
        print(f"Location: {ASSURANCE_LOCK_PATH}")
        lock_data = get_assurance_lock_data()
        if lock_data:
            print(f"Created: {lock_data.get('created', 'unknown')}")
            print(f"Mode: {lock_data.get('mode', 'unknown')}")
        return 0
    
    if create_assurance_lock():
        return 0
    else:
        return 1


if __name__ == "__main__":
    sys.exit(main())

