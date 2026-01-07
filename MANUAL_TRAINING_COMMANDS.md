# Path and File Name : /home/ransomeye/rebuild/MANUAL_TRAINING_COMMANDS.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Commands to run manually for long-running training tasks

# Manual Training Commands

## Overview

These commands are designed to be run manually in your terminal since they take significant time and Cursor has timeout limits. Run each command and provide the output when complete.

---

## 1. Complete Full Training (If Needed)

If you want to re-run the full training pipeline:

```bash
cd /home/ransomeye/rebuild && python3 train_all_ai_ml_llm.py 2>&1 | tee logs/training_execution_full.log
```

**Expected time:** 2-3 hours

---

## 2. Regenerate SHAP Baselines (Already Fixed - Should Work Now)

The SHAP generation has been fixed to work without the `shap` library. Regenerate to ensure all models have SHAP baselines:

```bash
cd /home/ransomeye/rebuild && python3 ransomeye_intelligence/baseline_pack/generate_shap_baselines.py 2>&1 | tee logs/shap_generation.log
```

**Expected time:** 5-10 minutes

---

## 3. Create RAG Index (Fallback - No Dependencies Required)

Create a basic RAG index that works without sentence-transformers/faiss:

```bash
cd /home/ransomeye/rebuild && python3 -c "
from train_all_ai_ml_llm import TrainingOrchestrator
from pathlib import Path
import sys

orchestrator = TrainingOrchestrator(Path('/home/ransomeye/rebuild'))
result = orchestrator._create_basic_rag_index()
if result:
    print('✓ RAG index created successfully')
    sys.exit(0)
else:
    print('✗ RAG index creation failed')
    sys.exit(1)
" 2>&1 | tee logs/rag_index_creation.log
```

**Expected time:** < 1 minute

---

## 4. Verify All Trained Models

Verify that all models are properly trained and not placeholders:

```bash
cd /home/ransomeye/rebuild && python3 verify_ai_ml_training.py 2>&1 | tee logs/model_verification.log
```

**Expected time:** 2-5 minutes

---

## 5. Complete Training Summary (Run All Remaining Tasks)

Run all remaining tasks in sequence:

```bash
cd /home/ransomeye/rebuild && \
echo "=== Regenerating SHAP Baselines ===" && \
python3 ransomeye_intelligence/baseline_pack/generate_shap_baselines.py 2>&1 | tee -a logs/complete_training.log && \
echo "" && \
echo "=== Creating RAG Index ===" && \
python3 -c "
from train_all_ai_ml_llm import TrainingOrchestrator
from pathlib import Path
orchestrator = TrainingOrchestrator(Path('/home/ransomeye/rebuild'))
result = orchestrator._create_basic_rag_index()
print('✓ RAG index created' if result else '✗ RAG index failed')
" 2>&1 | tee -a logs/complete_training.log && \
echo "" && \
echo "=== Verifying All Models ===" && \
python3 verify_ai_ml_training.py 2>&1 | tee -a logs/complete_training.log
```

**Expected time:** 10-15 minutes

---

## 6. Check Training Status

Quick check of what's been trained:

```bash
cd /home/ransomeye/rebuild && \
echo "=== Training Report ===" && \
cat logs/ai_ml_llm_training_report.json | python3 -m json.tool && \
echo "" && \
echo "=== Model Files ===" && \
find . -name "*.model" -o -name "*.pkl" | grep -E "(models|baseline_pack|threat_correlation|forensic|threat_intel|dpi_probe)" | head -20
```

**Expected time:** < 1 minute

---

## Expected Output Summary

After running all commands, you should have:

✅ **7/8 modules trained successfully:**
- Baseline Pack Models (ransomware_behavior, anomaly_baseline, confidence_calibration)
- Core AI Risk Model
- Threat Correlation Confidence Predictor
- Forensic Malware DNA Model
- Threat Intel Trust Scoring & Clustering
- DPI Probe Asset Classifier
- SHAP Baselines (all models)

⚠️ **RAG Index:** Basic index created (full index requires optional dependencies)

---

## Next Steps After Training

1. **Verify all models:**
   ```bash
   python3 verify_ai_ml_training.py
   ```

2. **Check model registry:**
   ```bash
   python3 register_models.py
   ```

3. **Sign model manifests** (if signing keys available):
   ```bash
   # Sign baseline pack
   cd ransomeye_intelligence/baseline_pack/models
   python3 ../../../../ransomeye_trust/sign_tool.py model_manifest.json
   ```

---

## Troubleshooting

If any command fails:
1. Check the log file (logs/*.log)
2. Share the error output
3. I'll help fix the issue

---

**Last Updated:** Generated after training orchestrator fixes
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7g0

