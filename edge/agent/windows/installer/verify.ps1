# Path and File Name: /home/ransomeye/rebuild/ransomeye_windows_agent/installer/verify.ps1
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: Post-install validation script for Windows Agent

$ErrorActionPreference = "Continue"

$InstallDir = "C:\Program Files\RansomEye\WindowsAgent"
$DataDir = "C:\ProgramData\RansomEye\WindowsAgent"
$ServiceName = "RansomEyeWindowsAgent"
$BinaryPath = Join-Path $InstallDir "ransomeye_windows_agent.exe"

$Passed = 0
$Failed = 0

function Check {
    param(
        [string]$Name,
        [scriptblock]$TestCmd
    )
    
    try {
        $result = & $TestCmd
        if ($result) {
            Write-Host "✓ $Name" -ForegroundColor Green
            $script:Passed++
            return $true
        } else {
            Write-Host "✗ $Name" -ForegroundColor Red
            $script:Failed++
            return $false
        }
    } catch {
        Write-Host "✗ $Name" -ForegroundColor Red
        Write-Host "  Error: $_" -ForegroundColor Red
        $script:Failed++
        return $false
    }
}

Write-Host "RansomEye Windows Agent - Installation Verification"
Write-Host "===================================================="
Write-Host ""

# Binary checks
Check "Binary exists" { Test-Path $BinaryPath }
Check "Binary is executable" { (Get-Item $BinaryPath).Extension -eq ".exe" }

# Verify binary signature (Authenticode)
try {
    $sig = Get-AuthenticodeSignature $BinaryPath
    if ($sig.Status -eq "Valid") {
        Write-Host "✓ Binary signature verified" -ForegroundColor Green
        $Passed++
    } else {
        Write-Host "✗ Binary signature invalid. Status: $($sig.Status)" -ForegroundColor Red
        $Failed++
    }
} catch {
    Write-Host "✗ Could not verify binary signature: $_" -ForegroundColor Red
    $Failed++
}

# Directory checks
Check "Install directory exists" { Test-Path $InstallDir }
Check "Data directory exists" { Test-Path $DataDir }
Check "Config directory exists" { Test-Path (Join-Path $DataDir "config") }
Check "Logs directory exists" { Test-Path (Join-Path $DataDir "logs") }

# Service checks
Check "Service exists" { Get-Service -Name $ServiceName -ErrorAction SilentlyContinue }
Check "Service file exists" { Test-Path "C:\Windows\System32\RansomEyeWindowsAgent.exe" -or (Get-Service -Name $ServiceName).BinaryPathName }

# Summary
Write-Host ""
Write-Host "===================================================="
Write-Host "Verification Summary: $Passed passed, $Failed failed"
Write-Host "===================================================="

if ($Failed -eq 0) {
    Write-Host "All checks passed!" -ForegroundColor Green
    exit 0
} else {
    Write-Host "Some checks failed. Please review the errors above." -ForegroundColor Red
    exit 1
}

