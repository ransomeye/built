# RansomEye Linux Agent (STANDALONE)

**Path and File Name:** `/home/ransomeye/rebuild/edge/agent/linux/docs/README.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Documentation for RansomEye Linux Agent - STANDALONE module

---

## ⚠️ CRITICAL: STANDALONE MODULE

**This is a STANDALONE module - NOT part of Core.**

- **Independent installer:** `/home/ransomeye/rebuild/edge/agent/linux/installer/install.sh`
- **Independent uninstaller:** `/home/ransomeye/rebuild/edge/agent/linux/installer/uninstall.sh`
- **Independent systemd unit:** `/home/ransomeye/rebuild/edge/agent/linux/systemd/ransomeye-linux-agent.service`
- **Independent runtime:** `/opt/ransomeye-linux-agent` (NOT `/opt/ransomeye`)
- **Independent user:** `ransomeye-agent` (NOT `ransomeye`)

**Core does NOT install, manage, or own this agent.**

---

## Architecture

### Directory Structure

```
/home/ransomeye/rebuild/edge/agent/linux/
├── systemd/
│   └── ransomeye-linux-agent.service    # Standalone systemd unit
├── installer/
│   ├── install.sh                       # Standalone installer
│   └── uninstall.sh                     # Standalone uninstaller
├── config/
│   └── linux-agent.env                   # Environment template
└── docs/
    └── README.md                         # This file
```

### Runtime Layout

```
/opt/ransomeye-linux-agent/               # Standalone runtime (NOT /opt/ransomeye)
├── bin/
│   └── ransomeye_linux_agent             # Agent binary
├── config/
│   └── environment.conf                 # Runtime config
└── lib/                                  # Agent libraries

/etc/ransomeye-linux-agent/               # Standalone config (NOT /etc/ransomeye)
└── linux-agent.env                       # Environment file

/var/lib/ransomeye-linux-agent/           # Standalone state (NOT /var/lib/ransomeye)
└── buffer/                               # Telemetry buffer
```

---

## Installation

### Prerequisites

- Linux (any modern distribution)
- Root/sudo privileges
- systemd for service management

### Install

1. **Build the binary:**
```bash
cd /home/ransomeye/rebuild/edge/agent/linux
cargo build --release
```

2. **Run the installer:**
```bash
sudo /home/ransomeye/rebuild/edge/agent/linux/installer/install.sh
```

The installer will:
- Create `ransomeye-agent` system user (nologin)
- Create `/opt/ransomeye-linux-agent` runtime directory
- Install agent binary to `/opt/ransomeye-linux-agent/bin/`
- Create environment file at `/etc/ransomeye-linux-agent/linux-agent.env`
- Install systemd unit to `/etc/systemd/system/ransomeye-linux-agent.service`
- Enable (but NOT start) the service

### Service Management

**Start service:**
```bash
sudo systemctl start ransomeye-linux-agent.service
```

**Stop service:**
```bash
sudo systemctl stop ransomeye-linux-agent.service
```

**Check status:**
```bash
sudo systemctl status ransomeye-linux-agent.service
```

**View logs:**
```bash
sudo journalctl -u ransomeye-linux-agent.service -f
```

---

## Uninstallation

```bash
sudo /home/ransomeye/rebuild/edge/agent/linux/installer/uninstall.sh
```

The uninstaller will:
- Stop and disable the service
- Remove systemd unit
- Remove runtime directory (`/opt/ransomeye-linux-agent`)
- Remove environment file (`/etc/ransomeye-linux-agent/linux-agent.env`)
- Optionally remove state directories
- Optionally remove `ransomeye-agent` system user

**Note:** The uninstaller does NOT touch Core files.

---

## Configuration

Edit the environment file:
```bash
sudo nano /etc/ransomeye-linux-agent/linux-agent.env
```

Key variables:
- `CORE_API_URL`: Core API endpoint
- `AGENT_ID`: Agent identifier (default: hostname)
- `AGENT_BUFFER_DIR`: Buffer directory for telemetry
- `ENABLE_EBPF`: Enable eBPF features
- `AGENT_CERT_PATH`: mTLS certificate path

After editing, restart the service:
```bash
sudo systemctl restart ransomeye-linux-agent.service
```

---

## Isolation Guarantees

### No Core Dependencies

- Agent does NOT reference Core binaries
- Agent does NOT use Core environment files
- Agent does NOT use Core runtime paths
- Agent does NOT use Core system user

### Standalone User

- Runs as `ransomeye-agent` (NOT `ransomeye`)
- Independent user account
- Independent group membership

### Standalone Paths

- Runtime: `/opt/ransomeye-linux-agent` (NOT `/opt/ransomeye`)
- Config: `/etc/ransomeye-linux-agent` (NOT `/etc/ransomeye`)
- State: `/var/lib/ransomeye-linux-agent` (NOT `/var/lib/ransomeye`)

---

## Security

- **Rootless execution:** Runs as `ransomeye-agent` user
- **Minimal capabilities:** Only required capabilities granted
- **Hardened systemd:** Strict security settings
- **No privilege escalation:** NoNewPrivileges=true

---

*Generated: 2025-12-30*  
*Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU*

