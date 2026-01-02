# Path and File Name : /home/ransomeye/rebuild/ransomeye_posture_engine/engine/drift_detector.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Drift detection engine - detects configuration and posture drift

"""
Drift Detector

Detects configuration and posture drift over time.
Compares current posture against historical baseline.
"""

import logging
import json
from pathlib import Path
from datetime import datetime, timedelta
from typing import List, Dict, Any, Optional
from dataclasses import dataclass, asdict

from .scorer import HostPostureScore, NetworkPostureScore
from .normalizer import PostureFact

logger = logging.getLogger("ransomeye_posture_engine.drift_detector")

@dataclass
class DriftAlert:
    """Drift detection alert."""
    alert_id: str
    host_id: Optional[str]
    drift_type: str  # "score_drift", "config_drift", "fact_drift"
    severity: str  # "critical", "high", "medium", "low"
    description: str
    baseline_value: Any
    current_value: Any
    drift_amount: float
    timestamp: datetime

class DriftDetector:
    """Detects posture drift."""
    
    def __init__(self, baseline_dir: Path, drift_window_hours: int):
        self.baseline_dir = baseline_dir
        self.drift_window_hours = drift_window_hours
        self.baseline_dir.mkdir(parents=True, exist_ok=True)
    
    def detect_host_drift(self, 
                         host_id: str,
                         current_score: HostPostureScore,
                         baseline_score: Optional[HostPostureScore] = None) -> List[DriftAlert]:
        """
        Detect drift in host posture score.
        
        Args:
            host_id: Host identifier
            current_score: Current host posture score
            baseline_score: Baseline score (if None, loads from disk)
        
        Returns:
            List of drift alerts
        """
        alerts = []
        
        # Load baseline if not provided
        if baseline_score is None:
            baseline_score = self._load_host_baseline(host_id)
        
        if baseline_score is None:
            # No baseline - save current as baseline
            self._save_host_baseline(host_id, current_score)
            logger.info(f"No baseline for host {host_id}, saved current score as baseline")
            return alerts
        
        # Compare scores
        score_drift = abs(current_score.overall_score - baseline_score.overall_score)
        
        if score_drift > 0.1:  # More than 10% drift
            severity = "critical" if score_drift > 0.3 else "high" if score_drift > 0.2 else "medium"
            
            alert = DriftAlert(
                alert_id=f"drift_{host_id}_{datetime.utcnow().timestamp()}",
                host_id=host_id,
                drift_type="score_drift",
                severity=severity,
                description=f"Host posture score drifted by {score_drift:.2%}",
                baseline_value=baseline_score.overall_score,
                current_value=current_score.overall_score,
                drift_amount=score_drift,
                timestamp=datetime.utcnow(),
            )
            alerts.append(alert)
        
        # Check category score drifts
        category_drifts = [
            ("host_hardening", current_score.host_hardening_score, baseline_score.host_hardening_score),
            ("auth_hygiene", current_score.auth_hygiene_score, baseline_score.auth_hygiene_score),
        ]
        
        for category, current, baseline in category_drifts:
            drift = abs(current - baseline)
            if drift > 0.15:  # 15% drift threshold for categories
                severity = "high" if drift > 0.25 else "medium"
                alert = DriftAlert(
                    alert_id=f"drift_{host_id}_{category}_{datetime.utcnow().timestamp()}",
                    host_id=host_id,
                    drift_type="category_drift",
                    severity=severity,
                    description=f"{category} score drifted by {drift:.2%}",
                    baseline_value=baseline,
                    current_value=current,
                    drift_amount=drift,
                    timestamp=datetime.utcnow(),
                )
                alerts.append(alert)
        
        # Update baseline
        self._save_host_baseline(host_id, current_score)
        
        return alerts
    
    def detect_fact_drift(self,
                          host_id: str,
                          current_facts: List[PostureFact],
                          baseline_facts: Optional[List[PostureFact]] = None) -> List[DriftAlert]:
        """
        Detect drift in posture facts.
        
        Args:
            host_id: Host identifier
            current_facts: Current posture facts
            baseline_facts: Baseline facts (if None, loads from disk)
        
        Returns:
            List of drift alerts
        """
        alerts = []
        
        # Load baseline if not provided
        if baseline_facts is None:
            baseline_facts = self._load_fact_baseline(host_id)
        
        if baseline_facts is None:
            # No baseline - save current as baseline
            self._save_fact_baseline(host_id, current_facts)
            return alerts
        
        # Compare fact counts by category
        baseline_by_category = {}
        for fact in baseline_facts:
            category = fact.category.value
            baseline_by_category[category] = baseline_by_category.get(category, 0) + 1
        
        current_by_category = {}
        for fact in current_facts:
            category = fact.category.value
            current_by_category[category] = current_by_category.get(category, 0) + 1
        
        # Detect significant changes
        for category in set(list(baseline_by_category.keys()) + list(current_by_category.keys())):
            baseline_count = baseline_by_category.get(category, 0)
            current_count = current_by_category.get(category, 0)
            
            if baseline_count == 0:
                continue
            
            change_ratio = abs(current_count - baseline_count) / baseline_count
            
            if change_ratio > 0.2:  # More than 20% change
                severity = "high" if change_ratio > 0.5 else "medium"
                alert = DriftAlert(
                    alert_id=f"drift_{host_id}_{category}_{datetime.utcnow().timestamp()}",
                    host_id=host_id,
                    drift_type="fact_drift",
                    severity=severity,
                    description=f"{category} fact count changed by {change_ratio:.2%}",
                    baseline_value=baseline_count,
                    current_value=current_count,
                    drift_amount=change_ratio,
                    timestamp=datetime.utcnow(),
                )
                alerts.append(alert)
        
        # Update baseline
        self._save_fact_baseline(host_id, current_facts)
        
        return alerts
    
    def _load_host_baseline(self, host_id: str) -> Optional[HostPostureScore]:
        """Load host baseline from disk."""
        baseline_file = self.baseline_dir / f"host_{host_id}_baseline.json"
        
        if not baseline_file.exists():
            return None
        
        try:
            with open(baseline_file, 'r') as f:
                data = json.load(f)
            
            # Reconstruct HostPostureScore
            return HostPostureScore(
                host_id=data['host_id'],
                overall_score=data['overall_score'],
                host_hardening_score=data['host_hardening_score'],
                auth_hygiene_score=data['auth_hygiene_score'],
                timestamp=datetime.fromisoformat(data['timestamp']),
                cis_score=data['cis_score'],
                stig_score=data['stig_score'],
                custom_policy_score=data['custom_policy_score'],
                findings_count=data['findings_count'],
                critical_findings_count=data['critical_findings_count'],
            )
        except Exception as e:
            logger.error(f"Error loading host baseline for {host_id}: {e}")
            return None
    
    def _save_host_baseline(self, host_id: str, score: HostPostureScore):
        """Save host baseline to disk."""
        baseline_file = self.baseline_dir / f"host_{host_id}_baseline.json"
        
        try:
            data = {
                'host_id': score.host_id,
                'overall_score': score.overall_score,
                'host_hardening_score': score.host_hardening_score,
                'auth_hygiene_score': score.auth_hygiene_score,
                'timestamp': score.timestamp.isoformat(),
                'cis_score': score.cis_score,
                'stig_score': score.stig_score,
                'custom_policy_score': score.custom_policy_score,
                'findings_count': score.findings_count,
                'critical_findings_count': score.critical_findings_count,
            }
            
            with open(baseline_file, 'w') as f:
                json.dump(data, f, indent=2)
        
        except Exception as e:
            logger.error(f"Error saving host baseline for {host_id}: {e}")
    
    def _load_fact_baseline(self, host_id: str) -> Optional[List[PostureFact]]:
        """Load fact baseline from disk."""
        baseline_file = self.baseline_dir / f"host_{host_id}_facts_baseline.json"
        
        if not baseline_file.exists():
            return None
        
        try:
            with open(baseline_file, 'r') as f:
                data = json.load(f)
            
            # Reconstruct PostureFacts (simplified - would need full deserialization)
            # For now, return None and let it create new baseline
            return None
        
        except Exception as e:
            logger.error(f"Error loading fact baseline for {host_id}: {e}")
            return None
    
    def _save_fact_baseline(self, host_id: str, facts: List[PostureFact]):
        """Save fact baseline to disk."""
        baseline_file = self.baseline_dir / f"host_{host_id}_facts_baseline.json"
        
        try:
            # Save simplified fact data
            data = {
                'host_id': host_id,
                'facts': [
                    {
                        'fact_id': f.fact_id,
                        'category': f.category.value,
                        'fact_type': f.fact_type,
                        'timestamp': f.timestamp.isoformat(),
                    }
                    for f in facts
                ],
                'timestamp': datetime.utcnow().isoformat(),
            }
            
            with open(baseline_file, 'w') as f:
                json.dump(data, f, indent=2)
        
        except Exception as e:
            logger.error(f"Error saving fact baseline for {host_id}: {e}")

