# Approval Workflows

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_enforcement/docs/approval_workflows.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Approval workflow documentation - human-in-the-loop enforcement

## Overview

Approval workflows ensure that **destructive or high-risk enforcement actions** require explicit human approval before execution.

## Approval Types

### Operator Approval
- **Type:** `operator`
- **Purpose:** Standard operator approval
- **Use Case:** Routine enforcement actions
- **Required For:** Most enforcement actions

### Manager Approval
- **Type:** `manager`
- **Purpose:** Management-level approval
- **Use Case:** High-impact enforcement actions
- **Required For:** Mass enforcement, production changes

### Security Approval
- **Type:** `security`
- **Purpose:** Security team approval
- **Use Case:** Security-critical enforcement actions
- **Required For:** Isolation, quarantine, block actions

### Automated Approval
- **Type:** `automated`
- **Purpose:** Automated approval (low-risk)
- **Use Case:** Monitoring, low-risk actions
- **Required For:** Non-destructive actions

## Approval Process

### 1. Decision Requires Approval
When a policy decision includes `required_approvals`, execution is **HELD** until all approvals are present.

### 2. Approval Recording
Approvals are recorded via:
```rust
dispatcher.record_approval(decision_id, approval_type, approver);
```

### 3. Approval Validation
Before execution, all required approvals are validated:
- Approval type matches required type
- Approval recorded for correct decision
- Approval timestamp valid

### 4. Execution Proceeds
Once all approvals are present, execution proceeds normally.

## Approval Workflow States

### Held
- **Status:** `ExecutionStatus::Held`
- **Meaning:** Waiting for required approvals
- **Action:** Record approvals to proceed

### Executed
- **Status:** `ExecutionStatus::Executed`
- **Meaning:** All approvals present, execution completed
- **Action:** None

### Rejected
- **Status:** `ExecutionStatus::Rejected`
- **Meaning:** Execution rejected (not due to approvals)
- **Action:** Review rejection reason

## Approval Examples

### Example 1: Single Approval Required
```json
{
  "decision_id": "decision-1",
  "required_approvals": ["operator"],
  "decision": "block"
}
```

**Workflow:**
1. Decision received → Execution HELD
2. Operator approval recorded → Execution proceeds
3. Execution completed

### Example 2: Multiple Approvals Required
```json
{
  "decision_id": "decision-2",
  "required_approvals": ["operator", "manager"],
  "decision": "isolate"
}
```

**Workflow:**
1. Decision received → Execution HELD
2. Operator approval recorded → Still HELD (manager approval missing)
3. Manager approval recorded → Execution proceeds
4. Execution completed

### Example 3: No Approval Required
```json
{
  "decision_id": "decision-3",
  "required_approvals": [],
  "decision": "monitor"
}
```

**Workflow:**
1. Decision received → Execution proceeds immediately
2. Execution completed

## Approval Recording

### API
```rust
pub fn record_approval(
    &self,
    decision_id: &str,
    approval_type: &str,
    approver: &str,
) -> Result<(), EnforcementError>
```

### Parameters
- `decision_id`: Decision identifier
- `approval_type`: Approval type (operator, manager, security, automated)
- `approver`: Approver identifier (username, system ID, etc.)

### Example
```rust
dispatcher.record_approval(
    "decision-1",
    "operator",
    "admin@ransomeye.tech"
)?;
```

## Approval Status

### Status Structure
```rust
pub struct ApprovalStatus {
    pub approval_type: String,
    pub approved: bool,
    pub approver: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
}
```

### Status Fields
- `approval_type`: Type of approval required
- `approved`: Whether approval is present
- `approver`: Who approved (if approved)
- `approved_at`: When approved (if approved)

## Approval Validation

### Validation Rules
1. **Approval Type Match**
   - Required approval type must match recorded approval type
   - Case-sensitive matching

2. **Decision Match**
   - Approval must be recorded for correct decision
   - Decision ID must match

3. **Approval Presence**
   - All required approvals must be present
   - Missing approval → Execution HELD

4. **Approval Timestamp**
   - Approval timestamp must be valid
   - Approval must not be expired (if expiration configured)

## Approval Best Practices

### Production
- Require approvals for all destructive actions
- Require multiple approvals for high-impact actions
- Log all approvals with full context

### Staging
- Require approvals for destructive actions
- Allow single approval for moderate-impact actions
- Log all approvals

### Development
- Allow automated approvals for testing
- Require approvals only for critical actions
- Log all approvals

## Approval Monitoring

### Metrics
- Approval holds per hour
- Average approval time
- Approval rejection rate
- Approval workflow delays

### Alerts
- High approval hold rate
- Long approval wait times
- Approval workflow failures
- Missing approvals

## Approval Integration

### External Systems
- Integrate with ticketing systems
- Integrate with approval workflows
- Integrate with notification systems

### API Integration
- REST API for approval recording
- Webhook for approval notifications
- Event stream for approval events

## Approval Security

### Authentication
- Approver authentication required
- Approver authorization validated
- Approval source verified

### Audit Trail
- All approvals logged
- Approval history maintained
- Approval revocation supported

### Non-Repudiation
- Approvals cryptographically signed
- Approval timestamps verified
- Approval integrity validated

