# Standalone Agents Declaration

**Path:** `/home/ransomeye/rebuild/edge/STANDALONE.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Formal declaration of standalone agent architecture and exclusion from unified installer

---

## Overview

This document formally declares that edge agents under `/home/ransomeye/rebuild/edge/` are **standalone components** with independent lifecycle and are **excluded from the unified installer and systemd model**.

---

## Standalone Agent Classification

### Edge Agents

The following components are classified as **standalone agents**:

1. **Linux Agent** (`edge/agent/linux/` or `ransomeye_linux_agent/`)
   - Standalone Linux agent
   - Independent installer required
   - Independent lifecycle
   - Excluded from unified systemd validation

2. **Windows Agent** (`edge/agent/windows/` or `ransomeye_windows_agent/`)
   - Standalone Windows agent
   - Independent MSI installer required
   - Independent lifecycle
   - Excluded from unified systemd validation

3. **DPI Probe** (`edge/dpi/` or `ransomeye_dpi_probe/`)
   - Standalone DPI probe
   - Independent installer required
   - Independent lifecycle
   - Excluded from unified systemd validation

---

## Exclusion Rules

### Unified Installer Exclusion

**Rule:** Standalone agents are **NOT installed by the main installer**.

- Main installer (`/home/ransomeye/rebuild/install.sh`) does NOT install standalone agents
- Standalone agents require dedicated installers
- Standalone agents have independent installation procedures

### Unified Systemd Exclusion

**Rule:** Standalone agents are **excluded from unified systemd validation**.

- Systemd units under `edge/agent/**` or `edge/dpi/**` are **NOT validated** by unified systemd rules
- Standalone agents may have their own systemd units
- Standalone agents are **NOT required** to be in `/home/ransomeye/rebuild/systemd/`

### Independent Lifecycle

**Rule:** Standalone agents have **independent lifecycle**.

- Standalone agents have their own versioning
- Standalone agents have their own update procedures
- Standalone agents have their own uninstall procedures
- Standalone agents are **NOT managed** by core installer/uninstaller

---

## Validation Rules

### Global Validator Behavior

The Global Forensic Consistency Validator (`core/global_validator/`) treats standalone agents as follows:

1. **Systemd Units:** Systemd units under `edge/agent/**` or `edge/dpi/**` are **excluded** from unified systemd validation
2. **Requirement:** If systemd units exist under `edge/agent/**` or `edge/dpi/**`, this `STANDALONE.md` file **MUST exist**
3. **Fail-Closed:** If edge units exist but `STANDALONE.md` is missing, validation **FAILS**

### Standalone Declaration Requirement

**Rule:** If any systemd unit exists under:
- `edge/agent/**`
- `edge/dpi/**`

Then:
- This `STANDALONE.md` file **MUST exist**
- Validation **FAILS** if edge units exist but declaration is missing

---

## Architecture Rationale

### Why Standalone?

1. **Deployment Flexibility:** Standalone agents can be deployed independently
2. **Platform-Specific:** Windows agents require MSI installers (not compatible with Linux installer)
3. **Lifecycle Independence:** Agents may have different update cadences
4. **Security Boundaries:** Edge agents operate in different trust boundaries

### Why Not Unified?

1. **Platform Differences:** Windows vs Linux require different installers
2. **Deployment Models:** Agents may be deployed to different hosts than core
3. **Update Cycles:** Agents may update independently of core
4. **Trust Boundaries:** Edge agents operate in different security contexts

---

## Status

✅ **STANDALONE DECLARATION ACTIVE**

This document formally declares the standalone architecture. All edge agents are excluded from unified installer and systemd validation.

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech

