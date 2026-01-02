# Schema Version Compatibility

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_ingestion/protocol/compatibility.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Schema version compatibility rules

---

## Overview

Schema version compatibility is **strict**. No auto-upgrade. No permissive parsing.

---

## Version Rules

### Version 1 (Current)

- **Status:** Supported
- **Compatibility:** Self-compatible only
- **Auto-upgrade:** Forbidden
- **Permissive parsing:** Forbidden

---

## Compatibility Matrix

| Producer Version | Ingestion Version | Compatible |
|-----------------|-------------------|------------|
| v1              | v1                | Yes        |
| v1              | v2 (future)       | No         |
| v2 (future)     | v1                | No         |

---

## Migration Rules

1. **No Auto-Upgrade:** Producers must use compatible schema versions
2. **Explicit Rejection:** Incompatible versions are explicitly rejected
3. **No Permissive Parsing:** Schema violations are not tolerated

---

## Last Updated

Phase 4 Implementation

