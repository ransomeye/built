# Path and File Name : /home/ransomeye/rebuild/TRAINING_EXECUTION_GUIDE.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Guide for executing the unified AI/ML/LLM training orchestrator

# RansomEye AI/ML/LLM Training Execution Guide

## Overview

The unified training orchestrator (`train_all_ai_ml_llm.py`) trains ALL AI, ML, and LLM modules across RansomEye end-to-end. This process can take **1-3 hours** depending on your system.

## Quick Start

### Option 1: Run Manually (Recommended for Long Training)

```bash
cd /home/ransomeye/rebuild
python3 train_all_ai_ml_llm.py 2>&1 | tee logs/training_execution.log
```

**Benefits:**
- You can monitor progress in real-time
- No timeout issues
- You can pause/resume if needed
- Full control over the process

### Option 2: Run in Background

```bash
cd /home/ransomeye/rebuild
nohup python3 train_all_ai_ml_llm.py > logs/training_execution.log 2>&1 &
echo $! > logs/training_pid.txt
```

**Monitor progress:**
```bash
tail -f logs/training_execution.log
```

**Check if still running:**
```bash
ps aux | grep train_all_ai_ml_llm.py
```

## What Gets Trained

The orchestrator trains the following modules in sequence:

1. **Baseline Pack Models** (~30-45 min)
   - Ransomware behavior classifier
   - Anomaly detection baseline
   - Confidence calibration model

2. **Core AI Risk Model** (~10-15 min)
   - Risk scoring model

3. **Threat Correlation Confidence Predictor** (~15-20 min)
   - Confidence scoring for threat correlations

4. **Forensic Malware DNA Model** (~15-20 min)
   - Malware DNA extraction classifier

5. **Threat Intel Trust Scoring** (~15-20 min)
   - Trust scorer for threat intelligence
   - IOC clustering model

6. **DPI Probe Asset Classifier** (~15-20 min)
   - Network asset classification model

7. **RAG Index for LLM/SOC Copilot** (~10-30 min)
   - Knowledge base indexing

8. **SHAP Explainability Generation** (~20-30 min)
   - SHAP baselines for all models

**Total Estimated Time: 2-3 hours**

## Expected Output

The script will output progress for each module:

```
================================================================================
RANSOMEYE UNIFIED AI/ML/LLM TRAINING ORCHESTRATOR
================================================================================

[2026-01-XX...] [INFO] Training Baseline Pack Models
[2026-01-XX...] [INFO] Executing: python3 .../train_baseline_models.py --use-feeds
...
[2026-01-XX...] [INFO] ✓ Baseline pack models trained successfully
...
================================================================================
TRAINING SUMMARY
================================================================================
Modules Trained: 8/8
...
```

## Output Files

After training completes, check:

1. **Training Report:**
   ```bash
   cat logs/ai_ml_llm_training_report.json
   ```

2. **Training Log:**
   ```bash
   cat logs/ai_ml_llm_training.log
   ```

3. **Execution Log:**
   ```bash
   cat logs/training_execution.log
   ```

## Verification

After training completes, verify all models:

```bash
python3 verify_ai_ml_training.py
```

This will check:
- All model files exist and are real (not dummy/placeholder)
- Model metadata is present
- SHAP files are generated
- Model hashes match manifests

## Troubleshooting

### If training fails for a specific module:

1. Check the error in `logs/training_execution.log`
2. The orchestrator will continue with other modules
3. Re-run just the failed module:
   ```bash
   # Example: Re-train just baseline pack
   cd ransomeye_intelligence/baseline_pack
   python3 train_baseline_models.py --use-feeds
   ```

### If SHAP generation fails:

This is non-critical. Models will still work, but explainability will be limited. You can regenerate SHAP later:

```bash
cd ransomeye_intelligence/baseline_pack
python3 generate_shap_baselines.py
```

### If RAG index build fails:

The orchestrator will create a basic RAG index. For a full index, run:

```bash
cd ransomeye_intelligence/llm_knowledge
python3 build_rag_index.py
```

## Next Steps After Training

1. **Verify all models:**
   ```bash
   python3 verify_ai_ml_training.py
   ```

2. **Sign model manifests** (if signing keys are available):
   ```bash
   # Sign baseline pack manifest
   cd ransomeye_intelligence/baseline_pack/models
   python3 ../../../../ransomeye_trust/sign_tool.py model_manifest.json
   ```

3. **Update model registry:**
   ```bash
   python3 register_models.py
   ```

## Command-Line Options

```bash
python3 train_all_ai_ml_llm.py [OPTIONS]

Options:
  --project-root PATH    Project root directory (default: /home/ransomeye/rebuild)
  --skip-baseline        Skip baseline pack training
  --skip-rag             Skip RAG index building
```

## Example: Skip RAG (if already built)

```bash
python3 train_all_ai_ml_llm.py --skip-rag
```

## Success Criteria

Training is successful if:
- ✅ All 8 modules report "✓ SUCCESS"
- ✅ `logs/ai_ml_llm_training_report.json` shows `success_rate >= 0.8`
- ✅ `verify_ai_ml_training.py` reports all models as "REAL"
- ✅ No critical errors in training log

## Notes

- All training uses **synthetic data + threat intelligence feeds**
- **NO customer/production data** is used
- Models are trained with **reproducible seeds** (RANDOM_SEED=42)
- All models include **SHAP explainability** support
- Training is **offline-capable** (uses cached threat intel feeds)

---

**Last Updated:** Generated by training orchestrator setup
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU

