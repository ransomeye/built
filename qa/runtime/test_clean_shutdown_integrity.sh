#!/bin/bash
# Path and File Name : /home/ransomeye/rebuild/qa/runtime/test_clean_shutdown_integrity.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Test script for clean shutdown integrity scenario

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPORT_DIR="$SCRIPT_DIR/reports"
mkdir -p "$REPORT_DIR"

TEST_NAME="clean_shutdown_integrity"
LOG_FILE="$REPORT_DIR/${TEST_NAME}.log"
RESULT_FILE="$REPORT_DIR/${TEST_NAME}.result.json"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

error() {
    echo -e "${RED}ERROR: $1${NC}" | tee -a "$LOG_FILE"
}

success() {
    echo -e "${GREEN}✓ $1${NC}" | tee -a "$LOG_FILE"
}

warning() {
    echo -e "${YELLOW}⚠ $1${NC}" | tee -a "$LOG_FILE"
}

log "=========================================="
log "TEST: Clean Shutdown Integrity"
log "=========================================="
log ""

# Check if running as root
if [[ $EUID -ne 0 ]]; then
    error "This test must be run as root (for systemctl operations)"
    exit 1
fi

# Ensure environment is clean (no dry-run)
ENV_FILE="/etc/ransomeye/ransomeye.env"
if [[ -f "$ENV_FILE" ]]; then
    # Remove dry-run if present
    if grep -q "^RANSOMEYE_DRY_RUN=" "$ENV_FILE"; then
        log "Removing dry-run mode for clean shutdown test"
        sed -i '/^RANSOMEYE_DRY_RUN=/d' "$ENV_FILE"
    fi
fi

# Record start time
START_TIME=$(date +%s.%N)

# Start orchestrator service successfully
log "Starting orchestrator service..."
if systemctl start ransomeye-orchestrator.service 2>&1 | tee -a "$LOG_FILE"; then
    START_EXIT_CODE=0
else
    START_EXIT_CODE=${PIPESTATUS[0]}
fi

# Wait for service to start
log "Waiting for service to start..."
sleep 3

# Check if service is active
SERVICE_STATUS=$(systemctl is-active ransomeye-orchestrator.service 2>&1 || echo "inactive")
log "Service status after start: $SERVICE_STATUS"

if [[ "$SERVICE_STATUS" != "active" ]]; then
    error "Service failed to start (status: $SERVICE_STATUS)"
    log "Capturing journal logs for diagnosis..."
    journalctl -u ransomeye-orchestrator.service --no-pager -n 50 > "$REPORT_DIR/${TEST_NAME}.startup_failure.log" 2>&1 || true
    exit 1
fi

success "Service started successfully"

# Wait a moment for service to stabilize
sleep 2

# Record shutdown start time
SHUTDOWN_START_TIME=$(date +%s.%N)

# Send SIGTERM (systemctl stop sends SIGTERM)
log "Sending SIGTERM (via systemctl stop)..."
if systemctl stop ransomeye-orchestrator.service 2>&1 | tee -a "$LOG_FILE"; then
    STOP_EXIT_CODE=0
else
    STOP_EXIT_CODE=${PIPESTATUS[0]}
fi

# Wait for shutdown to complete
log "Waiting for shutdown to complete..."
sleep 3

# Record shutdown completion time
SHUTDOWN_END_TIME=$(date +%s.%N)
SHUTDOWN_TIME=$(echo "$SHUTDOWN_END_TIME - $SHUTDOWN_START_TIME" | bc)

# Check service status
FINAL_STATUS=$(systemctl is-active ransomeye-orchestrator.service 2>&1 || echo "inactive")
log "Service status after shutdown: $FINAL_STATUS"

# Get exit code
EXIT_CODE=$(systemctl show ransomeye-orchestrator.service -p ExecMainStatus --value 2>/dev/null || echo "unknown")
log "Exit code: $EXIT_CODE"

# Get journal logs
log "Capturing journal logs..."
JOURNAL_LOG="$REPORT_DIR/${TEST_NAME}.journal.log"
journalctl -u ransomeye-orchestrator.service --no-pager -n 200 > "$JOURNAL_LOG" 2>&1 || true
log "Journal logs saved to: $JOURNAL_LOG"

# Check for ordered shutdown indicators
ORDERED_SHUTDOWN_DETECTED=false
if grep -qi "shutdown\|shutting.*down\|cleanup\|teardown\|stopping" "$JOURNAL_LOG"; then
    success "Ordered shutdown indicators found"
    ORDERED_SHUTDOWN_DETECTED=true
fi

# Check for data loss warnings
DATA_LOSS_WARNING=false
if grep -qi "data.*loss\|warning.*data\|queue.*not.*flushed\|buffer.*not.*saved" "$JOURNAL_LOG"; then
    warning "Data loss warnings detected in logs"
    DATA_LOSS_WARNING=true
else
    success "No data loss warnings detected"
fi

# Check for panic messages
PANIC_DETECTED=false
if grep -qi "panic\|fatal\|abort\|segmentation.*fault" "$JOURNAL_LOG"; then
    error "Panic or fatal error detected in logs"
    PANIC_DETECTED=true
else
    success "No panic or fatal errors detected"
fi

# Check exit code (should be 0 for clean shutdown)
if [[ "$EXIT_CODE" == "0" ]]; then
    success "Exit code is zero (expected for clean shutdown)"
    EXIT_CODE_CORRECT=true
else
    if [[ "$EXIT_CODE" == "unknown" ]]; then
        warning "Could not determine exit code"
        EXIT_CODE_CORRECT=false
    else
        error "Exit code is non-zero (expected 0 for clean shutdown): $EXIT_CODE"
        EXIT_CODE_CORRECT=false
    fi
fi

# Check for graceful shutdown message
GRACEFUL_SHUTDOWN_DETECTED=false
if grep -qi "graceful\|clean.*shutdown\|shutdown.*complete\|exiting.*cleanly" "$JOURNAL_LOG"; then
    success "Graceful shutdown message detected"
    GRACEFUL_SHUTDOWN_DETECTED=true
fi

# Generate result JSON
cat > "$RESULT_FILE" <<EOF
{
  "test_name": "$TEST_NAME",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "expected_behavior": {
    "ordered_shutdown": true,
    "no_data_loss_warnings": true,
    "no_panic": true,
    "exit_code_0": true
  },
  "observed_behavior": {
    "service_started": true,
    "final_status": "$FINAL_STATUS",
    "exit_code": "$EXIT_CODE",
    "shutdown_time_seconds": "$SHUTDOWN_TIME",
    "ordered_shutdown_detected": $ORDERED_SHUTDOWN_DETECTED,
    "data_loss_warning": $DATA_LOSS_WARNING,
    "panic_detected": $PANIC_DETECTED,
    "graceful_shutdown_detected": $GRACEFUL_SHUTDOWN_DETECTED
  },
  "test_results": {
    "exit_code_correct": $EXIT_CODE_CORRECT,
    "ordered_shutdown": $ORDERED_SHUTDOWN_DETECTED,
    "no_data_loss": $([ "$DATA_LOSS_WARNING" == "false" ] && echo "true" || echo "false"),
    "no_panic": $([ "$PANIC_DETECTED" == "false" ] && echo "true" || echo "false"),
    "graceful_shutdown": $GRACEFUL_SHUTDOWN_DETECTED
  },
  "pass": $([ "$EXIT_CODE_CORRECT" == "true" ] && [ "$ORDERED_SHUTDOWN_DETECTED" == "true" ] && [ "$DATA_LOSS_WARNING" == "false" ] && [ "$PANIC_DETECTED" == "false" ] && echo "true" || echo "false")
}
EOF

log "Result saved to: $RESULT_FILE"

# Summary
log ""
log "=========================================="
log "TEST SUMMARY"
log "=========================================="
log "Service Started: true"
log "Final Status: $FINAL_STATUS"
log "Exit Code: $EXIT_CODE"
log "Shutdown Time: ${SHUTDOWN_TIME}s"
log "Ordered Shutdown: $ORDERED_SHUTDOWN_DETECTED"
log "Data Loss Warning: $DATA_LOSS_WARNING"
log "Panic Detected: $PANIC_DETECTED"
log "Graceful Shutdown: $GRACEFUL_SHUTDOWN_DETECTED"
log ""

if [[ "$EXIT_CODE_CORRECT" == "true" ]] && [[ "$ORDERED_SHUTDOWN_DETECTED" == "true" ]] && [[ "$DATA_LOSS_WARNING" == "false" ]] && [[ "$PANIC_DETECTED" == "false" ]]; then
    success "TEST PASSED: Clean shutdown integrity verified - ordered shutdown, no data loss, no panic"
    exit 0
else
    error "TEST FAILED: Clean shutdown integrity not verified"
    exit 1
fi

