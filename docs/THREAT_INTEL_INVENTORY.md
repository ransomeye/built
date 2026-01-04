# Threat Intelligence Module Inventory
## PROMPT-45 — Threat Intelligence Full Training & Enablement

**Date:** 2026-01-04  
**Status:** Inventory Complete

---

## THREAT INTELLIGENCE MODULES IDENTIFIED

### 1. Feed Ingestion Module
**Location:** `ransomeye_intelligence/threat_intel/ingestion/`  
**Type:** Python modules  
**Components:**
- `fetch_all_feeds.py` - Unified feed fetcher (main initialization script)
- `malwarebazaar_feed.py` - MalwareBazaar feed collector
- `wiz_feed.py` - Wiz.io STIX feed collector
- `ransomware_live_feed.py` - Ransomware.live feed collector
- `additional_sources.py` - Additional feed sources (OTX, VirusTotal, URLhaus, ThreatFox)
- `offline_feeds.py` - Offline feed support
- `feed_validator.py` - Feed validation and poisoning detection

**Data Source:** External threat intelligence feeds (cached locally)  
**Output:** Cached JSON files in `threat_intel/cache/`

### 2. IOC Normalization Module
**Location:** `ransomeye_intelligence/threat_intel/normalization/`  
**Type:** Python modules  
**Components:**
- `ontology.py` - IOC ontology and type mapping
- `mapping.py` - Format conversion and mapping

**Data Source:** Cached feed files  
**Output:** Normalized IOC dictionaries

### 3. Feed Fusion Module
**Location:** `ransomeye_intelligence/threat_intel/fusion/`  
**Type:** Python modules  
**Components:**
- `correlation.py` - Multi-source IOC correlation
- `confidence.py` - Confidence scoring

**Data Source:** Normalized IOCs  
**Output:** Correlated IOCs with confidence scores

### 4. Incremental Retraining Module
**Location:** `ransomeye_intelligence/threat_intel/incremental_retrain.py`  
**Type:** Python script  
**Function:** Retrains ML models with new feed data  
**Data Source:** Cached feed data  
**Output:** Trained model files in `threat_intel/models/`

### 5. Training Governance Module
**Location:** `ransomeye_intelligence/threat_intel/training_governance.py`  
**Type:** Python module  
**Function:** Model validation, signing, and provenance tracking

### 6. Rust Threat Feed Module (Future)
**Location:** `core/threat_feed/`  
**Type:** Rust crate (feature-gated: "future-threat-feed")  
**Status:** Not active in current build

### 7. Rust Intel Module (Future)
**Location:** `core/intel/`  
**Type:** Rust crate  
**Status:** Correlation and confidence scoring (future)

---

## INITIALIZATION REQUIREMENTS

### Required Steps:
1. ✅ Fetch feeds (or load from cache) - `fetch_all_feeds.py`
2. ❌ **MISSING:** Database table for threat intelligence
3. ❌ **MISSING:** Script to load cached feeds into database
4. ❌ **MISSING:** Default initialization on system startup
5. ✅ Incremental retraining script exists - `incremental_retrain.py`

---

## CURRENT STATE

**Cached Feeds Available:**
- MalwareBazaar: Cached in `cache/malwarebazaar/`
- Wiz.io: Cached in `cache/wiz/`
- Ransomware.live: Cached in `cache/ransomware_live/`
- ThreatFox: Cached in `cache/threatfox/`
- URLhaus: Cached in `cache/urlhaus/`

**Database Tables:**
- ❌ No `threat_intel` table exists
- ❌ No `ioc` table exists
- ❌ No `threat_feed` table exists

**Integration:**
- Threat intel is advisory-only
- No enforcement authority
- Must integrate with detection pipeline (to be verified)

---

## ACTION ITEMS

1. Create database schema for threat intelligence
2. Create initialization script to load cached feeds into database
3. Execute initialization
4. Verify database population
5. Test fail-closed behavior
6. Verify integration with detection pipeline

