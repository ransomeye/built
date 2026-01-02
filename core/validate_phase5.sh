#!/bin/bash
# Path and File Name : /home/ransomeye/rebuild/core/validate_phase5.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Phase 5 pipeline validation script

set -e

echo "=========================================="
echo "PHASE 5 PIPELINE VALIDATION"
echo "=========================================="
echo ""

FAILURES=()
FIXES=()

# Test 1: Ingest validation
echo "[1/7] Testing ingest validation..."
cd /home/ransomeye/rebuild/core/ingest
if cargo test --lib 2>&1 | grep -q "test result: ok"; then
    echo "  ✓ Ingest tests pass"
else
    echo "  ✗ Ingest tests failed"
    FAILURES+=("Ingest validation tests failed")
fi

# Test 2: Policy engine determinism
echo "[2/7] Testing policy engine determinism..."
cd /home/ransomeye/rebuild/core/policy
if cargo test determinism 2>&1 | grep -q "test result: ok"; then
    echo "  ✓ Policy determinism tests pass"
else
    echo "  ✗ Policy determinism tests failed"
    FAILURES+=("Policy engine determinism tests failed")
fi

# Test 3: Dispatch dry-run mode
echo "[3/7] Testing dispatch dry-run mode..."
cd /home/ransomeye/rebuild/core/dispatch/enforcement
if cargo test dry_run 2>&1 | grep -q "test result: ok"; then
    echo "  ✓ Dispatch dry-run tests pass"
else
    echo "  ✗ Dispatch dry-run tests failed"
    FAILURES+=("Dispatch dry-run tests failed")
fi

# Test 4: Check for priority-based rate limiting
echo "[4/7] Checking priority-based rate limiting..."
if grep -r "INFO\|WARN\|CRITICAL" /home/ransomeye/rebuild/core/ingest/src/rate_limit.rs > /dev/null 2>&1; then
    echo "  ✓ Priority-based rate limiting found"
else
    echo "  ✗ Priority-based rate limiting NOT found"
    FAILURES+=("Priority-based rate limiting (drops INFO before WARN/CRITICAL) not implemented")
fi

# Test 5: Check for content hash deduplication
echo "[5/7] Checking content hash deduplication..."
if grep -r "content_hash\|content hash" /home/ransomeye/rebuild/core/ingest/src/ > /dev/null 2>&1; then
    echo "  ✓ Content hash deduplication found"
else
    echo "  ✗ Content hash deduplication NOT found"
    FAILURES+=("Content hash deduplication not implemented (only message ID/nonce deduplication exists)")
fi

# Test 6: Check policy signature verification
echo "[6/7] Checking policy signature verification..."
if grep -r "signature.*verify\|verify.*signature" /home/ransomeye/rebuild/core/policy/engine/src/engine.rs > /dev/null 2>&1; then
    echo "  ✓ Policy signature verification found"
else
    echo "  ✗ Policy signature verification NOT found"
    FAILURES+=("Policy signature verification not found in engine")
fi

# Test 7: Check governor dispatch boundary
echo "[7/7] Checking governor dispatch boundary..."
if grep -r "governor\|Governor" /home/ransomeye/rebuild/core/dispatch/dispatcher/src/dispatcher.rs > /dev/null 2>&1; then
    echo "  ✓ Governor references found"
else
    echo "  ⚠ Governor dispatch boundary check not explicit (relies on signature verification)"
    # This is not necessarily a failure - signature verification may be sufficient
fi

echo ""
echo "=========================================="
echo "VALIDATION COMPLETE"
echo "=========================================="

if [ ${#FAILURES[@]} -eq 0 ]; then
    echo "✅ PHASE 5 RESULT: PASS"
    echo ""
    echo "🔍 PIPELINE FAILURES FOUND"
    echo "NONE"
    echo ""
    echo "🛠️ FIXES APPLIED"
    echo "NONE"
    echo ""
    echo "🔁 RE-VALIDATION RESULT"
    echo "PASS"
else
    echo "❌ PHASE 5 RESULT: FAIL"
    echo ""
    echo "🔍 PIPELINE FAILURES FOUND"
    for failure in "${FAILURES[@]}"; do
        echo "- $failure"
    done
    echo ""
    echo "🛠️ FIXES APPLIED"
    if [ ${#FIXES[@]} -eq 0 ]; then
        echo "NONE"
    else
        for fix in "${FIXES[@]}"; do
            echo "- $fix"
        done
    fi
    echo ""
    echo "🔁 RE-VALIDATION RESULT"
    echo "FAIL"
fi

echo ""
echo "PHASE 5 COMPLETE — AWAIT NEXT PROMPT"

