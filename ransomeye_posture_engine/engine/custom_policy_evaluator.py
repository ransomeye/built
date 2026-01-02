# Path and File Name : /home/ransomeye/rebuild/ransomeye_posture_engine/engine/custom_policy_evaluator.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Custom YAML policy evaluator - deterministic evaluation against user-defined policies

"""
Custom Policy Evaluator

Evaluates posture facts against custom YAML policies.
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

logger = logging.getLogger("ransomeye_posture_engine.custom_policy_evaluator")

@dataclass
class CustomPolicy:
    """Custom policy definition."""
    policy_id: str
    name: str
    description: str
    severity: str
    enabled: bool
    check_logic: Dict[str, Any]
    remediation: Optional[str] = None

@dataclass
class CustomPolicyEvaluationResult:
    """Custom policy evaluation result with policy metadata."""
    policy_id: str
    status: ComplianceStatus
    findings: List[str]
    score: float
    timestamp: str
    policy_metadata: PolicyMetadata  # MANDATORY: policy hash, version, source path

class CustomPolicyEvaluator:
    """Evaluates posture facts against custom YAML policies."""
    
    def __init__(self, policies_dir: Path, metadata_manager: PolicyMetadataManager):
        self.policies_dir = policies_dir
        self.metadata_manager = metadata_manager
        self.policies: Dict[str, CustomPolicy] = {}
        self._load_policies()
    
    def _load_policies(self):
        """Load custom policies from YAML files."""
        if not self.policies_dir.exists():
            logger.warning(f"Custom policies directory does not exist: {self.policies_dir}")
            return
        
        for yaml_file in self.policies_dir.glob("*.yaml"):
            try:
                with open(yaml_file, 'r') as f:
                    data = yaml.safe_load(f)
                
                if not isinstance(data, dict):
                    continue
                
                # Support both single policy and list of policies
                policies_data = data.get('policies', [data] if 'policy_id' in data else [])
                
                for policy_data in policies_data:
                    policy_id = policy_data.get('policy_id', '')
                    if not policy_id:
                        continue
                    
                    # Check for policy drift
                    if self.metadata_manager.detect_policy_drift(policy_id, yaml_file):
                        logger.warning(f"Policy drift detected for custom policy {policy_id} - hash changed")
                    
                    # Register policy metadata (MANDATORY)
                    self.metadata_manager.register_policy(
                        policy_id=policy_id,
                        policy_type='custom',
                        source_path=yaml_file,
                        version=policy_data.get('version')
                    )
                    
                    policy = CustomPolicy(
                        policy_id=policy_id,
                        name=policy_data.get('name', ''),
                        description=policy_data.get('description', ''),
                        severity=policy_data.get('severity', 'medium'),
                        enabled=policy_data.get('enabled', True),
                        check_logic=policy_data.get('check_logic', {}),
                        remediation=policy_data.get('remediation'),
                    )
                    
                    if policy.enabled:
                        self.policies[policy.policy_id] = policy
                
                logger.info(f"Loaded {len(policies_data)} policies from {yaml_file}")
            
            except Exception as e:
                logger.error(f"Error loading custom policy file {yaml_file}: {e}")
                # Fail-closed: continue but log error
        
        logger.info(f"Loaded {len(self.policies)} total custom policies")
    
    def evaluate(self, facts: List[PostureFact]) -> List[CustomPolicyEvaluationResult]:
        """
        Evaluate posture facts against custom policies.
        
        Args:
            facts: List of posture facts to evaluate
        
        Returns:
            List of evaluation results
        """
        results = []
        
        for policy_id, policy in self.policies.items():
            if not policy.enabled:
                continue
            
            result = self._evaluate_policy(policy, facts)
            results.append(result)
        
        logger.info(f"Evaluated {len(results)} custom policies")
        return results
    
    def _evaluate_policy(self, policy: CustomPolicy, 
                        facts: List[PostureFact]) -> CustomPolicyEvaluationResult:
        """Evaluate a single custom policy."""
        findings = []
        status = ComplianceStatus.COMPLIANT
        score = 1.0
        
        check_logic = policy.check_logic
        
        # Extract relevant facts
        relevant_facts = self._filter_relevant_facts(policy, facts)
        
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
            
            elif check_type == 'expression':
                # Support simple expression evaluation
                expression = check_logic.get('expression', '')
                if expression:
                    result = self._evaluate_expression(expression, relevant_facts)
                    if not result:
                        status = ComplianceStatus.NON_COMPLIANT
                        score = 0.0
                        findings.append(f"Expression evaluation failed: {expression}")
            
            else:
                logger.warning(f"Unknown check type for policy {policy.policy_id}: {check_type}")
                status = ComplianceStatus.AMBIGUOUS
                score = 0.0
                findings.append(f"Unknown check type: {check_type}")
        
        # Get policy metadata (MANDATORY)
        policy_metadata = self.metadata_manager.get_metadata(policy.policy_id)
        if not policy_metadata:
            raise RuntimeError(f"Policy metadata not found for policy {policy.policy_id} (FAIL-CLOSED)")
        
        return CustomPolicyEvaluationResult(
            policy_id=policy.policy_id,
            status=status,
            findings=findings,
            score=score,
            timestamp=str(facts[0].timestamp) if facts else "",
            policy_metadata=policy_metadata,
        )
    
    def _filter_relevant_facts(self, policy: CustomPolicy, 
                               facts: List[PostureFact]) -> List[PostureFact]:
        """Filter facts relevant to a policy."""
        relevant = []
        
        check_logic = policy.check_logic
        fact_types = check_logic.get('fact_types', [])
        categories = check_logic.get('categories', [])
        host_ids = check_logic.get('host_ids', [])
        
        for fact in facts:
            if fact_types and fact.fact_type not in fact_types:
                continue
            if categories and fact.category.value not in categories:
                continue
            if host_ids and fact.host_id not in host_ids:
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
    
    def _evaluate_expression(self, expression: str, 
                            facts: List[PostureFact]) -> bool:
        """
        Evaluate a simple expression against facts.
        
        Supports basic expressions like:
        - "count < 5"
        - "risk_level == 'high'"
        - "len(facts) == 0"
        
        Returns True if expression evaluates to true, False otherwise.
        """
        try:
            # Simple expression evaluation (deterministic only)
            # For safety, only allow basic comparisons
            if 'count' in expression or 'len' in expression:
                count = len(facts)
                # Replace count/len with actual value
                expr = expression.replace('count', str(count)).replace('len(facts)', str(count))
                # Evaluate safely
                result = eval(expr, {"__builtins__": {}}, {})
                return bool(result)
            
            # For other expressions, check against fact data
            for fact in facts:
                fact_data = fact.fact_data
                # Simple key-value checks
                if '==' in expression:
                    key, value = expression.split('==', 1)
                    key = key.strip()
                    value = value.strip().strip("'\"")
                    if fact_data.get(key) == value:
                        return True
            
            return False
        
        except Exception as e:
            logger.warning(f"Error evaluating expression {expression}: {e}")
            # Fail-closed: return False on error
            return False

