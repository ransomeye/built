# Ransomware Kill-Chain Model

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_core/docs/kill_chain_model.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Formal ransomware-specific kill-chain model documentation

## Overview

The RansomEye correlation engine implements a **ransomware-specific kill-chain model** (NOT generic MITRE ATT&CK). This model defines the stages that ransomware attacks progress through, with deterministic transition rules and evidence requirements.

## Kill-Chain Stages

### 1. InitialAccess
- **Description:** Initial access to the target system
- **Required Signals:** Network connection, process creation
- **Entry Conditions:** At least 1 signal with confidence >= 0.6
- **Temporal Window:** 300 seconds

### 2. Execution
- **Description:** Malicious code execution
- **Required Signals:** Process creation, file execution
- **Entry Conditions:** At least 1 signal with confidence >= 0.7
- **Temporal Window:** 60 seconds

### 3. Persistence
- **Description:** Establishing persistence mechanisms
- **Required Signals:** Registry modification, service creation, scheduled task
- **Entry Conditions:** At least 1 signal with confidence >= 0.7
- **Temporal Window:** 300 seconds

### 4. PrivilegeEscalation
- **Description:** Gaining elevated privileges
- **Required Signals:** Privilege escalation, token manipulation
- **Entry Conditions:** At least 1 signal with confidence >= 0.6
- **Temporal Window:** 300 seconds

### 5. LateralMovement
- **Description:** Moving across network boundaries
- **Required Signals:** Network connection, remote execution
- **Entry Conditions:** At least 1 signal with confidence >= 0.6
- **Temporal Window:** 600 seconds

### 6. CredentialAccess
- **Description:** Stealing credentials
- **Required Signals:** Credential dump, LSASS access
- **Entry Conditions:** At least 1 signal with confidence >= 0.6
- **Temporal Window:** 300 seconds

### 7. Discovery
- **Description:** Discovering system resources and data
- **Required Signals:** File enumeration, network scanning, system info
- **Entry Conditions:** At least 2 signals with confidence >= 0.6
- **Temporal Window:** 600 seconds

### 8. EncryptionPreparation
- **Description:** Preparing for encryption (key generation, file enumeration)
- **Required Signals:** File enumeration, key generation, process creation
- **Entry Conditions:** At least 2 signals with confidence >= 0.7
- **Temporal Window:** 300 seconds

### 9. EncryptionExecution
- **Description:** Actively encrypting files
- **Required Signals:** File modification (>=10), encryption activity
- **Entry Conditions:** At least 2 signals with confidence >= 0.8
- **Temporal Window:** 60 seconds

### 10. Impact
- **Description:** Final impact (data encrypted, ransom note dropped)
- **Required Signals:** Ransom note, file encryption complete
- **Entry Conditions:** At least 1 signal with confidence >= 0.9
- **Temporal Window:** 300 seconds

## Transition Rules

### Allowed Transitions

1. **Sequential:** Each stage can transition to the next stage
2. **Self:** Each stage can remain in the same stage (with new evidence)
3. **Conditional Skips:** Some stages allow skipping with evidence:
   - InitialAccess → Execution (direct)
   - Execution → PrivilegeEscalation (skip Persistence with evidence)
   - Discovery → EncryptionPreparation (direct)

### Forbidden Transitions

1. **Regression:** Cannot go backwards in kill-chain
2. **Large Jumps:** Cannot skip multiple stages without evidence
3. **Invalid Starts:** Must start with InitialAccess

## Evidence Requirements

Each stage transition requires:
- Valid transition rule match
- Sufficient evidence signals
- Confidence threshold met
- Temporal constraints satisfied

## Confidence Decay

Each stage has a confidence decay rate (default: 0.1 per hour). If no new signals arrive, confidence decreases over time.

## Determinism

All stage transitions are **deterministic**:
- Same signals → same stage inference
- Same evidence → same transition result
- No probabilistic decisions
- No AI/ML influence on transitions

