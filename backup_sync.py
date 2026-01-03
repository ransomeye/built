# Path and File Name : /home/ransomeye/rebuild/backup_sync.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Continuous backup sync script that syncs rebuild folder to external pendrive, excluding venv files

"""
Continuous Backup Sync Script
Syncs /home/ransomeye/rebuild to external pendrive at /mnt/pendrive/rebuild
Excludes venv files and directories for efficient backup.
"""

import os
import sys
import time
import subprocess
import signal
import logging
from pathlib import Path
from datetime import datetime

# Configuration
SOURCE_DIR = Path("/home/ransomeye/rebuild")
BACKUP_DIR = Path("/mnt/pendrive/rebuild")
LOG_FILE = Path("/home/ransomeye/rebuild/logs/backup_sync.log")
SYNC_INTERVAL = 30  # seconds between syncs
PID_FILE = Path("/home/ransomeye/rebuild/logs/backup_sync.pid")

# Exclude patterns for rsync
EXCLUDE_PATTERNS = [
    "venv/",
    "**/venv/",
    "**/__pycache__/",
    "**/*.pyc",
    "**/*.pyo",
    "**/.git/",
    "**/target/",
    "**/node_modules/",
    "**/.pytest_cache/",
    "**/.mypy_cache/",
    "**/.ruff_cache/",
    "**/*.swp",
    "**/*.swo",
    "**/.DS_Store",
    "**/Thumbs.db",
    "logs/backup_sync.log",
    "logs/backup_sync.pid",
]

# Setup logging
def setup_logging():
    """Configure logging to file and console."""
    log_dir = LOG_FILE.parent
    log_dir.mkdir(parents=True, exist_ok=True)
    
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(levelname)s - %(message)s',
        handlers=[
            logging.FileHandler(LOG_FILE),
            logging.StreamHandler(sys.stdout)
        ]
    )
    return logging.getLogger(__name__)

logger = setup_logging()

# Global flag for graceful shutdown
shutdown_flag = False

def signal_handler(signum, frame):
    """Handle shutdown signals gracefully."""
    global shutdown_flag
    logger.info(f"Received signal {signum}, initiating graceful shutdown...")
    shutdown_flag = True

def check_pendrive_mounted():
    """Check if pendrive is mounted."""
    if not BACKUP_DIR.parent.exists():
        logger.error(f"Pendrive mount point {BACKUP_DIR.parent} does not exist!")
        return False
    
    # Check if it's actually a mount point
    result = subprocess.run(
        ["mountpoint", "-q", str(BACKUP_DIR.parent)],
        capture_output=True
    )
    if result.returncode != 0:
        logger.error(f"Pendrive not mounted at {BACKUP_DIR.parent}")
        return False
    
    return True

def create_backup_directory():
    """Create backup directory if it doesn't exist."""
    try:
        BACKUP_DIR.mkdir(parents=True, exist_ok=True)
        logger.info(f"Backup directory ready: {BACKUP_DIR}")
        return True
    except Exception as e:
        logger.error(f"Failed to create backup directory: {e}")
        return False

def build_rsync_command():
    """Build rsync command with exclude patterns."""
    exclude_args = []
    for pattern in EXCLUDE_PATTERNS:
        exclude_args.extend(["--exclude", pattern])
    
    # rsync command
    cmd = [
        "rsync",
        "-av",  # archive mode, verbose
        "--delete",  # delete files in destination that don't exist in source
        "--progress",  # show progress
        "--human-readable",  # human-readable sizes
        "--stats",  # show statistics
    ] + exclude_args + [
        f"{SOURCE_DIR}/",
        f"{BACKUP_DIR}/"
    ]
    
    return cmd

def perform_sync():
    """Perform a single sync operation."""
    if not check_pendrive_mounted():
        return False
    
    if not create_backup_directory():
        return False
    
    cmd = build_rsync_command()
    logger.info(f"Starting sync: {SOURCE_DIR} -> {BACKUP_DIR}")
    logger.debug(f"Command: {' '.join(cmd)}")
    
    try:
        start_time = time.time()
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=3600  # 1 hour timeout
        )
        elapsed = time.time() - start_time
        
        if result.returncode == 0:
            logger.info(f"Sync completed successfully in {elapsed:.2f} seconds")
            # Log statistics if available
            if result.stdout:
                stats_lines = [line for line in result.stdout.split('\n') if 'Total file size' in line or 'Number of files' in line]
                for stat in stats_lines:
                    logger.info(f"  {stat}")
            return True
        else:
            logger.error(f"Sync failed with return code {result.returncode}")
            logger.error(f"Error output: {result.stderr}")
            return False
            
    except subprocess.TimeoutExpired:
        logger.error("Sync operation timed out after 1 hour")
        return False
    except Exception as e:
        logger.error(f"Exception during sync: {e}")
        return False

def write_pid_file():
    """Write PID file for process management."""
    try:
        PID_FILE.parent.mkdir(parents=True, exist_ok=True)
        with open(PID_FILE, 'w') as f:
            f.write(str(os.getpid()))
        logger.info(f"PID file written: {PID_FILE}")
    except Exception as e:
        logger.error(f"Failed to write PID file: {e}")

def remove_pid_file():
    """Remove PID file on shutdown."""
    try:
        if PID_FILE.exists():
            PID_FILE.unlink()
            logger.info("PID file removed")
    except Exception as e:
        logger.error(f"Failed to remove PID file: {e}")

def main():
    """Main sync loop."""
    global shutdown_flag
    
    # Register signal handlers
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)
    
    # Write PID file
    write_pid_file()
    
    logger.info("=" * 60)
    logger.info("RansomEye Backup Sync Service Started")
    logger.info(f"Source: {SOURCE_DIR}")
    logger.info(f"Destination: {BACKUP_DIR}")
    logger.info(f"Sync Interval: {SYNC_INTERVAL} seconds")
    logger.info("=" * 60)
    
    # Initial sync
    logger.info("Performing initial sync...")
    perform_sync()
    
    # Continuous sync loop
    sync_count = 0
    while not shutdown_flag:
        try:
            time.sleep(SYNC_INTERVAL)
            
            if shutdown_flag:
                break
            
            sync_count += 1
            logger.info(f"\n--- Sync Cycle #{sync_count} at {datetime.now().strftime('%Y-%m-%d %H:%M:%S')} ---")
            perform_sync()
            
        except KeyboardInterrupt:
            logger.info("Keyboard interrupt received")
            shutdown_flag = True
            break
        except Exception as e:
            logger.error(f"Unexpected error in sync loop: {e}")
            time.sleep(5)  # Brief pause before retrying
    
    logger.info("Backup sync service shutting down...")
    remove_pid_file()
    logger.info("Shutdown complete")

if __name__ == "__main__":
    main()

