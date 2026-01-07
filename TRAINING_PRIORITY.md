# RansomEye Training Priority - MANDATORY

## Training is HIGHEST PRIORITY - NO EXCEPTIONS

All AI/ML/LLM models MUST be trained. There are NO exceptions.

## Mandatory Training Command

```bash
# Use the mandatory training script (ensures 100% completion)
python3 /home/ransomeye/rebuild/train_all_models_complete.py
```

Or use the convenience script:
```bash
/home/ransomeye/rebuild/train_missing_models.sh
```

## What Gets Trained

### Required Models (ALL must exist):

1. **Baseline Pack** (3 models):
   - `ransomware_behavior.model` ✓
   - `anomaly_detector.model` ✗
   - `confidence_calibrator.model` ✗

2. **Risk Model** (1 model):
   - `risk_predictor.model` ✗

3. **Threat Correlation** (1 model):
   - `confidence_predictor.model` ✓

4. **Forensic Malware DNA** (1 model):
   - `malware_dna.model` ✓

5. **Threat Intel Trust** (2 models):
   - `trust_scorer.model` ✓
   - `ioc_clusterer.model` ✓

6. **DPI Probe Classifier** (1 model):
   - `asset_classifier.model` ✗

7. **Threat Classifier Continuous** (1 model):
   - `threat_classifier_continuous.model` ✓

8. **RAG Index** (2 files):
   - `index.faiss` ✗
   - `index.pkl` ✗

**Total: 12 models/files required**

## Training Features

The mandatory training script:

1. **Checks current status** - Identifies all missing models
2. **Runs main training** - Executes `train_all_ai_ml_llm.py`
3. **Targeted training** - Trains any remaining missing models
4. **Retry logic** - Retries failed training up to 3 times
5. **Final validation** - Ensures 100% completion
6. **Auto-creates scripts** - Creates missing training scripts if needed
7. **Exit codes** - Exits with error if any model is missing

## Validation After Training

After training completes, always validate:

```bash
python3 /home/ransomeye/rebuild/validate_all_modules.py
```

Expected result:
- **Modules: 8/8 complete (100.0%)**
- **Models: 12/12 exist (100.0%)**

## Troubleshooting

### If Training Fails

1. Check logs for specific errors
2. Verify disk space: `df -h`
3. Check memory: `free -h`
4. Verify Python dependencies: `pip list | grep -E "(sklearn|numpy|faiss)"`
5. Retry training: `python3 /home/ransomeye/rebuild/train_all_models_complete.py`

### Missing Training Scripts

The mandatory trainer will automatically create missing training scripts for:
- DPI Probe Classifier
- RAG Index

### Training Timeouts

If training times out:
- Increase timeout values in the script
- Train modules individually
- Check system resources

## Priority Enforcement

Training is **MANDATORY** and **HIGHEST PRIORITY**:

- ✅ No skipping allowed (unless explicitly disabled)
- ✅ All models must exist after training
- ✅ Validation must pass 100%
- ✅ Exit with error if incomplete
- ✅ Retry failed training automatically

## Current Status

Run validation to check current status:
```bash
python3 /home/ransomeye/rebuild/validate_all_modules.py
```

## Next Steps

1. Run mandatory training: `python3 /home/ransomeye/rebuild/train_all_models_complete.py`
2. Validate completion: `python3 /home/ransomeye/rebuild/validate_all_modules.py`
3. Verify 100% completion before proceeding

© RansomEye.Tech | Support: Gagan@RansomEye.Tech

