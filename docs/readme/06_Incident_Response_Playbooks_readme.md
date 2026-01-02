# Phase 6: Incident Response Playbooks

**Path and File Name:** `/home/ransomeye/rebuild/docs/readme/06_Incident_Response_Playbooks_readme.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Forensic-grade documentation for Phase 6 - Incident Response Playbooks

---

## WHAT EXISTS

### Implementation Location
- **Directory:** `/home/ransomeye/rebuild/core/response_playbooks/`
- **Service:** `systemd/ransomeye-playbook-engine.service`
- **Binary:** `ransomeye-playbook-engine` (Rust)

### Core Components

1. **Playbook Registry** (`src/registry.rs`)
   - Loads playbooks from disk
   - Verifies cryptographic signatures (RSA-4096/Ed25519)
   - Validates YAML schema
   - Version control (one active version per playbook)

2. **Playbook Executor** (`src/executor.rs`)
   - Deterministic step execution
   - Step-level state tracking
   - Timeout enforcement
   - Crash-safe resume
   - Replay protection (nonce + execution ID)

3. **Rollback Engine** (`src/rollback.rs`)
   - Reverse-order execution
   - Restart-safe persistence
   - Fail-closed on rollback failure (enters SAFE-HALT state)

4. **Persistence** (`src/persistence.rs`)
   - PostgreSQL integration
   - Tables: `playbook_executions`, `playbook_rollback_states`, `playbook_nonces`, `playbook_audit_log`, `playbook_safe_halt_state`
   - Immutable execution records
   - Cryptographically chained audit log
   - Nonce tracking (replay protection)

5. **Policy Binding** (`src/binding.rs`)
   - Explicit policy outcome → playbook mapping
   - No implicit actions
   - Missing binding → NO ACTION (fail-closed)

6. **SOC Copilot Visibility** (`src/visibility.rs`)
   - Read-only access to playbook intent
   - Execution status tracking
   - Rollback status tracking
   - Cannot modify playbooks

---

## WHAT DOES NOT EXIST

1. **No automatic playbook execution** - Playbooks triggered only via explicit policy bindings
2. **No playbook authoring UI** - Playbooks authored in YAML
3. **No playbook marketplace** - Playbooks stored locally

---

## DATABASE SCHEMAS

**PostgreSQL Tables:**

1. **playbook_executions**
   - `execution_id` (VARCHAR(36), PRIMARY KEY)
   - `playbook_id` (VARCHAR(36), NOT NULL)
   - `state` (VARCHAR(20), NOT NULL)
   - `started_at` (TIMESTAMP WITH TIME ZONE, NOT NULL)
   - `completed_at` (TIMESTAMP WITH TIME ZONE)
   - `current_step` (INTEGER, NOT NULL, DEFAULT 0)
   - `step_results` (JSONB, NOT NULL, DEFAULT '{}')
   - `nonce` (VARCHAR(36), NOT NULL, UNIQUE)
   - `policy_decision_id` (VARCHAR(36))
   - `created_at`, `updated_at` (TIMESTAMP WITH TIME ZONE)

2. **playbook_rollback_states**
   - `rollback_id` (VARCHAR(36), PRIMARY KEY)
   - `execution_id` (VARCHAR(36), NOT NULL)
   - `playbook_id` (VARCHAR(36), NOT NULL)
   - `started_at`, `completed_at` (TIMESTAMP WITH TIME ZONE)
   - `rollback_step_results` (JSONB, NOT NULL, DEFAULT '[]')
   - `status` (VARCHAR(20), NOT NULL)
   - `created_at`, `updated_at` (TIMESTAMP WITH TIME ZONE)

3. **playbook_nonces**
   - `nonce` (VARCHAR(36), PRIMARY KEY)
   - `used_at` (TIMESTAMP WITH TIME ZONE, NOT NULL, DEFAULT NOW())
   - `execution_id` (VARCHAR(36))

4. **playbook_audit_log**
   - `audit_id` (SERIAL, PRIMARY KEY)
   - `execution_id`, `rollback_id` (VARCHAR(36))
   - `event_type` (VARCHAR(50), NOT NULL)
   - `event_data` (JSONB, NOT NULL)
   - `created_at` (TIMESTAMP WITH TIME ZONE, NOT NULL, DEFAULT NOW())

5. **playbook_safe_halt_state**
   - `halt_id` (SERIAL, PRIMARY KEY)
   - `rollback_id` (VARCHAR(36), NOT NULL)
   - `error_message` (TEXT)
   - `entered_at` (TIMESTAMP WITH TIME ZONE, NOT NULL, DEFAULT NOW())
   - `resolved_at` (TIMESTAMP WITH TIME ZONE)
   - `is_active` (BOOLEAN, NOT NULL, DEFAULT TRUE)

**Indexes:**
- `idx_playbook_executions_playbook_id` ON `playbook_executions(playbook_id)`
- `idx_playbook_executions_state` ON `playbook_executions(state)`

---

## RUNTIME SERVICES

**Service:** `ransomeye-playbook-engine.service`
- **Location:** `/home/ransomeye/rebuild/systemd/ransomeye-playbook-engine.service`
- **User:** `ransomeye`
- **Group:** `ransomeye`
- **Restart:** `always`
- **Dependencies:** `network.target`, `ransomeye-policy.service`
- **ExecStart:** `/usr/local/bin/ransomeye-playbook-engine`

**Service Configuration:**
- Rootless runtime (User=ransomeye)
- Capabilities: CAP_NET_BIND_SERVICE, CAP_NET_RAW, CAP_SYS_NICE
- ReadWritePaths: /home/ransomeye/rebuild, /var/lib/ransomeye/playbooks, /run/ransomeye/playbooks

---

## GUARDRAILS ALIGNMENT

Phase 6 enforces guardrails:

1. **Signed Playbooks Only** - Unsigned playbooks rejected
2. **Validated Schema** - Invalid schema rejected
3. **Replay Protection** - Duplicate nonce rejected
4. **Explicit Bindings** - Missing binding → NO ACTION
5. **Fail-Closed** - Rollback failure → SAFE-HALT state

---

## INSTALLER BEHAVIOR

**Installation:**
- Playbook engine service installed by main installer
- Service file created in `/home/ransomeye/rebuild/systemd/`
- Binary built from Rust crate: `core/response_playbooks/`
- Service disabled by default

---

## SYSTEMD INTEGRATION

**Service File:**
- Created by installer
- Located in unified systemd directory
- Rootless configuration
- Restart always
- Disabled by default

---

## AI/ML/LLM TRAINING REALITY

**NONE** - Phase 6 does not use AI/ML models.

**Playbooks are deterministic:**
- YAML-defined steps
- No ML inference
- No model loading
- Rule-based execution

---

## COPILOT REALITY

**SOC Copilot Visibility:**
- Read-only access to playbook intent
- Execution status tracking
- Rollback status tracking
- Cannot modify playbooks
- Cannot trigger playbooks

---

## UI REALITY

**NONE** - Phase 6 has no UI.

**Playbook execution:**
- Triggered via policy bindings
- Status available via API/visibility interface
- Logs available via systemd journal

---

## FAIL-CLOSED BEHAVIOR

**STRICT FAIL-CLOSED:**

1. **Unsigned Playbook** → REJECTED
2. **Invalid Signature** → REJECTED
3. **Invalid Schema** → REJECTED
4. **Replay Attempt** → REJECTED
5. **Missing Binding** → NO ACTION
6. **Rollback Failure** → SAFE-HALT state

**No Bypass:**
- No `--skip-signature` flag
- No `--skip-replay-check` flag
- All checks must pass

---

## FINAL VERDICT

**PRODUCTION-VIABLE**

Phase 6 is fully implemented and production-ready:

✅ **Complete Implementation**
- Playbook registry functional
- Playbook executor working
- Rollback engine working
- Persistence complete (PostgreSQL)
- Policy binding working
- SOC Copilot visibility working

✅ **Guardrails Alignment**
- Signed playbooks only
- Validated schema
- Replay protection
- Explicit bindings
- Fail-closed behavior

✅ **Fail-Closed Behavior**
- All validation checks fail-closed
- No bypass mechanisms
- Playbooks rejected on any failure
- Rollback failure → SAFE-HALT

**Recommendation:** Deploy as-is. Phase 6 meets all requirements and is production-ready.

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech
