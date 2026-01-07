#!/bin/bash
# Path and File Name : /home/ransomeye/rebuild/train_missing_models.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Script to train missing AI/ML/LLM models based on validation results

echo "=================================================================================="
echo "RansomEye - Training Missing Models"
echo "=================================================================================="
echo ""
echo "This script will train all missing AI/ML/LLM models."
echo ""

# Run validation to see what's missing
echo "Checking current model status..."
python3 /home/ransomeye/rebuild/validate_all_modules.py --json > /tmp/validation_before.json 2>&1

echo ""
echo "Starting MANDATORY training for all modules..."
echo "HIGHEST PRIORITY - NO EXCEPTIONS"
echo ""

# Use mandatory training script that ensures 100% completion
python3 /home/ransomeye/rebuild/train_all_models_complete.py

# If mandatory training fails, also try the standard training
if [ $? -ne 0 ]; then
    echo ""
    echo "⚠ Mandatory training had issues, running standard training as fallback..."
    python3 /home/ransomeye/rebuild/train_all_ai_ml_llm.py
fi

echo ""
echo "=================================================================================="
echo "Training Complete - Re-validating..."
echo "=================================================================================="
echo ""

# Re-validate to show improvements
python3 /home/ransomeye/rebuild/validate_all_modules.py

echo ""
echo "=================================================================================="
echo "Done!"
echo "=================================================================================="

