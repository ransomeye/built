# Swap Requirements

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_architecture/swap_policy/swap_requirements.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Swap space requirements for Core Engine and DPI Probe - mandatory enforcement

---

## Overview

Swap space is **mandatory** for Core Engine and DPI Probe. Missing swap results in startup failure. Linux Agent and Windows Agent are explicitly forbidden from requiring swap.

---

## Components Requiring Swap

### 1. Core Engine

**Location:** `/home/ransomeye/rebuild/ransomeye_master_core/`

**Requirement:**
- Minimum swap: max(16GB, RAM size)
- Created during installation
- Verified at startup
- Missing swap → FAIL STARTUP

**Rationale:**
- High memory usage
- Critical system component
- Must handle memory pressure
- Must not fail due to OOM

---

### 2. DPI Probe

**Location:** `/home/ransomeye/rebuild/ransomeye_dpi_probe/`

**Requirement:**
- Minimum swap: max(16GB, RAM size)
- Created during installation
- Verified at startup
- Missing swap → FAIL STARTUP

**Rationale:**
- High throughput processing
- Memory-intensive operations
- Must handle traffic spikes
- Must not fail due to OOM

---

## Components Forbidden from Swap

### 1. Linux Agent

**Location:** `/home/ransomeye/rebuild/ransomeye_linux_agent/`

**Requirement:**
- NO swap required
- NO swap check
- NO swap enforcement
- Lightweight operation

**Rationale:**
- Minimal memory footprint
- Host-based operation
- Must not impact host system
- Must be lightweight

---

### 2. Windows Agent

**Location:** `/home/ransomeye/rebuild/ransomeye_windows_agent/`

**Requirement:**
- NO swap required
- NO swap check
- NO swap enforcement
- Lightweight operation

**Rationale:**
- Minimal memory footprint
- Host-based operation
- Must not impact host system
- Must be lightweight

---

## Swap Calculation

### Formula

```
required_swap = max(16GB, RAM_size)
```

### Examples

- RAM = 8GB → Swap = 16GB (minimum)
- RAM = 32GB → Swap = 32GB (equal to RAM)
- RAM = 64GB → Swap = 64GB (equal to RAM)

---

## Swap Creation

### During Installation

**Process:**
1. Detect RAM size
2. Calculate required swap
3. Create swap file/partition
4. Enable swap
5. Verify swap active

### Swap File Creation

**Method:**
```bash
# Create swap file
dd if=/dev/zero of=/swapfile bs=1G count=<size_in_gb>

# Set permissions
chmod 600 /swapfile

# Format as swap
mkswap /swapfile

# Enable swap
swapon /swapfile

# Add to fstab
echo '/swapfile none swap sw 0 0' >> /etc/fstab
```

---

## Swap Verification

### At Startup

**Process:**
1. Check swap exists
2. Check swap active
3. Check swap size
4. Verify swap >= required

**Failure:** FAIL STARTUP, log error, human notification

### Verification Code

**Location:** `/home/ransomeye/rebuild/ransomeye_installer/system/swap_check.py`

**Function:** `check_swap()`

**Returns:**
- `meets_requirements`: bool
- `message`: str
- `info`: dict

---

## Enforcement Points

### Point 1: Installation

**Location:** Installer

**Enforcement:**
- Check swap during installation
- Create swap if missing
- Verify swap active
- Fail installation if swap cannot be created

### Point 2: Startup

**Location:** Core Engine, DPI Probe

**Enforcement:**
- Check swap at startup
- Verify swap >= required
- Fail startup if swap missing
- Log error if swap insufficient

### Point 3: Runtime

**Location:** Core Engine, DPI Probe

**Enforcement:**
- Monitor swap usage
- Alert if swap low
- Prevent OOM conditions
- Log swap events

---

## Failure Modes

### Mode 1: Swap Missing

**Detection:** Swap not found at startup

**Response:**
- FAIL STARTUP
- Log error
- Human notification
- Installation required

### Mode 2: Swap Insufficient

**Detection:** Swap < required size

**Response:**
- FAIL STARTUP
- Log error
- Human notification
- Swap expansion required

### Mode 3: Swap Inactive

**Detection:** Swap not active

**Response:**
- FAIL STARTUP
- Log error
- Human notification
- Swap activation required

---

## Last Updated

Phase 2 Implementation

