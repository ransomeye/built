# Enhanced Threat Detection & Auto-Evolution System

## Overview

RansomEye's Enhanced Threat Detection system provides comprehensive coverage of all cyber threat types with automatic continuous training and long-term evolution capabilities.

## Key Features

### 1. Comprehensive Threat Classification

The system classifies and detects **all cyber threat types** including:

- **Malware** (all variants)
- **Ransomware**
- **DDoS Attacks**
- **Trojans**
- **Spyware**
- **Worms**
- **Man-in-the-Middle (MitM)**
- **SQL Injection**
- **DNS Tunneling**
- **AI-Driven Attacks**
- **Supply Chain Attacks**
- **Zero-Day Exploits**
- **Cryptojacking**
- **Phishing**
- **APT (Advanced Persistent Threats)**
- **Botnets**
- **Keyloggers**
- **Rootkits**
- **Backdoors**
- **Fileless Malware**
- **Polymorphic/Metamorphic Malware**

### 2. Threat Intelligence Integration

#### Ransomware.live API Integration

The system now includes the Ransomware.live API key by default:
- **API Key**: `6c0cca08-3419-43e6-8014-0a4f87f353a3`
- **Auto-enabled**: Feed is automatically enabled when API key is available
- **Environment Variable**: `RANSOMEYE_FEED_RANSOMWARELIVE_API_KEY`

#### Additional Threat Intel Sources

- **MalwareBazaar**: Malware samples and metadata
- **Wiz.io**: Cloud threat landscape feeds
- **URLhaus**: Malware URL feed (Abuse.ch)
- **ThreatFox**: Malware IOCs (Abuse.ch)
- **CISA KEV**: Known Exploited Vulnerabilities
- **AlienVault OTX**: Threat intelligence pulses
- **VirusTotal**: Threat intelligence (requires API key)

### 3. Continuous Training System

The continuous training system automatically retrains models from:

- **Internal Telemetry**: Incidents, alerts, forensic data from the last 30 days
- **External Feeds**: All cached threat intelligence feeds
- **Novel Threat Detection**: Automatically detects and learns from unknown threats
- **Feedback Learning**: Learns from false positives and false negatives

#### Training Schedule

- **Weekly Continuous Training**: Runs every Monday at 2:00 AM
- **Monthly Auto-Evolution**: Runs on the 1st of every month at 3:00 AM

### 4. Auto-Evolution System

Ensures RansomEye remains effective against unknown threats over **10+ years** through:

- **Novel Threat Detection**: Identifies new threat patterns not seen before
- **Adaptive Learning**: Automatically adapts models to novel threats
- **Long-Term Knowledge Retention**: Retains important patterns for 10 years
- **Model Versioning**: Tracks evolution versions and allows rollback
- **Performance Drift Detection**: Monitors model performance and triggers retraining

### 5. Unified Threat Enrichment

All IOCs are enriched with:

- **Threat Classification**: Primary and secondary threat categories
- **Confidence Scores**: Per-category confidence levels
- **Behavioral Analysis**: Behavioral signal extraction
- **Cross-Source Correlation**: Links IOCs across multiple sources
- **Normalized Threat Types**: Standard taxonomy across all sources

## Architecture

```
ransomeye_intelligence/threat_intel/
├── classification/
│   ├── threat_classifier.py      # Comprehensive threat classifier
│   └── __init__.py
├── training/
│   ├── continuous_trainer.py     # Continuous training system
│   └── __init__.py
├── enrichment/
│   ├── unified_enricher.py       # Unified IOC enrichment
│   └── __init__.py
├── auto_evolution.py              # Auto-evolution system
└── ingestion/
    ├── ransomware_live_feed.py    # Ransomware.live feed (enhanced)
    └── ...
```

## Systemd Services

### Continuous Training Service

**Service**: `ransomeye-continuous-training.service`
**Timer**: `ransomeye-continuous-training.timer`
**Schedule**: Weekly (Monday 2:00 AM)

### Auto-Evolution Service

**Service**: `ransomeye-auto-evolution.service`
**Timer**: `ransomeye-auto-evolution.timer`
**Schedule**: Monthly (1st of month, 3:00 AM)

## Usage

### Manual Continuous Training

```bash
python3 /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/training/continuous_trainer.py --force
```

### Manual Auto-Evolution Cycle

```bash
python3 /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/auto_evolution.py --cycle
```

### Enrich IOCs

```bash
# Single IOC
python3 /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/enrichment/unified_enricher.py --ioc "malware.exe" --source "internal"

# Batch IOCs from file
python3 /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/enrichment/unified_enricher.py --file iocs.json --source "feed"
```

### Classify Threats

```python
from ransomeye_intelligence.threat_intel.classification import ThreatClassifier

classifier = ThreatClassifier()
result = classifier.classify(
    ioc_value="malware.exe.encrypted",
    ioc_type="filename",
    metadata={"description": "Encrypted file with ransom note"},
    behavior_signals=["file_encryption", "ransom_note"]
)

print(f"Primary Category: {result['primary_category']}")
print(f"Confidence: {result['confidence_scores']}")
```

## Configuration

### Environment Variables

```bash
# Ransomware.live API (pre-configured)
export RANSOMWARE_LIVE_API_KEY="6c0cca08-3419-43e6-8014-0a4f87f353a3"
export RANSOMEYE_FEED_RANSOMWARELIVE_ENABLED="true"
export RANSOMEYE_FEED_RANSOMWARELIVE_API_KEY="6c0cca08-3419-43e6-8014-0a4f87f353a3"

# Database (for internal telemetry)
export DB_HOST="localhost"
export DB_PORT="5432"
export DB_NAME="ransomeye"
export DB_USER="gagan"
export DB_PASS="gagan"

# Additional feeds (optional)
export OTX_KEY="your_otx_key"
export VIRUSTOTAL_KEY="your_vt_key"
```

## Training Data Sources

### Internal Data
- Incidents from `ransomeye.incidents` table
- Alerts from `ransomeye.alerts` table
- Forensic evidence from `ransomeye.forensic_evidence` table

### External Feeds
- Cached threat intelligence feeds from `/home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/cache/`

## Model Outputs

### Continuous Training Outputs
- **Model**: `threat_classifier_continuous.model`
- **Metrics**: `threat_classifier_continuous_metrics.json`
- **SHAP Explanations**: `threat_classifier_continuous_shap.json`

### Evolution Reports
- **Evolution Reports**: `/home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/evolution/evolution_report_*.json`
- **Evolution History**: `/home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/evolution/evolution_history.json`

## Long-Term Evolution

The auto-evolution system ensures:

1. **Novel Threat Detection**: Automatically identifies new threat patterns
2. **Adaptive Learning**: Retrains models when novel threats are detected
3. **Knowledge Retention**: Maintains 10-year knowledge base
4. **Performance Monitoring**: Tracks model drift and triggers retraining
5. **Version Control**: Tracks evolution versions for rollback capability

## Integration with Main Training Pipeline

The enhanced threat detection system integrates seamlessly with the main training pipeline:

```bash
# Main training (includes threat intel)
python3 /home/ransomeye/rebuild/train_all_ai_ml_llm.py

# Continuous training (runs automatically via systemd)
systemctl start ransomeye-continuous-training.service

# Auto-evolution (runs automatically via systemd)
systemctl start ransomeye-auto-evolution.service
```

## Monitoring

### Check Training Status

```bash
# Check continuous training logs
journalctl -u ransomeye-continuous-training.service -f

# Check auto-evolution logs
journalctl -u ransomeye-auto-evolution.service -f

# Check timer status
systemctl list-timers ransomeye-*
```

### View Evolution History

```bash
cat /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/evolution/evolution_history.json
```

## Support

For issues or questions:
- **Email**: Gagan@RansomEye.Tech
- **Documentation**: See main RansomEye documentation

© RansomEye.Tech | Support: Gagan@RansomEye.Tech

