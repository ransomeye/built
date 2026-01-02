# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/fusion/correlation.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Correlates threat intelligence from multiple sources - advisory only, never overrides Core decisions

"""
Threat Intelligence Correlation: Correlates threat intelligence from multiple sources.
Advisory only - never overrides Control Plane decisions.
"""

from typing import Dict, List, Tuple
from collections import defaultdict


class ThreatIntelligenceCorrelator:
    """Correlates threat intelligence from multiple sources."""
    
    def __init__(self):
        self.ioc_index: Dict[str, List[Dict]] = defaultdict(list)
    
    def index_ioc(self, ioc: Dict) -> None:
        """Index an IOC for correlation."""
        ioc_key = f"{ioc['type']}:{ioc['value']}"
        self.ioc_index[ioc_key].append(ioc)
    
    def correlate_ioc(self, ioc: Dict) -> Dict:
        """
        Correlate an IOC across multiple sources.
        
        Args:
            ioc: IOC to correlate
        
        Returns:
            Correlated IOC with confidence score
        """
        ioc_key = f"{ioc['type']}:{ioc['value']}"
        matching_iocs = self.ioc_index.get(ioc_key, [])
        
        if not matching_iocs:
            return {
                'ioc': ioc,
                'correlation_count': 0,
                'correlated_confidence': ioc.get('confidence', 0.0),
                'sources': [ioc.get('source', 'unknown')],
                'advisory': True
            }
        
        # Calculate correlated confidence
        confidences = [m.get('confidence', 0.0) for m in matching_iocs]
        avg_confidence = sum(confidences) / len(confidences)
        
        # Weight by number of sources
        source_count = len(set(m.get('source', 'unknown') for m in matching_iocs))
        correlated_confidence = min(1.0, avg_confidence * (1.0 + source_count * 0.1))
        
        return {
            'ioc': ioc,
            'correlation_count': len(matching_iocs),
            'correlated_confidence': correlated_confidence,
            'sources': list(set(m.get('source', 'unknown') for m in matching_iocs)),
            'advisory': True  # Always advisory
        }
    
    def correlate_feed(self, feed_iocs: List[Dict]) -> List[Dict]:
        """Correlate all IOCs in a feed."""
        correlated = []
        for ioc in feed_iocs:
            correlated.append(self.correlate_ioc(ioc))
        return correlated


def main():
    """CLI entry point for correlator."""
    correlator = ThreatIntelligenceCorrelator()
    
    # Example correlation
    ioc1 = {'type': 'ip', 'value': '192.0.2.1', 'confidence': 0.8, 'source': 'misp'}
    ioc2 = {'type': 'ip', 'value': '192.0.2.1', 'confidence': 0.9, 'source': 'otx'}
    
    correlator.index_ioc(ioc1)
    correlator.index_ioc(ioc2)
    
    correlated = correlator.correlate_ioc(ioc1)
    print(f"Correlated IOC: {correlated}")


if __name__ == '__main__':
    main()

