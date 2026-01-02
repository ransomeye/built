#!/bin/bash
# Path and File Name : /home/ransomeye/rebuild/qa/runtime/run_all_tests.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Main test runner for all runtime validation and failure injection tests

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
REPORT_DIR="$SCRIPT_DIR/reports"
mkdir -p "$REPORT_DIR"

MAIN_LOG="$REPORT_DIR/runtime_validation.log"
SUMMARY_FILE="$REPORT_DIR/test_summary.json"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$MAIN_LOG"
}

error() {
    echo -e "${RED}ERROR: $1${NC}" | tee -a "$MAIN_LOG"
}

success() {
    echo -e "${GREEN}✓ $1${NC}" | tee -a "$MAIN_LOG"
}

warning() {
    echo -e "${YELLOW}⚠ $1${NC}" | tee -a "$MAIN_LOG"
}

info() {
    echo -e "${BLUE}ℹ $1${NC}" | tee -a "$MAIN_LOG"
}

log "=========================================="
log "RUNTIME VALIDATION TEST SUITE"
log "=========================================="
log "Starting comprehensive runtime validation tests"
log ""

# Check if running as root
if [[ $EUID -ne 0 ]]; then
    error "This test suite must be run as root (for systemctl operations)"
    exit 1
fi

# Test scripts
FAILURE_INJECTION_TESTS=(
    "$PROJECT_ROOT/qa/failure_injection/test_missing_trust_material.sh"
    "$PROJECT_ROOT/qa/failure_injection/test_invalid_policy_signature.sh"
    "$PROJECT_ROOT/qa/failure_injection/test_missing_policy_directory.sh"
    "$PROJECT_ROOT/qa/failure_injection/test_health_gate_failure.sh"
)

RUNTIME_TESTS=(
    "$PROJECT_ROOT/qa/runtime/test_dry_run_validation.sh"
    "$PROJECT_ROOT/qa/runtime/test_clean_shutdown_integrity.sh"
)

# Make all test scripts executable
log "Making test scripts executable..."
for test in "${FAILURE_INJECTION_TESTS[@]}" "${RUNTIME_TESTS[@]}"; do
    if [[ -f "$test" ]]; then
        chmod +x "$test"
        log "  Made executable: $(basename "$test")"
    else
        warning "Test script not found: $test"
    fi
done

# Test results
declare -A TEST_RESULTS
declare -A TEST_EXIT_CODES
declare -A TEST_TIMES

# Run failure injection tests
log ""
log "=========================================="
log "FAILURE INJECTION TESTS"
log "=========================================="
log ""

FAILURE_TESTS_PASSED=0
FAILURE_TESTS_FAILED=0

for test in "${FAILURE_INJECTION_TESTS[@]}"; do
    if [[ ! -f "$test" ]]; then
        warning "Test script not found: $test"
        continue
    fi
    
    TEST_NAME=$(basename "$test" .sh)
    log "Running test: $TEST_NAME"
    
    TEST_START=$(date +%s.%N)
    
    if bash "$test" 2>&1 | tee -a "$MAIN_LOG"; then
        TEST_EXIT_CODE=0
        TEST_RESULTS["$TEST_NAME"]="PASS"
        ((FAILURE_TESTS_PASSED++)) || true
        success "Test passed: $TEST_NAME"
    else
        TEST_EXIT_CODE=${PIPESTATUS[0]}
        TEST_RESULTS["$TEST_NAME"]="FAIL"
        TEST_EXIT_CODES["$TEST_NAME"]=$TEST_EXIT_CODE
        ((FAILURE_TESTS_FAILED++)) || true
        error "Test failed: $TEST_NAME (exit code: $TEST_EXIT_CODE)"
    fi
    
    TEST_END=$(date +%s.%N)
    TEST_TIME=$(echo "$TEST_END - $TEST_START" | bc)
    TEST_TIMES["$TEST_NAME"]=$TEST_TIME
    
    log "Test completed in ${TEST_TIME}s"
    log ""
    
    # Brief pause between tests
    sleep 2
done

# Run runtime tests
log ""
log "=========================================="
log "RUNTIME VALIDATION TESTS"
log "=========================================="
log ""

RUNTIME_TESTS_PASSED=0
RUNTIME_TESTS_FAILED=0

for test in "${RUNTIME_TESTS[@]}"; do
    if [[ ! -f "$test" ]]; then
        warning "Test script not found: $test"
        continue
    fi
    
    TEST_NAME=$(basename "$test" .sh)
    log "Running test: $TEST_NAME"
    
    TEST_START=$(date +%s.%N)
    
    if bash "$test" 2>&1 | tee -a "$MAIN_LOG"; then
        TEST_EXIT_CODE=0
        TEST_RESULTS["$TEST_NAME"]="PASS"
        ((RUNTIME_TESTS_PASSED++)) || true
        success "Test passed: $TEST_NAME"
    else
        TEST_EXIT_CODE=${PIPESTATUS[0]}
        TEST_RESULTS["$TEST_NAME"]="FAIL"
        TEST_EXIT_CODES["$TEST_NAME"]=$TEST_EXIT_CODE
        ((RUNTIME_TESTS_FAILED++)) || true
        error "Test failed: $TEST_NAME (exit code: $TEST_EXIT_CODE)"
    fi
    
    TEST_END=$(date +%s.%N)
    TEST_TIME=$(echo "$TEST_END - $TEST_START" | bc)
    TEST_TIMES["$TEST_NAME"]=$TEST_TIME
    
    log "Test completed in ${TEST_TIME}s"
    log ""
    
    # Brief pause between tests
    sleep 2
done

# Generate summary
TOTAL_TESTS=$((${#FAILURE_INJECTION_TESTS[@]} + ${#RUNTIME_TESTS[@]}))
TOTAL_PASSED=$((FAILURE_TESTS_PASSED + RUNTIME_TESTS_PASSED))
TOTAL_FAILED=$((FAILURE_TESTS_FAILED + RUNTIME_TESTS_FAILED))

log ""
log "=========================================="
log "TEST SUMMARY"
log "=========================================="
log ""
log "Failure Injection Tests:"
log "  Passed: $FAILURE_TESTS_PASSED / ${#FAILURE_INJECTION_TESTS[@]}"
log "  Failed: $FAILURE_TESTS_FAILED / ${#FAILURE_INJECTION_TESTS[@]}"
log ""
log "Runtime Validation Tests:"
log "  Passed: $RUNTIME_TESTS_PASSED / ${#RUNTIME_TESTS[@]}"
log "  Failed: $RUNTIME_TESTS_FAILED / ${#RUNTIME_TESTS[@]}"
log ""
log "Overall:"
log "  Total Tests: $TOTAL_TESTS"
log "  Passed: $TOTAL_PASSED"
log "  Failed: $TOTAL_FAILED"
log ""

# Generate JSON summary
cat > "$SUMMARY_FILE" <<EOF
{
  "test_suite": "runtime_validation",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "summary": {
    "total_tests": $TOTAL_TESTS,
    "total_passed": $TOTAL_PASSED,
    "total_failed": $TOTAL_FAILED,
    "failure_injection": {
      "total": ${#FAILURE_INJECTION_TESTS[@]},
      "passed": $FAILURE_TESTS_PASSED,
      "failed": $FAILURE_TESTS_FAILED
    },
    "runtime_validation": {
      "total": ${#RUNTIME_TESTS[@]},
      "passed": $RUNTIME_TESTS_PASSED,
      "failed": $RUNTIME_TESTS_FAILED
    }
  },
  "test_results": {
EOF

# Add individual test results
FIRST=true
for test_name in "${!TEST_RESULTS[@]}"; do
    if [[ "$FIRST" == "true" ]]; then
        FIRST=false
    else
        echo "," >> "$SUMMARY_FILE"
    fi
    cat >> "$SUMMARY_FILE" <<EOF
    "$test_name": {
      "result": "${TEST_RESULTS[$test_name]}",
      "exit_code": ${TEST_EXIT_CODES[$test_name]:-0},
      "duration_seconds": "${TEST_TIMES[$test_name]}"
    }
EOF
done

cat >> "$SUMMARY_FILE" <<EOF
  },
  "overall_pass": $([ "$TOTAL_FAILED" -eq 0 ] && echo "true" || echo "false")
}
EOF

log "Summary saved to: $SUMMARY_FILE"

# Final status
if [[ $TOTAL_FAILED -eq 0 ]]; then
    success "ALL TESTS PASSED"
    log ""
    log "Runtime validation complete - all tests passed"
    log "Ready for GO / NO-GO decision"
    exit 0
else
    error "SOME TESTS FAILED"
    log ""
    log "Runtime validation incomplete - $TOTAL_FAILED test(s) failed"
    log "Review test results before making GO / NO-GO decision"
    exit 1
fi

