# Phase 18: Threat Intelligence Feed Engine

**Path and File Name:** `/home/ransomeye/rebuild/docs/readme/18_Threat_Intelligence_Feed_Engine_readme.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Forensic-grade documentation for Phase 18 - Threat Intelligence Feed Engine

---

## WHAT EXISTS

### Implementation Location
- **Directory:** `/home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/`
- **Service:** `systemd/ransomeye-feed-fetcher.service` (timer)
- **Type:** Python modules

### Core Components

1. **Feed Ingestion** (`threat_intel/ingestion/`)
   - Offline-capable feed ingestion
   - Multiple feed sources:
     - `auto_feed_fetcher.py` - Automatic feed fetching
     - `wiz_feed.py` - Wiz feed integration
     - `ransomware_live_feed.py` - Ransomware live feed
     - `malwarebazaar_feed.py` - MalwareBazaar feed
     - `additional_sources.py` - Additional feed sources
     - `offline_feeds.py` - Offline feed support
   - `fetch_all_feeds.py` - Fetches all configured feeds

2. **Feed Validation** (`threat_intel/ingestion/feed_validator.py`)
   - Feed validation
   - Poisoning detection
   - Schema validation
   - Fail-closed on invalid feeds

3. **IOC Normalization** (`threat_intel/normalization/`)
   - IOC normalization
   - Ontology mapping (`ontology.py`)
   - Format conversion (`mapping.py`)

4. **Feed Fusion** (`threat_intel/fusion/`)
   - Multi-source correlation (`correlation.py`)
   - Confidence scoring (`confidence.py`)
   - IOC deduplication

5. **Incremental Retraining** (`threat_intel/incremental_retrain.py`)
   - Retrains models with new feeds
   - Model versioning
   - Training provenance

6. **Training Governance** (`threat_intel/training_governance.py`)
   - Training governance
   - Model validation
   - Training provenance tracking

---

## WHAT DOES NOT EXIST

1. **No separate threat intel service** - Integrated into Intelligence (Phase 3)
2. **No dedup & clustering service** - Functionality in fusion module
3. **No trust scoring service** - Functionality in confidence module

**Canonical Mapping (from MODULE_PHASE_MAP.yaml):**
- `ransomeye_threat_intel_engine` → PHANTOM MODULE
- Functionality provided by `ransomeye_intelligence` (Phase 3)

---

## DATABASE SCHEMAS

**NONE** - Phase 18 does not create database tables.

**Feed Storage:**
- Feeds stored in filesystem
- IOC data in filesystem
- Model training data in filesystem

---

## RUNTIME SERVICES

**Service:** `ransomeye-feed-fetcher.service` (timer)
- **Location:** `/home/ransomeye/rebuild/systemd/ransomeye-feed-fetcher.service`
- **Type:** systemd timer
- **User:** `ransomeye`
- **Group:** `ransomeye`
- **ExecStart:** Python feed fetcher script

**Additional Service:** `ransomeye-feed-retraining.service` (timer)
- Periodic model retraining with new feeds
- Timer-based execution

---

## GUARDRAILS ALIGNMENT

Phase 18 enforces guardrails:

1. **Offline-Capable** - Feeds cached locally
2. **Feed Validation** - Invalid feeds rejected
3. **Poisoning Detection** - Poisoned feeds rejected
4. **Fail-Closed** - Invalid feeds → REJECTED

---

## INSTALLER BEHAVIOR

**Installation:**
- Threat intelligence modules installed by main installer
- Feed fetcher timer installed
- Feed retraining timer installed
- No separate installation step

---

## SYSTEMD INTEGRATION

**Service Files:**
- `ransomeye-feed-fetcher.service` (timer)
- `ransomeye-feed-retraining.service` (timer)
- Created by installer
- Located in unified systemd directory
- Rootless configuration
- Disabled by default

---

## AI/ML/LLM TRAINING REALITY

**Model Retraining:**
- Incremental retraining with new feeds
- `threat_intel/incremental_retrain.py` - Retraining script
- Training governance enforced
- Model versioning
- Training provenance tracked

**Training Sources:**
- Threat intelligence feeds
- IOC data
- Historical data

---

## COPILOT REALITY

**NONE** - Phase 18 does not provide copilot functionality.

**Threat intelligence:**
- Used by Phase 3 (Intelligence)
- Used by Phase 5 (Correlation)
- Used by Phase 8 (AI Advisory)

---

## UI REALITY

**NONE** - Phase 18 has no UI.

**Threat intelligence:**
- Consumed by other phases
- Available via Intelligence API
- Used in correlation and AI advisory

---

## FAIL-CLOSED BEHAVIOR

**STRICT FAIL-CLOSED:**

1. **Invalid Feed** → REJECTED
2. **Poisoned Feed** → REJECTED
3. **Invalid Schema** → REJECTED
4. **Validation Failure** → REJECTED

**No Bypass:**
- No `--skip-validation` flag
- All checks must pass

---

## FINAL VERDICT

**PRODUCTION-VIABLE**

Phase 18 is fully implemented and production-ready:

✅ **Complete Implementation**
- Feed ingestion functional
- Feed validation working
- IOC normalization working
- Feed fusion working
- Incremental retraining working
- Training governance working

✅ **Guardrails Alignment**
- Offline-capable
- Feed validation
- Poisoning detection
- Fail-closed behavior

✅ **Fail-Closed Behavior**
- All validation checks fail-closed
- No bypass mechanisms
- Feeds rejected on any failure

**Note:** Phase 18 functionality integrated into Phase 3 (Intelligence), not a separate service. Feed fetcher and retraining run as systemd timers.

**Recommendation:** Deploy as-is. Phase 18 meets all requirements and is production-ready.

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech
