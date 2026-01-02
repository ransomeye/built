# RansomEye Install State Enforcement

## Overview

The **Install State Enforcement System** provides cryptographically-signed, fail-closed database enablement for RansomEye. This system ensures that database services **only start** when explicitly enabled and properly configured, preventing accidental or malicious database activation.

## Security Model

### Fail-Closed Architecture

The install state system implements **fail-closed security**:

- **No fallbacks**: Missing prerequisites abort installation
- **No auto-fix**: System refuses to proceed with incomplete configuration
- **No silent defaults**: Database enablement must be explicit
- **No overwriting**: Signed state files are immutable
- **Cryptographic enforcement**: All state files are Ed25519-signed

### Trust Chain

```
Installer → Manifest (signed) → Install State (signed) → Systemd Units → Services
```

1. **Installer** generates installation manifest
2. **Manifest** is cryptographically signed (Ed25519)
3. **Install State** is generated and signed, referencing manifest hash
4. **Systemd Units** require valid install state to start
5. **Services** load configuration from secured environment files

## Architecture

### Core Components

```
/home/ransomeye/rebuild/core/install_state/
├── __init__.py                      # Module initialization
├── finalize_install_state.py        # Main implementation
├── state_schema.json                # JSON schema for validation
├── state_signer.py                  # Ed25519 signing utilities
└── tests/
    └── test_install_state.py        # Test suite
```

### Runtime Files

```
/var/lib/ransomeye/
├── install_state.json               # Signed installation state (0444)
├── install_state.sig                # Ed25519 signature (0444)
├── install_manifest.json            # Installation manifest (0444)
├── install_manifest.sig             # Manifest signature (0444)
└── keys/
    ├── signing_private.pem          # Ed25519 private key (0600)
    └── signing_public.pem           # Ed25519 public key (0644)

/etc/ransomeye/
└── db.env                           # Database credentials (0600, if DB enabled)
```

## Install State Schema

### Required Fields

```json
{
  "state_version": "1.0",
  "install_timestamp": "2025-12-29T12:34:56.789Z",
  "db_enabled": true,
  "db_schema_applied": true,
  "db_schema_signature_verified": true,
  "enabled_modules": ["ransomeye_intelligence", "ransomeye_correlation", ...],
  "manifest_hash": "sha256_hex_hash_of_install_manifest",
  "installer_identity_hash": "sha256_hex_hash_of_installer_identity",
  "signer_fingerprint": "sha256_hex_hash_of_public_key",
  "db_host": "localhost",
  "db_port": 5432,
  "db_name": "ransomeye"
}
```

### Database Enablement Logic

#### When `RANSOMEYE_ENABLE_DB` is NOT set:

```json
{
  "db_enabled": false,
  "db_schema_applied": false,
  "db_schema_signature_verified": false
}
```

- Database services **CANNOT START** (systemd condition fails)
- No `/etc/ransomeye/db.env` file created
- No database-related validation performed

#### When `RANSOMEYE_ENABLE_DB=true`:

```json
{
  "db_enabled": true,
  "db_schema_applied": true,
  "db_schema_signature_verified": true,
  "db_host": "localhost",
  "db_port": 5432,
  "db_name": "ransomeye"
}
```

**Fail-Closed Prerequisites:**

1. ✅ `DB_HOST`, `DB_PORT`, `DB_NAME`, `DB_USER`, `DB_PASS` environment variables set
2. ✅ Database schema file exists: `/home/ransomeye/rebuild/ransomeye_db_core/schema/unified_schema.sql`
3. ✅ Schema signature exists: `/home/ransomeye/rebuild/ransomeye_db_core/schema/unified_schema.sql.sig`
4. ✅ Schema signature is valid (Ed25519)
5. ✅ Schema has been applied (manifest flag: `db_initialized=true`)
6. ✅ `/etc/ransomeye/db.env` created with 0600 permissions

**Any missing prerequisite aborts installation.**

## Systemd Integration

### Unit File Conditions

The `ransomeye-db_core.service` unit includes **fail-closed conditions**:

```ini
[Unit]
Description=RansomEye ransomeye_db_core
After=network.target
Requires=network.target
ConditionPathExists=/var/lib/ransomeye/install_state.json
# DB ENFORCEMENT: Require cryptographically signed install state
ConditionPathExists=/var/lib/ransomeye/install_state.sig
# Pre-start validation: fail-closed if runtime layout invalid
ConditionPathExists=/opt/ransomeye
ConditionPathExists=/opt/ransomeye/bin/ransomeye-db_core

[Service]
Type=simple
User=ransomeye
Group=ransomeye
WorkingDirectory=/opt/ransomeye
# DB ENVIRONMENT: Load database credentials from secured file
EnvironmentFile=/etc/ransomeye/db.env
Environment="RANSOMEYE_ROOT=/opt/ransomeye"
Environment="PYTHONUNBUFFERED=1"
ExecStart=/opt/ransomeye/bin/ransomeye-db_core
...
```

### Condition Enforcement

If **any** condition fails, systemd **refuses to start the service**:

- Missing `install_state.json` → Service inactive (dead)
- Missing `install_state.sig` → Service inactive (dead)
- Missing `db.env` → Service fails to start (EnvironmentFile error)
- Invalid signature → Validator detects, blocks deployment

This ensures **fail-closed behavior at the systemd level**.

## Installation Workflow

### Execution Order

```
1. Generate install manifest
2. Install systemd units
3. Sign manifest (Ed25519)
4. ▶ Finalize install state ◀  [NEW STEP]
5. Run global validator
6. Generate post-install report
```

### Finalize Install State Step

**Location in `install.sh`:** Between manifest signing and global validator

**Actions:**

1. Check if `RANSOMEYE_ENABLE_DB` is set
2. If `db_enabled=false`:
   - Create minimal install_state.json
   - Sign state file
   - Skip database prerequisites
3. If `db_enabled=true`:
   - Verify all database prerequisites (FAIL-CLOSED)
   - Verify schema signature
   - Create `/etc/ransomeye/db.env` with credentials
   - Generate install_state.json with DB config
   - Sign state file
4. Make state file immutable (chmod 0444)
5. Exit 0 on success, exit 1 on failure (installer aborts)

### Install State Finalization Script

```python
from core.install_state.finalize_install_state import finalize_install_state

# This will ABORT if prerequisites missing (fail-closed)
install_state = finalize_install_state(
    private_key_path="/var/lib/ransomeye/keys/signing_private.pem"
)
```

## Validation

### Global Validator Integration

The **Global Forensic Consistency Validator** includes `InstallStateChecker`:

```python
from core.global_validator.install_state_checker import InstallStateChecker

# Validator automatically runs this checker
validator = GlobalForensicValidator()
result = validator.validate_all()
```

**InstallStateChecker Verifies:**

1. ✅ `install_state.json` exists
2. ✅ `install_state.sig` exists
3. ✅ Signature is valid (Ed25519)
4. ✅ File permissions are 0444 (immutable)
5. ✅ All required fields present
6. ✅ If `db_enabled=true`:
   - Schema applied
   - Schema signature verified
   - `/etc/ransomeye/db.env` exists
   - DB config fields present
7. ✅ Manifest hash matches

**Any violation is CRITICAL and fails validation.**

## Testing

### Test Suite

**Location:** `/home/ransomeye/rebuild/core/install_state/tests/test_install_state.py`

**Run Tests:**

```bash
cd /home/ransomeye/rebuild
python3 core/install_state/tests/test_install_state.py
```

**Tests:**

1. ✅ DB service requires `install_state.json`
2. ✅ DB service requires `install_state.sig`
3. ✅ DB service loads `db.env`
4. ✅ `install_state.json` exists and is valid JSON
5. ✅ Signature is valid
6. ✅ State file is immutable (0444)
7. ✅ Database enablement prerequisites consistent
8. ✅ Global validator includes install_state checker

### Manual Validation

**Verify install state:**

```bash
python3 -m core.install_state.finalize_install_state verify
```

**Check systemd conditions:**

```bash
systemctl cat ransomeye-db_core.service | grep Condition
```

**Test service start:**

```bash
# Should fail if install_state missing
sudo rm /var/lib/ransomeye/install_state.json
sudo systemctl start ransomeye-db_core
# Expected: Service inactive (dead) due to failed condition

# Restore state
sudo ./install.sh  # Re-run installer

# Should start if db_enabled=true
sudo systemctl start ransomeye-db_core
systemctl status ransomeye-db_core
```

## Security Considerations

### Cryptographic Guarantees

1. **Ed25519 Signatures**: Military-grade elliptic curve signatures
2. **Immutable Files**: State files are read-only (0444)
3. **Tamper Detection**: Any modification invalidates signature
4. **Trust Chain**: Manifest → Install State → Services

### Attack Surface Mitigation

| Attack | Mitigation |
|--------|------------|
| Unsigned state file | Systemd condition requires `.sig` file |
| Tampered state | Validator verifies signature before accepting |
| Missing DB config | EnvironmentFile directive fails service start |
| Accidental DB enable | Explicit `RANSOMEYE_ENABLE_DB` required |
| Schema corruption | Schema signature verification before enable |
| Credential leak | `db.env` has 0600 permissions (root/ransomeye only) |

### Fail-Closed Enforcement Points

1. **Installer**: Aborts if prerequisites missing
2. **Systemd**: Conditions prevent service start
3. **Validator**: Detects tampering/misconfiguration
4. **Runtime**: Services refuse to start without valid state

## Troubleshooting

### Service Won't Start

**Symptom:** `systemctl start ransomeye-db_core` fails with "Condition check resulted in ... being skipped"

**Cause:** Missing install state files

**Fix:**

```bash
# Check files exist
ls -l /var/lib/ransomeye/install_state.{json,sig}
ls -l /etc/ransomeye/db.env

# Verify signature
python3 -m core.install_state.finalize_install_state verify

# Re-run installer if files missing
sudo ./install.sh
```

### Database Not Enabled

**Symptom:** `db_enabled: false` in install_state.json

**Cause:** `RANSOMEYE_ENABLE_DB` not set during installation

**Fix:**

```bash
# Set environment variable
export RANSOMEYE_ENABLE_DB=true
export DB_HOST=localhost
export DB_PORT=5432
export DB_NAME=ransomeye
export DB_USER=gagan
export DB_PASS=gagan

# Re-run installer
sudo ./install.sh
```

### Signature Verification Failed

**Symptom:** Validator reports "Invalid signature on install_state.json"

**Cause:** State file modified after signing

**Fix:**

```bash
# DO NOT modify install_state.json manually
# Re-run installer to regenerate signed state
sudo rm /var/lib/ransomeye/install_state.{json,sig}
sudo ./install.sh
```

### Missing db.env

**Symptom:** Service fails with "Cannot load environment file"

**Cause:** `db.env` deleted or permissions incorrect

**Fix:**

```bash
# Check file exists and permissions
ls -l /etc/ransomeye/db.env

# If missing, re-run installer
sudo ./install.sh

# If permissions wrong
sudo chmod 600 /etc/ransomeye/db.env
sudo chown root:ransomeye /etc/ransomeye/db.env
```

## Implementation Checklist

### ✅ Completed

- [x] Create `core/install_state/` module
- [x] Implement `finalize_install_state.py`
- [x] Implement `state_signer.py` (Ed25519)
- [x] Define `state_schema.json`
- [x] Update `systemd_writer.py` for DB service conditions
- [x] Integrate into `install.sh`
- [x] Create `InstallStateChecker` for global validator
- [x] Register checker in validator
- [x] Create test suite
- [x] Create documentation

### 🔒 Security Guarantees

- [x] Fail-closed database enablement
- [x] Cryptographic signing (Ed25519)
- [x] Immutable state files (0444)
- [x] Secured credentials (0600)
- [x] Systemd condition enforcement
- [x] Validator signature verification
- [x] Tamper detection
- [x] No fallbacks or auto-fix

## References

- **Ed25519 Signing**: `core/install_state/state_signer.py`
- **Schema Definition**: `core/install_state/state_schema.json`
- **Finalization Logic**: `core/install_state/finalize_install_state.py`
- **Validator Integration**: `core/global_validator/install_state_checker.py`
- **Systemd Generator**: `ransomeye_installer/services/systemd_writer.py`
- **Test Suite**: `core/install_state/tests/test_install_state.py`

---

**© RansomEye.Tech | Support: Gagan@RansomEye.Tech**

---

**Document Version:** 1.0  
**Last Updated:** 2025-12-29  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU

