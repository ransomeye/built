# Threat Intelligence Feed Collectors

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/ingestion/README.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Threat intelligence feed collectors for training data enhancement

## Overview

This directory contains collectors for threat intelligence feeds that enhance RansomEye's AI/ML/LLM training data. All feeds are cached locally for offline training use.

## Supported Feeds

### 1. MalwareBazaar

**Collector:** `malwarebazaar_feed.py`

**API:** https://mb-api.abuse.ch/api/v1/

**Credentials:**
- Auth-Key: `483ce60ba7c8a3d7358e3c8afd6e6d23a746eb2a5a42479f`
- Set via `RANSOMEYE_FEED_MALWAREBAZAAR_API_KEY` environment variable
- Enable via `RANSOMEYE_FEED_MALWAREBAZAAR_ENABLED=true` environment variable

**Usage:**
```bash
export RANSOMEYE_FEED_MALWAREBAZAAR_ENABLED="true"
export RANSOMEYE_FEED_MALWAREBAZAAR_API_KEY="483ce60ba7c8a3d7358e3c8afd6e6d23a746eb2a5a42479f"
cd /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/ingestion
python3 malwarebazaar_feed.py --limit 100
```

**Cache Location:** `/home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/cache/malwarebazaar/`

### 2. Wiz.io Cloud Threat Landscape

**Collector:** `wiz_feed.py`

**API:** https://www.wiz.io/api/feed/cloud-threat-landscape/stix.json

**Format:** STIX 2.1 JSON

**Enable via:** `RANSOMEYE_FEED_WIZ_ENABLED=true` environment variable

**Usage:**
```bash
export RANSOMEYE_FEED_WIZ_ENABLED="true"
cd /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/ingestion
python3 wiz_feed.py
```

**Cache Location:** `/home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/cache/wiz/`

### 3. Ransomware.live

**Collector:** `ransomware_live_feed.py`

**API:** https://api.ransomware.live/v1

**Credentials:**
- API Key: `6c0cca08-3419-43e6-8014-0a4f87f353a3`
- Set via `RANSOMEYE_FEED_RANSOMWARELIVE_API_KEY` environment variable
- Enable via `RANSOMEYE_FEED_RANSOMWARELIVE_ENABLED=true` environment variable

**Usage:**
```bash
export RANSOMEYE_FEED_RANSOMWARELIVE_ENABLED="true"
export RANSOMEYE_FEED_RANSOMWARELIVE_API_KEY="6c0cca08-3419-43e6-8014-0a4f87f353a3"
cd /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/ingestion
python3 ransomware_live_feed.py --limit 100
```

**Cache Location:** `/home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/cache/ransomware_live/`

### 4. Additional Enterprise-Grade Sources

**Collector:** `additional_sources.py`

**Available Feeds:**
- **URLhaus** (Abuse.ch) - Malware URL feed (no API key required)
- **ThreatFox** (Abuse.ch) - Malware IOCs feed (no API key required)
- **CISA KEV** - Known Exploited Vulnerabilities catalog (no API key required) - Enterprise-grade vulnerability intelligence
- **AlienVault OTX** - Open Threat Exchange (requires `OTX_KEY` environment variable)
- **VirusTotal** - VirusTotal Intelligence API (requires `VIRUSTOTAL_KEY` environment variable)

**Usage:**
All additional sources are automatically included when running `fetch_all_feeds.py`.

## Unified Feed Fetcher

**Script:** `fetch_all_feeds.py`

Fetches all feeds in one command:

```bash
export RANSOMEYE_FEED_MALWAREBAZAAR_ENABLED="true"
export RANSOMEYE_FEED_MALWAREBAZAAR_API_KEY="483ce60ba7c8a3d7358e3c8afd6e6d23a746eb2a5a42479f"
export RANSOMEYE_FEED_WIZ_ENABLED="true"
export RANSOMEYE_FEED_RANSOMWARELIVE_ENABLED="true"
export RANSOMEYE_FEED_RANSOMWARELIVE_API_KEY="6c0cca08-3419-43e6-8014-0a4f87f353a3"
cd /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/ingestion
python3 fetch_all_feeds.py
```

**Alternative (using PYTHONPATH):**
```bash
export RANSOMEYE_FEED_MALWAREBAZAAR_API_KEY="483ce60ba7c8a3d7358e3c8afd6e6d23a746eb2a5a42479f"
export RANSOMEYE_FEED_RANSOMWARELIVE_API_KEY="6c0cca08-3419-43e6-8014-0a4f87f353a3"
PYTHONPATH=/home/ransomeye/rebuild python3 ransomeye_intelligence/threat_intel/ingestion/fetch_all_feeds.py
```

**Cache-only mode:**
```bash
cd /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/ingestion
python3 fetch_all_feeds.py --cache-only
```

## Integration with Training

The feeds are automatically integrated into training via `enhance_training_with_feeds.py`:

```bash
# Train with enhanced data (synthetic + threat intelligence)
python3 ransomeye_intelligence/baseline_pack/train_baseline_models.py --use-feeds

# Train with threat intelligence only
python3 ransomeye_intelligence/baseline_pack/train_baseline_models.py --feed-only
```

## Offline Operation

All feeds are cached locally. Once cached, training can proceed completely offline:

1. Fetch feeds once: `python fetch_all_feeds.py`
2. Train offline: `python train_baseline_models.py --use-feeds`

## Data Privacy

- All feeds are cached locally
- No customer data is used
- Feeds are used only for training data enhancement
- All training data is synthetic or from public threat intelligence

## Phase 6 Compliance

- ✅ All feeds cached for offline training
- ✅ Feed data used to enhance synthetic training data
- ✅ No inference-only models (all models have training scripts)
- ✅ SHAP explainability maintained
- ✅ Model signing enforced

