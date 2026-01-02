# Installing Automatic Feed Fetcher

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/ingestion/INSTALL_AUTO_FETCHER.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Installation instructions for automatic threat intelligence feed fetcher

## Overview

The automatic feed fetcher runs daily at 2 AM and retries every hour if the previous fetch failed. It checks for internet connectivity before attempting fetches.

## Installation Steps

### 1. Create Required Directories

```bash
sudo mkdir -p /var/log/ransomeye /var/lib/ransomeye
sudo chown -R ransomeye:ransomeye /var/log/ransomeye /var/lib/ransomeye
```

### 2. Install Systemd Service and Timer

```bash
# Copy service file
sudo cp /home/ransomeye/rebuild/systemd/ransomeye-feed-fetcher.service /etc/systemd/system/

# Copy timer file
sudo cp /home/ransomeye/rebuild/systemd/ransomeye-feed-fetcher.timer /etc/systemd/system/

# Reload systemd
sudo systemctl daemon-reload

# Enable and start timer
sudo systemctl enable ransomeye-feed-fetcher.timer
sudo systemctl start ransomeye-feed-fetcher.timer
```

### 3. Verify Installation

```bash
# Check timer status
sudo systemctl status ransomeye-feed-fetcher.timer

# Check service status
sudo systemctl status ransomeye-feed-fetcher.service

# List all timers
sudo systemctl list-timers ransomeye-feed-fetcher.timer

# View logs
sudo journalctl -u ransomeye-feed-fetcher.service -f
```

### 4. Manual Test

```bash
# Run fetcher manually
sudo systemctl start ransomeye-feed-fetcher.service

# Check logs
sudo journalctl -u ransomeye-feed-fetcher.service --since "5 minutes ago"
```

## Configuration

### Environment Variables

The service uses these environment variables (set in service file):
- `MALWARBAZAAR_AUTH_KEY`: MalwareBazaar API key
- `RANSOMWARE_LIVE_API_KEY`: Ransomware.live API key
- `OTX_KEY`: (Optional) AlienVault OTX API key
- `VIRUSTOTAL_KEY`: (Optional) VirusTotal API key

### Timer Schedule

- **Daily:** Runs at 2:00 AM
- **Hourly Retries:** Runs every hour if previous fetch failed
- **Randomized Delay:** Up to 5 minutes to avoid thundering herd

### Retry Logic

- Maximum retries: 24 (once per hour for 24 hours)
- Retry interval: 1 hour
- Internet check: Verifies connectivity before attempting fetch

## Additional Feed Sources

The fetcher automatically includes additional sources if available:

1. **URLhaus** (Abuse.ch) - No API key required
2. **ThreatFox** (Abuse.ch) - No API key required
3. **AlienVault OTX** - Requires `OTX_KEY` environment variable
4. **VirusTotal** - Requires `VIRUSTOTAL_KEY` environment variable

To enable optional sources, add to service file:

```ini
Environment="OTX_KEY=your-otx-key"
Environment="VIRUSTOTAL_KEY=your-vt-key"
```

Then reload:

```bash
sudo systemctl daemon-reload
sudo systemctl restart ransomeye-feed-fetcher.timer
```

## Monitoring

### Check Last Successful Fetch

```bash
cat /var/lib/ransomeye/feed_fetcher_state.json
```

### View Logs

```bash
# Recent logs
sudo journalctl -u ransomeye-feed-fetcher.service --since "1 hour ago"

# Follow logs
sudo journalctl -u ransomeye-feed-fetcher.service -f

# Log file
tail -f /var/log/ransomeye/feed_fetcher.log
```

### Check Feed Cache

```bash
# List cached feeds
ls -lh /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/cache/*/

# Count cached items
find /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/cache -name "*.json" | wc -l
```

## Troubleshooting

### Service Not Running

```bash
# Check service status
sudo systemctl status ransomeye-feed-fetcher.service

# Check for errors
sudo journalctl -u ransomeye-feed-fetcher.service --since "1 hour ago" | grep -i error
```

### No Internet Connectivity

The fetcher checks for internet before attempting fetches. If no internet is available, it will:
1. Skip the fetch
2. Increment retry count
3. Retry in 1 hour

### Feed Fetch Failures

Check individual feed errors in logs:

```bash
sudo journalctl -u ransomeye-feed-fetcher.service | grep -E "(✗|Error|Failed)"
```

## Uninstallation

```bash
# Stop and disable timer
sudo systemctl stop ransomeye-feed-fetcher.timer
sudo systemctl disable ransomeye-feed-fetcher.timer

# Remove service files
sudo rm /etc/systemd/system/ransomeye-feed-fetcher.service
sudo rm /etc/systemd/system/ransomeye-feed-fetcher.timer

# Reload systemd
sudo systemctl daemon-reload
```

