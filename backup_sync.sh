# Path and File Name : /home/ransomeye/rebuild/backup_sync.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Shell wrapper script to start/stop/status the backup sync service

#!/bin/bash

# Backup Sync Service Control Script
# Manages the continuous backup sync process

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PYTHON_SCRIPT="${SCRIPT_DIR}/backup_sync.py"
PID_FILE="${SCRIPT_DIR}/logs/backup_sync.pid"
LOG_FILE="${SCRIPT_DIR}/logs/backup_sync.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

check_pid() {
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if ps -p "$PID" > /dev/null 2>&1; then
            return 0
        else
            rm -f "$PID_FILE"
            return 1
        fi
    fi
    return 1
}

start() {
    if check_pid; then
        echo -e "${YELLOW}Backup sync service is already running (PID: $PID)${NC}"
        return 1
    fi
    
    echo -e "${GREEN}Starting backup sync service...${NC}"
    
    # Check if pendrive is mounted
    if ! mountpoint -q /mnt/pendrive; then
        echo -e "${RED}Error: Pendrive not mounted at /mnt/pendrive${NC}"
        echo "Please mount the pendrive first:"
        echo "  sudo mkdir -p /mnt/pendrive"
        echo "  sudo mount /dev/sdc1 /mnt/pendrive"
        return 1
    fi
    
    # Start the Python script in background
    nohup python3 "$PYTHON_SCRIPT" >> "$LOG_FILE" 2>&1 &
    
    sleep 2
    
    if check_pid; then
        echo -e "${GREEN}Backup sync service started successfully (PID: $PID)${NC}"
        echo "Log file: $LOG_FILE"
        echo "Monitor with: tail -f $LOG_FILE"
        return 0
    else
        echo -e "${RED}Failed to start backup sync service${NC}"
        echo "Check log file: $LOG_FILE"
        return 1
    fi
}

stop() {
    if ! check_pid; then
        echo -e "${YELLOW}Backup sync service is not running${NC}"
        return 1
    fi
    
    echo -e "${YELLOW}Stopping backup sync service (PID: $PID)...${NC}"
    kill "$PID"
    
    # Wait for process to stop
    for i in {1..10}; do
        if ! ps -p "$PID" > /dev/null 2>&1; then
            echo -e "${GREEN}Backup sync service stopped${NC}"
            rm -f "$PID_FILE"
            return 0
        fi
        sleep 1
    done
    
    # Force kill if still running
    if ps -p "$PID" > /dev/null 2>&1; then
        echo -e "${RED}Force killing backup sync service...${NC}"
        kill -9 "$PID"
        rm -f "$PID_FILE"
    fi
    
    return 0
}

status() {
    if check_pid; then
        echo -e "${GREEN}Backup sync service is running (PID: $PID)${NC}"
        
        # Show last few log lines
        if [ -f "$LOG_FILE" ]; then
            echo ""
            echo "Last 5 log entries:"
            tail -n 5 "$LOG_FILE"
        fi
        
        return 0
    else
        echo -e "${RED}Backup sync service is not running${NC}"
        return 1
    fi
}

restart() {
    stop
    sleep 2
    start
}

case "$1" in
    start)
        start
        ;;
    stop)
        stop
        ;;
    restart)
        restart
        ;;
    status)
        status
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status}"
        exit 1
        ;;
esac

exit $?

