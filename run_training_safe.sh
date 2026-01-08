# Path and File Name : /home/ransomeye/rebuild/run_training_safe.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Safe training script that survives SSH disconnections

#!/bin/bash
# Safe training runner that survives SSH disconnections

set -e

PROJECT_ROOT="/home/ransomeye/rebuild"
LOG_DIR="${PROJECT_ROOT}/logs"
PID_FILE="${LOG_DIR}/training_pid.txt"
LOG_FILE="${LOG_DIR}/training_large_models.log"

# Ensure log directory exists
mkdir -p "${LOG_DIR}"

# Function to log with timestamp
log_with_timestamp() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "${LOG_FILE}"
}

# Check if training is already running
if [ -f "${PID_FILE}" ]; then
    PID=$(cat "${PID_FILE}")
    if ps -p "${PID}" > /dev/null 2>&1; then
        log_with_timestamp "Training already running with PID: ${PID}"
        log_with_timestamp "To monitor: tail -f ${LOG_FILE}"
        exit 1
    else
        log_with_timestamp "Stale PID file found, removing..."
        rm -f "${PID_FILE}"
    fi
fi

# Change to project root
cd "${PROJECT_ROOT}"

# Start training in background with nohup
log_with_timestamp "Starting RansomEye training with large datasets..."
log_with_timestamp "This will take 12-18 hours. Training will continue even if SSH disconnects."
log_with_timestamp "Log file: ${LOG_FILE}"
log_with_timestamp "PID file: ${PID_FILE}"

# Run with unbuffered output and nohup
nohup python3 -u "${PROJECT_ROOT}/train_all_models_complete.py" >> "${LOG_FILE}" 2>&1 &
TRAINING_PID=$!

# Save PID
echo "${TRAINING_PID}" > "${PID_FILE}"
log_with_timestamp "Training started with PID: ${TRAINING_PID}"
log_with_timestamp ""
log_with_timestamp "To monitor progress:"
log_with_timestamp "  tail -f ${LOG_FILE}"
log_with_timestamp ""
log_with_timestamp "To check if still running:"
log_with_timestamp "  ps -p ${TRAINING_PID}"
log_with_timestamp ""
log_with_timestamp "To stop training:"
log_with_timestamp "  kill ${TRAINING_PID}"
log_with_timestamp ""
log_with_timestamp "Training is running in background and will continue even if you disconnect SSH."

exit 0
