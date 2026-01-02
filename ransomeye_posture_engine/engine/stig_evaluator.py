# Path and File Name : /home/ransomeye/rebuild/ransomeye_posture_engine/engine/stig_evaluator.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: STIG profiles evaluator - deterministic evaluation against STIG requirements

"""
STIG Profiles Evaluator

Evaluates posture facts against STIG (Security Technical Implementation Guide) profiles.
Deterministic logic only - NO ML.
Fail-closed on ambiguity.
"""

import logging
import yaml
from pathlib import Path
from typing import List, Dict, Any, Optional
from dataclasses import dataclass
from enum import Enum

from .normalizer import PostureFact
from .cis_evaluator import ComplianceStatus
from .policy_metadata import PolicyMetadataManager, PolicyMetadata

logger = logging.getLogger("ransomeye_posture_engine.stig_evaluator")

@dataclass
class STIGRequirement:
    """STIG requirement definition."""
    requirement_id: str
    title: str
    description: str
    severity: str  # "CAT I", "CAT II", "CAT III"
    vuln_id: str  # STIG vulnerability ID
    check_type: str
    check_logic: Dict[str, Any]
    remediation: Optional[str] = None

@dataclass
class STIGEvaluationResult:
    """STIG evaluation result with policy metadata."""
    requirement_id: str
    vuln_id: str
    status: ComplianceStatus
    findings: List[str]
    score: float
    timestamp: str
    policy_metadata: PolicyMetadata  # MANDATORY: policy hash, version, source path

class STIGEvaluator:
    """Evaluates posture facts against STIG profiles."""
    
    def __init__(self, stig_dir: Path, metadata_manager: PolicyMetadataManager):
        self.stig_dir = stig_dir
        self.metadata_manager = metadata_manager
        self.requirements: Dict[str, STIGRequirement] = {}
        self._load_requirements()
    
    def _load_requirements(self):
        """Load STIG requirements from YAML files."""
        if not self.stig_dir.exists():
            logger.warning(f"STIG profiles directory does not exist: {self.stig_dir}")
            return
        
        for yaml_file in self.stig_dir.glob("*.yaml"):
            try:
                with open(yaml_file, 'r') as f:
                    data = yaml.safe_load(f)
                
                if not isinstance(data, dict) or 'requirements' not in data:
                    continue
                
                for req_data in data['requirements']:
                    req_id = req_data.get('id', '')
                    if not req_id:
                        continue
                    
                    # Check for policy drift
                    if self.metadata_manager.detect_policy_drift(req_id, yaml_file):
                        logger.warning(f"Policy drift detected for STIG requirement {req_id} - hash changed")
                    
                    # Register policy metadata (MANDATORY)
                    self.metadata_manager.register_policy(
                        policy_id=req_id,
                        policy_type='stig',
                        source_path=yaml_file,
                        version=req_data.get('version')
                    )
                    
                    requirement = STIGRequirement(
                        requirement_id=req_id,
                        title=req_data.get('title', ''),
                        description=req_data.get('description', ''),
                        severity=req_data.get('severity', 'CAT III'),
                        vuln_id=req_data.get('vuln_id', ''),
                        check_type=req_data.get('check_type', 'deterministic'),
                        check_logic=req_data.get('check_logic', {}),
                        remediation=req_data.get('remediation'),
                    )
                    self.requirements[requirement.requirement_id] = requirement
                
                logger.info(f"Loaded {len(data.get('requirements', []))} requirements from {yaml_file}")
            
            except Exception as e:
                logger.error(f"Error loading STIG requirement file {yaml_file}: {e}")
                # Fail-closed: continue but log error
        
        logger.info(f"Loaded {len(self.requirements)} total STIG requirements")
    
    def evaluate(self, facts: List[PostureFact]) -> List[STIGEvaluationResult]:
        """
        Evaluate posture facts against STIG requirements.
        
        Args:
            facts: List of posture facts to evaluate
        
        Returns:
            List of evaluation results
        """
        results = []
        
        for req_id, requirement in self.requirements.items():
            if requirement.check_type != 'deterministic':
                continue
            
            result = self._evaluate_requirement(requirement, facts)
            results.append(result)
        
        logger.info(f"Evaluated {len(results)} STIG requirements")
        return results
    
    def _evaluate_requirement(self, requirement: STIGRequirement, 
                             facts: List[PostureFact]) -> STIGEvaluationResult:
        """Evaluate a single STIG requirement."""
        findings = []
        status = ComplianceStatus.COMPLIANT
        score = 1.0
        
        check_logic = requirement.check_logic
        
        # Extract relevant facts
        relevant_facts = self._filter_relevant_facts(requirement, facts)
        
        if not relevant_facts:
            status = ComplianceStatus.NOT_APPLICABLE
            score = 1.0
        else:
            check_type = check_logic.get('type', 'presence')
            
            if check_type == 'absence':
                if relevant_facts:
                    status = ComplianceStatus.NON_COMPLIANT
                    score = 0.0
                    findings.append(f"Found {len(relevant_facts)} violating facts")
            
            elif check_type == 'threshold':
                threshold = check_logic.get('threshold', 0)
                if len(relevant_facts) > threshold:
                    status = ComplianceStatus.NON_COMPLIANT
                    score = max(0.0, 1.0 - (len(relevant_facts) - threshold) / max(threshold, 1))
                    findings.append(f"Fact count {len(relevant_facts)} exceeds threshold {threshold}")
            
            elif check_type == 'pattern':
                pattern = check_logic.get('pattern', {})
                violations = self._check_pattern(pattern, relevant_facts)
                if violations:
                    status = ComplianceStatus.NON_COMPLIANT
                    score = max(0.0, 1.0 - len(violations) / max(len(relevant_facts), 1))
                    findings.extend(violations)
            
            else:
                logger.warning(f"Unknown check type for requirement {requirement.requirement_id}: {check_type}")
                status = ComplianceStatus.AMBIGUOUS
                score = 0.0
                findings.append(f"Unknown check type: {check_type}")
        
        # Get policy metadata (MANDATORY)
        policy_metadata = self.metadata_manager.get_metadata(requirement.requirement_id)
        if not policy_metadata:
            raise RuntimeError(f"Policy metadata not found for requirement {requirement.requirement_id} (FAIL-CLOSED)")
        
        return STIGEvaluationResult(
            requirement_id=requirement.requirement_id,
            vuln_id=requirement.vuln_id,
            status=status,
            findings=findings,
            score=score,
            timestamp=str(facts[0].timestamp) if facts else "",
            policy_metadata=policy_metadata,
        )
    
    def _filter_relevant_facts(self, requirement: STIGRequirement, 
                              facts: List[PostureFact]) -> List[PostureFact]:
        """Filter facts relevant to a requirement."""
        relevant = []
        
        check_logic = requirement.check_logic
        fact_types = check_logic.get('fact_types', [])
        categories = check_logic.get('categories', [])
        
        for fact in facts:
            if fact_types and fact.fact_type not in fact_types:
                continue
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
            
            for key, expected_value in pattern.items():
                actual_value = fact_data.get(key)
                
                if actual_value != expected_value:
                    violations.append(
                        f"Fact {fact.fact_id}: {key} = {actual_value}, expected {expected_value}"
                    )
        
        return violations

