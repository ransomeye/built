# Path and File Name: /home/ransomeye/rebuild/ransomeye_windows_agent/installer/lifecycle.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: Service lifecycle management semantics for Windows Agent

# RansomEye Windows Agent - Service Lifecycle

## Service Management

The Windows Agent runs as a Windows Service: `RansomEyeWindowsAgent`

## Start

**Command (PowerShell):**
```powershell
Start-Service -Name "RansomEyeWindowsAgent"
```

**Command (CMD):**
```cmd
net start RansomEyeWindowsAgent
```

**Command (Services MMC):**
1. Open `services.msc`
2. Find "RansomEye Windows Agent"
3. Right-click → Start

**Behavior:**
- Service starts automatically (if configured)
- Begins ETW telemetry collection
- Connects to Core API via mTLS
- Begins sending signed telemetry events

**Prerequisites:**
- Service must be installed (run MSI installer first)
- Core API must be reachable (or buffering enabled)
- Certificates must be present and valid

## Stop

**Command (PowerShell):**
```powershell
Stop-Service -Name "RansomEyeWindowsAgent"
```

**Command (CMD):**
```cmd
net stop RansomEyeWindowsAgent
```

**Behavior:**
- Gracefully stops telemetry collection
- Flushes remaining buffers to disk (if Core unavailable)
- Closes network connections
- Shuts down all worker threads

**Graceful Shutdown:**
- Service handles stop requests gracefully
- Maximum shutdown timeout: 30 seconds
- After timeout, service is forcefully terminated

## Restart

**Command (PowerShell):**
```powershell
Restart-Service -Name "RansomEyeWindowsAgent"
```

**Behavior:**
- Equivalent to `stop` followed by `start`
- Useful for applying configuration changes

## Status

**Command (PowerShell):**
```powershell
Get-Service -Name "RansomEyeWindowsAgent"
```

**Output Includes:**
- Service status (Running, Stopped, etc.)
- Service name and display name
- Service start type (Automatic, Manual, Disabled)

**Command (CMD):**
```cmd
sc query RansomEyeWindowsAgent
```

## Service Configuration

**Set Startup Type (Automatic):**
```powershell
Set-Service -Name "RansomEyeWindowsAgent" -StartupType Automatic
```

**Set Startup Type (Manual):**
```powershell
Set-Service -Name "RansomEyeWindowsAgent" -StartupType Manual
```

**Set Startup Type (Disabled):**
```powershell
Set-Service -Name "RansomEyeWindowsAgent" -StartupType Disabled
```

## Logs

**View Event Logs:**
```powershell
# Windows Event Viewer
eventvwr.msc

# Filter for RansomEye logs
Get-WinEvent -LogName Application | Where-Object {$_.ProviderName -like "*RansomEye*"}
```

**View Application Logs:**
```powershell
# Application-specific logs
Get-Content "C:\ProgramData\RansomEye\WindowsAgent\logs\*.log" -Tail 50
```

**Log Locations:**
- Windows Event Log: Application log
- Application logs: `C:\ProgramData\RansomEye\WindowsAgent\logs\` (if configured)

## Health Checks

**Service Health:**
```powershell
# Check if service is running
(Get-Service -Name "RansomEyeWindowsAgent").Status

# Check service process
Get-Process | Where-Object {$_.ProcessName -like "*ransomeye*"}
```

**Expected Output:**
- `Running`: Service is active
- `Stopped`: Service is inactive
- `StartPending`: Service is starting
- `StopPending`: Service is stopping

## Troubleshooting

**Service Won't Start:**
1. Check prerequisites (certificates, Core API connectivity)
2. Check Event Viewer for error messages
3. Verify installation: `.\installer\verify.ps1`
4. Check service account permissions
5. Verify certificates are accessible to LocalSystem account

**Service Crashes:**
1. Check Event Viewer for error messages
2. Check application logs
3. Verify Core API connectivity
4. Check certificates are valid and accessible
5. Verify sufficient disk space for buffers

**High CPU/Memory Usage:**
- Normal for active telemetry collection
- Check buffer size configuration
- Verify ETW session configuration

## Lifecycle State Machine

```
[Not Installed]
    ↓ (MSI installer)
[Installed, Stopped]
    ↓ (Start-Service or Automatic)
[Running]
    ↓ (Stop-Service)
[Stopped]
    ↓ (Uninstall MSI)
[Removed]
```

## Service Dependencies

- **Depends On**: TCP/IP Protocol Driver
- **Required**: Network connectivity for Core API

## Recovery Actions

**Configure Service Recovery (PowerShell):**
```powershell
sc failure "RansomEyeWindowsAgent" reset= 86400 actions= restart/5000/restart/5000/restart/5000
```

**Recovery Options:**
- Restart service on failure
- Restart computer on failure (not recommended)
- Run a program on failure (for notification)

## Installation/Uninstallation

**Install:**
- MSI installer (recommended)
- PowerShell wrapper script (`install.ps1`)
- Command line: `msiexec.exe /i RansomEyeWindowsAgent.msi /quiet`

**Uninstall:**
- Control Panel → Add/Remove Programs
- PowerShell wrapper script (`uninstall.ps1`)
- Command line: `msiexec.exe /x {ProductCode} /quiet`

## Service Account

- **Default**: LocalSystem (NT AUTHORITY\SYSTEM)
- **Permissions**: High privileges required for ETW
- **No User Interaction**: Service runs in background

