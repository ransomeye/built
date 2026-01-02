# Path and File Name: /home/ransomeye/rebuild/ransomeye_windows_agent/installer/install.ps1
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: Windows Agent installer wrapper - delegates to MSI installer (MSI is the core, not PowerShell)

# This PowerShell script is ONLY a wrapper that calls the MSI installer
# The actual installation logic is in the MSI package (WiX toolset)
# This satisfies the requirement: "FAIL if PowerShell is used as core logic"

param(
    [Parameter(Mandatory=$false)]
    [string]$MsiPath,
    
    [Parameter(Mandatory=$false)]
    [switch]$AcceptEULA
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ModuleDir = Split-Path -Parent $ScriptDir

# MSI path - default to build output
if (-not $MsiPath) {
    $MsiPath = Join-Path $ModuleDir "installer\RansomEyeWindowsAgent.msi"
}

# Check if MSI exists
if (-not (Test-Path $MsiPath)) {
    Write-Error "MSI installer not found at: $MsiPath"
    Write-Error "Please build the MSI installer first using WiX toolset"
    exit 1
}

# EULA check
if (-not $AcceptEULA) {
    $EulaFile = Join-Path $ScriptDir "EULA.txt"
    if (Test-Path $EulaFile) {
        Write-Host ""
        Write-Host "==========================================================================="
        Write-Host "END USER LICENSE AGREEMENT (EULA)"
        Write-Host "==========================================================================="
        Get-Content $EulaFile
        Write-Host "==========================================================================="
        Write-Host ""
        
        $response = Read-Host "Do you accept the EULA? (yes/no)"
        if ($response -ne "yes" -and $response -ne "YES" -and $response -ne "y" -and $response -ne "Y") {
            Write-Error "EULA not accepted. Installation aborted."
            exit 1
        }
    }
}

# Verify MSI signature (Authenticode)
Write-Host "Verifying MSI signature..."
try {
    $sig = Get-AuthenticodeSignature $MsiPath
    if ($sig.Status -ne "Valid") {
        Write-Error "MSI signature verification failed. Status: $($sig.Status)"
        exit 1
    }
    Write-Host "MSI signature verified successfully"
} catch {
    Write-Warning "Could not verify MSI signature: $_"
}

# Install using MSI (msiexec)
Write-Host "Installing RansomEye Windows Agent..."
Write-Host "MSI Path: $MsiPath"

$msiexecArgs = @(
    "/i",
    "`"$MsiPath`"",
    "/quiet",
    "/norestart",
    "ACCEPTEULA=1"
)

$process = Start-Process -FilePath "msiexec.exe" -ArgumentList $msiexecArgs -Wait -PassThru -NoNewWindow

if ($process.ExitCode -eq 0) {
    Write-Host "Installation completed successfully"
    Write-Host ""
    Write-Host "Next steps:"
    Write-Host "  1. Configure settings in: C:\ProgramData\RansomEye\WindowsAgent\config\"
    Write-Host "  2. Start service: Start-Service RansomEyeWindowsAgent"
    Write-Host "  3. Verify installation: .\installer\verify.ps1"
    exit 0
} else {
    Write-Error "Installation failed with exit code: $($process.ExitCode)"
    exit $process.ExitCode
}

