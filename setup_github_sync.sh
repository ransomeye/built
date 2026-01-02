#!/bin/bash
# Path and File Name : /home/ransomeye/rebuild/setup_github_sync.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Setup script to configure GitHub remote and enable continuous sync

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}RansomEye GitHub Sync Setup${NC}"
echo "============================================"

# Check if running from correct directory
if [ ! -d ".git" ]; then
    echo -e "${RED}Error: Must be run from the git repository root${NC}"
    exit 1
fi

# Get GitHub repository URL
if [ -z "$1" ]; then
    echo -e "${YELLOW}Usage: $0 <github-repo-url>${NC}"
    echo "Example: $0 https://github.com/username/ransomeye.git"
    echo "     or: $0 git@github.com:username/ransomeye.git"
    exit 1
fi

GITHUB_URL="$1"

echo -e "${GREEN}Setting up GitHub remote...${NC}"

# Remove existing remote if it exists
if git remote | grep -q "^origin$"; then
    echo "Removing existing origin remote..."
    git remote remove origin
fi

# Add new remote
git remote add origin "$GITHUB_URL"
echo -e "${GREEN}✓ Remote 'origin' added${NC}"

# Verify remote
echo "Remote URL: $(git remote get-url origin)"

# Push to GitHub
echo -e "${GREEN}Pushing to GitHub...${NC}"
if git push -u origin main; then
    echo -e "${GREEN}✓ Successfully pushed to GitHub${NC}"
else
    echo -e "${RED}Error: Failed to push to GitHub${NC}"
    echo "Please check:"
    echo "  1. GitHub repository exists"
    echo "  2. You have push permissions"
    echo "  3. SSH keys are configured (for git@ URLs)"
    exit 1
fi

# Install systemd service and timer
echo -e "${GREEN}Setting up continuous sync (systemd)...${NC}"

sudo cp systemd/ransomeye-git-sync.service /etc/systemd/system/
sudo cp systemd/ransomeye-git-sync.timer /etc/systemd/system/

sudo systemctl daemon-reload
sudo systemctl enable ransomeye-git-sync.timer
sudo systemctl start ransomeye-git-sync.timer

echo -e "${GREEN}✓ Continuous sync enabled${NC}"
echo ""
echo "============================================"
echo -e "${GREEN}Setup Complete!${NC}"
echo ""
echo "Your repository will now automatically sync to GitHub every hour."
echo ""
echo "Useful commands:"
echo "  - Check sync status:     systemctl status ransomeye-git-sync.timer"
echo "  - View sync logs:        journalctl -u ransomeye-git-sync.service -f"
echo "  - Manual sync now:       systemctl start ransomeye-git-sync.service"
echo "  - Disable auto-sync:     sudo systemctl stop ransomeye-git-sync.timer"
echo ""

