# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/fusion/confidence.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Calculates confidence scores for threat intelligence - advisory only

"""
Threat Intelligence Confidence: Calculates confidence scores for threat intelligence.
All confidence scores are advisory only and never override Control Plane decisions.
"""

from typing import Dict, List
from datetime import datetime


class ThreatIntelligenceConfidence:
    """Calculates confidence scores for threat intelligence."""
    
    def __init__(self):
        self.source_weights = {
            'misp': 0.9,
            'otx': 0.8,
            'talos': 0.85,
            'threatfox': 0.75,
            'internal': 0.95
        }
        self.recency_decay_days = 90  # Confidence decays over 90 days
    
    def calculate_confidence(self, ioc: Dict) -> float:
        """
        Calculate confidence score for an IOC.
        
        Args:
            ioc: IOC dictionary
        
        Returns:
            Confidence score (0.0 to 1.0)
        """
        base_confidence = float(ioc.get('confidence', 0.5))
        source = ioc.get('source', 'unknown')
        
        # Apply source weight
        source_weight = self.source_weights.get(source, 0.5)
        weighted_confidence = base_confidence * source_weight
        
        # Apply recency decay
        last_seen = ioc.get('last_seen', '')
        if last_seen:
            try:
                last_seen_dt = datetime.fromisoformat(last_seen)
                days_ago = (datetime.utcnow() - last_seen_dt).days
                if days_ago > self.recency_decay_days:
                    decay_factor = max(0.0, 1.0 - (days_ago - self.recency_decay_days) / self.recency_decay_days)
                    weighted_confidence *= decay_factor
            except Exception:
                pass
        
        # Apply tag-based adjustments
        tags = ioc.get('tags', [])
        if 'malware' in tags or 'ransomware' in tags:
            weighted_confidence *= 1.1  # Slight boost
        if 'false_positive' in tags:
            weighted_confidence *= 0.5  # Significant reduction
        
        return min(1.0, max(0.0, weighted_confidence))
    
    def calculate_correlated_confidence(self, correlated_ioc: Dict) -> float:
        """
        Calculate confidence for correlated IOC.
        
        Args:
            correlated_ioc: Correlated IOC dictionary
        
        Returns:
            Correlated confidence score
        """
        base_confidence = correlated_ioc.get('correlated_confidence', 0.5)
        correlation_count = correlated_ioc.get('correlation_count', 0)
        source_count = len(correlated_ioc.get('sources', []))
        
        # Boost confidence based on correlation
        if correlation_count > 1:
            boost = min(0.2, correlation_count * 0.05)
            base_confidence += boost
        
        # Boost confidence based on source diversity
        if source_count > 1:
            boost = min(0.15, source_count * 0.05)
            base_confidence += boost
        
        return min(1.0, max(0.0, base_confidence))


def main():
    """CLI entry point for confidence calculator."""
    confidence_calc = ThreatIntelligenceConfidence()
    
    # Example confidence calculation
    ioc = {
        'type': 'ip',
        'value': '192.0.2.1',
        'confidence': 0.8,
        'source': 'misp',
        'last_seen': datetime.utcnow().isoformat(),
        'tags': ['malware']
    }
    
    conf = confidence_calc.calculate_confidence(ioc)
    print(f"Calculated confidence: {conf}")


if __name__ == '__main__':
    main()

