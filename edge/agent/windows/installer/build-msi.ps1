# Path and File Name: /home/ransomeye/rebuild/ransomeye_windows_agent/installer/build-msi.ps1
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: Build script for MSI installer (requires WiX toolset)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ModuleDir = Split-Path -Parent $ScriptDir
$WxsFile = Join-Path $ScriptDir "RansomEyeWindowsAgent.wxs"

Write-Host "Building MSI installer..."
Write-Host "WiX Source: $WxsFile"

# Check for WiX toolset
$candle = Get-Command candle.exe -ErrorAction SilentlyContinue
$light = Get-Command light.exe -ErrorAction SilentlyContinue

if (-not $candle -or -not $light) {
    Write-Error "WiX toolset not found. Please install WiX Toolset v3.11+ from https://wixtoolset.org/"
    exit 1
}

# Build Rust binary first
Write-Host "Building Rust binary..."
Push-Location $ModuleDir
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to build Rust binary"
    exit 1
}
Pop-Location

# Compile WiX source
Write-Host "Compiling WiX source..."
$wixobjFile = Join-Path $ScriptDir "RansomEyeWindowsAgent.wixobj"
& candle.exe -out $wixobjFile $WxsFile
if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to compile WiX source"
    exit 1
}

# Link MSI
Write-Host "Linking MSI..."
$msiFile = Join-Path $ScriptDir "RansomEyeWindowsAgent.msi"
& light.exe -out $msiFile $wixobjFile
if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to link MSI"
    exit 1
}

Write-Host "MSI built successfully: $msiFile"
Write-Host ""
Write-Host "Next steps:"
Write-Host "  1. Sign MSI: signtool sign /f certificate.pfx /p password $msiFile"
Write-Host "  2. Install: .\install.ps1 -MsiPath $msiFile -AcceptEULA"

