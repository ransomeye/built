# Path and File Name : /home/ransomeye/rebuild/ransomeye_posture_engine/engine/scorer.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Posture scoring engine - calculates host and network posture scores

"""
Posture Scorer

Calculates host and network posture scores based on evaluation results.
Deterministic scoring - NO ML.
"""

import logging
from typing import List, Dict, Any, Optional
from dataclasses import dataclass
from datetime import datetime

from .cis_evaluator import CISEvaluationResult
from .stig_evaluator import STIGEvaluationResult
from .custom_policy_evaluator import CustomPolicyEvaluationResult
from .normalizer import PostureFact

logger = logging.getLogger("ransomeye_posture_engine.scorer")

@dataclass
class HostPostureScore:
    """Host posture score."""
    host_id: str
    overall_score: float  # 0.0 to 1.0
    host_hardening_score: float
    auth_hygiene_score: float
    timestamp: datetime
    cis_score: float
    stig_score: float
    custom_policy_score: float
    findings_count: int
    critical_findings_count: int

@dataclass
class NetworkPostureScore:
    """Network posture score."""
    network_id: str  # Can be subnet, VLAN, or "global"
    overall_score: float
    exposure_score: float
    timestamp: datetime
    findings_count: int
    critical_findings_count: int

class PostureScorer:
    """Calculates posture scores."""
    
    def __init__(self):
        pass
    
    def calculate_host_score(self, 
                            host_id: str,
                            facts: List[PostureFact],
                            cis_results: List[CISEvaluationResult],
                            stig_results: List[STIGEvaluationResult],
                            custom_results: List[CustomPolicyEvaluationResult]) -> HostPostureScore:
        """
        Calculate host posture score.
        
        Args:
            host_id: Host identifier
            facts: Posture facts for the host
            cis_results: CIS evaluation results
            stig_results: STIG evaluation results
            custom_results: Custom policy evaluation results
        
        Returns:
            Host posture score
        """
        # Calculate category scores
        host_hardening_facts = [f for f in facts if f.category.value == 'host_hardening']
        auth_hygiene_facts = [f for f in facts if f.category.value == 'auth_hygiene']
        
        host_hardening_score = self._calculate_category_score(host_hardening_facts)
        auth_hygiene_score = self._calculate_category_score(auth_hygiene_facts)
        
        # Calculate framework scores
        cis_score = self._calculate_framework_score(cis_results)
        stig_score = self._calculate_framework_score(stig_results)
        custom_policy_score = self._calculate_framework_score(custom_results)
        
        # Calculate overall score (weighted average)
        # Weights: CIS 40%, STIG 30%, Custom 20%, Category scores 10%
        overall_score = (
            cis_score * 0.4 +
            stig_score * 0.3 +
            custom_policy_score * 0.2 +
            (host_hardening_score + auth_hygiene_score) / 2 * 0.1
        )
        
        # Count findings
        findings_count = len([r for r in cis_results + stig_results + custom_results 
                            if r.status.value == 'non_compliant'])
        critical_findings_count = len([f for f in facts 
                                     if f.fact_data.get('risk_level') == 'high'])
        
        return HostPostureScore(
            host_id=host_id,
            overall_score=overall_score,
            host_hardening_score=host_hardening_score,
            auth_hygiene_score=auth_hygiene_score,
            timestamp=datetime.utcnow(),
            cis_score=cis_score,
            stig_score=stig_score,
            custom_policy_score=custom_policy_score,
            findings_count=findings_count,
            critical_findings_count=critical_findings_count,
        )
    
    def calculate_network_score(self,
                                network_id: str,
                                facts: List[PostureFact]) -> NetworkPostureScore:
        """
        Calculate network posture score.
        
        Args:
            network_id: Network identifier
            facts: Network-related posture facts
        
        Returns:
            Network posture score
        """
        # Filter network exposure facts
        network_facts = [f for f in facts if f.category.value == 'network_exposure']
        
        exposure_score = self._calculate_category_score(network_facts)
        
        # Overall network score is primarily based on exposure
        overall_score = exposure_score
        
        findings_count = len(network_facts)
        critical_findings_count = len([f for f in network_facts 
                                     if f.fact_data.get('risk_level') == 'high'])
        
        return NetworkPostureScore(
            network_id=network_id,
            overall_score=overall_score,
            exposure_score=exposure_score,
            timestamp=datetime.utcnow(),
            findings_count=findings_count,
            critical_findings_count=critical_findings_count,
        )
    
    def _calculate_category_score(self, facts: List[PostureFact]) -> float:
        """Calculate score for a category of facts."""
        if not facts:
            return 1.0  # No facts = no issues = perfect score
        
        # Score based on fact confidence and risk level
        total_weight = 0.0
        weighted_sum = 0.0
        
        for fact in facts:
            risk_level = fact.fact_data.get('risk_level', 'medium')
            confidence = fact.confidence
            
            # Weight by risk level
            if risk_level == 'high':
                weight = 1.0
            elif risk_level == 'medium':
                weight = 0.5
            else:
                weight = 0.25
            
            # Higher confidence = more reliable = higher impact on score
            impact = (1.0 - confidence) * weight
            weighted_sum += impact
            total_weight += weight
        
        if total_weight == 0:
            return 1.0
        
        # Score is inverse of weighted impact
        score = max(0.0, 1.0 - (weighted_sum / total_weight))
        return score
    
    def _calculate_framework_score(self, results: List) -> float:
        """Calculate score for a framework (CIS, STIG, or Custom)."""
        if not results:
            return 1.0  # No results = not applicable = perfect score
        
        # Average of individual result scores
        total_score = sum(r.score for r in results)
        avg_score = total_score / len(results) if results else 1.0
        
        return avg_score

