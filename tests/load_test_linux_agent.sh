#!/bin/bash
# Path and File Name : /home/ransomeye/rebuild/tests/load_test_linux_agent.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Load test script for Linux Agent (10k events/min)

set -euo pipefail

# Configuration
EVENTS_PER_MIN=10000
EVENTS_PER_SEC=$((EVENTS_PER_MIN / 60))
DURATION_SEC=600  # 10 minutes (as per requirement)
METRICS_INTERVAL=1  # seconds

# Find agent PID
AGENT_PID=$(pgrep -f ransomeye_linux_agent || echo "")
if [ -z "$AGENT_PID" ]; then
    echo "ERROR: Linux Agent not running"
    exit 1
fi

# Metrics collection directory
METRICS_DIR="/tmp/ransomeye_load_test_$(date +%s)"
mkdir -p "$METRICS_DIR"

echo "Starting load test: $EVENTS_PER_MIN events/min for $DURATION_SEC seconds"
echo "Agent PID: $AGENT_PID"
echo "Metrics directory: $METRICS_DIR"

# Start metrics collection
(
    echo "timestamp,pid,cpu_percent,mem_percent,rss_kb,vsz_kb,threads" > "$METRICS_DIR/cpu_mem.csv"
    while true; do
        if [ -n "$AGENT_PID" ] && kill -0 "$AGENT_PID" 2>/dev/null; then
            TIMESTAMP=$(date +%s)
            STATS=$(ps -p "$AGENT_PID" -o pid,pcpu,pmem,rss,vsz,nlwp --no-headers | awk '{print $1","$2","$3","$4","$5","$6}')
            echo "$TIMESTAMP,$STATS" >> "$METRICS_DIR/cpu_mem.csv"
            
            # Service status
            systemctl status ransomeye-linux-agent.service --no-pager >> "$METRICS_DIR/service_status.log" 2>&1 || true
        else
            echo "Agent process not found, stopping metrics collection"
            break
        fi
        sleep "$METRICS_INTERVAL"
    done
) &
METRICS_PID=$!

# Event injection - generate syscall activity to trigger agent monitoring
# We'll create processes, file operations, and network connections at target rate
START_TIME=$(date +%s)
END_TIME=$((START_TIME + DURATION_SEC))
EVENT_COUNTER=0
TEST_DIR="/tmp/ransomeye_load_test_events_$$"

# Create test directory for file operations
mkdir -p "$TEST_DIR"

echo "Load test started at $(date)"
echo "Target: $EVENTS_PER_SEC events/second"
echo "Test directory: $TEST_DIR"

# Event injection loop
while [ $(date +%s) -lt $END_TIME ]; do
    ELAPSED=$(( $(date +%s) - START_TIME ))
    REMAINING=$(( END_TIME - $(date +%s) ))
    
    # Generate events at target rate (batch per second)
    for i in $(seq 1 $EVENTS_PER_SEC); do
        # Process exec events (triggers syscall monitoring)
        ( /bin/echo "load_test_$EVENT_COUNTER" > /dev/null 2>&1 ) &
        
        # File operations (triggers filesystem monitoring)
        echo "load_test_data_$EVENT_COUNTER" > "$TEST_DIR/file_$EVENT_COUNTER.txt" 2>/dev/null || true
        
        # Network connections (triggers network monitoring)
        ( timeout 0.1 bash -c "echo > /dev/tcp/127.0.0.1/80" 2>/dev/null || true ) &
        
        EVENT_COUNTER=$((EVENT_COUNTER + 1))
    done
    
    # Cleanup old files to prevent disk fill
    if [ $((EVENT_COUNTER % 1000)) -eq 0 ]; then
        find "$TEST_DIR" -type f -mmin +1 -delete 2>/dev/null || true
    fi
    
    # Progress reporting
    if [ $((ELAPSED % 10)) -eq 0 ]; then
        echo "Progress: ${ELAPSED}s / ${DURATION_SEC}s (${REMAINING}s remaining) - Events: $EVENT_COUNTER"
    fi
    
    # Sleep to maintain rate (1 second per batch)
    sleep 1
done

# Cleanup test directory
rm -rf "$TEST_DIR" 2>/dev/null || true

echo "Event injection complete. Total events generated: $EVENT_COUNTER"

# Stop metrics collection
kill $METRICS_PID 2>/dev/null || true
wait $METRICS_PID 2>/dev/null || true

echo "Load test complete. Analyzing results..."

# Check for backpressure and dropped events in logs
echo "Analyzing agent logs for backpressure and dropped events..."
journalctl -u ransomeye-linux-agent.service --since "@$START_TIME" --until "@$END_TIME" \
    | grep -i "backpressure\|drop\|queue" > "$METRICS_DIR/backpressure.log" || true

# Generate summary report
cat > "$METRICS_DIR/summary.txt" <<EOF
Load Test Summary
================
Test Duration: $DURATION_SEC seconds ($(($DURATION_SEC / 60)) minutes)
Target Rate: $EVENTS_PER_MIN events/min ($EVENTS_PER_SEC events/sec)
Events Generated: $EVENT_COUNTER
Agent PID: $AGENT_PID

CPU Metrics:
$(awk -F',' 'NR>1 {cpu+=$3; count++} END {if(count>0) print "Average CPU: " cpu/count "%"; else print "No data"}' "$METRICS_DIR/cpu_mem.csv")

Memory Metrics:
$(awk -F',' 'NR>1 {mem+=$4; rss+=$5; count++} END {if(count>0) print "Average Memory: " mem/count "%\nAverage RSS: " rss/count " KB"; else print "No data"}' "$METRICS_DIR/cpu_mem.csv")

Peak Metrics:
$(awk -F',' 'NR>1 {if($3>max_cpu) max_cpu=$3; if($4>max_mem) max_mem=$4; if($5>max_rss) max_rss=$5} END {print "Peak CPU: " max_cpu "%\nPeak Memory: " max_mem "%\nPeak RSS: " max_rss " KB"}' "$METRICS_DIR/cpu_mem.csv")

Backpressure Events:
$(wc -l < "$METRICS_DIR/backpressure.log" | awk '{print $1 " backpressure/drop log entries"}')
EOF

cat "$METRICS_DIR/summary.txt"
echo ""
echo "Detailed metrics available in: $METRICS_DIR"

