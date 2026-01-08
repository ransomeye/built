#!/bin/bash
# Path and File Name : /home/ransomeye/rebuild/monitor_training.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Monitor comprehensive AI/ML/LLM training progress

cd /home/ransomeye/rebuild

echo "=========================================="
echo "RansomEye Training Monitor"
echo "=========================================="
echo ""

# Check if training is running
echo "1. Training Process Status:"
echo "----------------------------"
if pgrep -f "train_all_ai_ml_llm.py" > /dev/null; then
    echo "✓ Training orchestrator is RUNNING"
    ps aux | grep -E "train_all_ai_ml_llm|train_baseline|train_risk" | grep -v grep | awk '{print "  PID:", $2, "CPU:", $3"%", "MEM:", $4"%", "CMD:", substr($0, index($0,$11))}'
else
    echo "✗ Training orchestrator is NOT running"
fi
echo ""

# Check training log
echo "2. Training Log (Last 20 lines):"
echo "----------------------------------"
if [ -f "logs/comprehensive_training.log" ]; then
    LOG_SIZE=$(wc -l < logs/comprehensive_training.log)
    if [ "$LOG_SIZE" -gt 0 ]; then
        tail -20 logs/comprehensive_training.log
    else
        echo "  Log file exists but is empty (training may be initializing)"
    fi
else
    echo "  Log file not found: logs/comprehensive_training.log"
fi
echo ""

# Check model file sizes
echo "3. Model File Sizes (Recent):"
echo "------------------------------"
find . -name "*.model" -type f -exec ls -lh {} \; 2>/dev/null | \
    awk '{print $5, $9}' | \
    sort -h | \
    tail -10
echo ""

# Check for training subprocesses
echo "4. Active Training Subprocesses:"
echo "--------------------------------"
ps aux | grep -E "train_baseline|train_risk|train_confidence|train_malware|train_trust|train_classifier" | \
    grep -v grep | \
    awk '{print "  " substr($0, index($0,$11))}'
echo ""

# Check system resources
echo "5. System Resources:"
echo "--------------------"
echo "  CPU Usage:"
top -bn1 | grep "Cpu(s)" | awk '{print "    " $0}'
echo "  Memory Usage:"
free -h | grep Mem | awk '{print "    Total:", $2, "Used:", $3, "Free:", $4}'
echo ""

echo "=========================================="
echo "To monitor in real-time, run:"
echo "  tail -f /home/ransomeye/rebuild/logs/comprehensive_training.log"
echo "=========================================="
