# Compliance Validation Report

Generated: 2025-12-21T14:05:29.506882039+00:00

## Status: FAIL

## Compliance Checks

- **Critical:** Evidence integrity violation: Evidence store path does not exist: "/home/ransomeye/rebuild/var/lib/ransomeye/evidence"
- **High:** Retention enforcement violation: Evidence store path does not exist: "/home/ransomeye/rebuild/var/lib/ransomeye/evidence"
- **High:** Audit trail incomplete: Audit log not found in any expected location: ["/home/ransomeye/rebuild/var/log/ransomeye/audit.json", "/home/ransomeye/rebuild/logs/audit.json", "/home/ransomeye/rebuild/ransomeye_reporting/audit.json"]
- **Medium:** Reproducibility violation: Report directory not found in any expected location: ["/home/ransomeye/rebuild/var/lib/ransomeye/reports", "/home/ransomeye/rebuild/ransomeye_reporting/reports", "/home/ransomeye/rebuild/reports"]
