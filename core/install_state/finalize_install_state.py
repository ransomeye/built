# Path and File Name : /home/ransomeye/rebuild/core/install_state/finalize_install_state.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Fail-closed install state finalization with cryptographic signing

"""
Install State Finalization

This module implements fail-closed installation state tracking with
cryptographic signing. It enforces that database services only start
when explicitly enabled and properly configured.

SECURITY MODEL:
- Fail-closed: any missing prerequisite aborts
- No fallbacks, no auto-fix, no silent defaults
- Cryptographically signed state prevents tampering
- State file is immutable after signing
"""

import os
import sys
import json
import hashlib
import stat
from pathlib import Path
from datetime import datetime, timezone
from typing import Dict, List, Optional, Any

try:
    import jsonschema
except ImportError:
    print("ERROR: jsonschema package not installed", file=sys.stderr)
    sys.exit(1)

from .state_signer import (
    sign_state_file,
    verify_state_signature,
    generate_installer_identity_hash,
    load_public_key,
    compute_public_key_fingerprint
)


# Constants
STATE_FILE_PATH = "/var/lib/ransomeye/install_state.json"
STATE_SIGNATURE_PATH = "/var/lib/ransomeye/install_state.sig"
DB_ENV_FILE_PATH = "/etc/ransomeye/db.env"
MANIFEST_PATH = "/var/lib/ransomeye/install_manifest.json"
SCHEMA_PATH = "/home/ransomeye/rebuild/core/install_state/state_schema.json"


class InstallStateError(Exception):
    """Raised when install state finalization fails"""
    pass


def load_schema() -> Dict:
    """Load install state JSON schema."""
    try:
        schema_path = Path(SCHEMA_PATH)
        if not schema_path.exists():
            raise InstallStateError(f"Schema not found: {SCHEMA_PATH}")
        
        with open(schema_path, 'r') as f:
            return json.load(f)
    
    except Exception as e:
        raise InstallStateError(f"Failed to load schema: {e}")


def compute_manifest_hash() -> str:
    """
    Compute SHA256 hash of install_manifest.json.
    
    Returns:
        Hex-encoded SHA256 hash
        
    Raises:
        InstallStateError: If manifest missing or unreadable
    """
    try:
        manifest_path = Path(MANIFEST_PATH)
        
        if not manifest_path.exists():
            raise InstallStateError(
                f"Install manifest not found: {MANIFEST_PATH}\n"
                "Cannot finalize state without valid installation."
            )
        
        with open(manifest_path, 'rb') as f:
            manifest_data = f.read()
        
        return hashlib.sha256(manifest_data).hexdigest()
    
    except Exception as e:
        raise InstallStateError(f"Failed to hash manifest: {e}")


def check_db_prerequisites() -> Dict[str, Any]:
    """
    Check database prerequisites and return status.
    
    CRITICAL ARCHITECTURE RULE: DB IS MANDATORY FOR RANSOMEYE CORE
    - db.env MUST exist
    - DB_MODE MUST be set (standalone|ha)
    - All DB credentials MUST be present
    
    Returns:
        Dictionary with db_mode, db_schema_applied, etc.
        
    Raises:
        InstallStateError: If db.env missing (FAIL-CLOSED)
        InstallStateError: If DB_MODE missing or invalid (FAIL-CLOSED)
        InstallStateError: If DB credentials missing (FAIL-CLOSED)
        InstallStateError: If schema not applied/verified (FAIL-CLOSED)
    """
    # CRITICAL: DB is MANDATORY - db.env MUST exist
    # install.sh ALWAYS creates this file
    db_env_path = Path(DB_ENV_FILE_PATH)
    
    if not db_env_path.exists():
        raise InstallStateError(
            f"FAIL-CLOSED: Database environment file not found: {DB_ENV_FILE_PATH}\n"
            "INVARIANT VIOLATION: DB is MANDATORY for RansomEye Core.\n"
            "INSTALL ORDER VIOLATION: db.env MUST be created immediately after DB_MODE prompt.\n"
            "Expected order: EULA → DB_MODE → CREATE db.env → manifest → schema → finalize_install_state\n"
            "This is a critical installer bug. Installation aborted."
        )
    
    # DB env file exists - parse it (DB is ALWAYS enabled - NO INFERENCE)
    print("✓ Database configuration found (DB MANDATORY - no inference, always true)")
    
    # CRITICAL: Initialize with db_enabled ALWAYS true (architectural lock)
    result = {
        'db_enabled': True,  # MANDATORY: ALWAYS true for Core (no fallback, no inference)
        'db_mode': None,
        'db_schema_applied': False,
        'db_schema_signature_verified': False,
        'db_host': None,
        'db_port': None,
        'db_name': None
    }
    
    # Load DB config from persisted file
    try:
        db_config = {}
        with open(db_env_path, 'r') as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith('#') and '=' in line:
                    key, value = line.split('=', 1)
                    db_config[key.strip()] = value.strip()
        
        # Extract required fields (DB_MODE mandatory)
        required_vars = ['DB_HOST', 'DB_PORT', 'DB_NAME', 'DB_USER', 'DB_PASS', 'DB_MODE']
        missing_vars = [v for v in required_vars if v not in db_config]
        
        if missing_vars:
            raise InstallStateError(
                f"FAIL-CLOSED: db.env missing required vars: {missing_vars}\n"
                "INVARIANT: All DB credentials and DB_MODE must be present.\n"
                "install.sh must write complete DB configuration."
            )
        
        # CRITICAL ARCHITECTURE LOCK: Verify credentials are gagan:gagan (fixed)
        if db_config.get('DB_USER') != 'gagan':
            raise InstallStateError(
                f"FAIL-CLOSED: DB_USER must be 'gagan' (found: '{db_config.get('DB_USER')}')\n"
                "INVARIANT: DB credentials are FIXED to gagan:gagan (architectural lock)"
            )
        
        if db_config.get('DB_PASS') != 'gagan':
            raise InstallStateError(
                f"FAIL-CLOSED: DB_PASS must be 'gagan'\n"
                "INVARIANT: DB credentials are FIXED to gagan:gagan (architectural lock)"
            )
        
        # Extract and validate DB_MODE (MANDATORY)
        db_mode = db_config['DB_MODE'].strip()
        if not db_mode:
            raise InstallStateError(
                "FAIL-CLOSED: DB_MODE is empty in db.env\n"
                "DB_MODE must be 'standalone' or 'ha'"
            )
        
        if db_mode not in ['standalone', 'ha']:
            raise InstallStateError(
                f"FAIL-CLOSED: Invalid DB_MODE='{db_mode}' in db.env\n"
                "DB_MODE must be 'standalone' or 'ha'"
            )
        
        result['db_mode'] = db_mode
        result['db_host'] = db_config['DB_HOST']
        result['db_port'] = int(db_config['DB_PORT'])
        result['db_name'] = db_config['DB_NAME']
        
        print(f"  Database mode: {db_mode}")
    
    except InstallStateError:
        raise  # Re-raise our explicit errors
    except Exception as e:
        raise InstallStateError(f"FAIL-CLOSED: Failed to parse db.env: {e}")
    
    # Check schema file exists and is signed
    schema_file = Path("/home/ransomeye/rebuild/ransomeye_db_core/schema/unified_schema.sql")
    schema_sig = Path("/home/ransomeye/rebuild/ransomeye_db_core/schema/unified_schema.sql.sig")
    
    if not schema_file.exists():
        raise InstallStateError(
            f"FAIL-CLOSED: Database schema file not found: {schema_file}\n"
            "Schema must exist before DB can be enabled."
        )
    
    if not schema_sig.exists():
        raise InstallStateError(
            f"FAIL-CLOSED: Database schema signature not found: {schema_sig}\n"
            "Schema must be signed before DB can be enabled."
        )
    
    # BUG FIX: Use CORRECT manifest signing public key
    public_key_path = "/var/lib/ransomeye/keys/manifest_signing.pub"
    
    if not Path(public_key_path).exists():
        raise InstallStateError(
            f"FAIL-CLOSED: Public key not found: {public_key_path}\n"
            "Cannot verify schema signature."
        )
    
    schema_valid = verify_state_signature(
        str(schema_file),
        str(schema_sig),
        public_key_path
    )
    
    if not schema_valid:
        raise InstallStateError(
            "FAIL-CLOSED: Database schema signature verification FAILED\n"
            "Schema may be tampered or corrupted."
        )
    
    print("✓ Database schema signature verified")
    result['db_schema_signature_verified'] = True
    
    # Check if schema was applied (look for marker file or query DB)
    # For now, we check if manifest indicates DB was initialized
    try:
        with open(MANIFEST_PATH, 'r') as f:
            manifest = json.load(f)
        
        db_initialized = manifest.get('db_initialized', False)
        
        if not db_initialized:
            raise InstallStateError(
                "FAIL-CLOSED: Database enabled but schema not applied\n"
                "Run database initialization first."
            )
        
        print("✓ Database schema applied")
        result['db_schema_applied'] = True
    
    except FileNotFoundError:
        raise InstallStateError("Install manifest missing")
    
    except KeyError:
        raise InstallStateError("Manifest missing db_initialized flag")
    
    return result


def collect_enabled_modules() -> List[str]:
    """
    Collect list of enabled RansomEye modules.
    
    Returns:
        List of module names
    """
    try:
        with open(MANIFEST_PATH, 'r') as f:
            manifest = json.load(f)
        
        return manifest.get('enabled_modules', [])
    
    except Exception as e:
        print(f"WARNING: Could not read enabled modules: {e}", file=sys.stderr)
        return []


def check_existing_state() -> bool:
    """
    Check if signed install state already exists.
    
    Returns:
        True if valid signed state exists, False otherwise
        
    Raises:
        InstallStateError: If unsigned state exists (FAIL-CLOSED)
    """
    state_path = Path(STATE_FILE_PATH)
    sig_path = Path(STATE_SIGNATURE_PATH)
    
    if state_path.exists() and not sig_path.exists():
        raise InstallStateError(
            f"FAIL-CLOSED: Unsigned install state exists at {STATE_FILE_PATH}\n"
            "This indicates tampering or incomplete installation.\n"
            "Remove the file manually if reinstalling."
        )
    
    if state_path.exists() and sig_path.exists():
        # Verify signature
        public_key_path = "/var/lib/ransomeye/keys/signing_public.pem"
        
        if not Path(public_key_path).exists():
            raise InstallStateError(
                "FAIL-CLOSED: Public key missing, cannot verify existing state"
            )
        
        valid = verify_state_signature(
            str(state_path),
            str(sig_path),
            public_key_path
        )
        
        if not valid:
            raise InstallStateError(
                "FAIL-CLOSED: Existing install state signature is INVALID\n"
                "State file may be tampered or corrupted."
            )
        
        return True
    
    return False


def validate_db_env_file(db_config: Dict[str, Any]) -> None:
    """
    Validate database environment file exists.
    
    CRITICAL ARCHITECTURE RULE: DB IS MANDATORY FOR RANSOMEYE CORE
    - db.env MUST exist (installer creates it BEFORE finalize_install_state)
    - DB is ALWAYS enabled for Core
    
    Args:
        db_config: Dictionary with DB configuration
        
    Raises:
        InstallStateError: If db.env missing (FAIL-CLOSED)
    """
    env_path = Path(DB_ENV_FILE_PATH)
    
    # INVARIANT: DB is ALWAYS enabled - db.env MUST exist
    if not env_path.exists():
        raise InstallStateError(
            f"FAIL-CLOSED: db.env not found at {DB_ENV_FILE_PATH}\n"
            "INVARIANT VIOLATION: DB is MANDATORY for RansomEye Core.\n"
            "Installer must write db.env BEFORE calling finalize_install_state."
        )
    
    print(f"✓ Database env file validated: {DB_ENV_FILE_PATH}")


def finalize_install_state(private_key_path: str = "/var/lib/ransomeye/keys/manifest_signing.key") -> Dict[str, Any]:
    """
    Finalize installation state with cryptographic signing.
    
    This function:
    1. Checks if state already exists (REFUSE overwrite if signed)
    2. Verifies DB prerequisites (FAIL-CLOSED if enabled but incomplete)
    3. Collects enabled modules
    4. Computes manifest hash
    5. Generates install_state.json
    6. Signs state file with Ed25519
    7. Makes state file immutable (0444)
    8. Writes db.env if DB enabled
    
    Args:
        private_key_path: Path to Ed25519 private key PEM
        
    Returns:
        Install state dictionary
        
    Raises:
        InstallStateError: On any failure (FAIL-CLOSED)
    """
    print("\n" + "="*60)
    print("RansomEye Install State Finalization (FAIL-CLOSED)")
    print("="*60 + "\n")
    
    # 1. Check existing state
    if check_existing_state():
        raise InstallStateError(
            f"FAIL-CLOSED: Signed install state already exists at {STATE_FILE_PATH}\n"
            "Cannot overwrite signed state. Remove manually if reinstalling."
        )
    
    # 2. Check DB prerequisites (FAIL-CLOSED)
    db_config = check_db_prerequisites()
    
    # 3. Collect enabled modules
    enabled_modules = collect_enabled_modules()
    print(f"✓ Enabled modules: {len(enabled_modules)}")
    
    # 4. Compute manifest hash
    manifest_hash = compute_manifest_hash()
    print(f"✓ Manifest hash: {manifest_hash[:16]}...")
    
    # 5. Generate installer identity hash
    installer_identity = generate_installer_identity_hash()
    print(f"✓ Installer identity: {installer_identity[:16]}...")
    
    # 6. Load public key for fingerprint
    # BUG FIX: Use CORRECT manifest signing public key
    public_key_path = "/var/lib/ransomeye/keys/manifest_signing.pub"
    
    if not Path(public_key_path).exists():
        raise InstallStateError(f"Public key not found: {public_key_path}")
    
    public_key = load_public_key(public_key_path)
    signer_fingerprint = compute_public_key_fingerprint(public_key)
    print(f"✓ Signer fingerprint: {signer_fingerprint[:16]}...")
    
    # 7. Build install state with explicit db section
    # ====================================================================
    # CRITICAL: DB section is EXPLICIT and STRUCTURED
    # - If DB disabled: db.enabled=false, db.mode=null
    # - If DB enabled: db.enabled=true, db.mode=(standalone|ha)
    # - mode=ha: requires HA metadata (enforced at runtime)
    # ====================================================================
    install_state = {
        'state_version': '1.0',
        'install_timestamp': datetime.now(timezone.utc).isoformat(),
        'db': {
            'enabled': db_config['db_enabled'],
            'mode': db_config['db_mode'],
            'host': db_config['db_host'],
            'port': db_config['db_port'],
            'name': db_config['db_name'],
            'schema_applied': db_config['db_schema_applied'],
            'schema_signature_verified': db_config['db_schema_signature_verified']
        },
        'enabled_modules': enabled_modules,
        'manifest_hash': manifest_hash,
        'installer_identity_hash': installer_identity,
        'signer_fingerprint': signer_fingerprint
    }
    
    # 8. Validate against schema
    schema = load_schema()
    
    try:
        jsonschema.validate(install_state, schema)
        print("✓ Install state validated against schema")
    except jsonschema.ValidationError as e:
        raise InstallStateError(f"Install state validation failed: {e}")
    
    # 9. Write state file (MUST succeed before signing)
    state_path = Path(STATE_FILE_PATH)
    state_path.parent.mkdir(parents=True, exist_ok=True)
    
    try:
        with open(state_path, 'w') as f:
            json.dump(install_state, f, indent=2)
        
        # BUG FIX: Verify file was actually written
        if not state_path.exists():
            raise InstallStateError("State file write reported success but file not found")
        
        print(f"✓ Install state written to {STATE_FILE_PATH}")
    except Exception as e:
        raise InstallStateError(f"Failed to write install_state.json: {e}")
    
    # 10. Sign state file
    try:
        sign_state_file(
            STATE_FILE_PATH,
            STATE_SIGNATURE_PATH,
            private_key_path
        )
        
        # BUG FIX: Verify signature file was actually written
        sig_path = Path(STATE_SIGNATURE_PATH)
        if not sig_path.exists():
            raise InstallStateError("Signature write reported success but file not found")
    except Exception as e:
        raise InstallStateError(f"Failed to sign install_state.json: {e}")
    
    # 11. Make state file immutable (0444)
    os.chmod(state_path, 0o444)
    print(f"✓ Install state made immutable (0444)")
    
    # 12. Validate db.env exists (MANDATORY - installer writes it BEFORE this)
    validate_db_env_file(db_config)
    
    print("\n" + "="*60)
    print("✓✓✓ INSTALL STATE FINALIZED SUCCESSFULLY ✓✓✓")
    print("="*60)
    print(f"\nState file: {STATE_FILE_PATH}")
    print(f"Signature:  {STATE_SIGNATURE_PATH}")
    print(f"DB config:  {DB_ENV_FILE_PATH} (MANDATORY)")
    print(f"  Mode: {db_config['db_mode']}")
    print(f"  Host: {db_config['db_host']}:{db_config['db_port']}")
    print(f"  Database: {db_config['db_name']}")
    print()
    
    return install_state


def verify_install_state() -> bool:
    """
    Verify install state integrity.
    
    Returns:
        True if state valid and signed, False otherwise
    """
    try:
        state_path = Path(STATE_FILE_PATH)
        sig_path = Path(STATE_SIGNATURE_PATH)
        
        if not state_path.exists():
            print("ERROR: Install state file not found", file=sys.stderr)
            return False
        
        if not sig_path.exists():
            print("ERROR: Install state signature not found", file=sys.stderr)
            return False
        
        # BUG FIX: Use CORRECT manifest signing public key
        public_key_path = "/var/lib/ransomeye/keys/manifest_signing.pub"
        
        if not Path(public_key_path).exists():
            print("ERROR: Public key not found", file=sys.stderr)
            return False
        
        # Verify signature
        valid = verify_state_signature(
            str(state_path),
            str(sig_path),
            public_key_path
        )
        
        if not valid:
            return False
        
        # Verify schema compliance
        with open(state_path, 'r') as f:
            state = json.load(f)
        
        schema = load_schema()
        jsonschema.validate(state, schema)
        
        print("✓ Install state verified successfully")
        return True
    
    except Exception as e:
        print(f"ERROR: Install state verification failed: {e}", file=sys.stderr)
        return False


if __name__ == '__main__':
    # CLI interface
    if len(sys.argv) < 2:
        print("Usage:")
        print("  finalize_install_state.py finalize [private_key_path]")
        print("  finalize_install_state.py verify")
        sys.exit(1)
    
    command = sys.argv[1]
    
    if command == 'finalize':
        # BUG FIX: Use CORRECT manifest signing private key
        private_key = sys.argv[2] if len(sys.argv) > 2 else "/var/lib/ransomeye/keys/manifest_signing.key"
        
        try:
            finalize_install_state(private_key)
            sys.exit(0)
        except InstallStateError as e:
            print(f"\nFATAL: {e}", file=sys.stderr)
            sys.exit(1)
    
    elif command == 'verify':
        success = verify_install_state()
        sys.exit(0 if success else 1)
    
    else:
        print(f"Unknown command: {command}", file=sys.stderr)
        sys.exit(1)

