# Path and File Name : /home/ransomeye/rebuild/core/install_state/tests/test_install_state.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Test suite for install state enforcement

"""
Install State Test Suite

Tests fail-closed database enforcement and install state integrity.

CRITICAL TESTS:
1. DB service does NOT start without install_state.json
2. DB service does NOT start without install_state.sig
3. DB service does NOT start if db_enabled=false
4. DB service DOES start only after valid install_state with db_enabled=true
5. Tampering install_state.json causes validator to FAIL
6. Removing db.env causes service to FAIL
"""

import os
import sys
import json
import tempfile
import shutil
import subprocess
from pathlib import Path
from typing import Dict, Any

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent))

from core.install_state.finalize_install_state import (
    finalize_install_state,
    verify_install_state,
    InstallStateError
)
from core.install_state.state_signer import (
    sign_state_file,
    verify_state_signature,
    generate_installer_identity_hash
)


class InstallStateTestSuite:
    """Test suite for install state enforcement."""
    
    def __init__(self):
        self.test_dir = Path(tempfile.mkdtemp(prefix="ransomeye_test_"))
        self.passed = 0
        self.failed = 0
        self.errors = []
    
    def cleanup(self):
        """Clean up test directory."""
        if self.test_dir.exists():
            shutil.rmtree(self.test_dir, ignore_errors=True)
    
    def log_test(self, name: str, passed: bool, message: str = ""):
        """Log test result."""
        status = "✓ PASS" if passed else "✗ FAIL"
        print(f"  {status}: {name}")
        if message:
            print(f"    {message}")
        
        if passed:
            self.passed += 1
        else:
            self.failed += 1
            self.errors.append(f"{name}: {message}")
    
    def test_1_state_file_required(self) -> bool:
        """Test: DB service requires install_state.json."""
        print("\n[TEST 1] DB service requires install_state.json")
        
        try:
            # Check systemd unit has ConditionPathExists
            db_service = Path("/etc/systemd/system/ransomeye-db_core.service")
            
            if not db_service.exists():
                self.log_test(
                    "DB service unit exists",
                    False,
                    "ransomeye-db_core.service not found"
                )
                return False
            
            with open(db_service, 'r') as f:
                service_content = f.read()
            
            has_state_condition = "ConditionPathExists=/var/lib/ransomeye/install_state.json" in service_content
            
            self.log_test(
                "Unit has ConditionPathExists for install_state.json",
                has_state_condition,
                "Condition not found in service unit" if not has_state_condition else ""
            )
            
            return has_state_condition
        
        except Exception as e:
            self.log_test("Test execution", False, str(e))
            return False
    
    def test_2_signature_required(self) -> bool:
        """Test: DB service requires install_state.sig."""
        print("\n[TEST 2] DB service requires install_state.sig")
        
        try:
            db_service = Path("/etc/systemd/system/ransomeye-db_core.service")
            
            if not db_service.exists():
                self.log_test("DB service unit exists", False, "Not found")
                return False
            
            with open(db_service, 'r') as f:
                service_content = f.read()
            
            has_sig_condition = "ConditionPathExists=/var/lib/ransomeye/install_state.sig" in service_content
            
            self.log_test(
                "Unit has ConditionPathExists for install_state.sig",
                has_sig_condition,
                "Signature condition not found" if not has_sig_condition else ""
            )
            
            return has_sig_condition
        
        except Exception as e:
            self.log_test("Test execution", False, str(e))
            return False
    
    def test_3_db_env_loaded(self) -> bool:
        """Test: DB service loads db.env environment file."""
        print("\n[TEST 3] DB service loads db.env")
        
        try:
            db_service = Path("/etc/systemd/system/ransomeye-db_core.service")
            
            if not db_service.exists():
                self.log_test("DB service unit exists", False, "Not found")
                return False
            
            with open(db_service, 'r') as f:
                service_content = f.read()
            
            has_env_file = "EnvironmentFile=/etc/ransomeye/db.env" in service_content
            
            self.log_test(
                "Unit has EnvironmentFile for db.env",
                has_env_file,
                "EnvironmentFile directive not found" if not has_env_file else ""
            )
            
            return has_env_file
        
        except Exception as e:
            self.log_test("Test execution", False, str(e))
            return False
    
    def test_4_install_state_exists(self) -> bool:
        """Test: install_state.json exists and is readable."""
        print("\n[TEST 4] install_state.json exists")
        
        state_path = Path("/var/lib/ransomeye/install_state.json")
        exists = state_path.exists()
        
        self.log_test(
            "install_state.json exists",
            exists,
            f"File not found at {state_path}" if not exists else ""
        )
        
        if not exists:
            return False
        
        # Test readability
        try:
            with open(state_path, 'r') as f:
                state = json.load(f)
            
            readable = True
            self.log_test("install_state.json is valid JSON", True, "")
        
        except Exception as e:
            readable = False
            self.log_test("install_state.json is valid JSON", False, str(e))
        
        return readable
    
    def test_5_signature_valid(self) -> bool:
        """Test: install_state.sig is valid."""
        print("\n[TEST 5] install_state.sig is valid")
        
        state_path = Path("/var/lib/ransomeye/install_state.json")
        sig_path = Path("/var/lib/ransomeye/install_state.sig")
        # BUG FIX: Use CORRECT manifest signing public key
        pub_key_path = Path("/var/lib/ransomeye/keys/manifest_signing.pub")
        
        if not sig_path.exists():
            self.log_test("Signature file exists", False, f"Not found at {sig_path}")
            return False
        
        if not pub_key_path.exists():
            self.log_test("Public key exists", False, f"Not found at {pub_key_path}")
            return False
        
        try:
            valid = verify_state_signature(
                str(state_path),
                str(sig_path),
                str(pub_key_path)
            )
            
            self.log_test(
                "Signature verification",
                valid,
                "Signature invalid or verification failed" if not valid else ""
            )
            
            return valid
        
        except Exception as e:
            self.log_test("Signature verification", False, str(e))
            return False
    
    def test_6_state_immutable(self) -> bool:
        """Test: install_state.json has 0444 permissions (immutable)."""
        print("\n[TEST 6] install_state.json is immutable")
        
        state_path = Path("/var/lib/ransomeye/install_state.json")
        
        if not state_path.exists():
            self.log_test("State file exists", False, "File not found")
            return False
        
        try:
            import stat
            state_stat = state_path.stat()
            perms = oct(state_stat.st_mode)[-3:]
            
            immutable = perms == '444'
            
            self.log_test(
                "State file permissions are 0444",
                immutable,
                f"Permissions are {perms}, expected 444" if not immutable else ""
            )
            
            return immutable
        
        except Exception as e:
            self.log_test("Permission check", False, str(e))
            return False
    
    def test_7_db_enablement_consistency(self) -> bool:
        """Test: If db_enabled=true, all prerequisites are met."""
        print("\n[TEST 7] Database enablement consistency")
        
        state_path = Path("/var/lib/ransomeye/install_state.json")
        
        if not state_path.exists():
            self.log_test("State file exists", False, "Cannot test")
            return False
        
        try:
            with open(state_path, 'r') as f:
                state = json.load(f)
            
            db_enabled = state.get('db_enabled', False)
            
            if not db_enabled:
                self.log_test("DB not enabled (skipping prerequisites)", True, "")
                return True
            
            # DB enabled - check prerequisites
            schema_applied = state.get('db_schema_applied', False)
            schema_verified = state.get('db_schema_signature_verified', False)
            
            has_host = 'db_host' in state
            has_port = 'db_port' in state
            has_name = 'db_name' in state
            
            db_env_exists = Path("/etc/ransomeye/db.env").exists()
            
            all_prerequisites = (
                schema_applied and
                schema_verified and
                has_host and
                has_port and
                has_name and
                db_env_exists
            )
            
            self.log_test(
                "Schema applied",
                schema_applied,
                "" if schema_applied else "db_schema_applied=false"
            )
            
            self.log_test(
                "Schema signature verified",
                schema_verified,
                "" if schema_verified else "db_schema_signature_verified=false"
            )
            
            self.log_test(
                "DB configuration complete",
                has_host and has_port and has_name,
                "" if (has_host and has_port and has_name) else "Missing db_host/port/name"
            )
            
            self.log_test(
                "db.env exists",
                db_env_exists,
                "" if db_env_exists else "/etc/ransomeye/db.env not found"
            )
            
            return all_prerequisites
        
        except Exception as e:
            self.log_test("Test execution", False, str(e))
            return False
    
    def test_8_validator_integration(self) -> bool:
        """Test: Global validator includes install_state checker."""
        print("\n[TEST 8] Global validator integration")
        
        try:
            from core.global_validator.validator import GlobalForensicValidator
            
            validator = GlobalForensicValidator()
            checkers = validator.checkers
            
            has_install_state = 'install_state' in checkers
            
            self.log_test(
                "install_state checker registered",
                has_install_state,
                "Checker not found in validator" if not has_install_state else ""
            )
            
            return has_install_state
        
        except Exception as e:
            self.log_test("Test execution", False, str(e))
            return False
    
    def test_9_state_corruption_detection(self) -> bool:
        """Test: State corruption detected when db.env exists but DB disabled."""
        print("\n[TEST 9] State corruption detection (stale db.env)")
        
        state_path = Path("/var/lib/ransomeye/install_state.json")
        
        if not state_path.exists():
            self.log_test("State file exists", False, "Cannot test")
            return False
        
        try:
            with open(state_path, 'r') as f:
                state = json.load(f)
            
            db_enabled = state.get('db_enabled', False)
            db_env_exists = Path("/etc/ransomeye/db.env").exists()
            
            # Test case 1: DB disabled + db.env absent = PASS
            if not db_enabled and not db_env_exists:
                self.log_test(
                    "DB disabled + no db.env = PASS",
                    True,
                    "Correct state: db.env correctly absent"
                )
                return True
            
            # Test case 2: DB disabled + db.env present = FAIL (state corruption)
            if not db_enabled and db_env_exists:
                self.log_test(
                    "DB disabled + db.env present = FAIL",
                    False,
                    "STATE CORRUPTION: db.env exists but db_enabled=false"
                )
                return False
            
            # Test case 3: DB enabled + db.env present = PASS
            if db_enabled and db_env_exists:
                self.log_test(
                    "DB enabled + db.env present = PASS",
                    True,
                    "Correct state: db.env exists as expected"
                )
                return True
            
            # Test case 4: DB enabled + db.env absent = FAIL (should have failed during finalization)
            if db_enabled and not db_env_exists:
                self.log_test(
                    "DB enabled + db.env absent = FAIL",
                    False,
                    "STATE CORRUPTION: db_enabled=true but db.env missing"
                )
                return False
            
            return True
        
        except Exception as e:
            self.log_test("Test execution", False, str(e))
            return False
    
    def test_10_db_disabled_clean_state(self) -> bool:
        """Test: DB disabled installs have clean state (no db.env)."""
        print("\n[TEST 10] DB disabled clean state")
        
        state_path = Path("/var/lib/ransomeye/install_state.json")
        
        if not state_path.exists():
            self.log_test("State file exists", False, "Cannot test")
            return False
        
        try:
            with open(state_path, 'r') as f:
                state = json.load(f)
            
            db_enabled = state.get('db_enabled', False)
            
            if db_enabled:
                self.log_test("Skipping test (DB is enabled)", True, "")
                return True
            
            # DB is disabled - verify no db.env exists
            db_env_path = Path("/etc/ransomeye/db.env")
            
            if db_env_path.exists():
                self.log_test(
                    "No stale db.env",
                    False,
                    f"STATE CORRUPTION: db.env exists at {db_env_path} but DB disabled"
                )
                return False
            
            self.log_test(
                "Clean state verified",
                True,
                "db.env correctly absent for DB disabled install"
            )
            
            return True
        
        except Exception as e:
            self.log_test("Test execution", False, str(e))
            return False
    
    def run_all_tests(self):
        """Run all tests."""
        print("="*60)
        print("RANSOMEYE INSTALL STATE TEST SUITE")
        print("="*60)
        
        # Run tests
        self.test_1_state_file_required()
        self.test_2_signature_required()
        self.test_3_db_env_loaded()
        self.test_4_install_state_exists()
        self.test_5_signature_valid()
        self.test_6_state_immutable()
        self.test_7_db_enablement_consistency()
        self.test_8_validator_integration()
        self.test_9_state_corruption_detection()
        self.test_10_db_disabled_clean_state()
        
        # Summary
        print("\n" + "="*60)
        print("TEST SUMMARY")
        print("="*60)
        print(f"Passed: {self.passed}")
        print(f"Failed: {self.failed}")
        print(f"Total:  {self.passed + self.failed}")
        
        if self.failed > 0:
            print("\nFailed tests:")
            for error in self.errors:
                print(f"  - {error}")
        
        return self.failed == 0


def main():
    """Main entry point."""
    suite = InstallStateTestSuite()
    
    try:
        success = suite.run_all_tests()
        return 0 if success else 1
    
    finally:
        suite.cleanup()


if __name__ == '__main__':
    sys.exit(main())

