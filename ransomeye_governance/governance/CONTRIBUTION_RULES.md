# Path: /home/ransomeye/rebuild/ransomeye_governance/governance/CONTRIBUTION_RULES.md
# Author: RansomEye Core Team
# Purpose: Defines contribution rules for RansomEye development, including code standards, review process, and enforcement

# RansomEye Contribution Rules

## Overview

This document defines the rules and processes for contributing to RansomEye. All contributions must comply with these rules, which are enforced by automated tooling and CI.

**These rules are enforced by code, not documentation.**

---

## Code Standards

### Standard 1: File Headers

**MANDATORY:** Every file MUST start with:

```
# Path: <absolute repo path>
# Author: RansomEye Core Team
# Purpose: <explicit purpose>
```

Applies to: `.rs .c .cpp .py .sh .yaml .json .toml .service`

**Violation = Build Failure**

### Standard 2: License Compliance

- All code must comply with LICENSE_POLICY.md
- No GPL, AGPL, SSPL dependencies
- License headers in source files

**Violation = Build Failure**

### Standard 3: Security Compliance

- No hardcoded secrets
- No hardcoded credentials
- All secrets via environment variables
- Security scanning passes

**Violation = Build Failure**

### Standard 4: Code Quality

- Code must compile/run without errors
- Code must pass linting
- Code must pass static analysis
- Code must be documented

**Violation = Build Failure**

---

## Environment Configuration

### Rule 1: ENV-Only Configuration

- All configuration via environment variables
- No hardcoded defaults for secrets
- Missing required ENV = **STARTUP FAILURE**
- Unknown ENV = **STARTUP FAILURE**

### Rule 2: ENV Validation

- ENV variables validated at startup
- ENV schema documented
- Invalid ENV = **STARTUP FAILURE**

---

## Testing Requirements

### Requirement 1: Test Coverage

- All new code must have tests
- Tests must be runnable
- Tests must pass
- Test coverage must be maintained

### Requirement 2: Test Types

- Unit tests for all functions
- Integration tests for components
- End-to-end tests for workflows
- Security tests for security features

### Requirement 3: Test Execution

- Tests run in CI
- Tests must pass before merge
- Test failures block merge

---

## Review Process

### Process 1: Pre-Commit Checks

Before commit:
- File headers validated
- License compliance checked
- Secret scanning performed
- Code formatting checked

**Failure = Commit Blocked**

### Process 2: Pull Request Review

Pull requests must:
- Pass all CI checks
- Have code review approval
- Have test coverage
- Have documentation updates

**Failure = Merge Blocked**

### Process 3: Merge Requirements

Before merge:
- All CI checks pass
- All reviews approved
- All tests pass
- Documentation updated

**Failure = Merge Blocked**

---

## Enforcement

### Automated Enforcement

1. **Pre-Commit Hooks** - Headers, licenses, secrets checked
2. **CI Pipeline** - All checks run automatically
3. **Build System** - Build fails on violations

### Manual Enforcement

1. **Code Review** - Human review for quality
2. **Security Review** - Security team review for security
3. **Architecture Review** - Architecture team review for design

---

## Prohibited Practices

### Prohibited 1: Placeholders

- No TODO comments without tickets
- No placeholder code
- No "future phase will handle this"
- No assumptions

### Prohibited 2: Hardcoded Values

- No hardcoded IPs
- No hardcoded ports
- No hardcoded paths
- No hardcoded secrets

### Prohibited 3: Bypassing Checks

- No bypassing CI checks
- No bypassing security checks
- No bypassing license checks
- No bypassing header checks

---

## Compliance

These rules are enforced by:
- `ransomeye_governance/tooling/header_validator.py`
- `ransomeye_governance/tooling/license_validator.py`
- `ransomeye_governance/tooling/secret_validator.py`
- `ransomeye_governance/ci/header_enforcement.yml`
- `ransomeye_governance/ci/license_scan.yml`
- `ransomeye_governance/ci/secret_scan.yml`

**Violation = Build Failure = CI Failure = Merge Blocked**

---

## References

- LICENSE_POLICY.md
- SECURITY_POLICY.md
- AI_DAY1_READINESS.md
- SUPPLY_CHAIN_SECURITY.md

