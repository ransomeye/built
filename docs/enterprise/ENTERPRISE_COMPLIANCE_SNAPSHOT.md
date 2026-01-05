# Enterprise Compliance Snapshot

**Generated:** 2026-01-05T11:00:55.144496+00:00
**Version:** v1.0.0-enterprise-ship

## Asset Inventory

- **Agents:** 442
- **Components:** 1
- **Models:** 4

## Data Flow

```
Ingestion → Normalization → Detection → Threat Intel → Audit
```

## Security Controls

- **Authentication:** mTLS certificates
- **Authorization:** Component-based access control
- **Encryption:** AES-256 for PII fields
- **Integrity:** SHA-256 hashing, immutable audit chain
- **Monitoring:** Continuous verifier, audit log

## Audit Retention

- **Retention Period:** 7 years
- **Total Entries:** 7294
- **Oldest Entry:** 2026-01-03T11:46:26.693897+00:00
- **Newest Entry:** 2026-01-05T11:00:54.726876+00:00

## AI Explainability

- **SHAP Explanations:** 0
- **Compliance:** ✗

## Evidence

```json
{
  "asset_inventory": {
    "agents": 442,
    "components": 1,
    "models": 4,
    "timestamp": "2026-01-05T11:00:55.142058+00:00"
  },
  "data_flow": {
    "ingestion": "raw_events",
    "normalization": "normalized_events",
    "detection": "detection_results",
    "threat_intel": "threat_intel_matches",
    "audit": "immutable_audit_log"
  },
  "security_controls": {
    "authentication": "mTLS certificates",
    "authorization": "Component-based access control",
    "encryption": "AES-256 for PII fields",
    "integrity": "SHA-256 hashing, audit chain",
    "monitoring": "Continuous verifier, immutable audit log"
  },
  "audit_retention": {
    "oldest_entry": "2026-01-03T11:46:26.693897+00:00",
    "newest_entry": "2026-01-05T11:00:54.726876+00:00",
    "total_entries": 7294,
    "retention_years": 7
  },
  "ai_explainability": {
    "shap_explanations_count": 0,
    "sample_shap": {
      "shap_id": null,
      "inference_id": null,
      "created_at": null
    },
    "compliance": false,
    "warning": "No SHAP explanations found (may indicate no inferences run yet)"
  }
}
```

## Failures

None

## Conclusion

PASS
