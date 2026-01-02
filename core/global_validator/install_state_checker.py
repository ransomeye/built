# Path and File Name : /home/ransomeye/rebuild/core/global_validator/install_state_checker.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Install state verification checker for global validator

"""
Install State Checker

Verifies install_state.json integrity and enforces fail-closed database requirements.

CRITICAL ARCHITECTURE RULE: DB IS MANDATORY FOR RANSOMEYE CORE

CRITICAL CHECKS:
1. install_state.json exists
2. install_state.sig exists and valid
3. State file permissions are 0444 (immutable)
4. Database validation (MANDATORY):
   - db.enabled MUST be true
   - db.mode MUST be 'standalone' or 'ha'
   - db.schema_applied MUST be true
   - db.schema_signature_verified MUST be true
   - db.env MUST exist with correct permissions (0600)
   - db.host, db.port, db.name MUST be present
5. State matches manifest hash

FAIL-CLOSED: Any failure is a CRITICAL violation.
"""

import json
import sys
import stat
from pathlib import Path
from typing import List, Dict, Any

from .validator import ValidationResult, Violation, ViolationSeverity


# Constants
INSTALL_STATE_PATH = Path("/var/lib/ransomeye/install_state.json")
INSTALL_STATE_SIG_PATH = Path("/var/lib/ransomeye/install_state.sig")
DB_ENV_PATH = Path("/etc/ransomeye/db.env")
MANIFEST_PATH = Path("/var/lib/ransomeye/install_manifest.json")


class InstallStateChecker:
    """Validates install state integrity and fail-closed database enablement."""
    
    def __init__(self, validator_instance):
        """
        Initialize install state checker.
        
        Args:
            validator_instance: GlobalForensicValidator instance
        """
        self.validator = validator_instance
        self.violations: List[Violation] = []
    
    def _add_violation(self, severity: ViolationSeverity, message: str, details: Dict = None):
        """Add a violation."""
        self.violations.append(Violation(
            checker='install_state',
            severity=severity,
            message=message,
            details=details or {}
        ))
    
    def validate(self) -> ValidationResult:
        """
        Run install state validation.
        
        Returns:
            ValidationResult with violations
        """
        self.violations.clear()
        
        print("[VALIDATOR] Checking install state integrity...")
        
        # Check 1: install_state.json exists
        if not INSTALL_STATE_PATH.exists():
            self._add_violation(
                ViolationSeverity.CRITICAL,
                f"install_state.json not found at {INSTALL_STATE_PATH}",
                {
                    'expected_path': str(INSTALL_STATE_PATH),
                    'reason': 'Installation incomplete or state file missing'
                }
            )
            return ValidationResult(passed=False, violations=self.violations)
        
        print(f"  ✓ install_state.json exists")
        
        # Check 2: install_state.sig exists
        if not INSTALL_STATE_SIG_PATH.exists():
            self._add_violation(
                ViolationSeverity.CRITICAL,
                f"install_state.sig not found at {INSTALL_STATE_SIG_PATH}",
                {
                    'expected_path': str(INSTALL_STATE_SIG_PATH),
                    'reason': 'State file not signed - installation incomplete or tampering'
                }
            )
            return ValidationResult(passed=False, violations=self.violations)
        
        print(f"  ✓ install_state.sig exists")
        
        # Check 3: Verify signature
        try:
            sys.path.insert(0, str(Path(__file__).parent.parent))
            from install_state.state_signer import verify_state_signature
            
            # BUG FIX: Use CORRECT manifest signing public key
            public_key_path = "/var/lib/ransomeye/keys/manifest_signing.pub"
            
            if not Path(public_key_path).exists():
                self._add_violation(
                    ViolationSeverity.CRITICAL,
                    f"Public key not found at {public_key_path}",
                    {'expected_path': public_key_path}
                )
                return ValidationResult(passed=False, violations=self.violations)
            
            signature_valid = verify_state_signature(
                str(INSTALL_STATE_PATH),
                str(INSTALL_STATE_SIG_PATH),
                public_key_path
            )
            
            if not signature_valid:
                self._add_violation(
                    ViolationSeverity.CRITICAL,
                    "install_state.json signature verification FAILED",
                    {
                        'state_file': str(INSTALL_STATE_PATH),
                        'signature_file': str(INSTALL_STATE_SIG_PATH),
                        'reason': 'State file may be tampered or signature corrupted'
                    }
                )
                return ValidationResult(passed=False, violations=self.violations)
            
            print(f"  ✓ install_state.json signature valid")
        
        except Exception as e:
            self._add_violation(
                ViolationSeverity.CRITICAL,
                f"Failed to verify install_state.json signature: {e}",
                {'exception': str(e)}
            )
            return ValidationResult(passed=False, violations=self.violations)
        
        # Check 4: File permissions (should be 0444 - immutable)
        try:
            state_stat = INSTALL_STATE_PATH.stat()
            state_mode = stat.filemode(state_stat.st_mode)
            state_perms = oct(state_stat.st_mode)[-3:]
            
            if state_perms != '444':
                self._add_violation(
                    ViolationSeverity.WARNING,
                    f"install_state.json permissions are {state_perms}, expected 444 (immutable)",
                    {
                        'current_permissions': state_perms,
                        'expected_permissions': '444',
                        'reason': 'State file should be immutable'
                    }
                )
        
        except Exception as e:
            self._add_violation(
                ViolationSeverity.WARNING,
                f"Could not check install_state.json permissions: {e}",
                {'exception': str(e)}
            )
        
        # Check 5: Load and validate state content
        try:
            with open(INSTALL_STATE_PATH, 'r') as f:
                install_state = json.load(f)
            
            print(f"  ✓ install_state.json loaded successfully")
        
        except Exception as e:
            self._add_violation(
                ViolationSeverity.CRITICAL,
                f"Failed to parse install_state.json: {e}",
                {'exception': str(e)}
            )
            return ValidationResult(passed=False, violations=self.violations)
        
        # Check 6: Validate required fields (UPDATED FOR STRUCTURED DB MODEL)
        required_fields = [
            'state_version',
            'install_timestamp',
            'db',  # ← STRUCTURED DB SECTION
            'enabled_modules',
            'manifest_hash',
            'installer_identity_hash',
            'signer_fingerprint'
        ]
        
        missing_fields = [f for f in required_fields if f not in install_state]
        
        if missing_fields:
            self._add_violation(
                ViolationSeverity.CRITICAL,
                f"install_state.json missing required fields: {missing_fields}",
                {'missing_fields': missing_fields}
            )
            return ValidationResult(passed=False, violations=self.violations)
        
        # Validate db section structure
        db_section = install_state.get('db', {})
        if not isinstance(db_section, dict):
            self._add_violation(
                ViolationSeverity.CRITICAL,
                "install_state.json 'db' field must be an object",
                {'db_type': type(db_section).__name__}
            )
            return ValidationResult(passed=False, violations=self.violations)
        
        # Validate db section required fields
        required_db_fields = ['enabled', 'mode', 'host', 'port', 'name', 'schema_applied', 'schema_signature_verified']
        missing_db_fields = [f for f in required_db_fields if f not in db_section]
        
        if missing_db_fields:
            self._add_violation(
                ViolationSeverity.CRITICAL,
                f"install_state.json db section missing required fields: {missing_db_fields}",
                {'missing_fields': missing_db_fields}
            )
            return ValidationResult(passed=False, violations=self.violations)
        
        print(f"  ✓ All required fields present (structured db model)")
        
        # Check 7: Database validation (MANDATORY - FAIL-CLOSED)
        # CRITICAL ARCHITECTURE RULE: DB IS MANDATORY FOR RANSOMEYE CORE
        db_section = install_state.get('db', {})
        db_enabled = db_section.get('enabled', False)
        db_mode = db_section.get('mode')
        
        # FAIL-CLOSED: DB MUST be enabled for Core
        if not db_enabled:
            self._add_violation(
                ViolationSeverity.CRITICAL,
                "FAIL-CLOSED: db.enabled=false (INVARIANT VIOLATION: DB is MANDATORY for Core)",
                {
                    'db_enabled': False,
                    'reason': 'Database must always be enabled for RansomEye Core'
                }
            )
            # Continue validation to report all violations
        
        print(f"  ℹ Database validation (MANDATORY, mode={db_mode})...")
        
        # FAIL-CLOSED: DB mode must be valid
        if db_mode not in ['standalone', 'ha']:
            self._add_violation(
                ViolationSeverity.CRITICAL,
                f"FAIL-CLOSED: db.mode='{db_mode}' (must be 'standalone' or 'ha')",
                {
                    'db_mode': db_mode,
                    'reason': 'Database mode must be explicitly set to standalone or ha'
                }
            )
        
        # FAIL-CLOSED: Schema must be applied
        if not db_section.get('schema_applied', False):
            self._add_violation(
                ViolationSeverity.CRITICAL,
                "FAIL-CLOSED: db.schema_applied=false (schema must be applied)",
                {
                    'db_schema_applied': False,
                    'reason': 'Database schema must be applied for Core'
                }
            )
        
        # FAIL-CLOSED: Schema signature must be verified
        if not db_section.get('schema_signature_verified', False):
            self._add_violation(
                ViolationSeverity.CRITICAL,
                "FAIL-CLOSED: db.schema_signature_verified=false (signature must be verified)",
                {
                    'db_schema_signature_verified': False,
                    'reason': 'Database schema signature must be verified'
                }
            )
        
        # FAIL-CLOSED: db.env must exist
        if not DB_ENV_PATH.exists():
            self._add_violation(
                ViolationSeverity.CRITICAL,
                f"FAIL-CLOSED: {DB_ENV_PATH} not found (MANDATORY for Core)",
                {
                    'db_env_path': str(DB_ENV_PATH),
                    'reason': 'Database environment file required for Core'
                }
            )
        else:
            print(f"    ✓ db.env exists")
            
            # Check db.env permissions (should be 0600)
            try:
                env_stat = DB_ENV_PATH.stat()
                env_perms = oct(env_stat.st_mode)[-3:]
                
                if env_perms != '600':
                    self._add_violation(
                        ViolationSeverity.WARNING,
                        f"db.env permissions are {env_perms}, expected 600 (secure)",
                        {
                            'current_permissions': env_perms,
                            'expected_permissions': '600',
                            'reason': 'Database credentials should be secured'
                        }
                    )
            
            except Exception as e:
                self._add_violation(
                    ViolationSeverity.WARNING,
                    f"Could not check db.env permissions: {e}",
                    {'exception': str(e)}
                )
            
            # CRITICAL ARCHITECTURE LOCK: Verify DB credentials are gagan:gagan (FIXED)
            # Parse db.env and verify credentials
            try:
                db_env_config = {}
                with open(DB_ENV_PATH, 'r') as f:
                    for line in f:
                        line = line.strip()
                        if line and not line.startswith('#') and '=' in line:
                            key, value = line.split('=', 1)
                            db_env_config[key.strip()] = value.strip()
                
                db_user = db_env_config.get('DB_USER')
                db_pass = db_env_config.get('DB_PASS')
                
                # FAIL-CLOSED: Credentials MUST be gagan:gagan
                if db_user != 'gagan':
                    self._add_violation(
                        ViolationSeverity.CRITICAL,
                        f"FAIL-CLOSED: DB_USER must be 'gagan' (found: '{db_user}')",
                        {
                            'expected_user': 'gagan',
                            'actual_user': db_user,
                            'reason': 'ARCHITECTURAL LOCK: DB credentials are FIXED to gagan:gagan (non-overridable)'
                        }
                    )
                else:
                    print(f"    ✓ DB_USER verified: gagan (architectural lock)")
                
                if db_pass != 'gagan':
                    self._add_violation(
                        ViolationSeverity.CRITICAL,
                        f"FAIL-CLOSED: DB_PASS must be 'gagan'",
                        {
                            'expected_pass': 'gagan',
                            'reason': 'ARCHITECTURAL LOCK: DB credentials are FIXED to gagan:gagan (non-overridable)'
                        }
                    )
                else:
                    print(f"    ✓ DB_PASS verified: gagan (architectural lock)")
            
            except Exception as e:
                self._add_violation(
                    ViolationSeverity.CRITICAL,
                    f"FAIL-CLOSED: Could not verify DB credentials from db.env: {e}",
                    {'exception': str(e)}
                )
        
        # Check required DB connection fields
        db_host = db_section.get('host')
        db_port = db_section.get('port')
        db_name = db_section.get('name')
        
        missing_connection_fields = []
        if not db_host or db_host is None:
            missing_connection_fields.append('host')
        if not db_port or db_port is None:
            missing_connection_fields.append('port')
        if not db_name or db_name is None:
            missing_connection_fields.append('name')
        
        if missing_connection_fields:
            self._add_violation(
                ViolationSeverity.CRITICAL,
                f"FAIL-CLOSED: Missing DB connection fields: {missing_connection_fields}",
                {
                    'missing_fields': missing_connection_fields,
                    'reason': 'Database configuration incomplete'
                }
            )
        else:
            print(f"    ✓ Database configuration complete: {db_host}:{db_port}/{db_name}")
        
        # Check 8: Verify manifest hash matches (if manifest exists)
        if MANIFEST_PATH.exists():
            try:
                import hashlib
                
                with open(MANIFEST_PATH, 'rb') as f:
                    manifest_data = f.read()
                
                computed_hash = hashlib.sha256(manifest_data).hexdigest()
                state_manifest_hash = install_state.get('manifest_hash', '')
                
                if computed_hash != state_manifest_hash:
                    self._add_violation(
                        ViolationSeverity.CRITICAL,
                        "Manifest hash mismatch - manifest may be modified after installation",
                        {
                            'state_manifest_hash': state_manifest_hash,
                            'computed_manifest_hash': computed_hash,
                            'reason': 'Manifest modified after install_state finalization'
                        }
                    )
                else:
                    print(f"  ✓ Manifest hash matches")
            
            except Exception as e:
                self._add_violation(
                    ViolationSeverity.WARNING,
                    f"Could not verify manifest hash: {e}",
                    {'exception': str(e)}
                )
        
        # Determine result
        critical_violations = [
            v for v in self.violations
            if v.severity == ViolationSeverity.CRITICAL
        ]
        
        passed = len(critical_violations) == 0
        
        if passed:
            print(f"[VALIDATOR] ✓✓✓ Install state validation PASSED")
        else:
            print(f"[VALIDATOR] ✗✗✗ Install state validation FAILED ({len(critical_violations)} critical)")
        
        return ValidationResult(passed=passed, violations=self.violations)


if __name__ == '__main__':
    # Standalone test
    print("Install State Checker - Standalone Test")
    print("="*60)
    
    class DummyValidator:
        pass
    
    checker = InstallStateChecker(DummyValidator())
    result = checker.validate()
    
    print(f"\nResult: {'PASSED' if result.passed else 'FAILED'}")
    print(f"Violations: {len(result.violations)}")
    
    for v in result.violations:
        print(f"  [{v.severity.value.upper()}] {v.message}")
    
    sys.exit(0 if result.passed else 1)

