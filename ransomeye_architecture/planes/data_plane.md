# Data Plane Architecture

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_architecture/planes/data_plane.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Data Plane definition - untrusted, high-volume, non-authoritative components

---

## Overview

The Data Plane is the **untrusted boundary** of RansomEye. It handles high-volume, potentially hostile data from network and host sources.

---

## Components

### 1. DPI Probe (Stand-alone)

**Location:** `/home/ransomeye/rebuild/ransomeye_dpi_probe/`

**Function:**
- High-throughput packet capture
- ML-based flow classification
- Network traffic analysis

**Properties:**
- Stand-alone (no dependencies on other planes)
- High volume (10Gbps+ sustained)
- Potentially hostile input (network traffic)
- Never authoritative
- Never long-term stateful
- Never enforces policy
- Never invokes AI

**Swap Policy:** REQUIRED (max(16GB, RAM))

**Identity:** Unique per-instance keypair

**Data Flow:**
- Input: Network packets (untrusted)
- Output: Signed telemetry → Control Plane only

---

### 2. Linux Agent (Stand-alone)

**Location:** `/home/ransomeye/rebuild/ransomeye_linux_agent/`

**Function:**
- Host telemetry collection
- Process monitoring
- File system monitoring
- System call interception

**Properties:**
- Stand-alone (no dependencies on other planes)
- High volume (thousands of events/second)
- Potentially hostile input (compromised hosts)
- Never authoritative
- Never long-term stateful
- Never enforces policy
- Never invokes AI

**Swap Policy:** FORBIDDEN (no swap required)

**Identity:** Unique per-instance keypair

**Data Flow:**
- Input: Host events (untrusted)
- Output: Signed telemetry → Control Plane only

---

### 3. Windows Agent (Stand-alone)

**Location:** `/home/ransomeye/rebuild/ransomeye_windows_agent/`

**Function:**
- Host telemetry collection (ETW)
- Process monitoring
- Registry monitoring
- Kernel driver (optional)

**Properties:**
- Stand-alone (no dependencies on other planes)
- High volume (thousands of events/second)
- Potentially hostile input (compromised hosts)
- Never authoritative
- Never long-term stateful
- Never enforces policy
- Never invokes AI

**Swap Policy:** FORBIDDEN (no swap required)

**Identity:** Unique per-instance keypair

**Data Flow:**
- Input: Host events (untrusted)
- Output: Signed telemetry → Control Plane only

---

## Trust Properties

### Untrusted Input

All Data Plane components receive **untrusted input**:
- Network packets may be malicious
- Host events may be from compromised systems
- No assumption of benign behavior

### Non-Authoritative

Data Plane components are **never authoritative**:
- Cannot make policy decisions
- Cannot authorize enforcement
- Cannot modify system state
- Cannot access long-term storage

### Stateless

Data Plane components are **stateless**:
- No persistent state between restarts
- No long-term memory
- All state in Control Plane

### No Policy Enforcement

Data Plane components **never enforce policy**:
- Cannot block traffic
- Cannot terminate processes
- Cannot modify files
- Enforcement is Control Plane only

### No AI Invocation

Data Plane components **never invoke AI**:
- Cannot call ML models
- Cannot use LLM
- Cannot request AI analysis
- AI is Intelligence Plane only

---

## Allowed Operations

1. **Data Collection**
   - Capture packets
   - Collect host events
   - Monitor system calls

2. **Data Signing**
   - Sign all telemetry with component identity
   - Include timestamp and sequence number
   - Include integrity hash

3. **Data Transmission**
   - Send signed telemetry to Control Plane
   - Use authenticated channels (mTLS)
   - Include nonce for replay protection

---

## Forbidden Operations

1. **Policy Decisions**
   - Cannot decide what to block
   - Cannot decide what to allow
   - Cannot modify enforcement rules

2. **State Modification**
   - Cannot modify database
   - Cannot modify configuration
   - Cannot modify other components

3. **AI Invocation**
   - Cannot call ML models
   - Cannot use LLM
   - Cannot request AI analysis

4. **Direct Enforcement**
   - Cannot block traffic
   - Cannot terminate processes
   - Cannot modify files

---

## Failure Modes

### Compromise Detection

If Data Plane component is compromised:
- Control Plane must detect via signature verification
- Communication must be terminated
- Component must be revoked
- Audit log must be generated

### Resource Exhaustion

If Data Plane component exhausts resources:
- Component must fail gracefully
- No impact on other planes
- State must be recoverable

---

## Enforcement Points

All Data Plane operations are enforced at:
1. **Identity Verification** - Control Plane verifies component identity
2. **Signature Validation** - All telemetry must be signed
3. **Rate Limiting** - Control Plane limits data volume
4. **Resource Quotas** - Swap policy enforced for Core/DPI only

---

## Last Updated

Phase 2 Implementation

