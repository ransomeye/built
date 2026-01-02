# Global Forensic Consistency Validator

**Path:** `/home/ransomeye/rebuild/core/global_validator/`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Machine-enforced consistency validator ensuring phase status, DB ownership, systemd/installer consistency, fail-open/fail-closed correctness, and AI/ML claims match reality

## Overview

The Global Forensic Consistency Validator is a read-only validation tool that ensures consistency across:

- **Phase status** ↔ **guardrails.yaml** ↔ **installer** ↔ **systemd** ↔ **READMEs**
- **DB ownership** (no conflicts)
- **Fail-open vs fail-closed** correctness
- **AI/ML claims** vs actual files

This validator **fails-closed** - any violation results in a non-zero exit code, making it suitable for CI/CD pipelines.

## Components

### 1. Phase Consistency Checker (`phase_consistency.py`)

Validates:
- Phase marked `IMPLEMENTED` → module exists on disk
- Phase marked `NOT_IMPLEMENTED` → no service, no installer reference
- README verdict matches guardrails status
- `installable`/`runnable` flags match actual capabilities

**Mismatch → FAIL (fail-closed)**

### 2. DB Ownership Validator (`db_ownership.py`)

Detects:
- Multiple writers to same table without arbitration
- Phases writing to tables they don't own
- Tables without owning phase

**Mismatch → FAIL (fail-closed)**

### 3. systemd/Installer Validator (`systemd_installer.py`)

Ensures:
- systemd units only for `IMPLEMENTED` phases
- Units exist only in unified directory (`/home/ransomeye/rebuild/systemd/`)
- Installer never installs `NOT_IMPLEMENTED` phases
- systemd units match phase `installable`/`runnable` flags

**Mismatch → FAIL (fail-closed)**

### 4. Fail-Closed/Fail-Open Auditor (`fail_closed_auditor.py`)

Detects:
- Sensors allowed fail-open (correct behavior)
- Enforcement allowed fail-open (**FORBIDDEN** - must fail-closed)
- Core services failing open (**FORBIDDEN** - must fail-closed)

**Policy:**
- Sensors (agents, probes, scanners) → fail-open allowed (buffer to disk)
- Enforcement/Policy/Decision engines → **MUST fail-closed**
- Core services → **MUST fail-closed**
- Playbook execution → **MUST fail-closed**

**Violation → FAIL (fail-closed)**

### 5. AI/ML Claim Validator (`ai_ml_claims.py`)

Cross-checks:
- README claims vs actual model registry
- Training scripts existence
- SHAP presence (if model exists)
- Signature enforcement claims

**Mismatch → FAIL (fail-closed)**

## Usage

### Command Line

```bash
# Run validation (outputs JSON to stdout)
python3 /home/ransomeye/rebuild/core/global_validator/validate.py

# Save report to file
python3 /home/ransomeye/rebuild/core/global_validator/validate.py --output /tmp/validation_report.json

# Use custom project root
python3 /home/ransomeye/rebuild/core/global_validator/validate.py --project-root /path/to/rebuild
```

### Exit Codes

- `0`: Validation passed (no critical violations)
- `1`: Validation failed (critical violations detected)
- `2`: Error during validation

### JSON Output Format

```json
{
  "validation_timestamp": "2025-01-27T12:00:00Z",
  "passed": false,
  "summary": {
    "total_violations": 3,
    "critical_violations": 2,
    "warning_violations": 1,
    "info_violations": 0,
    "checkers_run": [
      "phase_consistency",
      "db_ownership",
      "systemd_installer",
      "fail_closed",
      "ai_ml_claims"
    ]
  },
  "violations": [
    {
      "checker": "phase_consistency",
      "severity": "critical",
      "message": "Phase 9 (Network Scanner) marked IMPLEMENTED but path does not exist: /home/ransomeye/rebuild/core/network_scanner",
      "details": {
        "expected_path": "/home/ransomeye/rebuild/core/network_scanner"
      },
      "phase_id": 9,
      "phase_name": "Network Scanner"
    }
  ]
}
```

## Integration

### CI/CD Pipeline

Add to your CI pipeline:

```yaml
- name: Validate Consistency
  run: |
    python3 /home/ransomeye/rebuild/core/global_validator/validate.py --output validation_report.json
    # Exit code 1 will fail the build
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

python3 /home/ransomeye/rebuild/core/global_validator/validate.py
if [ $? -ne 0 ]; then
    echo "Validation failed. Commit rejected."
    exit 1
fi
```

### Installer Integration

Add to `install.sh`:

```bash
# Run consistency validation
python3 /home/ransomeye/rebuild/core/global_validator/validate.py
if [ $? -ne 0 ]; then
    error "Consistency validation failed. Installation aborted."
fi
```

## Testing

Run tests (these intentionally create violations to verify detection):

```bash
cd /home/ransomeye/rebuild/core/global_validator
python -m pytest tests/test_validator.py -v
```

**Note:** Tests intentionally create violations and expect failures. This verifies fail-closed behavior.

## Design Principles

1. **Read-Only**: Validator never modifies files or system state
2. **Fail-Closed**: Any violation results in non-zero exit code
3. **No Assumptions**: Validates against actual disk state and artifacts
4. **Machine-Verifiable**: JSON output suitable for automated processing
5. **Comprehensive**: Cross-checks all consistency rules

## Limitations

1. **Code Parsing**: DB ownership validation uses heuristics to extract table definitions from source code. Complex cases may require manual review.

2. **README Parsing**: Phase status extraction from READMEs uses pattern matching. Explicit status markers improve accuracy.

3. **Name Matching**: systemd unit to phase matching uses heuristics. Clear naming conventions improve detection.

4. **Model Detection**: AI/ML model detection searches for common file extensions. Custom formats may require configuration.

## Future Enhancements

- [ ] Add database connection to verify actual table ownership
- [ ] Improve README status extraction with explicit markers
- [ ] Add configuration file for custom rules
- [ ] Add HTML report generation
- [ ] Add signed verdict output
- [ ] Add historical trend tracking

## Status

✅ **COMPLETE**

- [x] Phase Consistency Checker
- [x] DB Ownership Validator
- [x] systemd/Installer Validator
- [x] Fail-Closed/Fail-Open Auditor
- [x] AI/ML Claim Validator
- [x] Test suite with intentional violations
- [x] CLI entry point
- [x] Documentation

## Author

nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU

## License

Proprietary - RansomEye.Tech

