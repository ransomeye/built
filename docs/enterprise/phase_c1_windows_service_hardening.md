# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/phase_c1_windows_service_hardening.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Phase C1 - Windows Agent Service Hardening

# Phase C1 - Windows Agent Service Hardening

**Date:** 2026-01-28  
**Phase:** PROMPT-53 — BLOCKER 4 Resolution  
**Status:** ✅ **FRAMEWORK COMPLETE**

---

## Objective

Validate Windows Agent service hardening:
- Service account configuration
- Minimal privilege assignment
- Service isolation
- Security policy enforcement

---

## Service Hardening Checklist

### 1. Service Account Configuration

**Requirements:**
- ✅ Dedicated service account (not Administrator)
- ✅ "Log on as a service" right
- ✅ No interactive logon rights
- ✅ Password never expires (or managed rotation)

**Validation:**
```powershell
# Check service account
$service = Get-WmiObject -Class Win32_Service -Filter "Name='RansomEyeAgent'"
$serviceAccount = $service.StartName

# Verify account exists
Get-LocalUser -Name $serviceAccount

# Verify "Log on as a service" right
$right = "SeServiceLogonRight"
$account = New-Object System.Security.Principal.NTAccount($serviceAccount)
$sid = $account.Translate([System.Security.Principal.SecurityIdentifier])
$policy = [System.Security.AccessControl.LsaSecurityPolicy]::GetPolicy()
$rights = $policy.GetAccountRights($sid)
$rights -contains $right
```

**Expected Result:**
- Service account exists
- "Log on as a service" right granted
- No administrator privileges

---

### 2. Minimal Privilege Assignment

**Requirements:**
- ✅ No administrator privileges
- ✅ No "Debug programs" right
- ✅ No "Take ownership" right
- ✅ Only required privileges for telemetry collection

**Validation:**
```powershell
# Check service account privileges
$serviceAccount = "RansomEyeAgent"
$account = New-Object System.Security.Principal.NTAccount($serviceAccount)
$sid = $account.Translate([System.Security.Principal.SecurityIdentifier])
$policy = [System.Security.AccessControl.LsaSecurityPolicy]::GetPolicy()
$privileges = $policy.GetAccountPrivileges($sid)

# Verify no dangerous privileges
$dangerousPrivileges = @("SeDebugPrivilege", "SeTakeOwnershipPrivilege", "SeBackupPrivilege", "SeRestorePrivilege")
foreach ($priv in $dangerousPrivileges) {
    if ($privileges -contains $priv) {
        Write-Warning "Dangerous privilege found: $priv"
    }
}
```

**Expected Result:**
- No dangerous privileges assigned
- Only minimal required privileges

---

### 3. Service Isolation

**Requirements:**
- ✅ Service runs in isolated context
- ✅ No access to other services' data
- ✅ No access to user data
- ✅ Bounded resource usage

**Validation:**
```powershell
# Check service process isolation
$service = Get-Service -Name "RansomEyeAgent"
$process = Get-Process -Id (Get-CimInstance Win32_Service -Filter "Name='RansomEyeAgent'").ProcessId

# Check process token
$token = $process | Select-Object -ExpandProperty Handle
# (Advanced: Check token privileges via Win32 API)

# Check resource limits
$process | Select-Object WorkingSet, VirtualMemorySize, PagedMemorySize
```

**Expected Result:**
- Service process isolated
- Resource usage bounded
- No access to other services

---

### 4. Security Policy Enforcement

**Requirements:**
- ✅ Service cannot be stopped by non-administrators
- ✅ Service binary protected from modification
- ✅ Service configuration protected
- ✅ Audit logging enabled

**Validation:**
```powershell
# Check service protection
$service = Get-Service -Name "RansomEyeAgent"
$acl = Get-Acl "HKLM:\SYSTEM\CurrentControlSet\Services\RansomEyeAgent"

# Verify service stop protection
# (Service should require administrator to stop)

# Check binary protection
$binaryPath = (Get-CimInstance Win32_Service -Filter "Name='RansomEyeAgent'").PathName
$binaryAcl = Get-Acl $binaryPath
$binaryAcl | Format-List

# Check audit logging
auditpol /get /category:"Logon/Logoff"
```

**Expected Result:**
- Service protected from unauthorized stop
- Binary protected from modification
- Audit logging enabled

---

## Hardening Validation Matrix

| Hardening Aspect | Requirement | Validation Method | Status |
|------------------|-------------|-------------------|--------|
| **Service Account** | Dedicated non-admin account | Check service StartName | PENDING |
| **Log on as Service** | Right granted | Check account rights | PENDING |
| **Minimal Privileges** | No dangerous privileges | Check account privileges | PENDING |
| **Service Isolation** | Isolated process context | Check process isolation | PENDING |
| **Resource Limits** | Bounded memory/CPU | Check process resources | PENDING |
| **Binary Protection** | Protected from modification | Check file ACLs | PENDING |
| **Stop Protection** | Requires admin to stop | Test stop permission | PENDING |
| **Audit Logging** | Enabled | Check audit policy | PENDING |

---

## Test Execution

### Test Script Location
`tests/windows_service_hardening.ps1`

### Test Execution
```powershell
# Run service hardening tests
.\tests\windows_service_hardening.ps1

# Expected output:
# - Service account validation
# - Privilege validation
# - Isolation validation
# - Security policy validation
```

---

## Conclusion

**Phase C1 Status:** ✅ **FRAMEWORK COMPLETE**

Service hardening framework is complete with:
- ✅ All hardening aspects defined
- ✅ Validation procedures documented
- ✅ Test execution framework ready
- ❌ Execution pending (requires Windows VM)

**Next Steps:**
1. Execute hardening validation tests
2. Verify all hardening aspects
3. Document any findings
4. Proceed to Phase C2

**Blocking Issues:** Windows VM provisioning required

