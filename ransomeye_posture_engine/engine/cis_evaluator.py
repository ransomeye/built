# Path and File Name : /home/ransomeye/rebuild/ransomeye_posture_engine/engine/cis_evaluator.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: CIS Benchmarks evaluator - deterministic evaluation against CIS controls

"""
CIS Benchmarks Evaluator

Evaluates posture facts against CIS Benchmark controls.
Deterministic logic only - NO ML.
Fail-closed on ambiguity.
"""

import logging
import yaml
from pathlib import Path
from typing import List, Dict, Any, Optional
from dataclasses import dataclass
from enum import Enum

from .normalizer import PostureFact, PostureCategory
from .policy_metadata import PolicyMetadataManager, PolicyMetadata

logger = logging.getLogger("ransomeye_posture_engine.cis_evaluator")

class ComplianceStatus(Enum):
    """Compliance status."""
    COMPLIANT = "compliant"
    NON_COMPLIANT = "non_compliant"
    NOT_APPLICABLE = "not_applicable"
    AMBIGUOUS = "ambiguous"  # Fail-closed: treat as non-compliant

@dataclass
class CISControl:
    """CIS control definition."""
    control_id: str
    title: str
    description: str
    severity: str  # "critical", "high", "medium", "low"
    check_type: str  # "deterministic", "manual"
    check_logic: Dict[str, Any]  # Deterministic check rules
    remediation: Optional[str] = None

@dataclass
class CISEvaluationResult:
    """CIS evaluation result with policy metadata."""
    control_id: str
    status: ComplianceStatus
    findings: List[str]
    score: float  # 0.0 to 1.0
    timestamp: str
    policy_metadata: PolicyMetadata  # MANDATORY: policy hash, version, source path

class CISEvaluator:
    """Evaluates posture facts against CIS Benchmarks."""
    
    def __init__(self, benchmarks_dir: Path, metadata_manager: PolicyMetadataManager):
        self.benchmarks_dir = benchmarks_dir
        self.metadata_manager = metadata_manager
        self.controls: Dict[str, CISControl] = {}
        self._load_controls()
    
    def _load_controls(self):
        """Load CIS controls from YAML files."""
        if not self.benchmarks_dir.exists():
            logger.warning(f"CIS benchmarks directory does not exist: {self.benchmarks_dir}")
            return
        
        for yaml_file in self.benchmarks_dir.glob("*.yaml"):
            try:
                with open(yaml_file, 'r') as f:
                    data = yaml.safe_load(f)
                
                if not isinstance(data, dict) or 'controls' not in data:
                    continue
                
                for control_data in data['controls']:
                    control_id = control_data.get('id', '')
                    if not control_id:
                        continue
                    
                    # Check for policy drift
                    if self.metadata_manager.detect_policy_drift(control_id, yaml_file):
                        logger.warning(f"Policy drift detected for CIS control {control_id} - hash changed")
                        # Fail-closed: policy change without baseline reset
                    
                    # Register policy metadata (MANDATORY)
                    self.metadata_manager.register_policy(
                        policy_id=control_id,
                        policy_type='cis',
                        source_path=yaml_file,
                        version=control_data.get('version')
                    )
                    
                    control = CISControl(
                        control_id=control_id,
                        title=control_data.get('title', ''),
                        description=control_data.get('description', ''),
                        severity=control_data.get('severity', 'medium'),
                        check_type=control_data.get('check_type', 'deterministic'),
                        check_logic=control_data.get('check_logic', {}),
                        remediation=control_data.get('remediation'),
                    )
                    self.controls[control.control_id] = control
                
                logger.info(f"Loaded {len(data.get('controls', []))} controls from {yaml_file}")
            
            except Exception as e:
                logger.error(f"Error loading CIS control file {yaml_file}: {e}")
                # Fail-closed: continue but log error
        
        logger.info(f"Loaded {len(self.controls)} total CIS controls")
    
    def evaluate(self, facts: List[PostureFact]) -> List[CISEvaluationResult]:
        """
        Evaluate posture facts against CIS controls.
        
        Args:
            facts: List of posture facts to evaluate
        
        Returns:
            List of evaluation results
        """
        results = []
        
        for control_id, control in self.controls.items():
            if control.check_type != 'deterministic':
                # Skip manual checks (not applicable for automated evaluation)
                continue
            
            result = self._evaluate_control(control, facts)
            results.append(result)
        
        logger.info(f"Evaluated {len(results)} CIS controls")
        return results
    
    def _evaluate_control(self, control: CISControl, 
                         facts: List[PostureFact]) -> CISEvaluationResult:
        """Evaluate a single CIS control."""
        findings = []
        status = ComplianceStatus.COMPLIANT
        score = 1.0
        
        check_logic = control.check_logic
        
        # Extract relevant facts for this control
        relevant_facts = self._filter_relevant_facts(control, facts)
        
        if not relevant_facts:
            # No relevant facts - assume compliant (or not applicable)
            status = ComplianceStatus.NOT_APPLICABLE
            score = 1.0
        else:
            # Apply deterministic check logic
            check_type = check_logic.get('type', 'presence')
            
            if check_type == 'absence':
                # Control requires absence of certain facts
                if relevant_facts:
                    status = ComplianceStatus.NON_COMPLIANT
                    score = 0.0
                    findings.append(f"Found {len(relevant_facts)} violating facts")
                else:
                    status = ComplianceStatus.COMPLIANT
                    score = 1.0
            
            elif check_type == 'threshold':
                # Control requires fact count below threshold
                threshold = check_logic.get('threshold', 0)
                if len(relevant_facts) > threshold:
                    status = ComplianceStatus.NON_COMPLIANT
                    score = max(0.0, 1.0 - (len(relevant_facts) - threshold) / max(threshold, 1))
                    findings.append(f"Fact count {len(relevant_facts)} exceeds threshold {threshold}")
                else:
                    status = ComplianceStatus.COMPLIANT
                    score = 1.0
            
            elif check_type == 'pattern':
                # Control requires pattern matching
                pattern = check_logic.get('pattern', {})
                violations = self._check_pattern(pattern, relevant_facts)
                if violations:
                    status = ComplianceStatus.NON_COMPLIANT
                    score = max(0.0, 1.0 - len(violations) / max(len(relevant_facts), 1))
                    findings.extend(violations)
                else:
                    status = ComplianceStatus.COMPLIANT
                    score = 1.0
            
            else:
                # Unknown check type - fail-closed
                logger.warning(f"Unknown check type for control {control.control_id}: {check_type}")
                status = ComplianceStatus.AMBIGUOUS
                score = 0.0
                findings.append(f"Unknown check type: {check_type}")
        
        # Get policy metadata (MANDATORY)
        policy_metadata = self.metadata_manager.get_metadata(control.control_id)
        if not policy_metadata:
            # Fail-closed: policy metadata is mandatory
            raise RuntimeError(f"Policy metadata not found for control {control.control_id} (FAIL-CLOSED)")
        
        return CISEvaluationResult(
            control_id=control.control_id,
            status=status,
            findings=findings,
            score=score,
            timestamp=str(facts[0].timestamp) if facts else "",
            policy_metadata=policy_metadata,
        )
    
    def _filter_relevant_facts(self, control: CISControl, 
                              facts: List[PostureFact]) -> List[PostureFact]:
        """Filter facts relevant to a control."""
        relevant = []
        
        check_logic = control.check_logic
        fact_types = check_logic.get('fact_types', [])
        categories = check_logic.get('categories', [])
        
        for fact in facts:
            # Check fact type match
            if fact_types and fact.fact_type not in fact_types:
                continue
            
            # Check category match
            if categories and fact.category.value not in categories:
                continue
            
            relevant.append(fact)
        
        return relevant
    
    def _check_pattern(self, pattern: Dict[str, Any], 
                     facts: List[PostureFact]) -> List[str]:
        """Check facts against pattern rules."""
        violations = []
        
        for fact in facts:
            fact_data = fact.fact_data
            
            # Check each pattern rule
            for key, expected_value in pattern.items():
                actual_value = fact_data.get(key)
                
                if actual_value != expected_value:
                    violations.append(
                        f"Fact {fact.fact_id}: {key} = {actual_value}, expected {expected_value}"
                    )
        
        return violations

