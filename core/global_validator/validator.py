# Path and File Name : /home/ransomeye/rebuild/core/global_validator/validator.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Main validator orchestrator - reads all artifacts and validates consistency

"""
Global Forensic Consistency Validator - Main Entry Point

Reads:
- guardrails.yaml
- install_manifest.json
- systemd units
- DB schemas (parsed from code)
- Phase READMEs

Validates:
- Phase status consistency
- DB ownership
- systemd/installer consistency
- Fail-open vs fail-closed
- AI/ML claims

FAILS on any violation (fail-closed).
"""

import json
import yaml
import sys
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple
from dataclasses import dataclass, field
from enum import Enum

PROJECT_ROOT = Path("/home/ransomeye/rebuild")
GUARDRAILS_PATH = PROJECT_ROOT / "core/guardrails/guardrails.yaml"
INSTALL_MANIFEST_PATH = Path("/var/lib/ransomeye/install_manifest.json")
SYSTEMD_DIR = PROJECT_ROOT / "systemd"
README_DIR = PROJECT_ROOT / "docs/readme"


class ViolationSeverity(Enum):
    """Violation severity levels."""
    CRITICAL = "critical"  # Fail-closed violation
    WARNING = "warning"    # Consistency issue
    INFO = "info"          # Informational


@dataclass
class Violation:
    """Represents a consistency violation."""
    checker: str
    severity: ViolationSeverity
    message: str
    details: Dict = field(default_factory=dict)
    phase_id: Optional[int] = None
    phase_name: Optional[str] = None


@dataclass
class ValidationResult:
    """Validation result with violations."""
    passed: bool
    violations: List[Violation] = field(default_factory=list)
    summary: Dict = field(default_factory=dict)


class GlobalForensicValidator:
    """Main validator orchestrator."""
    
    def __init__(self, project_root: Path = PROJECT_ROOT):
        self.project_root = Path(project_root)
        self.guardrails_path = self.project_root / "core/guardrails/guardrails.yaml"
        self.install_manifest_path = INSTALL_MANIFEST_PATH
        self.systemd_dir = self.project_root / "systemd"
        self.readme_dir = self.project_root / "docs/readme"
        self.violations: List[Violation] = []
        
        # Loaded artifacts
        self.guardrails: Optional[Dict] = None
        self.install_manifest: Optional[Dict] = None
        self.systemd_units: List[str] = []
        self.phase_readmes: Dict[int, Path] = {}
        
        # Import checkers (lazy import to avoid circular dependencies)
        self._checkers = None
        
        # Import manifest verifier
        self._manifest_verifier = None
    
    def _get_checkers(self):
        """Lazy load checkers."""
        if self._checkers is None:
            from .phase_consistency import PhaseConsistencyChecker
            from .db_ownership import DBOwnershipValidator
            from .systemd_installer import SystemdInstallerValidator
            from .fail_closed_auditor import FailClosedAuditor
            from .ai_ml_claims import AIMLClaimValidator
            from .install_state_checker import InstallStateChecker
            
            self._checkers = {
                'phase_consistency': PhaseConsistencyChecker(self),
                'db_ownership': DBOwnershipValidator(self),
                'systemd_installer': SystemdInstallerValidator(self),
                'fail_closed': FailClosedAuditor(self),
                'ai_ml_claims': AIMLClaimValidator(self),
                'install_state': InstallStateChecker(self),
            }
        return self._checkers
    
    @property
    def checkers(self):
        """Get checkers (lazy loaded)."""
        return self._get_checkers()
    
    def _verify_manifest_signature(self) -> bool:
        """
        Verify manifest signature BEFORE loading manifest content.
        
        FAIL-CLOSED: Returns False on any verification failure.
        
        Returns:
            True if signature valid, False otherwise
        """
        if self._manifest_verifier is None:
            try:
                from .manifest_verifier import ManifestVerifier
                self._manifest_verifier = ManifestVerifier()
            except Exception as e:
                self._add_violation(
                    'manifest_signature',
                    ViolationSeverity.CRITICAL,
                    f"Failed to import manifest verifier: {e}",
                )
                return False
        
        is_valid, error_message = self._manifest_verifier.verify_signature()
        
        if not is_valid:
            self._add_violation(
                'manifest_signature',
                ViolationSeverity.CRITICAL,
                f"Manifest signature verification FAILED: {error_message}",
                details={
                    'manifest_path': str(self.install_manifest_path),
                    'signature_path': str(self._manifest_verifier.SIGNATURE_PATH),
                    'public_key_path': str(self._manifest_verifier.PUBLIC_KEY_PATH),
                }
            )
            return False
        
        return True
    
    def load_artifacts(self) -> None:
        """Load all artifacts for validation."""
        # Load guardrails.yaml
        if not self.guardrails_path.exists():
            self._add_violation(
                'validator',
                ViolationSeverity.CRITICAL,
                f"guardrails.yaml not found at {self.guardrails_path}",
            )
            return
        
        try:
            with open(self.guardrails_path, 'r') as f:
                self.guardrails = yaml.safe_load(f)
        except Exception as e:
            self._add_violation(
                'validator',
                ViolationSeverity.CRITICAL,
                f"Failed to load guardrails.yaml: {e}",
            )
            return
        
        # Load install manifest (optional - may not exist)
        # CRITICAL: Verify signature BEFORE loading content
        if self.install_manifest_path.exists():
            # STEP 1: Verify signature (FAIL-CLOSED)
            if not self._verify_manifest_signature():
                # Signature verification failed - violation already added
                # DO NOT load manifest content
                return
            
            # STEP 2: Load manifest content (signature verified)
            try:
                with open(self.install_manifest_path, 'r') as f:
                    self.install_manifest = json.load(f)
            except Exception as e:
                self._add_violation(
                    'validator',
                    ViolationSeverity.CRITICAL,
                    f"Failed to load install_manifest.json: {e}",
                )
        
        # Load systemd units
        if self.systemd_dir.exists():
            self.systemd_units = [
                f.name for f in self.systemd_dir.iterdir()
                if f.is_file() and f.suffix == '.service'
            ]
        
        # Load phase READMEs
        if self.readme_dir.exists():
            for readme_file in self.readme_dir.glob("*_readme.md"):
                # Extract phase number from filename (e.g., "00_Guardrails_readme.md" -> 0)
                try:
                    phase_str = readme_file.stem.split('_')[0]
                    phase_id = int(phase_str)
                    self.phase_readmes[phase_id] = readme_file
                except (ValueError, IndexError):
                    continue
    
    def validate_all(self) -> ValidationResult:
        """Run all validators."""
        self.violations.clear()
        
        # Load artifacts first
        self.load_artifacts()
        
        if not self.guardrails:
            return ValidationResult(
                passed=False,
                violations=self.violations,
                summary={'error': 'guardrails.yaml not loaded'}
            )
        
        # Run all checkers
        for checker_name, checker in self.checkers.items():
            try:
                result = checker.validate()
                if not result.passed:
                    self.violations.extend(result.violations)
            except Exception as e:
                self._add_violation(
                    checker_name,
                    ViolationSeverity.CRITICAL,
                    f"Checker {checker_name} failed with exception: {e}",
                    details={'exception_type': type(e).__name__}
                )
        
        # Determine if validation passed (no critical violations)
        critical_violations = [
            v for v in self.violations
            if v.severity == ViolationSeverity.CRITICAL
        ]
        
        passed = len(critical_violations) == 0
        
        summary = {
            'total_violations': len(self.violations),
            'critical_violations': len(critical_violations),
            'warning_violations': len([v for v in self.violations if v.severity == ViolationSeverity.WARNING]),
            'info_violations': len([v for v in self.violations if v.severity == ViolationSeverity.INFO]),
            'checkers_run': list(self.checkers.keys()),
        }
        
        return ValidationResult(
            passed=passed,
            violations=self.violations,
            summary=summary
        )
    
    def _add_violation(self, checker: str, severity: ViolationSeverity, message: str,
                      details: Dict = None, phase_id: Optional[int] = None,
                      phase_name: Optional[str] = None) -> None:
        """Add a violation."""
        self.violations.append(Violation(
            checker=checker,
            severity=severity,
            message=message,
            details=details or {},
            phase_id=phase_id,
            phase_name=phase_name,
        ))
    
    def generate_report(self, result: ValidationResult) -> Dict:
        """Generate JSON report."""
        return {
            'validation_timestamp': self._timestamp(),
            'passed': result.passed,
            'summary': result.summary,
            'violations': [
                {
                    'checker': v.checker,
                    'severity': v.severity.value,
                    'message': v.message,
                    'details': v.details,
                    'phase_id': v.phase_id,
                    'phase_name': v.phase_name,
                }
                for v in result.violations
            ],
        }
    
    def _timestamp(self) -> str:
        """Get current timestamp."""
        from datetime import datetime
        return datetime.utcnow().isoformat() + 'Z'


def main():
    """CLI entry point."""
    validator = GlobalForensicValidator()
    result = validator.validate_all()
    
    # Generate report
    report = validator.generate_report(result)
    
    # Print report to stdout
    print(json.dumps(report, indent=2))
    
    # Exit with appropriate code
    sys.exit(0 if result.passed else 1)


if __name__ == '__main__':
    main()

