# Personal Backup System - Quick Reference

## Overview
This is a personal backup system that continuously syncs your `/home/ransomeye/rebuild` folder to an external pendrive at `/mnt/pendrive/rebuild`, excluding venv files and other unnecessary files.

## Features
- **Continuous Sync**: Automatically syncs every 1 hour
- **Git Commit**: Automatically commits changes to git before each sync
- **Auto-Mount**: Pendrive automatically mounts at boot
- **Auto-Start**: Backup service automatically starts at boot
- **Excludes venv**: All virtual environment files are excluded from backup
- **Non-Stop**: Service runs continuously and restarts automatically if it fails

## Files Created
- `backup_sync.py` - Main Python sync script
- `backup_sync.sh` - Manual control script (start/stop/status)
- `mnt-pendrive.mount` - Systemd mount unit for pendrive
- `backup-sync.service` - Systemd service for backup sync

## Manual Control

### Using the shell script:
```bash
# Start backup sync
./backup_sync.sh start

# Stop backup sync
./backup_sync.sh stop

# Check status
./backup_sync.sh status

# Restart backup sync
./backup_sync.sh restart
```

### Using systemctl:
```bash
# Check mount status
sudo systemctl status mnt-pendrive.mount

# Check backup service status
sudo systemctl status backup-sync.service

# View logs
sudo journalctl -u backup-sync.service -f

# Restart services
sudo systemctl restart mnt-pendrive.mount
sudo systemctl restart backup-sync.service
```

## Logs
- Service logs: `sudo journalctl -u backup-sync.service -f`
- Application logs: `/home/ransomeye/rebuild/logs/backup_sync.log`
- View log file: `tail -f /home/ransomeye/rebuild/logs/backup_sync.log`

## Excluded Files/Directories
The backup excludes:
- `venv/` and all `**/venv/` directories
- `__pycache__/` directories
- `.pyc` and `.pyo` files
- `.git/` directories
- `target/` directories (Rust build artifacts)
- `node_modules/` directories
- Various cache directories (`.pytest_cache`, `.mypy_cache`, `.ruff_cache`)
- Temporary files (`.swp`, `.swo`, `.DS_Store`, `Thumbs.db`)
- Backup sync log and PID files

## Backup Location
- Source: `/home/ransomeye/rebuild`
- Destination: `/mnt/pendrive/rebuild`
- Pendrive UUID: `1601-1054`
- Pendrive Device: `/dev/sdc1`

## Auto-Start Configuration
Both services are enabled to start at boot:
- `mnt-pendrive.mount` - Mounts the pendrive
- `backup-sync.service` - Starts the backup sync

To disable auto-start:
```bash
sudo systemctl disable mnt-pendrive.mount backup-sync.service
```

To re-enable:
```bash
sudo systemctl enable mnt-pendrive.mount backup-sync.service
```

## Troubleshooting

### Pendrive not mounting
1. Check if pendrive is connected: `lsblk | grep sdc`
2. Check mount status: `sudo systemctl status mnt-pendrive.mount`
3. Manually mount: `sudo mount /dev/sdc1 /mnt/pendrive`

### Backup service not running
1. Check service status: `sudo systemctl status backup-sync.service`
2. Check logs: `sudo journalctl -u backup-sync.service -n 50`
3. Check if pendrive is mounted: `mount | grep pendrive`
4. Restart service: `sudo systemctl restart backup-sync.service`

### Check sync progress
```bash
# View real-time logs
tail -f /home/ransomeye/rebuild/logs/backup_sync.log

# Check what's being synced
sudo journalctl -u backup-sync.service -f
```

## Notes
- Sync interval is set to 1 hour (3600 seconds, configurable in `backup_sync.py`)
- Git commit is performed automatically before each sync (if source is a git repository)
- Git push is attempted after commit (non-blocking if it fails)
- The service will wait for the pendrive to be mounted before starting
- If the pendrive is removed, the service will keep retrying until it's reconnected
- The backup uses `rsync` with `--delete` flag, so files deleted in source will be deleted in backup

