# RansomEye GitHub Sync Setup

## Quick Start

### Step 1: Create GitHub Repository

1. Go to [GitHub](https://github.com) and sign in
2. Click the **+** icon (top right) → **New repository**
3. Name it (e.g., `ransomeye` or `ransomeye-enterprise`)
4. Choose **Private** (recommended for security)
5. **Do NOT initialize** with README, .gitignore, or license
6. Click **Create repository**
7. Copy the repository URL (you'll see it on the next page)

### Step 2: Run Setup Script

```bash
cd /home/ransomeye/rebuild
./setup_github_sync.sh <your-github-repo-url>
```

**Examples:**
- HTTPS: `./setup_github_sync.sh https://github.com/yourusername/ransomeye.git`
- SSH: `./setup_github_sync.sh git@github.com:yourusername/ransomeye.git`

### Step 3: Authentication

#### For HTTPS URLs:
You'll be prompted for your GitHub username and password/token.
- **Recommended:** Use a Personal Access Token instead of password
- Generate token: GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
- Required scopes: `repo` (full control of private repositories)

#### For SSH URLs:
Ensure SSH keys are configured:
```bash
# Check if SSH key exists
ls -la ~/.ssh/id_rsa.pub

# If not, generate one
ssh-keygen -t rsa -b 4096 -C "Gagan@RansomEye.Tech"

# Copy public key and add to GitHub (Settings → SSH and GPG keys)
cat ~/.ssh/id_rsa.pub
```

## What Gets Synced

The setup creates:
1. **Systemd Timer** - Automatically syncs every hour
2. **Git Tracking** - All files except those in `.gitignore`
3. **Auto-commits** - Changes are auto-committed with timestamp

## Managing Sync

### Check Sync Status
```bash
systemctl status ransomeye-git-sync.timer
```

### View Sync Logs
```bash
journalctl -u ransomeye-git-sync.service -f
```

### Manual Sync Now
```bash
sudo systemctl start ransomeye-git-sync.service
```

### Disable Auto-Sync
```bash
sudo systemctl stop ransomeye-git-sync.timer
sudo systemctl disable ransomeye-git-sync.timer
```

### Re-enable Auto-Sync
```bash
sudo systemctl start ransomeye-git-sync.timer
sudo systemctl enable ransomeye-git-sync.timer
```

## Manual Operations

### Push Changes Manually
```bash
cd /home/ransomeye/rebuild
git add .
git commit -m "Your commit message"
git push origin main
```

### Pull Latest Changes
```bash
cd /home/ransomeye/rebuild
git pull origin main
```

### View Commit History
```bash
cd /home/ransomeye/rebuild
git log --oneline -10
```

### Check Repository Status
```bash
cd /home/ransomeye/rebuild
git status
```

## What's Excluded (Not Synced)

Check `.gitignore` for full list. Key exclusions:
- Virtual environments (`venv/`, `.venv/`)
- Python cache (`__pycache__/`, `*.pyc`)
- Logs (`logs/*.log`)
- Environment files (`.env`)
- Certificates and keys (`*.key`, `*.pem`, `*.crt`)
- Compiled binaries
- Database files
- Temporary files

## Troubleshooting

### Authentication Failed
- HTTPS: Generate and use Personal Access Token
- SSH: Ensure SSH key is added to GitHub account

### Permission Denied
```bash
# Check remote URL
git remote -v

# Update to correct URL if needed
git remote set-url origin <correct-url>
```

### Sync Service Not Running
```bash
# Check service status
systemctl status ransomeye-git-sync.service
systemctl status ransomeye-git-sync.timer

# View detailed logs
journalctl -xe -u ransomeye-git-sync.service
```

### Merge Conflicts
```bash
# If auto-sync fails due to conflicts
cd /home/ransomeye/rebuild
git pull origin main
# Resolve conflicts manually
git add .
git commit -m "Resolved conflicts"
git push origin main
```

## Security Recommendations

1. **Use Private Repository** - Keep RansomEye source code private
2. **SSH Authentication** - More secure than HTTPS with tokens
3. **Restrict Access** - Limit GitHub repository collaborators
4. **Review .gitignore** - Ensure no secrets are committed
5. **Enable 2FA** - On your GitHub account

## Support

For issues or questions:
- Email: Gagan@RansomEye.Tech
- Check logs: `journalctl -u ransomeye-git-sync.service`

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech

