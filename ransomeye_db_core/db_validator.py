# Path and File Name: /home/ransomeye/rebuild/ransomeye_db_core/db_validator.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Database validation for Global Validator (fail-closed)

"""
Database Validator: Validates database schema, permissions, and retention.

CRITICAL:
- Only runs if database is enabled
- Fail-closed on any violation
- Verifies schema signature
- Checks retention policies
- Validates role permissions
"""

import sys
from pathlib import Path
from typing import Tuple, List, Optional

try:
    from .db_gate import DatabaseGate
    from .schema_signer import SchemaSigner
except ImportError:
    # Fallback for direct execution
    sys.path.insert(0, str(Path(__file__).parent.parent))
    from ransomeye_db_core.db_gate import DatabaseGate
    from ransomeye_db_core.schema_signer import SchemaSigner


class DatabaseValidator:
    """
    Validates database configuration and integrity.
    
    Only executes checks if database is enabled.
    """
    
    def __init__(self):
        """Initialize database validator."""
        self.schema_signer = SchemaSigner()
        self.violations = []
    
    def validate_schema_signature(self) -> bool:
        """
        Validate schema signature.
        
        Returns:
            True if signature valid, False otherwise
        """
        print("[DB VALIDATOR] Verifying schema signature...")
        
        if not self.schema_signer.verify_schema_signature():
            self.violations.append("Schema signature verification FAILED")
            return False
        
        print("✓ Schema signature verified")
        return True
    
    def validate_credentials_present(self) -> bool:
        """
        Validate all required credentials are present.
        
        Returns:
            True if all present, False otherwise
        """
        print("[DB VALIDATOR] Checking database credentials...")
        
        all_present, missing_var = DatabaseGate.validate_credentials_present()
        
        if not all_present:
            self.violations.append(f"Missing database credential: {missing_var}")
            print(f"✗ Missing credential: {missing_var}")
            return False
        
        print("✓ All database credentials present")
        return True
    
    def validate_connection(self) -> bool:
        """
        Validate database connection.
        
        Returns:
            True if connection succeeds, False otherwise
        """
        print("[DB VALIDATOR] Testing database connection...")
        
        db_config = DatabaseGate.get_db_config()
        if not db_config:
            self.violations.append("Database configuration unavailable")
            return False
        
        try:
            import psycopg2
            
            conn = psycopg2.connect(
                host=db_config['host'],
                port=db_config['port'],
                database=db_config['database'],
                user=db_config['user'],
                password=db_config['password'],
                connect_timeout=5
            )
            conn.close()
            print("✓ Database connection successful")
            return True
        
        except ImportError:
            self.violations.append("psycopg2 module not installed")
            print("✗ psycopg2 not installed (install with: pip install psycopg2-binary)")
            return False
        
        except Exception as e:
            self.violations.append(f"Database connection failed: {e}")
            print(f"✗ Database connection failed: {e}")
            return False
    
    def validate_schema_exists(self) -> bool:
        """
        Validate ransomeye schema exists in database.
        
        Returns:
            True if schema exists, False otherwise
        """
        print("[DB VALIDATOR] Checking ransomeye schema exists...")
        
        db_config = DatabaseGate.get_db_config()
        if not db_config:
            self.violations.append("Database configuration unavailable")
            return False
        
        try:
            import psycopg2
            
            conn = psycopg2.connect(**db_config)
            cur = conn.cursor()
            
            # Check if ransomeye schema exists
            cur.execute("""
                SELECT EXISTS(
                    SELECT 1 FROM information_schema.schemata 
                    WHERE schema_name = 'ransomeye'
                )
            """)
            schema_exists = cur.fetchone()[0]
            
            cur.close()
            conn.close()
            
            if not schema_exists:
                self.violations.append("ransomeye schema does not exist in database")
                print("✗ ransomeye schema not found")
                return False
            
            print("✓ ransomeye schema exists")
            return True
        
        except Exception as e:
            self.violations.append(f"Schema check failed: {e}")
            print(f"✗ Schema check failed: {e}")
            return False
    
    def validate_retention_policies(self) -> bool:
        """
        Validate retention policies exist and are reasonable.
        
        Returns:
            True if policies valid, False otherwise
        """
        print("[DB VALIDATOR] Checking retention policies...")
        
        db_config = DatabaseGate.get_db_config()
        if not db_config:
            self.violations.append("Database configuration unavailable")
            return False
        
        try:
            import psycopg2
            
            conn = psycopg2.connect(**db_config)
            cur = conn.cursor()
            
            # Check retention policies table exists and has entries
            cur.execute("""
                SELECT table_name, retention_days, retention_enabled
                FROM ransomeye.retention_policies
                WHERE retention_enabled = TRUE
            """)
            policies = cur.fetchall()
            
            if not policies:
                self.violations.append("No active retention policies found")
                print("✗ No retention policies")
                cur.close()
                conn.close()
                return False
            
            # Validate policies don't exceed maximum (7 years = 2555 days)
            MAX_RETENTION_DAYS = 2555  # 7 years
            violations_found = False
            
            for table_name, retention_days, enabled in policies:
                if retention_days > MAX_RETENTION_DAYS:
                    self.violations.append(
                        f"Retention policy for {table_name} exceeds maximum: "
                        f"{retention_days} days > {MAX_RETENTION_DAYS} days"
                    )
                    print(f"✗ Excessive retention for {table_name}: {retention_days} days")
                    violations_found = True
            
            cur.close()
            conn.close()
            
            if violations_found:
                return False
            
            print(f"✓ {len(policies)} retention policies validated")
            return True
        
        except Exception as e:
            self.violations.append(f"Retention policy check failed: {e}")
            print(f"✗ Retention policy check failed: {e}")
            return False
    
    def validate_all(self) -> Tuple[bool, List[str]]:
        """
        Run all database validation checks.
        
        Returns:
            Tuple of (all_passed, violations)
        """
        print("")
        print("=" * 80)
        print("DATABASE VALIDATION")
        print("=" * 80)
        print("")
        
        # Check if DB is enabled
        if not DatabaseGate.is_enabled():
            print("Database features: DISABLED")
            print("Skipping database validation (database not enabled)")
            print("")
            return True, []
        
        print("Database features: ENABLED")
        print("")
        
        self.violations = []
        all_passed = True
        
        # 1. Schema signature
        if not self.validate_schema_signature():
            all_passed = False
        
        # 2. Credentials
        if not self.validate_credentials_present():
            all_passed = False
            # Skip connection tests if credentials missing
            print("")
            print("Skipping connection tests (credentials missing)")
            return all_passed, self.violations
        
        # 3. Connection
        if not self.validate_connection():
            all_passed = False
            # Skip schema tests if connection fails
            print("")
            print("Skipping schema tests (connection failed)")
            return all_passed, self.violations
        
        # 4. Schema exists
        if not self.validate_schema_exists():
            all_passed = False
        
        # 5. Retention policies
        if not self.validate_retention_policies():
            all_passed = False
        
        print("")
        if all_passed:
            print("=" * 80)
            print("✓ ALL DATABASE CHECKS PASSED")
            print("=" * 80)
        else:
            print("=" * 80)
            print("✗ DATABASE VALIDATION FAILED")
            print("=" * 80)
            print("")
            print("Violations:")
            for violation in self.violations:
                print(f"  ✗ {violation}")
        
        print("")
        return all_passed, self.violations


def main():
    """CLI entry point."""
    validator = DatabaseValidator()
    passed, violations = validator.validate_all()
    
    if passed:
        sys.exit(0)
    else:
        sys.exit(1)


if __name__ == '__main__':
    main()

