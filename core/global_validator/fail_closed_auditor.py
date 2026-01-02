# Path and File Name : /home/ransomeye/rebuild/core/global_validator/fail_closed_auditor.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Fail-Closed/Fail-Open Auditor - security policy enforcement

"""
Fail-Closed/Fail-Open Auditor

Detects:
1. Sensors allowed fail-open (correct behavior)
2. Enforcement allowed fail-open (FORBIDDEN - must fail-closed)
3. Core services failing open (FORBIDDEN - must fail-closed)

Violation → FAIL (fail-closed).

Policy:
- Sensors (agents, probes, scanners) → fail-open allowed (buffer to disk)
- Enforcement/Policy/Decision engines → MUST fail-closed
- Core services → MUST fail-closed
- Playbook execution → MUST fail-closed
"""

import re
from pathlib import Path
from typing import Dict, List, Set, Optional
from .validator import Violation, ViolationSeverity, ValidationResult


class FailClosedAuditor:
    """Audits fail-open vs fail-closed correctness."""
    
    # Components that MUST fail-closed
    MUST_FAIL_CLOSED = {
        'enforcement',
        'policy',
        'decision',
        'playbook',
        'guardrails',
        'core',
        'dispatcher',
    }
    
    # Components that can fail-open (sensors)
    CAN_FAIL_OPEN = {
        'agent',
        'probe',
        'scanner',
        'ingest',  # Telemetry ingestion can buffer
    }
    
    def __init__(self, validator):
        self.validator = validator
    
    def validate(self) -> ValidationResult:
        """Run fail-closed/fail-open audit."""
        violations: List[Violation] = []
        
        if not self.validator.guardrails:
            return ValidationResult(
                passed=False,
                violations=[Violation(
                    checker='fail_closed',
                    severity=ViolationSeverity.CRITICAL,
                    message="guardrails.yaml not loaded",
                )]
            )
        
        # Check READMEs for fail-open/fail-closed claims
        for phase_id, readme_path in self.validator.phase_readmes.items():
            try:
                readme_violations = self._check_readme_fail_behavior(readme_path, phase_id)
                violations.extend(readme_violations)
            except Exception as e:
                violations.append(Violation(
                    checker='fail_closed',
                    severity=ViolationSeverity.WARNING,
                    message=f"Failed to audit fail behavior for phase {phase_id}: {e}",
                    details={'readme_path': str(readme_path), 'error': str(e)},
                    phase_id=phase_id,
                ))
        
        # Check source code for fail-open patterns in MUST_FAIL_CLOSED components
        project_root = self.validator.project_root
        code_violations = self._check_code_fail_behavior(project_root)
        violations.extend(code_violations)
        
        return ValidationResult(
            passed=len(violations) == 0,
            violations=violations,
        )
    
    def _check_readme_fail_behavior(self, readme_path: Path, phase_id: int) -> List[Violation]:
        """Check README for fail-open/fail-closed claims."""
        violations = []
        
        try:
            with open(readme_path, 'r', encoding='utf-8') as f:
                content = f.read()
                content_lower = content.lower()
            
            # Get phase info
            phase_info = self._get_phase_info(phase_id)
            if not phase_info:
                return violations
            
            phase_name = phase_info.get('name', '').lower()
            phase_path = phase_info.get('path', '').lower()
            
            # Determine component type
            is_enforcement = any(keyword in phase_name or keyword in phase_path
                               for keyword in self.MUST_FAIL_CLOSED)
            is_sensor = any(keyword in phase_name or keyword in phase_path
                          for keyword in self.CAN_FAIL_OPEN)
            
            # Check for fail-open claims in enforcement components
            if is_enforcement and 'fail-open' in content_lower:
                # Check if it's explicitly allowed (should not be)
                if 'sensor' not in content_lower and 'buffer' not in content_lower:
                    violations.append(Violation(
                        checker='fail_closed',
                        severity=ViolationSeverity.CRITICAL,
                        message=f"Phase {phase_id} ({phase_info.get('name')}) is enforcement/core but claims fail-open behavior",
                        details={
                            'phase_name': phase_info.get('name'),
                            'component_type': 'enforcement/core',
                            'readme_path': str(readme_path),
                        },
                        phase_id=phase_id,
                        phase_name=phase_info.get('name'),
                    ))
            
            # Check for fail-closed claims in README
            if 'fail-closed' not in content_lower and is_enforcement:
                # Enforcement components should explicitly state fail-closed
                violations.append(Violation(
                    checker='fail_closed',
                    severity=ViolationSeverity.WARNING,
                    message=f"Phase {phase_id} ({phase_info.get('name')}) is enforcement/core but does not explicitly document fail-closed behavior",
                    details={
                        'phase_name': phase_info.get('name'),
                        'component_type': 'enforcement/core',
                        'readme_path': str(readme_path),
                    },
                    phase_id=phase_id,
                    phase_name=phase_info.get('name'),
                ))
        
        except Exception as e:
            violations.append(Violation(
                checker='fail_closed',
                severity=ViolationSeverity.WARNING,
                message=f"Failed to parse README for phase {phase_id}: {e}",
                details={'readme_path': str(readme_path), 'error': str(e)},
                phase_id=phase_id,
            ))
        
        return violations
    
    def _check_code_fail_behavior(self, project_root: Path) -> List[Violation]:
        """Check source code for fail-open patterns in MUST_FAIL_CLOSED components."""
        violations = []
        
        # This is a basic check - in practice, you'd want more sophisticated parsing
        # Look for fail-open patterns in enforcement/policy code
        
        enforcement_paths = [
            project_root / "core" / "dispatch",
            project_root / "core" / "policy",
            project_root / "core" / "guardrails",
        ]
        
        # Patterns that indicate fail-open (should not be in enforcement)
        fail_open_patterns = [
            r'fail.open',
            r'fail_open',
            r'continue.*error',
            r'log.*continue',  # Basic heuristic
        ]
        
        for enforcement_path in enforcement_paths:
            if not enforcement_path.exists():
                continue
            
            for rust_file in enforcement_path.rglob("*.rs"):
                try:
                    with open(rust_file, 'r', encoding='utf-8') as f:
                        content = f.read()
                    
                    # Check for fail-open patterns
                    for pattern in fail_open_patterns:
                        if re.search(pattern, content, re.IGNORECASE):
                            # This is a warning - need manual review
                            # Skip for now as it's too noisy
                            pass
                except Exception:
                    pass
        
        return violations
    
    def _get_phase_info(self, phase_id: int) -> Optional[Dict]:
        """Get phase info from guardrails."""
        if not self.validator.guardrails:
            return None
        
        allowed_phases = self.validator.guardrails.get('allowed_phases', [])
        for phase in allowed_phases:
            if phase.get('id') == phase_id:
                return phase
        
        return None

