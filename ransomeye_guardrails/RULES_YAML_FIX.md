# Phase 0 Correction - rules.yaml Hardening

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_guardrails/RULES_YAML_FIX.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Documentation of rules.yaml fixes and schema validation implementation

---

## Summary

Phase 0 has been hardened to prevent malformed YAML and regex patterns from breaking the scanner. All rules are now schema-validated before use, with fail-closed enforcement.

---

## Changes Made

### 1. Fixed rules.yaml Structure

**Modified:** `/home/ransomeye/rebuild/ransomeye_guardrails/rules.yaml`

**Fixes:**
- All regex patterns properly quoted with double quotes
- All special characters properly escaped (`\` → `\\`, `.` → `\\.`)
- Consistent structure with required fields: `name`, `regex`, `description`, `rule_name`, `severity`
- Removed inline comments that could break YAML parsing
- Standardized indentation (2 spaces)

**New Format:**
```yaml
hardcoded_patterns:
  - name: "Hardcoded IPv4 address"
    regex: "\\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\\b"
    description: "Hardcoded IPv4 address detected"
    exception: "127\\.0\\.0\\.1|localhost|0\\.0\\.0\\.0"
    rule_name: "NO_HARDCODED_IPV4"
    severity: "CRITICAL"
```

**Key Changes:**
- `pattern` → `regex` (more explicit)
- All patterns double-quoted
- All backslashes escaped (`\\`)
- All dots escaped (`\\.`)
- Added `severity` field to all rules
- Added `name` field for human-readable rule names

---

### 2. Schema Validation Module

**Created:** `/home/ransomeye/rebuild/ransomeye_guardrails/rules_schema.py`

**Functionality:**
- Validates YAML syntax before parsing
- Validates required fields for each rule
- Validates regex pattern syntax (compiles each pattern)
- Validates severity values (CRITICAL, HIGH, MEDIUM, LOW)
- Validates rule_name format (must match `[A-Z_][A-Z0-9_]*`)
- Validates top-level structure
- Fails-closed on any validation error

**Required Fields:**
- `name`: Human-readable rule name
- `regex`: Regex pattern (validated for syntax)
- `description`: Rule description
- `rule_name`: Rule identifier (must match `[A-Z_][A-Z0-9_]*`)
- `severity`: One of CRITICAL, HIGH, MEDIUM, LOW

**Validation Checks:**
1. File exists
2. YAML syntax valid
3. Required top-level keys present
4. Each pattern rule has required fields
5. Regex patterns compile successfully
6. Severity values are valid
7. Rule names match format
8. Allowed exceptions structure valid
9. Scan config structure valid

---

### 3. Scanner Hardening

**Modified:** `/home/ransomeye/rebuild/ransomeye_guardrails/scanner.py`

**Changes:**
- Added schema validation BEFORE loading rules
- Rules validated using `validate_rules_file()` before `yaml.safe_load()`
- Scanner fails-closed if rules validation fails
- Updated to use `regex` field (with fallback to `pattern` for backward compatibility)
- No scanning occurs with partial or invalid rules

**Key Code:**
```python
def _load_rules(self, rules_path: str) -> Dict:
    # CRITICAL: Validate rules.yaml schema BEFORE loading
    if not validate_rules_file(rules_path):
        # validate_rules_file will fail-closed on errors
        fail_closed(...)
    
    # Load validated rules
    with open(rules_path, 'r') as f:
        rules = yaml.safe_load(f)
```

---

### 4. Test Suite

**Created:** `/home/ransomeye/rebuild/ransomeye_guardrails/tests/test_rules_yaml.py`

**Test Cases:**
- ✅ Valid rules.yaml → PASS
- ✅ Missing required field → FAIL
- ✅ Invalid regex pattern → FAIL
- ✅ Malformed YAML → FAIL
- ✅ Invalid severity → FAIL
- ✅ Invalid rule_name format → FAIL
- ✅ Empty regex → FAIL
- ✅ Missing top-level key → FAIL/WARN
- ✅ Bad indentation → FAIL
- ✅ Unescaped special chars → Validated

**Coverage:**
- All validation paths tested
- Edge cases covered
- Fail-closed behavior verified

---

### 5. CI/CD Enforcement

**Modified:** `/home/ransomeye/rebuild/ci/global_guardrails.yml`

**Added Step:**
```yaml
- name: Validate rules.yaml Schema
  run: |
    cd /home/ransomeye/rebuild
    python3 -m ransomeye_guardrails.rules_schema --rules ransomeye_guardrails/rules.yaml
  continue-on-error: false
```

**Enforcement:**
- Rules validation runs BEFORE guardrails checks
- CI fails if rules.yaml is invalid
- No guardrails checks run with invalid rules
- Explicit validation step for visibility

---

## Fail-Closed Enforcement

All validation is **fail-closed**:
- Invalid YAML syntax → Exit 1 with error details
- Missing required field → Exit 1 with field name
- Invalid regex pattern → Exit 1 with regex error
- Invalid severity → Exit 1 with valid options
- Invalid rule_name format → Exit 1 with format requirement
- No warnings-only mode
- No partial rule loading
- No silent failures

---

## Validation Flow

1. **File Exists Check** → Fail if missing
2. **YAML Syntax Check** → Fail if malformed
3. **Top-Level Structure** → Fail if required keys missing
4. **Pattern Rule Validation** → Fail if any rule invalid
5. **Regex Compilation** → Fail if regex syntax error
6. **Field Validation** → Fail if required fields missing
7. **Format Validation** → Fail if formats invalid
8. **Load Rules** → Only if all validation passes

---

## Backward Compatibility

Scanner supports both formats:
- `regex` field (preferred, validated)
- `pattern` field (legacy, fallback)

This ensures existing rules continue to work while encouraging migration to new format.

---

## Usage

### Validate rules.yaml

```bash
python3 -m ransomeye_guardrails.rules_schema --rules ransomeye_guardrails/rules.yaml
```

### Run Tests

```bash
python3 -m unittest ransomeye_guardrails.tests.test_rules_yaml
```

### Scanner Auto-Validation

Scanner automatically validates rules before loading:
```bash
python3 -m ransomeye_guardrails.scanner --rules ransomeye_guardrails/rules.yaml
```

---

## Acceptance Criteria

✅ Valid rules.yaml passes validation  
✅ Malformed YAML causes immediate failure  
✅ Invalid regex syntax detected early  
✅ Missing required fields detected  
✅ Scanner never runs with partial rules  
✅ CI blocks invalid rule definitions  
✅ All tests pass  
✅ Fail-closed on all error conditions  

---

## Files Modified/Created

1. ✅ `ransomeye_guardrails/rules.yaml` (FIXED)
2. ✅ `ransomeye_guardrails/rules_schema.py` (NEW)
3. ✅ `ransomeye_guardrails/scanner.py` (HARDENED)
4. ✅ `ransomeye_guardrails/tests/test_rules_yaml.py` (NEW)
5. ✅ `ci/global_guardrails.yml` (UPDATED)

---

## Last Updated

Phase 0 Correction - rules.yaml Hardening  
**Status:** Complete  
**Enforcement:** Fail-Closed  
**Validation:** Comprehensive

