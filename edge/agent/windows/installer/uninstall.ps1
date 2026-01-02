# Path and File Name: /home/ransomeye/rebuild/ransomeye_windows_agent/installer/uninstall.ps1
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: Windows Agent uninstaller wrapper - delegates to MSI uninstaller

# This PowerShell script is ONLY a wrapper that calls MSI uninstaller
# The actual uninstall logic is in the MSI package (handled by Windows Installer)

param(
    [Parameter(Mandatory=$false)]
    [switch]$PreserveLogs
)

$ErrorActionPreference = "Stop"

$ProductCode = "{YOUR-PRODUCT-GUID-HERE}"  # Will be replaced by MSI build

Write-Host "Uninstalling RansomEye Windows Agent..."

# Find installed product
$installed = Get-WmiObject Win32_Product -Filter "Name='RansomEye Windows Agent'" -ErrorAction SilentlyContinue

if (-not $installed) {
    Write-Warning "RansomEye Windows Agent is not installed"
    exit 0
}

# Stop service
$service = Get-Service -Name "RansomEyeWindowsAgent" -ErrorAction SilentlyContinue
if ($service -and $service.Status -eq "Running") {
    Write-Host "Stopping service..."
    Stop-Service -Name "RansomEyeWindowsAgent" -Force
    Write-Host "Service stopped"
}

# Uninstall via MSI
$msiexecArgs = @(
    "/x",
    $installed.IdentifyingNumber,
    "/quiet",
    "/norestart"
)

if ($PreserveLogs) {
    $msiexecArgs += "PRESERVELOGS=1"
}

$process = Start-Process -FilePath "msiexec.exe" -ArgumentList $msiexecArgs -Wait -PassThru -NoNewWindow

if ($process.ExitCode -eq 0) {
    Write-Host "Uninstallation completed successfully"
    exit 0
} else {
    Write-Error "Uninstallation failed with exit code: $($process.ExitCode)"
    exit $process.ExitCode
}

