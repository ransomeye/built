# RansomEye Validation Commands

## Quick Validation Command

To check if everything is built and verify all trained AI/ML/LLM models:

```bash
python3 /home/ransomeye/rebuild/validate_all_modules.py
```

## Detailed Validation with JSON Output

To get detailed JSON output for programmatic processing:

```bash
python3 /home/ransomeye/rebuild/validate_all_modules.py --json
```

## Save Validation Report

To save the validation report to a specific file:

```bash
python3 /home/ransomeye/rebuild/validate_all_modules.py --save /path/to/report.json
```

## What the Validation Checks

### 1. AI/ML/LLM Modules
- **Baseline Pack**: ransomware_behavior, anomaly_detector, confidence_calibrator
- **Risk Model**: risk_predictor
- **Threat Correlation**: confidence_predictor
- **Forensic Malware DNA**: malware_dna
- **Threat Intel Trust**: trust_scorer, ioc_clusterer
- **DPI Probe Classifier**: asset_classifier
- **Threat Classifier Continuous**: threat_classifier_continuous
- **RAG Index**: index.faiss, index.pkl

For each model, it checks:
- ✓ Model file exists
- ✓ Model size and hash
- ✓ Last modified date
- ✓ SHAP explainability files
- ✓ Metadata files

### 2. Systemd Services
- ransomeye-master-core.service
- ransomeye-ai-core.service
- ransomeye-alert-engine.service
- ransomeye-threat-intel.service
- ransomeye-continuous-training.timer
- ransomeye-auto-evolution.timer

### 3. Database Connectivity
- PostgreSQL connection
- Database version

### 4. Threat Intelligence Feeds
- Cached feed files
- Feed sources (MalwareBazaar, Ransomware.live, Wiz, etc.)

## Output Format

The validation script provides:
- **Status Icons**:
  - ✓ = Complete/Active/Exists
  - ⚠ = Partial
  - ✗ = Missing/Inactive
  - ○ = Exists but inactive

- **Summary Statistics**:
  - Module completion percentage
  - Model count and status
  - Systemd service status
  - Database connectivity
  - Feed cache status

## Example Output

```
================================================================================
RansomEye Comprehensive Module Validation
================================================================================

Validating AI/ML/LLM Modules...
--------------------------------------------------------------------------------
Checking baseline_pack...
  ⚠ baseline_pack: partial
    ✓ ransomware_behavior.model (2.96 MB)
    ✗ anomaly_detector.model (missing)
Checking threat_correlation...
  ✓ threat_correlation: complete
    ✓ confidence_predictor.model (0.46 MB)

================================================================================
VALIDATION SUMMARY
================================================================================
Modules: 4/8 complete (50.0%)
Models: 6/12 exist (50.0%)
Systemd Services: 2/6 active, 2 enabled
Database: ✓ Connected
Threat Intel Feeds: 6 feeds cached
================================================================================
```

## Exit Codes

- **0** = All modules complete (100% validation)
- **1** = Some modules missing or incomplete

## Validation Report Location

The validation report is automatically saved to:
```
/home/ransomeye/rebuild/logs/validation_report.json
```

This JSON file contains detailed information about:
- All module statuses
- Model metadata (size, hash, dates)
- SHAP file locations
- Systemd service statuses
- Database connection details
- Threat intel feed cache information

## Additional Quick Checks

### Check Specific Module Models
```bash
# List all models in a module
ls -lh /home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack/models/

# Check model metadata
cat /home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack/models/model_manifest.json
```

### Check Systemd Services
```bash
# List all RansomEye services
systemctl list-units --type=service --state=running | grep ransomeye

# Check timer status
systemctl list-timers ransomeye-*
```

### Check Threat Intel Feeds
```bash
# List cached feeds
ls -lh /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/cache/*/
```

### Check Database
```bash
# Test database connection
psql -h localhost -U gagan -d ransomeye -c "SELECT version();"
```

## Training Missing Models

If the validation shows missing models, you can train them using:

```bash
# Train all missing models
python3 /home/ransomeye/rebuild/train_all_ai_ml_llm.py

# Or use the convenience script
/home/ransomeye/rebuild/train_missing_models.sh
```

The training script will:
- Skip already trained models (if configured)
- Train all missing models
- Generate SHAP explainability files
- Create metadata files

## Troubleshooting

### Missing Models
If models are missing, run the training script:
```bash
python3 /home/ransomeye/rebuild/train_all_ai_ml_llm.py
```

### Permission Errors When Saving Reports
If you get permission errors when using `--save` with a custom path, the script will automatically fall back to the default location (`/home/ransomeye/rebuild/logs/validation_report.json`).

### Missing Systemd Services
If services are not found, check installation:
```bash
sudo systemctl daemon-reload
sudo systemctl list-unit-files | grep ransomeye
```

### Database Connection Issues
Check environment variables:
```bash
echo $DB_HOST $DB_PORT $DB_NAME $DB_USER
```

© RansomEye.Tech | Support: Gagan@RansomEye.Tech

