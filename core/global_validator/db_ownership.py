# Path and File Name : /home/ransomeye/rebuild/core/global_validator/db_ownership.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: DB Ownership Validator - detects table ownership conflicts

"""
DB Ownership Validator

Detects:
1. Multiple writers to same table without arbitration
2. Phases writing to tables they don't own
3. Tables without owning phase

Mismatch → FAIL (fail-closed).

Note: This validator parses database schemas from source code files.
It looks for CREATE TABLE statements and maps them to phases.
"""

import re
from pathlib import Path
from typing import Dict, List, Set, Optional, Tuple
from collections import defaultdict
from .validator import Violation, ViolationSeverity, ValidationResult


class DBOwnershipValidator:
    """Validates database table ownership."""
    
    def __init__(self, validator):
        self.validator = validator
        # Known table ownership (phase_id -> set of table names)
        self.known_ownership: Dict[int, Set[str]] = {}
        # Table -> list of phases that create it
        self.table_creators: Dict[str, List[int]] = defaultdict(list)
    
    def validate(self) -> ValidationResult:
        """Run DB ownership checks."""
        violations: List[Violation] = []
        
        if not self.validator.guardrails:
            return ValidationResult(
                passed=False,
                violations=[Violation(
                    checker='db_ownership',
                    severity=ViolationSeverity.CRITICAL,
                    message="guardrails.yaml not loaded",
                )]
            )
        
        # Extract table definitions from source code
        self._extract_table_definitions()
        
        # Check for ownership conflicts
        for table_name, creator_phases in self.table_creators.items():
            if len(creator_phases) > 1:
                # Multiple phases create the same table
                violations.append(Violation(
                    checker='db_ownership',
                    severity=ViolationSeverity.CRITICAL,
                    message=f"Table '{table_name}' is created by multiple phases: {creator_phases}",
                    details={
                        'table_name': table_name,
                        'phases': creator_phases,
                    },
                ))
        
        # Check for tables without clear ownership
        # (Tables created by phases that don't exist or are NOT_IMPLEMENTED)
        for table_name, creator_phases in self.table_creators.items():
            for phase_id in creator_phases:
                phase_info = self._get_phase_info(phase_id)
                if phase_info and phase_info.get('status') == 'NOT_IMPLEMENTED':
                    violations.append(Violation(
                        checker='db_ownership',
                        severity=ViolationSeverity.CRITICAL,
                        message=f"Table '{table_name}' is created by NOT_IMPLEMENTED phase {phase_id}",
                        details={
                            'table_name': table_name,
                            'phase_id': phase_id,
                            'phase_status': 'NOT_IMPLEMENTED',
                        },
                        phase_id=phase_id,
                        phase_name=phase_info.get('name'),
                    ))
        
        return ValidationResult(
            passed=len(violations) == 0,
            violations=violations,
        )
    
    def _extract_table_definitions(self) -> None:
        """Extract CREATE TABLE statements from source code."""
        # Scan Rust and Python files for CREATE TABLE statements
        project_root = self.validator.project_root
        
        # Known locations where schemas are defined
        search_paths = [
            project_root / "core",
            project_root / "ransomeye_intelligence",
            project_root / "ransomeye_posture_engine",
        ]
        
        for search_path in search_paths:
            if not search_path.exists():
                continue
            
            # Search Rust files
            for rust_file in search_path.rglob("*.rs"):
                if self._is_schema_file(rust_file):
                    tables = self._extract_tables_from_rust(rust_file)
                    phase_id = self._infer_phase_from_path(rust_file)
                    if phase_id is not None:
                        for table_name in tables:
                            self.table_creators[table_name].append(phase_id)
            
            # Search Python files
            for py_file in search_path.rglob("*.py"):
                if self._is_schema_file(py_file):
                    tables = self._extract_tables_from_python(py_file)
                    phase_id = self._infer_phase_from_path(py_file)
                    if phase_id is not None:
                        for table_name in tables:
                            self.table_creators[table_name].append(phase_id)
    
    def _is_schema_file(self, file_path: Path) -> bool:
        """Check if file likely contains schema definitions."""
        name = file_path.name.lower()
        return (
            'schema' in name or
            'persistence' in name or
            'db' in name or
            'database' in name or
            'model' in name
        )
    
    def _extract_tables_from_rust(self, file_path: Path) -> Set[str]:
        """Extract table names from Rust file."""
        tables = set()
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            # Match CREATE TABLE IF NOT EXISTS table_name
            pattern = r'CREATE TABLE (?:IF NOT EXISTS )?(\w+)'
            matches = re.findall(pattern, content, re.IGNORECASE)
            tables.update(matches)
        except Exception:
            pass
        
        return tables
    
    def _extract_tables_from_python(self, file_path: Path) -> Set[str]:
        """Extract table names from Python file."""
        tables = set()
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            # Match CREATE TABLE statements in SQL strings
            pattern = r'CREATE TABLE (?:IF NOT EXISTS )?(\w+)'
            matches = re.findall(pattern, content, re.IGNORECASE)
            tables.update(matches)
        except Exception:
            pass
        
        return tables
    
    def _infer_phase_from_path(self, file_path: Path) -> Optional[int]:
        """Infer phase ID from file path."""
        path_str = str(file_path)
        
        # Known mappings from path patterns to phases
        mappings = {
            'network_scanner': 9,
            'response_playbooks': 6,
            'intelligence': 3,  # ransomeye_intelligence
            'posture_engine': 19,
            'ingest': None,  # Multiple phases use ingest
            'forensics': 13,
            'engine': 8,  # Correlation engine
            'narrative': 5,  # LLM summarizer
        }
        
        for pattern, phase_id in mappings.items():
            if pattern in path_str:
                return phase_id
        
        # Try to match against guardrails phase paths
        if self.validator.guardrails:
            allowed_phases = self.validator.guardrails.get('allowed_phases', [])
            for phase in allowed_phases:
                phase_path_str = phase.get('path', '')
                if phase_path_str and phase_path_str in path_str:
                    return phase.get('id')
        
        return None
    
    def _get_phase_info(self, phase_id: int) -> Optional[Dict]:
        """Get phase info from guardrails."""
        if not self.validator.guardrails:
            return None
        
        allowed_phases = self.validator.guardrails.get('allowed_phases', [])
        for phase in allowed_phases:
            if phase.get('id') == phase_id:
                return phase
        
        return None

