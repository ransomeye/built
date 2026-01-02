# Path and File Name: /home/ransomeye/rebuild/ransomeye_windows_agent/installer/requirements.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: OS, kernel, and privilege requirements for Windows Agent

# RansomEye Windows Agent - System Requirements

## Operating System

- **Required**: Microsoft Windows
- **Minimum**: Windows 10 (Build 1809+) or Windows Server 2019+
- **Recommended**: Windows 11 or Windows Server 2022+

## Architecture

- **Supported**: x64 (64-bit) only
- **Unsupported**: x86 (32-bit), ARM, ARM64

## Memory Requirements

- **RAM**: Minimum 512MB, recommended 1GB+
- **Swap**: Uses Windows virtual memory (pagefile) - managed by OS

## Disk Space

- **Installation**: 150MB for binaries and configuration
- **Runtime**: Additional space for logs and buffers
  - Buffer directory (default): `C:\ProgramData\RansomEye\WindowsAgent\buffer`
  - Configurable via `BUFFER_DIR` environment variable
  - Default buffer size: 512MB (configurable via `MAX_BUFFER_SIZE_MB`)

## Privileges

- **Installation**: Administrator privileges required
- **Runtime**: Service runs as LocalSystem (NT AUTHORITY\SYSTEM)
  - High privileges required for ETW telemetry collection
  - No user-mode restrictions

## Software Dependencies

- **Windows Installer**: MSI 5.0+ (included in Windows)
- **.NET Runtime**: Not required (native Rust binary)
- **Visual C++ Runtime**: May be required (included in Windows 10+)

## Network Configuration

- **mTLS**: Requires client certificate and private key
  - Configurable via `AGENT_CERT_PATH` and `AGENT_KEY_PATH`
- **CA Certificate**: Required for Core API connection
  - Configurable via `CA_CERT_PATH`

## Security Requirements

- **Binary Signing**: Binaries must be Authenticode signed
- **MSI Signing**: MSI installer must be Authenticode signed
- **EULA Acceptance**: Required during installation (no bypass)
- **No Kernel Drivers**: Runs in user mode only (no kernel drivers)

## Installation Requirements

- **WiX Toolset**: Required to build MSI installer
  - Download from: https://wixtoolset.org/
  - Version 3.11+ recommended
- **SignTool**: Required for signing MSI and binaries
  - Included in Windows SDK

## Build Requirements

To build the MSI installer:

1. Install WiX Toolset v3.11+
2. Install Windows SDK (for SignTool)
3. Build Rust binary: `cargo build --release`
4. Build MSI: `candle RansomEyeWindowsAgent.wxs && light RansomEyeWindowsAgent.wixobj`
5. Sign MSI: `signtool sign /f certificate.pfx /p password RansomEyeWindowsAgent.msi`

## Uninstallation

- **Via Control Panel**: Add/Remove Programs (standard Windows uninstall)
- **Via MSI**: `msiexec.exe /x {ProductCode}`
- **Via PowerShell**: Wrapper script `uninstall.ps1`

## Performance Considerations

- **ETW Collection**: Uses Windows Event Tracing for telemetry
- **CPU Usage**: Minimal during idle, scales with event volume
- **Memory**: Bounded buffers prevent memory exhaustion

## Registry

- **Installation Key**: `HKEY_LOCAL_MACHINE\SOFTWARE\RansomEye\WindowsAgent`
- **Service Configuration**: Windows Service Control Manager

## File Locations

- **Installation**: `C:\Program Files\RansomEye\WindowsAgent\`
- **Data/Config**: `C:\ProgramData\RansomEye\WindowsAgent\`
- **Service Binary**: `C:\Windows\System32\` (registered service)

