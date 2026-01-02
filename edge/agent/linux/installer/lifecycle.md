# Path and File Name: /home/ransomeye/rebuild/ransomeye_linux_agent/installer/lifecycle.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: Service lifecycle management semantics for Linux Agent

# RansomEye Linux Agent - Service Lifecycle

## Service Management

The Linux Agent runs as a systemd service: `ransomeye-linux-agent.service`

## Start

**Command:**
```bash
sudo systemctl start ransomeye-linux-agent
```

**Behavior:**
- Service starts with root privileges (for privilege downgrade)
- Automatically drops to unprivileged user (`ransomeye`) after initialization
- Begins telemetry collection
- Connects to Core API via mTLS
- Begins sending signed telemetry events

**Prerequisites:**
- Service must be installed (run installer first)
- Core API must be reachable (or buffering enabled)
- Certificates must be present and valid
- NO swap enabled (installation validates this)

**Privilege Downgrade:**
- Binary has SetUID bit set
- Starts as root to acquire necessary capabilities
- Immediately drops to `ransomeye` user after startup
- Runs with minimal privileges for security

## Stop

**Command:**
```bash
sudo systemctl stop ransomeye-linux-agent
```

**Behavior:**
- Gracefully stops telemetry collection
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
sudo systemctl restart ransomeye-linux-agent
```

**Behavior:**
- Equivalent to `stop` followed by `start`
- Useful for applying configuration changes

## Status

**Command:**
```bash
sudo systemctl status ransomeye-linux-agent
```

**Output Includes:**
- Service state (active/inactive)
- Process ID (running as `ransomeye` user)
- Memory and CPU usage
- Recent log entries

## Enable (Auto-start)

**Command:**
```bash
sudo systemctl enable ransomeye-linux-agent
```

**Behavior:**
- Service automatically starts on system boot
- Service starts after network.target is available

## Disable (Manual Start Only)

**Command:**
```bash
sudo systemctl disable ransomeye-linux-agent
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

**Note:** Linux Agent reads configuration from environment variables and files. Service file changes require `daemon-reload`, but configuration changes may require service restart.

## Logs

**View Logs:**
```bash
# Journal logs (systemd)
sudo journalctl -u ransomeye-linux-agent -f

# Application logs
tail -f /opt/ransomeye/linux_agent/logs/*.log
```

**Log Locations:**
- Systemd journal: All service output
- Application logs: `/opt/ransomeye/linux_agent/logs/` (if configured)

## Health Checks

**Service Health:**
```bash
# Check if service is running
sudo systemctl is-active ransomeye-linux-agent

# Check if service is enabled
sudo systemctl is-enabled ransomeye-linux-agent

# Check process user (should be 'ransomeye')
ps aux | grep ransomeye_linux_agent
```

**Expected Output:**
- `active`: Service is running
- `inactive`: Service is stopped
- `enabled`: Service starts on boot
- `disabled`: Service does not start on boot
- Process user: `ransomeye` (after privilege downgrade)

## Troubleshooting

**Service Won't Start:**
1. Check prerequisites (certificates, Core API connectivity)
2. Check logs: `journalctl -u ransomeye-linux-agent -n 50`
3. Verify installation: `./installer/verify.sh`
4. Check for swap: `free -h` (should show 0 swap)
5. Verify user exists: `id ransomeye`

**Service Crashes:**
1. Check logs for error messages
2. Verify Core API connectivity
3. Check certificates are valid and accessible
4. Verify sufficient RAM (agent must not use swap)

**High Memory Usage:**
- Normal for active telemetry collection
- Check buffer size configuration
- Verify swap is disabled (agent must not swap)

## Lifecycle State Machine

```
[Not Installed]
    ↓ (install.sh)
[Installed, Disabled]
    ↓ (systemctl enable)
[Enabled]
    ↓ (systemctl start)
[Active (root)]
    ↓ (privilege downgrade)
[Active (ransomeye user)]
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

## Privilege Model

- **Startup**: Root (via SetUID binary)
- **Runtime**: Unprivileged user (`ransomeye`)
- **Capabilities**: Minimal (only what's necessary)
- **No Kernel Drivers**: Runs entirely in user space

