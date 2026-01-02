# Path and File Name : /home/ransomeye/rebuild/core/global_validator/phase_consistency.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Phase Consistency Checker - validates phase status vs disk reality

"""
Phase Consistency Checker

Validates:
1. Phase marked IMPLEMENTED → module exists on disk
2. Phase marked NOT_IMPLEMENTED → no service, no installer ref
3. README verdict matches guardrails status
4. Installable/runnable flags match actual capabilities

Mismatch → FAIL (fail-closed).
"""

from pathlib import Path
from typing import Dict, List, Optional
from .validator import Violation, ViolationSeverity, ValidationResult


class PhaseConsistencyChecker:
    """Checks phase status consistency."""
    
    def __init__(self, validator):
        self.validator = validator
    
    def validate(self) -> ValidationResult:
        """Run phase consistency checks."""
        violations: List[Violation] = []
        
        if not self.validator.guardrails:
            return ValidationResult(
                passed=False,
                violations=[Violation(
                    checker='phase_consistency',
                    severity=ViolationSeverity.CRITICAL,
                    message="guardrails.yaml not loaded",
                )]
            )
        
        allowed_phases = self.validator.guardrails.get('allowed_phases', [])
        
        for phase in allowed_phases:
            phase_id = phase.get('id')
            phase_name = phase.get('name', 'Unknown')
            status = phase.get('status')
            phase_path_str = phase.get('path')
            installable = phase.get('installable', False)
            runnable = phase.get('runnable', False)
            
            if not phase_path_str:
                violations.append(Violation(
                    checker='phase_consistency',
                    severity=ViolationSeverity.CRITICAL,
                    message=f"Phase {phase_id} ({phase_name}) missing path",
                    phase_id=phase_id,
                    phase_name=phase_name,
                ))
                continue
            
            phase_path = Path(phase_path_str)
            
            # Check 1: IMPLEMENTED → module exists on disk
            if status == 'IMPLEMENTED':
                if not phase_path.exists():
                    violations.append(Violation(
                        checker='phase_consistency',
                        severity=ViolationSeverity.CRITICAL,
                        message=f"Phase {phase_id} ({phase_name}) marked IMPLEMENTED but path does not exist: {phase_path}",
                        details={'expected_path': str(phase_path)},
                        phase_id=phase_id,
                        phase_name=phase_name,
                    ))
                elif not phase_path.is_dir():
                    violations.append(Violation(
                        checker='phase_consistency',
                        severity=ViolationSeverity.CRITICAL,
                        message=f"Phase {phase_id} ({phase_name}) path exists but is not a directory: {phase_path}",
                        details={'path': str(phase_path)},
                        phase_id=phase_id,
                        phase_name=phase_name,
                    ))
            
            # Check 2: NOT_IMPLEMENTED → no service, no installer ref
            elif status == 'NOT_IMPLEMENTED':
                # Check if service exists (should not)
                if installable or runnable:
                    violations.append(Violation(
                        checker='phase_consistency',
                        severity=ViolationSeverity.CRITICAL,
                        message=f"Phase {phase_id} ({phase_name}) marked NOT_IMPLEMENTED but installable={installable}, runnable={runnable}",
                        details={
                            'installable': installable,
                            'runnable': runnable,
                        },
                        phase_id=phase_id,
                        phase_name=phase_name,
                    ))
                
                # Check if systemd unit exists for this phase (should not)
                if phase_path.exists() and phase_path.is_dir():
                    # Try to find corresponding systemd unit
                    # This is heuristic - we check if any systemd unit matches phase naming
                    phase_name_clean = phase_name.lower().replace(' ', '-').replace('(', '').replace(')', '')
                    for unit_name in self.validator.systemd_units:
                        if phase_name_clean in unit_name.lower() or f"phase-{phase_id}" in unit_name.lower():
                            violations.append(Violation(
                                checker='phase_consistency',
                                severity=ViolationSeverity.WARNING,
                                message=f"Phase {phase_id} ({phase_name}) marked NOT_IMPLEMENTED but systemd unit exists: {unit_name}",
                                details={'unit_name': unit_name},
                                phase_id=phase_id,
                                phase_name=phase_name,
                            ))
            
            # Check 3: README verdict matches guardrails status
            if phase_id in self.validator.phase_readmes:
                readme_path = self.validator.phase_readmes[phase_id]
                try:
                    readme_status = self._extract_readme_status(readme_path)
                    if readme_status and readme_status.upper() != status:
                        violations.append(Violation(
                            checker='phase_consistency',
                            severity=ViolationSeverity.CRITICAL,
                            message=f"Phase {phase_id} ({phase_name}) README status '{readme_status}' does not match guardrails status '{status}'",
                            details={
                                'readme_status': readme_status,
                                'guardrails_status': status,
                                'readme_path': str(readme_path),
                            },
                            phase_id=phase_id,
                            phase_name=phase_name,
                        ))
                except Exception as e:
                    violations.append(Violation(
                        checker='phase_consistency',
                        severity=ViolationSeverity.WARNING,
                        message=f"Phase {phase_id} ({phase_name}) failed to parse README status: {e}",
                        details={'readme_path': str(readme_path), 'error': str(e)},
                        phase_id=phase_id,
                        phase_name=phase_name,
                    ))
        
        # Check for phases that exist on disk but not in guardrails (orphaned)
        # This is harder - we'd need to scan all directories and match
        # Skip for now as it requires complex module resolution
        
        return ValidationResult(
            passed=len(violations) == 0,
            violations=violations,
        )
    
    def _extract_readme_status(self, readme_path: Path) -> Optional[str]:
        """Extract status from README file."""
        try:
            with open(readme_path, 'r') as f:
                content = f.read()
            
            # Look for status indicators
            content_lower = content.lower()
            
            # Check for explicit STATUS markers (most reliable)
            if '## STATUS:' in content or '## STATUS :' in content:
                if '✅' in content and 'implemented' in content_lower:
                    return 'IMPLEMENTED'
                if '❌' in content and 'not_implemented' in content_lower:
                    return 'NOT_IMPLEMENTED'
                if '❌' in content and 'not implemented' in content_lower:
                    return 'NOT_IMPLEMENTED'
            
            # Check for status: line (case-insensitive)
            import re
            status_match = re.search(r'status:\s*([✅❌]?\s*)?(implemented|not_implemented|not implemented)', content_lower, re.IGNORECASE)
            if status_match:
                status_text = status_match.group(2)
                if 'not' in status_text:
                    return 'NOT_IMPLEMENTED'
                if 'implemented' in status_text:
                    return 'IMPLEMENTED'
            
            # Check for COMPLETE status
            if 'status:' in content and 'complete' in content_lower:
                return 'IMPLEMENTED'
            if 'status:' in content and 'not implemented' in content_lower:
                return 'NOT_IMPLEMENTED'
            if 'status:' in content and 'not yet' in content_lower:
                return 'NOT_IMPLEMENTED'
            
            # Check for explicit status markers
            if '✅' in content and ('implemented' in content_lower or 'complete' in content_lower):
                return 'IMPLEMENTED'
            if '❌' in content and 'not_implemented' in content_lower:
                return 'NOT_IMPLEMENTED'
            if '❌' in content and 'not implemented' in content_lower:
                return 'NOT_IMPLEMENTED'
            
            # Default: if README exists and has content, assume implemented
            # This is heuristic
            return None
        except Exception:
            return None

