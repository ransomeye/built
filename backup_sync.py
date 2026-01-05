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
import shutil
import select
import fcntl
from pathlib import Path
from datetime import datetime

# Configuration
SOURCE_DIR = Path("/home/ransomeye/rebuild")
BACKUP_DIR = Path("/mnt/pendrive/rebuild")
LOG_FILE = Path("/home/ransomeye/rebuild/logs/backup_sync.log")
SYNC_INTERVAL = 3600  # 1 hour between syncs
PID_FILE = Path("/home/ransomeye/rebuild/logs/backup_sync.pid")

# Exclude patterns for rsync
EXCLUDE_PATTERNS = [
    "venv/",
    "**/venv/",
    ".venv/",
    "**/.venv/",
    ".venv-*/",
    "**/.venv-*/",
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
    # Note: FAT32 doesn't support symlinks, so we copy them as files or skip
    cmd = [
        "rsync",
        "-av",  # archive mode, verbose
        "--delete",  # delete files in destination that don't exist in source
        "--info=progress2",  # show overall progress
        "--human-readable",  # human-readable sizes
        "--stats",  # show statistics
        "--partial",  # keep partial files on interruption
        "--partial-dir=.rsync-partial",  # store partial files here
        "--copy-links",  # Copy symlinks as files (FAT32 doesn't support symlinks)
        "--safe-links",  # Ignore symlinks that point outside the tree
        "--iconv=utf-8,utf-8",  # Handle character encoding
        "--modify-window=2",  # Allow 2 second time difference (FAT32 precision)
        "--timeout=300",  # 5 minute timeout per file operation
        "--contimeout=60",  # 1 minute connection timeout
        "--bwlimit=0",  # No bandwidth limit (use full speed)
        "--whole-file",  # Transfer whole files (faster for local USB)
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
        last_progress_log = start_time
        MAX_SYNC_DURATION = 7200  # 2 hours maximum sync time
        
        # Log initial sync start
        logger.info("Calculating directory sizes...")
        try:
            # Use du command for accurate directory sizes
            source_result = subprocess.run(
                ["du", "-sb", str(SOURCE_DIR)],
                capture_output=True,
                text=True,
                timeout=60
            )
            if source_result.returncode == 0:
                source_size = int(source_result.stdout.split()[0])
                logger.info(f"Source directory size: {source_size / (1024**3):.2f} GB")
            else:
                source_size = 0
                logger.info("Could not calculate source size")
        except Exception as e:
            logger.debug(f"Size calculation error: {e}")
            source_size = 0
        
        # Run rsync with progress monitoring
        process = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,  # Combine stderr into stdout
            text=True,
            bufsize=1
        )
        
        # Set stdout to non-blocking
        fd = process.stdout.fileno()
        fl = fcntl.fcntl(fd, fcntl.F_GETFL)
        fcntl.fcntl(fd, fcntl.F_SETFL, fl | os.O_NONBLOCK)
        
        output_lines = []
        last_progress_bytes = 0
        last_progress_time = start_time
        stuck_threshold = 300  # 5 minutes without progress = stuck
        
        # Monitor progress - parse rsync output in real-time
        while True:
            return_code = process.poll()
            if return_code is not None:
                # Process finished, read remaining output
                try:
                    remaining = process.stdout.read()
                    if remaining:
                        output_lines.append(remaining)
                except:
                    pass
                break
            
            current_time = time.time()
            elapsed_so_far = current_time - start_time
            
            # Try to read rsync output (non-blocking)
            try:
                # Use select to check if data is available (with timeout)
                ready, _, _ = select.select([process.stdout], [], [], 1.0)
                if ready:
                    chunk = process.stdout.read(4096)
                    if chunk:
                        output_lines.append(chunk)
                        
                        # Parse progress from rsync output
                        # rsync --info=progress2 outputs lines like:
                        # "  7,234,567  73%  123.45M/s    0:00:45  (xfr#1234, to-chk=567/890)"
                        for line in chunk.split('\n'):
                            line = line.strip()
                            if '%' in line and ('xfr#' in line or 'to-chk=' in line):
                                # Extract percentage
                                try:
                                    parts = line.split()
                                    for part in parts:
                                        if '%' in part:
                                            pct = float(part.replace('%', ''))
                                            # Extract bytes transferred
                                            if len(parts) > 0:
                                                bytes_str = parts[0].replace(',', '')
                                                if bytes_str.isdigit():
                                                    last_progress_bytes = int(bytes_str)
                                                    last_progress_time = current_time
                                            break
                                except:
                                    pass
            except (OSError, ValueError):
                # No data available or parsing error, continue
                pass
            
            # Check for maximum sync duration timeout
            if elapsed_so_far > MAX_SYNC_DURATION:
                logger.error(f"Sync exceeded maximum duration ({MAX_SYNC_DURATION // 60} minutes). Terminating...")
                process.terminate()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    logger.warning("Process did not terminate gracefully, killing...")
                    process.kill()
                    process.wait()
                return False
            
            # Log progress every 30 seconds
            if current_time - last_progress_log >= 30:
                elapsed_min = int(elapsed_so_far // 60)
                elapsed_sec = int(elapsed_so_far % 60)
                
                # Check if stuck (no progress for 5+ minutes)
                time_since_progress = current_time - last_progress_time
                if time_since_progress > stuck_threshold and elapsed_so_far > stuck_threshold:
                    logger.warning(f"Sync appears stuck! No progress for {int(time_since_progress // 60)}m {int(time_since_progress % 60)}s")
                    logger.warning(f"Last progress: {last_progress_bytes / (1024**3):.2f} GB at {int((current_time - last_progress_time) // 60)}m ago")
                    logger.warning("This may indicate:")
                    logger.warning("  - Slow USB pendrive I/O")
                    logger.warning("  - Large file transfer in progress")
                    logger.warning("  - Filesystem issues on destination")
                    logger.warning("  - Network issues (if using network path)")
                    logger.warning("Consider checking:")
                    logger.warning(f"  - Disk I/O: iostat -x 1")
                    logger.warning(f"  - rsync process: ps aux | grep rsync")
                    logger.warning(f"  - Destination mount: df -h {BACKUP_DIR}")
                    # If stuck for more than 10 minutes, consider terminating
                    if time_since_progress > 600:  # 10 minutes
                        logger.error("Sync stuck for over 10 minutes. Terminating rsync process...")
                        process.terminate()
                        try:
                            process.wait(timeout=10)
                        except subprocess.TimeoutExpired:
                            logger.warning("Process did not terminate gracefully, killing...")
                            process.kill()
                            process.wait()
                        return False
                
                # Calculate progress if we have source size
                if source_size > 0 and last_progress_bytes > 0:
                    progress_pct = (last_progress_bytes / source_size * 100)
                    logger.info(f"Sync in progress... ({elapsed_min}m {elapsed_sec}s elapsed, ~{progress_pct:.1f}% complete, {last_progress_bytes / (1024**3):.2f} GB synced)")
                else:
                    logger.info(f"Sync in progress... ({elapsed_min}m {elapsed_sec}s elapsed)")
                
                last_progress_log = current_time
            
            time.sleep(1)  # Check every 1 second (more responsive)
        
        # Read any remaining output
        try:
            remaining = process.stdout.read()
            if remaining:
                output_lines.append(remaining)
        except:
            pass
        
        elapsed = time.time() - start_time
        result_stdout = ''.join(output_lines)
        
        # Create a result-like object
        class Result:
            def __init__(self, returncode, stdout, stderr):
                self.returncode = returncode
                self.stdout = stdout
                self.stderr = stderr or ""
        
        result = Result(return_code, result_stdout, "")
        
        if result.returncode == 0:
            elapsed_min = int(elapsed // 60)
            elapsed_sec = int(elapsed % 60)
            logger.info(f"Sync completed successfully in {elapsed_min}m {elapsed_sec}s")
            # Log statistics if available
            if result.stdout:
                stats_lines = [line for line in result.stdout.split('\n') 
                             if any(keyword in line.lower() for keyword in 
                                   ['total file size', 'number of files', 'total transferred', 'speedup'])]
                for stat in stats_lines:
                    if stat.strip():
                        logger.info(f"  {stat.strip()}")
            return True
        elif result.returncode == 23:
            # Exit code 23: partial transfer due to error (common with FAT32)
            elapsed_min = int(elapsed // 60)
            elapsed_sec = int(elapsed % 60)
            logger.warning(f"Sync completed with warnings (exit code 23) in {elapsed_min}m {elapsed_sec}s")
            logger.warning("Some files may not have been transferred due to FAT32 limitations:")
            logger.warning("  - Symlinks are copied as files (FAT32 doesn't support symlinks)")
            logger.warning("  - Files with invalid characters may be skipped")
            logger.warning("  - Very long filenames may be truncated")
            # Log statistics if available
            if result.stdout:
                stats_lines = [line for line in result.stdout.split('\n') 
                             if any(keyword in line.lower() for keyword in 
                                   ['total file size', 'number of files', 'total transferred', 'speedup'])]
                for stat in stats_lines:
                    if stat.strip():
                        logger.info(f"  {stat.strip()}")
            # Still return True as partial success is acceptable for backup
            return True
        else:
            logger.error(f"Sync failed with return code {result.returncode}")
            if result.stderr:
                # Show first few error lines to avoid log spam
                error_lines = result.stderr.split('\n')[:10]
                for line in error_lines:
                    if line.strip():
                        logger.error(f"  {line.strip()}")
                if len(result.stderr.split('\n')) > 10:
                    logger.error(f"  ... and {len(result.stderr.split('\n')) - 10} more error lines")
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

def is_git_repository():
    """Check if source directory is a git repository."""
    git_dir = SOURCE_DIR / ".git"
    return git_dir.exists() and git_dir.is_dir()

def perform_git_commit():
    """Perform git add, commit before sync."""
    if not is_git_repository():
        logger.debug("Source directory is not a git repository, skipping git commit")
        return True
    
    try:
        # Change to source directory
        os.chdir(SOURCE_DIR)
        
        # Check if there are any changes
        result = subprocess.run(
            ["git", "status", "--porcelain"],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        if result.returncode != 0:
            logger.warning(f"Git status check failed: {result.stderr}")
            return False
        
        # If no changes, skip commit
        if not result.stdout.strip():
            logger.debug("No git changes detected, skipping commit")
            return True
        
        # Add all changes
        logger.info("Staging git changes...")
        add_result = subprocess.run(
            ["git", "add", "-A"],
            capture_output=True,
            text=True,
            timeout=60
        )
        
        if add_result.returncode != 0:
            logger.error(f"Git add failed: {add_result.stderr}")
            return False
        
        # Create commit with timestamp
        commit_message = f"Auto-backup commit: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}"
        logger.info(f"Creating git commit: {commit_message}")
        
        commit_result = subprocess.run(
            ["git", "commit", "-m", commit_message],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        if commit_result.returncode == 0:
            logger.info("Git commit created successfully")
            # Try to push if remote is configured (non-blocking)
            try:
                push_result = subprocess.run(
                    ["git", "push"],
                    capture_output=True,
                    text=True,
                    timeout=60
                )
                if push_result.returncode == 0:
                    logger.info("Git push completed successfully")
                else:
                    logger.debug(f"Git push skipped or failed (non-critical): {push_result.stderr[:100]}")
            except Exception as e:
                logger.debug(f"Git push skipped (non-critical): {e}")
            return True
        elif "nothing to commit" in commit_result.stdout.lower() or "nothing to commit" in commit_result.stderr.lower():
            logger.debug("No changes to commit")
            return True
        else:
            logger.warning(f"Git commit failed: {commit_result.stderr}")
            return False
            
    except subprocess.TimeoutExpired:
        logger.error("Git operation timed out")
        return False
    except Exception as e:
        logger.error(f"Exception during git commit: {e}")
        return False
    finally:
        # Restore working directory
        os.chdir(SOURCE_DIR.parent if SOURCE_DIR.parent else "/")

def main():
    """Main sync loop."""
    global shutdown_flag
    
    # Register signal handlers
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)
    
    # Write PID file
    write_pid_file()
    
    logger.info("=" * 60)
    logger.info("Personal Backup Sync Service Started")
    logger.info(f"Source: {SOURCE_DIR}")
    logger.info(f"Destination: {BACKUP_DIR}")
    logger.info(f"Sync Interval: {SYNC_INTERVAL} seconds ({SYNC_INTERVAL // 60} minutes)")
    logger.info("=" * 60)
    
    # Initial sync
    logger.info("Performing initial sync...")
    perform_git_commit()
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
            # Perform git commit before sync
            perform_git_commit()
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

