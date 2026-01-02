# Path and File Name : /home/ransomeye/rebuild/ransomeye_guardrails/runtime_check.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Verifies python3 runtime exists and meets minimum version requirements - fails-closed if invalid

"""
Runtime Check: Verifies python3 runtime exists and meets minimum version.
Fails-closed if runtime is missing or invalid.
"""

import sys
import subprocess
from typing import Tuple, Optional


def check_python3_exists() -> Tuple[bool, Optional[str]]:
    """
    Check if python3 executable exists.
    
    Returns:
        Tuple of (exists: bool, version_output: Optional[str])
    """
    try:
        result = subprocess.run(
            ['python3', '--version'],
            capture_output=True,
            text=True,
            timeout=5
        )
        if result.returncode == 0:
            return True, result.stdout.strip()
        return False, None
    except FileNotFoundError:
        return False, None
    except Exception:
        return False, None


def parse_version(version_string: str) -> Tuple[int, int, int]:
    """
    Parse Python version string to (major, minor, patch).
    
    Args:
        version_string: Version string like "Python 3.10.5"
    
    Returns:
        Tuple of (major, minor, patch)
    """
    try:
        # Extract version number
        parts = version_string.split()
        version_part = None
        for part in parts:
            if part.startswith('3.'):
                version_part = part
                break
        
        if not version_part:
            return (0, 0, 0)
        
        # Parse version numbers
        version_numbers = version_part.split('.')
        major = int(version_numbers[0])
        minor = int(version_numbers[1]) if len(version_numbers) > 1 else 0
        patch = int(version_numbers[2]) if len(version_numbers) > 2 else 0
        
        return (major, minor, patch)
    except Exception:
        return (0, 0, 0)


def check_minimum_version(major: int, minor: int, patch: int, min_major: int = 3, min_minor: int = 10) -> bool:
    """
    Check if version meets minimum requirements.
    
    Args:
        major: Major version number
        minor: Minor version number
        patch: Patch version number
        min_major: Minimum major version
        min_minor: Minimum minor version
    
    Returns:
        True if version meets requirements
    """
    if major > min_major:
        return True
    if major == min_major and minor >= min_minor:
        return True
    return False


def validate_runtime() -> None:
    """
    Validate python3 runtime exists and meets minimum version.
    Fails-closed if validation fails.
    """
    # Check if python3 exists
    exists, version_output = check_python3_exists()
    
    if not exists:
        print("="*80, file=sys.stderr)
        print("RANSOMEYE RUNTIME CHECK FAILED", file=sys.stderr)
        print("="*80, file=sys.stderr)
        print("ERROR: python3 executable not found", file=sys.stderr)
        print("", file=sys.stderr)
        print("REQUIREMENTS:", file=sys.stderr)
        print("  - python3 must be installed and available in PATH", file=sys.stderr)
        print("  - Minimum version: Python 3.10", file=sys.stderr)
        print("", file=sys.stderr)
        print("SUPPORTED PLATFORMS:", file=sys.stderr)
        print("  - Ubuntu 22.04+", file=sys.stderr)
        print("  - RHEL 8+", file=sys.stderr)
        print("", file=sys.stderr)
        print("INSTALLATION:", file=sys.stderr)
        print("  Ubuntu/Debian: sudo apt-get install python3 python3-pip", file=sys.stderr)
        print("  RHEL/CentOS: sudo yum install python3 python3-pip", file=sys.stderr)
        print("="*80, file=sys.stderr)
        sys.exit(1)
    
    # Parse version
    major, minor, patch = parse_version(version_output)
    
    # Check minimum version
    if not check_minimum_version(major, minor, patch, min_major=3, min_minor=10):
        print("="*80, file=sys.stderr)
        print("RANSOMEYE RUNTIME CHECK FAILED", file=sys.stderr)
        print("="*80, file=sys.stderr)
        print(f"ERROR: Python version {major}.{minor}.{patch} does not meet minimum requirements", file=sys.stderr)
        print("", file=sys.stderr)
        print("REQUIREMENTS:", file=sys.stderr)
        print("  - Minimum version: Python 3.10", file=sys.stderr)
        print(f"  - Detected version: Python {major}.{minor}.{patch}", file=sys.stderr)
        print("", file=sys.stderr)
        print("UPGRADE:", file=sys.stderr)
        print("  Ubuntu/Debian: sudo apt-get install python3.10 python3.10-pip", file=sys.stderr)
        print("  RHEL/CentOS: sudo yum install python3.10 python3.10-pip", file=sys.stderr)
        print("="*80, file=sys.stderr)
        sys.exit(1)
    
    # Success
    print(f"✓ Runtime check passed: {version_output}", file=sys.stderr)


def main():
    """CLI entry point for runtime check."""
    validate_runtime()


if __name__ == '__main__':
    main()

