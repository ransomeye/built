# Path and File Name : /home/ransomeye/rebuild/check_training_status.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYylg8CMw1iGsq7gU
# Details of functionality of this file: Check training status and progress

#!/bin/bash

PROJECT_ROOT="/home/ransomeye/rebuild"
LOG_DIR="${PROJECT_ROOT}/logs"
PID_FILE="${LOG_DIR}/training_pid.txt"
LOG_FILE="${LOG_DIR}/training_large_models.log"

echo "=========================================="
echo "RansomEye Training Status Check"
echo "=========================================="
echo ""

if [ -f "${PID_FILE}" ]; then
    PID=$(cat "${PID_FILE}")
    if ps -p "${PID}" > /dev/null 2>&1; then
        echo "✓ Training is RUNNING (PID: ${PID})"
        echo ""
        echo "Process info:"
        ps -p "${PID}" -o pid,ppid,cmd,etime,%mem,%cpu
        echo ""
        echo "Last 20 log lines:"
        echo "----------------------------------------"
        tail -20 "${LOG_FILE}"
        echo ""
        echo "To view live logs: tail -f ${LOG_FILE}"
    else
        echo "✗ Training is NOT running (stale PID file)"
        echo "Checking log file for completion status..."
        echo ""
        tail -50 "${LOG_FILE}" | grep -E "(COMPLETE|SUCCESS|FAILED|ERROR|✓|✗)" | tail -10
        rm -f "${PID_FILE}"
    fi
else
    echo "✗ No training process found (no PID file)"
    echo "Checking if training completed..."
    if [ -f "${LOG_FILE}" ]; then
        echo ""
        echo "Last log entries:"
        echo "----------------------------------------"
        tail -30 "${LOG_FILE}" | grep -E "(COMPLETE|SUCCESS|FAILED|ERROR|✓|✗)" | tail -10
    else
        echo "No log file found. Training may not have been started."
    fi
fi

echo ""
echo "Model sizes:"
find "${PROJECT_ROOT}" -name "*.model" -type f -exec ls -lh {} \; 2>/dev/null | awk '{printf "  %-60s %8s\n", $9, $5}' | head -15
