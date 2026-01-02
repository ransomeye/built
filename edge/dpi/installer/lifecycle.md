# Path and File Name: /home/ransomeye/rebuild/ransomeye_dpi_probe/installer/lifecycle.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: Service lifecycle management semantics for DPI Probe

# RansomEye DPI Probe - Service Lifecycle

## Service Management

The DPI Probe runs as a systemd service: `ransomeye-dpi-probe.service`

## Start

**Command:**
```bash
sudo systemctl start ransomeye-dpi-probe
```

**Behavior:**
- Service starts and begins packet capture
- Initializes capture engine on configured network interface
- Connects to Core API via mTLS
- Begins sending signed telemetry events

**Prerequisites:**
- Service must be installed (run installer first)
- Network interface must exist and be accessible
- Core API must be reachable (or buffering enabled)
- Certificates must be present and valid

**Exit Codes:**
- `0`: Success
- `1`: Service already running
- Other: Service failed to start (check logs)

## Stop

**Command:**
```bash
sudo systemctl stop ransomeye-dpi-probe
```

**Behavior:**
- Gracefully stops packet capture
- Flushes remaining buffers to disk (if Core unavailable)
- Closes network connections
- Shuts down all worker threads

**Graceful Shutdown:**
- Service handles SIGTERM for graceful shutdown
- Maximum shutdown timeout: 30 seconds
- After timeout, SIGKILL is sent

## Restart

**Command:**
```bash
sudo systemctl restart ransomeye-dpi-probe
```

**Behavior:**
- Equivalent to `stop` followed by `start`
- Useful for applying configuration changes

## Status

**Command:**
```bash
sudo systemctl status ransomeye-dpi-probe
```

**Output Includes:**
- Service state (active/inactive)
- Process ID
- Memory and CPU usage
- Recent log entries

## Enable (Auto-start)

**Command:**
```bash
sudo systemctl enable ransomeye-dpi-probe
```

**Behavior:**
- Service automatically starts on system boot
- Service starts after network.target is available

## Disable (Manual Start Only)

**Command:**
```bash
sudo systemctl disable ransomeye-dpi-probe
```

**Behavior:**
- Service does not start automatically on boot
- Service can still be started manually

## Reload Configuration

**Command:**
```bash
sudo systemctl daemon-reload
```

**When to Use:**
- After modifying service file
- After installing/updating service

**Note:** DPI Probe reads configuration from environment variables and files. Service file changes require `daemon-reload`, but configuration changes may require service restart.

## Logs

**View Logs:**
```bash
# Journal logs (systemd)
sudo journalctl -u ransomeye-dpi-probe -f

# Application logs
tail -f /opt/ransomeye/dpi_probe/logs/*.log
```

**Log Locations:**
- Systemd journal: All service output
- Application logs: `/opt/ransomeye/dpi_probe/logs/` (if configured)

## Health Checks

**Service Health:**
```bash
# Check if service is running
sudo systemctl is-active ransomeye-dpi-probe

# Check if service is enabled
sudo systemctl is-enabled ransomeye-dpi-probe
```

**Expected Output:**
- `active`: Service is running
- `inactive`: Service is stopped
- `enabled`: Service starts on boot
- `disabled`: Service does not start on boot

## Troubleshooting

**Service Won't Start:**
1. Check prerequisites (network interface, certificates)
2. Check logs: `journalctl -u ransomeye-dpi-probe -n 50`
3. Verify installation: `./installer/verify.sh`
4. Check swap: `free -h`
5. Verify kernel version: `uname -r`

**Service Crashes:**
1. Check logs for error messages
2. Verify network interface is accessible
3. Check Core API connectivity
4. Verify sufficient resources (CPU, memory, swap)

**High CPU Usage:**
- Normal for high-throughput packet capture
- Consider CPU affinity tuning
- Check for misconfigured capture filters

## Lifecycle State Machine

```
[Not Installed]
    ↓ (install.sh)
[Installed, Disabled]
    ↓ (systemctl enable)
[Enabled]
    ↓ (systemctl start)
[Active]
    ↓ (systemctl stop)
[Inactive]
    ↓ (systemctl disable)
[Disabled]
    ↓ (uninstall.sh)
[Removed]
```

## Service Dependencies

- **After**: `network.target`
- **Requires**: `network.target`
- **Wants**: None (optional dependencies)

## Restart Policy

- **Restart**: `always`
- **RestartSec**: 10 seconds
- Service automatically restarts on failure

