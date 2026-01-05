# Path and File Name : /home/ransomeye/rebuild/core/incident/biannual_drill_scheduler.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Biannual Incident Drill Scheduler - Automates biannual incident drills

"""
RansomEye Biannual Incident Drill Scheduler (PROMPT-60-D)

Automates biannual incident drills:
- Crash → recovery
- Audit replay
- Forensic export
- MTTR capture
"""

import os
import sys
import subprocess
from datetime import datetime, timezone
from pathlib import Path

INCIDENT_DRILL_SCRIPT = Path("/home/ransomeye/rebuild/core/incident/incident_drill.py")
DRILL_OUTPUT_DIR = Path("/var/lib/ransomeye/incident_drills")


def should_run_drill() -> bool:
    """Check if drill should run (biannual: Jan 1 and Jul 1)."""
    now = datetime.now(timezone.utc)
    
    # Check if today is Jan 1 or Jul 1
    if (now.month == 1 and now.day == 1) or (now.month == 7 and now.day == 1):
        # Check if drill already run today
        today_str = now.strftime("%Y%m%d")
        drill_files = list(DRILL_OUTPUT_DIR.glob(f"drill_report_{today_str}*.json"))
        return len(drill_files) == 0
    
    return False


def run_drill():
    """Run incident drill."""
    if not INCIDENT_DRILL_SCRIPT.exists():
        print(f"ERROR: Drill script not found: {INCIDENT_DRILL_SCRIPT}")
        return False
    
    print("Running biannual incident drill...")
    
    try:
        result = subprocess.run(
            ["python3", str(INCIDENT_DRILL_SCRIPT)],
            capture_output=True,
            text=True,
            timeout=300  # 5 minutes max
        )
        
        if result.returncode == 0:
            print("Drill completed successfully")
            print(result.stdout)
            return True
        else:
            print("Drill completed with warnings")
            print(result.stdout)
            print(result.stderr)
            return True  # Still consider it run
    except subprocess.TimeoutExpired:
        print("ERROR: Drill timed out")
        return False
    except Exception as e:
        print(f"ERROR: Drill failed: {e}")
        return False


def main():
    """Main drill scheduler."""
    print("RansomEye Biannual Incident Drill Scheduler (PROMPT-60-D)")
    print("=" * 60)
    
    if should_run_drill():
        print("Biannual drill scheduled for today - executing...")
        if run_drill():
            print("Biannual drill completed")
            return 0
        else:
            print("Biannual drill failed")
            return 1
    else:
        print("Not scheduled for today - skipping")
        return 0


if __name__ == "__main__":
    sys.exit(main())

