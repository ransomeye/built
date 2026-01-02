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
- Set via `MALWARBAZAAR_AUTH_KEY` environment variable

**Usage:**
```bash
export MALWARBAZAAR_AUTH_KEY="483ce60ba7c8a3d7358e3c8afd6e6d23a746eb2a5a42479f"
python3 -m ransomeye_intelligence.threat_intel.ingestion.malwarebazaar_feed --limit 100
```

**Cache Location:** `/home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/cache/malwarebazaar/`

### 2. Wiz.io Cloud Threat Landscape

**Collector:** `wiz_feed.py`

**API:** https://www.wiz.io/api/feed/cloud-threat-landscape/stix.json

**Format:** STIX 2.1 JSON

**Usage:**
```bash
python3 -m ransomeye_intelligence.threat_intel.ingestion.wiz_feed
```

**Cache Location:** `/home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/cache/wiz/`

### 3. Ransomware.live

**Collector:** `ransomware_live_feed.py`

**API:** https://api.ransomware.live/v1

**Credentials:**
- API Key: `6c0cca08-3419-43e6-8014-0a4f87f353a3`
- Set via `RANSOMWARE_LIVE_API_KEY` environment variable

**Usage:**
```bash
export RANSOMWARE_LIVE_API_KEY="6c0cca08-3419-43e6-8014-0a4f87f353a3"
python3 -m ransomeye_intelligence.threat_intel.ingestion.ransomware_live_feed --limit 100
```

**Cache Location:** `/home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/cache/ransomware_live/`

## Unified Feed Fetcher

**Script:** `fetch_all_feeds.py`

Fetches all feeds in one command:

```bash
export MALWARBAZAAR_AUTH_KEY="483ce60ba7c8a3d7358e3c8afd6e6d23a746eb2a5a42479f"
export RANSOMWARE_LIVE_API_KEY="6c0cca08-3419-43e6-8014-0a4f87f353a3"
python -m ransomeye_intelligence.threat_intel.ingestion.fetch_all_feeds
```

**Cache-only mode:**
```bash
python3 -m ransomeye_intelligence.threat_intel.ingestion.fetch_all_feeds --cache-only
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

