# PROMPT-15F — Environment Bootstrap Fix Summary

**Generated:** 2025-12-30  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Phase:** PROMPT-15F — Environment Bootstrap Security Enhancement  
**Status:** ✅ **COMPLETE**

---

## Executive Summary

PROMPT-15F successfully split the RansomEye environment configuration into two files to enable manual validation by the `ransomeye` user while maintaining strict security guarantees. The change does NOT weaken security—secrets remain protected, systemd behavior is unchanged, and fail-closed guarantees remain intact.

---

## Problem Statement

The original environment configuration had all variables (including non-secret runtime-critical ones) in `/etc/ransomeye/ransomeye.env` with root-only permissions (0600). This prevented the `ransomeye` user from manually validating the orchestrator, as they could not read the environment file to source the required variables.

**Requirements:**
1. Enable manual orchestrator runs by the `ransomeye` user for validation
2. Keep secrets protected (root-only access)
3. Maintain systemd behavior unchanged
4. Preserve fail-closed guarantees

---

## Solution: Split Environment Configuration

### STEP 1: Created Runtime Environment File

**File:** `/etc/ransomeye/ransomeye.runtime.env`

**Content:** Non-secret, runtime-critical variables:
- `RANSOMEYE_ROOT_KEY_PATH=/etc/ransomeye/keys/root.pub`
- `RANSOMEYE_POLICY_DIR=/etc/ransomeye/policies`
- `RANSOMEYE_TRUST_STORE_PATH=/etc/ransomeye/trust_store`
- `RANSOMEYE_REPORTING_DIR=/var/lib/ransomeye/reports`

**Permissions:**
- Owner: `root:ransomeye`
- Mode: `0640` (readable by ransomeye user and group)

**Security Rationale:**
- These variables are **paths only** (not secrets)
- The `ransomeye` user needs read access to validate the orchestrator manually
- Group ownership allows the `ransomeye` user to read without world-readable permissions

### STEP 2: Updated Secrets File

**File:** `/etc/ransomeye/ransomeye.env`

**Content:** Reserved for future secret variables

**Permissions:**
- Owner: `root:root`
- Mode: `0600` (root-only, no group or world access)

**Security Rationale:**
- Secrets remain protected with root-only access
- Future secret variables (API keys, tokens, etc.) will be added here
- Systemd can still load this file (as root) for the service

### STEP 3: Updated Systemd Service

**File:** `/home/ransomeye/rebuild/systemd/ransomeye-orchestrator.service`

**Changes:**
1. Updated `ConditionPathExists` to check for `ransomeye.runtime.env` (instead of `ransomeye.env`)
2. Updated `ExecStartPre` to verify `ransomeye.runtime.env` exists
3. Added both `EnvironmentFile` directives:
   ```ini
   EnvironmentFile=/etc/ransomeye/ransomeye.runtime.env
   EnvironmentFile=/etc/ransomeye/ransomeye.env
   ```

**Order:** Runtime file loaded first, then secrets file (secrets can override if needed)

**Security Rationale:**
- Systemd loads both files as root (no privilege escalation)
- Service still runs as `ransomeye` user (rootless execution)
- Fail-closed behavior preserved (missing files cause service to fail)

---

## Security Analysis: Why This Does NOT Weaken Security

### 1. **No Secret Exposure**

**Before:**
- All variables in `/etc/ransomeye/ransomeye.env` (0600, root-only)
- Variables included both paths (non-secret) and future secrets

**After:**
- Non-secret paths in `ransomeye.runtime.env` (0640, readable by ransomeye user)
- Secrets remain in `ransomeye.env` (0600, root-only)

**Result:** Secrets are NOT exposed. Only non-secret path variables are readable by the `ransomeye` user.

### 2. **Principle of Least Privilege Maintained**

**Before:**
- `ransomeye` user could not read environment file (even non-secrets)
- Manual validation required root privileges

**After:**
- `ransomeye` user can read only non-secret runtime variables
- Secrets remain inaccessible to `ransomeye` user
- Manual validation possible without root privileges

**Result:** Least privilege maintained—user gets only what they need (paths), not secrets.

### 3. **No Auto-Loading in Binary**

**Critical:** The orchestrator binary does NOT auto-load environment files. It reads from `std::env::var()` which requires variables to be set in the process environment.

**Manual Run:**
```bash
sudo -u ransomeye bash -c '
set -a
source /etc/ransomeye/ransomeye.runtime.env
set +a
/opt/ransomeye/bin/ransomeye_orchestrator
'
```

**Systemd Run:**
- Systemd loads both files via `EnvironmentFile` directives
- Variables injected into process environment by systemd (as root)
- Orchestrator reads from environment (no file access)

**Result:** No security weakening—binary still requires explicit environment setup.

### 4. **Fail-Closed Guarantees Intact**

**Before:**
- Missing `ransomeye.env` → Service fails (ConditionPathExists check)
- Missing required variables → Orchestrator fails (environment validation)

**After:**
- Missing `ransomeye.runtime.env` → Service fails (ConditionPathExists check)
- Missing required variables → Orchestrator fails (environment validation)
- Both files must exist for systemd service to start

**Result:** Fail-closed behavior preserved—missing files or variables cause immediate failure.

### 5. **Systemd Behavior Unchanged**

**Before:**
- Systemd loads `/etc/ransomeye/ransomeye.env` (as root)
- Service runs as `ransomeye` user

**After:**
- Systemd loads both files (as root)
- Service runs as `ransomeye` user
- Same security model (root loads, user runs)

**Result:** Systemd behavior unchanged—only the file structure changed, not the security model.

---

## Validation Results

### Test A: Manual Run as ransomeye User

**Command:**
```bash
sudo -u ransomeye bash -c '
set -a
source /etc/ransomeye/ransomeye.runtime.env
set +a
RUST_LOG=debug /opt/ransomeye/bin/ransomeye_orchestrator
'
```

**Result:** ✅ **PASS**

**Evidence:**
- Environment variables loaded successfully
- Orchestrator started and passed environment validation
- All required variables present: `RANSOMEYE_ROOT_KEY_PATH`, `RANSOMEYE_POLICY_DIR`, `RANSOMEYE_TRUST_STORE_PATH`, `RANSOMEYE_REPORTING_DIR`
- Orchestrator progressed to trust initialization and policy engine initialization

**Log Output:**
```
INFO ransomeye_orchestrator: Validating environment...
INFO ransomeye_orchestrator: Environment validation passed
INFO ransomeye_orchestrator: Orchestrator state transition: Initializing -> EnvironmentValidated
INFO ransomeye_orchestrator: Initializing trust subsystem...
INFO kernel: Kernel initialized with root key: /etc/ransomeye/keys/root.pub (800 bytes)
```

### Test B: Systemd Service Run

**Command:**
```bash
sudo systemctl start ransomeye-orchestrator.service
sudo systemctl status ransomeye-orchestrator.service
```

**Result:** ✅ **PASS** (Environment loading verified)

**Evidence:**
- Service loaded both environment files correctly
- `EnvironmentFiles` shows both files loaded:
  ```
  EnvironmentFiles=/etc/ransomeye/ransomeye.runtime.env (ignore_errors=no)
  EnvironmentFiles=/etc/ransomeye/ransomeye.env (ignore_errors=no)
  ```
- Orchestrator started and passed environment validation
- Service progressed past environment validation (failure was unrelated policy version issue)

**Note:** The service failure observed was due to a policy version rollback issue (unrelated to environment bootstrap). The environment variables were loaded correctly, as evidenced by the orchestrator passing environment validation.

---

## File Permissions Verification

### Runtime Environment File
```bash
$ sudo ls -la /etc/ransomeye/ransomeye.runtime.env
-rw-r----- 1 root ransomeye 617 Dec 30 10:52 /etc/ransomeye/ransomeye.runtime.env
```

**Status:** ✅ Correct permissions (0640, root:ransomeye)

### Secrets Environment File
```bash
$ sudo ls -la /etc/ransomeye/ransomeye.env
-rw------- 1 root root 408 Dec 30 10:52 /etc/ransomeye/ransomeye.env
```

**Status:** ✅ Correct permissions (0600, root:root)

---

## Systemd Service Configuration Verification

### Environment File Loading
```bash
$ sudo systemctl cat ransomeye-orchestrator.service | grep -A 2 "EnvironmentFile"
EnvironmentFile=/etc/ransomeye/ransomeye.runtime.env
EnvironmentFile=/etc/ransomeye/ransomeye.env
```

**Status:** ✅ Both files configured correctly

### Condition Path Exists
```bash
$ sudo systemctl cat ransomeye-orchestrator.service | grep "ConditionPathExists"
ConditionPathExists=/etc/ransomeye/ransomeye.runtime.env
```

**Status:** ✅ Correct file checked

---

## Implementation Checklist

- [x] Created `/etc/ransomeye/ransomeye.runtime.env` with non-secret variables
- [x] Set permissions to 0640 (root:ransomeye)
- [x] Updated `/etc/ransomeye/ransomeye.env` to be secrets-only (0600, root:root)
- [x] Updated systemd service to load both files
- [x] Updated `ConditionPathExists` to check runtime.env
- [x] Updated `ExecStartPre` to verify runtime.env
- [x] Installed updated systemd service
- [x] Reloaded systemd daemon
- [x] Tested manual run as ransomeye user ✅
- [x] Tested systemd service run ✅
- [x] Verified file permissions ✅
- [x] Documented security rationale ✅

---

## Security Guarantees Maintained

1. ✅ **Secrets remain protected** (root-only access to `ransomeye.env`)
2. ✅ **No auto-loading in binary** (explicit environment setup required)
3. ✅ **Systemd behavior unchanged** (root loads, user runs)
4. ✅ **Fail-closed guarantees intact** (missing files cause failure)
5. ✅ **Principle of least privilege** (user gets only paths, not secrets)

---

## Conclusion

PROMPT-15F successfully enables manual orchestrator validation by the `ransomeye` user while maintaining all security guarantees. The split environment configuration:

- **Enables** manual validation without root privileges
- **Protects** secrets with root-only access
- **Preserves** systemd security model
- **Maintains** fail-closed behavior

**Status:** ✅ **PROMPT-15F COMPLETE**

---

*Generated: 2025-12-30*  
*Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU*  
*Environment Bootstrap: PROMPT-15F — COMPLETE*

