#!/bin/bash
# Path and File Name : /home/ransomeye/rebuild/qa/runtime/test_dry_run_validation.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Test script for dry-run validation scenario

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPORT_DIR="$SCRIPT_DIR/reports"
mkdir -p "$REPORT_DIR"

TEST_NAME="dry_run_validation"
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
log "TEST: Dry-Run Validation"
log "=========================================="
log ""

# Check if running as root
if [[ $EUID -ne 0 ]]; then
    error "This test must be run as root (for systemctl operations)"
    exit 1
fi

# Backup original environment file
ENV_FILE="/etc/ransomeye/ransomeye.env"
ENV_BACKUP="/tmp/ransomeye.env.backup.$$"

if [[ ! -f "$ENV_FILE" ]]; then
    error "Environment file not found: $ENV_FILE"
    exit 1
fi

log "Backing up environment file: $ENV_BACKUP"
cp "$ENV_FILE" "$ENV_BACKUP"

# Set dry-run mode
log "Setting RANSOMEYE_DRY_RUN=1"
if grep -q "^RANSOMEYE_DRY_RUN=" "$ENV_FILE"; then
    sed -i 's/^RANSOMEYE_DRY_RUN=.*/RANSOMEYE_DRY_RUN=1/' "$ENV_FILE"
else
    echo "RANSOMEYE_DRY_RUN=1" >> "$ENV_FILE"
fi

# Verify dry-run is set
if ! grep -q "^RANSOMEYE_DRY_RUN=1" "$ENV_FILE"; then
    error "Failed to set RANSOMEYE_DRY_RUN=1"
    mv "$ENV_BACKUP" "$ENV_FILE"
    exit 1
fi

log "Dry-run mode enabled"

# Record start time
START_TIME=$(date +%s.%N)

# Start orchestrator service with dry-run
log "Starting orchestrator service with dry-run mode..."
if systemctl start ransomeye-orchestrator.service 2>&1 | tee -a "$LOG_FILE"; then
    START_EXIT_CODE=0
else
    START_EXIT_CODE=${PIPESTATUS[0]}
fi

# Wait for orchestrator to complete (dry-run should exit quickly)
log "Waiting for orchestrator to complete (dry-run mode)..."
sleep 5

# Check if service has exited (dry-run should exit after READY)
SERVICE_STATUS=$(systemctl is-active ransomeye-orchestrator.service 2>&1 || echo "inactive")
log "Service status after dry-run: $SERVICE_STATUS"

# Record completion time
COMPLETION_TIME=$(date +%s.%N)
STARTUP_TIME=$(echo "$COMPLETION_TIME - $START_TIME" | bc)

# Get exit code
EXIT_CODE=$(systemctl show ransomeye-orchestrator.service -p ExecMainStatus --value 2>/dev/null || echo "unknown")
log "Exit code: $EXIT_CODE"

# Get journal logs
log "Capturing journal logs..."
JOURNAL_LOG="$REPORT_DIR/${TEST_NAME}.journal.log"
journalctl -u ransomeye-orchestrator.service --no-pager -n 200 > "$JOURNAL_LOG" 2>&1 || true
log "Journal logs saved to: $JOURNAL_LOG"

# Check for dry-run indicators
DRY_RUN_DETECTED=false
if grep -qi "dry.*run\|DRY_RUN\|dry-run" "$JOURNAL_LOG"; then
    success "Dry-run mode detected in logs"
    DRY_RUN_DETECTED=true
fi

# Check for READY state
READY_STATE_DETECTED=false
if grep -qi "ready\|READY\|initialized.*successfully" "$JOURNAL_LOG"; then
    success "READY state reached"
    READY_STATE_DETECTED=true
fi

# Check for RUNNING state (should NOT be present in dry-run)
RUNNING_STATE_DETECTED=false
if grep -qi "running\|RUNNING\|entering.*running" "$JOURNAL_LOG"; then
    warning "RUNNING state detected (should not occur in dry-run)"
    RUNNING_STATE_DETECTED=true
else
    success "RUNNING state not detected (expected for dry-run)"
fi

# Check exit code (should be 0 for successful dry-run)
if [[ "$EXIT_CODE" == "0" ]]; then
    success "Exit code is zero (expected for successful dry-run)"
    EXIT_CODE_CORRECT=true
else
    if [[ "$EXIT_CODE" == "unknown" ]]; then
        warning "Could not determine exit code"
        EXIT_CODE_CORRECT=false
    else
        error "Exit code is non-zero (expected 0 for successful dry-run): $EXIT_CODE"
        EXIT_CODE_CORRECT=false
    fi
fi

# Check for clean exit message
CLEAN_EXIT_DETECTED=false
if grep -qi "dry.*run.*complete\|initialization.*complete\|exiting\|shutdown" "$JOURNAL_LOG"; then
    success "Clean exit message detected"
    CLEAN_EXIT_DETECTED=true
fi

# Restore original environment file
log "Restoring original environment file"
mv "$ENV_BACKUP" "$ENV_FILE"

# Generate result JSON
cat > "$RESULT_FILE" <<EOF
{
  "test_name": "$TEST_NAME",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "expected_behavior": {
    "full_startup_sequence_executes": true,
    "ready_reached": true,
    "no_running_state": true,
    "clean_exit_code_0": true
  },
  "observed_behavior": {
    "service_status": "$SERVICE_STATUS",
    "exit_code": "$EXIT_CODE",
    "startup_time_seconds": "$STARTUP_TIME",
    "dry_run_detected": $DRY_RUN_DETECTED,
    "ready_state_detected": $READY_STATE_DETECTED,
    "running_state_detected": $RUNNING_STATE_DETECTED,
    "clean_exit_detected": $CLEAN_EXIT_DETECTED
  },
  "test_results": {
    "exit_code_correct": $EXIT_CODE_CORRECT,
    "dry_run_detected": $DRY_RUN_DETECTED,
    "ready_reached": $READY_STATE_DETECTED,
    "no_running_state": $([ "$RUNNING_STATE_DETECTED" == "false" ] && echo "true" || echo "false"),
    "clean_exit": $CLEAN_EXIT_DETECTED
  },
  "pass": $([ "$EXIT_CODE_CORRECT" == "true" ] && [ "$DRY_RUN_DETECTED" == "true" ] && [ "$READY_STATE_DETECTED" == "true" ] && [ "$RUNNING_STATE_DETECTED" == "false" ] && echo "true" || echo "false")
}
EOF

log "Result saved to: $RESULT_FILE"

# Summary
log ""
log "=========================================="
log "TEST SUMMARY"
log "=========================================="
log "Service Status: $SERVICE_STATUS"
log "Exit Code: $EXIT_CODE"
log "Startup Time: ${STARTUP_TIME}s"
log "Dry-Run Detected: $DRY_RUN_DETECTED"
log "READY State Reached: $READY_STATE_DETECTED"
log "RUNNING State Detected: $RUNNING_STATE_DETECTED"
log "Clean Exit: $CLEAN_EXIT_DETECTED"
log ""

if [[ "$EXIT_CODE_CORRECT" == "true" ]] && [[ "$DRY_RUN_DETECTED" == "true" ]] && [[ "$READY_STATE_DETECTED" == "true" ]] && [[ "$RUNNING_STATE_DETECTED" == "false" ]]; then
    success "TEST PASSED: Dry-run validation correctly executes startup sequence and exits cleanly"
    exit 0
else
    error "TEST FAILED: Dry-run validation did not behave as expected"
    exit 1
fi

