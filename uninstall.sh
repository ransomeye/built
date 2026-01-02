#!/bin/bash
# Path and File Name: /home/ransomeye/rebuild/uninstall.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: Root-level uninstallation entrypoint - idempotent, clean removal in reverse dependency order

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR"
LOG_FILE="/var/log/ransomeye/uninstall.log"

# Default flags
PRESERVE_LOGS=false
PRESERVE_EVIDENCE=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --preserve-logs)
            PRESERVE_LOGS=true
            shift
            ;;
        --preserve-evidence)
            PRESERVE_EVIDENCE=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--preserve-logs] [--preserve-evidence]"
            exit 1
            ;;
    esac
done

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE" 2>/dev/null || echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

error() {
    echo -e "${RED}ERROR: $1${NC}" | tee -a "$LOG_FILE" >&2
    exit 1
}

success() {
    echo -e "${GREEN}✓ $1${NC}" | tee -a "$LOG_FILE"
}

warning() {
    echo -e "${YELLOW}⚠ $1${NC}" | tee -a "$LOG_FILE"
}

# Ensure log directory exists
mkdir -p "$(dirname "$LOG_FILE")" 2>/dev/null || true

log "Starting RansomEye uninstallation"

# Check root (but allow idempotent runs even if not root for some checks)
if [[ $EUID -ne 0 ]]; then
    warning "Not running as root. Some operations may fail."
    warning "For complete uninstallation, run: sudo ./uninstall.sh"
fi

# Confirmation
echo ""
echo "==========================================================================="
echo "RANSOMEYE UNINSTALLATION"
echo "==========================================================================="
echo ""
echo "This will uninstall RansomEye and all installed components."
echo ""

if [[ "$PRESERVE_LOGS" == "true" ]]; then
    echo "Logs will be preserved."
fi

if [[ "$PRESERVE_EVIDENCE" == "true" ]]; then
    echo "Evidence will be preserved."
fi

echo ""

read -p "Are you sure you want to proceed? (yes/no): " confirm
if [[ "$confirm" != "yes" ]]; then
    log "Uninstallation cancelled by user"
    exit 0
fi

# ============================================================================
# 1. STOP ALL SERVICES (IDEMPOTENT)
# ============================================================================
log "Stopping all RansomEye services"

if command -v systemctl &> /dev/null && [[ $EUID -eq 0 ]]; then
    # Explicitly handle orchestrator service first (fail-closed service)
    if systemctl list-units --type=service --all --no-legend 2>/dev/null | grep -q "ransomeye-orchestrator.service"; then
        log "Stopping orchestrator service (fail-closed service)"
        if systemctl is-active --quiet ransomeye-orchestrator.service 2>/dev/null; then
            systemctl stop ransomeye-orchestrator.service 2>/dev/null || warning "Failed to stop orchestrator service"
        fi
        
        if systemctl is-enabled --quiet ransomeye-orchestrator.service 2>/dev/null; then
            log "Disabling orchestrator service"
            systemctl disable ransomeye-orchestrator.service 2>/dev/null || warning "Failed to disable orchestrator service"
        fi
        success "Orchestrator service stopped and disabled"
    fi
    
    # Find all other RansomEye services
    SERVICES=$(systemctl list-units --type=service --all --no-legend 2>/dev/null | grep -i ransomeye | grep -v "ransomeye-orchestrator.service" | awk '{print $1}' || true)
    
    if [[ -n "$SERVICES" ]]; then
        for service in $SERVICES; do
            if systemctl is-active --quiet "$service" 2>/dev/null; then
                log "Stopping service: $service"
                systemctl stop "$service" 2>/dev/null || warning "Failed to stop $service"
            fi
            
            if systemctl is-enabled --quiet "$service" 2>/dev/null; then
                log "Disabling service: $service"
                systemctl disable "$service" 2>/dev/null || warning "Failed to disable $service"
            fi
        done
        success "All services stopped and disabled"
    else
        log "No other active RansomEye services found"
    fi
else
    warning "Cannot stop services (systemctl not available or not root)"
fi

# ============================================================================
# 2. UNINSTALL STANDALONE MODULES (REVERSE DEPENDENCY ORDER)
# ============================================================================
log "Uninstalling standalone modules"

# DPI Probe
if [[ -f "$PROJECT_ROOT/ransomeye_dpi_probe/installer/uninstall.sh" ]] && [[ $EUID -eq 0 ]]; then
    if [[ -f "/opt/ransomeye/dpi_probe/.install_receipt.json" ]]; then
        log "Uninstalling DPI Probe"
        if bash "$PROJECT_ROOT/ransomeye_dpi_probe/installer/uninstall.sh" 2>&1 | tee -a "$LOG_FILE"; then
            success "DPI Probe uninstalled"
        else
            warning "DPI Probe uninstallation had issues"
        fi
    else
        log "DPI Probe not installed (no receipt found)"
    fi
fi

# Linux Agent
if [[ -f "$PROJECT_ROOT/ransomeye_linux_agent/installer/uninstall.sh" ]] && [[ $EUID -eq 0 ]]; then
    if [[ -f "/opt/ransomeye/linux_agent/.install_receipt.json" ]]; then
        log "Uninstalling Linux Agent"
        if bash "$PROJECT_ROOT/ransomeye_linux_agent/installer/uninstall.sh" 2>&1 | tee -a "$LOG_FILE"; then
            success "Linux Agent uninstalled"
        else
            warning "Linux Agent uninstallation had issues"
        fi
    else
        log "Linux Agent not installed (no receipt found)"
    fi
fi

# ============================================================================
# 3. UNINSTALL CORE STACK
# ============================================================================
log "Uninstalling core stack"

# Remove systemd service files
if [[ -d "/etc/systemd/system" ]] && [[ $EUID -eq 0 ]]; then
    # Explicitly remove orchestrator service file
    ORCHESTRATOR_SERVICE="/etc/systemd/system/ransomeye-orchestrator.service"
    if [[ -f "$ORCHESTRATOR_SERVICE" ]]; then
        log "Removing orchestrator systemd service: ransomeye-orchestrator.service"
        rm -f "$ORCHESTRATOR_SERVICE"
        success "Orchestrator service file removed"
    fi
    
    # Remove all other RansomEye service files
    SYSTEMD_SERVICES=$(find /etc/systemd/system -name "ransomeye-*.service" -type f 2>/dev/null || true)
    
    if [[ -n "$SYSTEMD_SERVICES" ]]; then
        for service_file in $SYSTEMD_SERVICES; do
            SERVICE_NAME=$(basename "$service_file")
            log "Removing systemd service: $SERVICE_NAME"
            rm -f "$service_file"
        done
    fi
    
    # Reload systemd after removal
    if command -v systemctl &> /dev/null; then
        systemctl daemon-reload 2>/dev/null || true
        success "Systemd daemon reloaded"
    fi
    
    if [[ -n "$SYSTEMD_SERVICES" ]] || [[ -f "$ORCHESTRATOR_SERVICE" ]]; then
        success "Systemd services removed"
    else
        log "No systemd service files found"
    fi
fi

# Remove service files from project systemd directory (if they exist)
if [[ -d "$PROJECT_ROOT/systemd" ]] && [[ $EUID -eq 0 ]]; then
    # Don't remove the directory, just log that it exists
    log "Service definitions preserved in: $PROJECT_ROOT/systemd/"
fi

# Remove install state
INSTALL_STATE="$PROJECT_ROOT/ransomeye_installer/config/install_state.json"
if [[ -f "$INSTALL_STATE" ]] && [[ $EUID -eq 0 ]]; then
    log "Removing install state"
    rm -f "$INSTALL_STATE"
    success "Install state removed"
fi

# ============================================================================
# 4. CLEANUP (WITH PRESERVATION FLAGS)
# ============================================================================
log "Cleaning up installation artifacts"

# Log preservation
if [[ "$PRESERVE_LOGS" == "false" ]] && [[ $EUID -eq 0 ]]; then
    LOG_DIRS=(
        "/var/log/ransomeye"
        "$PROJECT_ROOT/logs"
    )
    
    for log_dir in "${LOG_DIRS[@]}"; do
        if [[ -d "$log_dir" ]]; then
            log "Removing logs: $log_dir"
            rm -rf "$log_dir"
        fi
    done
    success "Logs removed"
else
    log "Logs preserved (--preserve-logs flag or not root)"
fi

# Evidence preservation
if [[ "$PRESERVE_EVIDENCE" == "false" ]] && [[ $EUID -eq 0 ]]; then
    EVIDENCE_DIRS=(
        "/var/lib/ransomeye"
        "$PROJECT_ROOT/evidence"
    )
    
    for evidence_dir in "${EVIDENCE_DIRS[@]}"; do
        if [[ -d "$evidence_dir" ]]; then
            log "Removing evidence: $evidence_dir"
            rm -rf "$evidence_dir"
        fi
    done
    success "Evidence removed"
else
    log "Evidence preserved (--preserve-evidence flag or not root)"
fi

# ============================================================================
# 5. VERIFY NO ORPHANED FILES
# ============================================================================
log "Verifying no orphaned files"

if [[ $EUID -eq 0 ]]; then
    # Remove orchestrator binary explicitly
    ORCHESTRATOR_BINARY="/opt/ransomeye/bin/ransomeye_orchestrator"
    if [[ -f "$ORCHESTRATOR_BINARY" ]]; then
        log "Removing orchestrator binary: $ORCHESTRATOR_BINARY"
        rm -f "$ORCHESTRATOR_BINARY"
        success "Orchestrator binary removed"
    fi
    
    # Check for orphaned binaries in standard locations
    ORPHANED_PATHS=(
        "/usr/local/bin/ransomeye*"
        "/usr/bin/ransomeye*"
        "/opt/ransomeye"
    )
    
    ORPHANS_FOUND=false
    for path_pattern in "${ORPHANED_PATHS[@]}"; do
        if ls $path_pattern 2>/dev/null | grep -q .; then
            ORPHANS_FOUND=true
            warning "Potential orphaned files found: $path_pattern"
        fi
    done
    
    if [[ "$ORPHANS_FOUND" == "false" ]]; then
        success "No orphaned files detected"
    fi
fi

# ============================================================================
# 6. COMPLETION
# ============================================================================
log "Uninstallation completed"

echo ""
echo "==========================================================================="
echo "UNINSTALLATION COMPLETE"
echo "==========================================================================="
echo ""
if [[ "$PRESERVE_LOGS" == "true" ]]; then
    echo "Logs preserved at: /var/log/ransomeye/"
fi
if [[ "$PRESERVE_EVIDENCE" == "true" ]]; then
    echo "Evidence preserved at: /var/lib/ransomeye/"
fi
echo ""
echo "Uninstallation log: $LOG_FILE"
echo ""
echo "Note: Project directory ($PROJECT_ROOT) was not removed."
echo "      To remove it completely, delete it manually."
echo "==========================================================================="
echo ""

success "Uninstallation completed successfully"

exit 0

