# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/enrichment/unified_enricher.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Unified threat intelligence enrichment module that normalizes all threat types

"""
Unified Threat Intelligence Enricher
======================================
Enriches IOCs from all sources with:
- Comprehensive threat classification
- Threat type normalization
- Confidence scoring
- Behavioral analysis
- Cross-source correlation
"""

import os
import sys
import json
from pathlib import Path
from datetime import datetime, timezone
from typing import Dict, List, Optional
import logging

# Add parent directories to path
sys.path.insert(0, str(Path(__file__).parent.parent))
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from classification.threat_classifier import ThreatClassifier, ThreatCategory

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger('unified_enricher')


class UnifiedThreatEnricher:
    """Unified threat intelligence enricher."""
    
    def __init__(self):
        self.threat_classifier = ThreatClassifier()
        
    def enrich_ioc(self, ioc: Dict, source: str = None) -> Dict:
        """
        Enrich a single IOC with comprehensive threat intelligence.
        
        Args:
            ioc: IOC dictionary
            source: Source of the IOC
        
        Returns:
            Enriched IOC
        """
        # Classify threat
        classification = self.threat_classifier.classify(
            ioc_value=ioc.get('value', ''),
            ioc_type=ioc.get('type', ''),
            metadata=ioc.get('metadata', {}),
            behavior_signals=ioc.get('behavior_signals', [])
        )
        
        # Build enriched IOC
        enriched = {
            'ioc_value': ioc.get('value', ''),
            'ioc_type': ioc.get('type', ''),
            'source': source or ioc.get('source', 'unknown'),
            'threat_category': classification['primary_category'],
            'threat_categories': classification['all_categories'],
            'classification_confidence': max(classification['confidence_scores'].values()) if classification['confidence_scores'] else 0.0,
            'confidence_scores': classification['confidence_scores'],
            'classification_matches': classification.get('classifications', []),
            'metadata': ioc.get('metadata', {}),
            'behavior_signals': ioc.get('behavior_signals', []),
            'enriched_at': datetime.now(timezone.utc).isoformat().replace('+00:00', '') + 'Z',
            'enrichment_version': '1.0.0'
        }
        
        return enriched
    
    def enrich_batch(self, iocs: List[Dict], source: str = None) -> List[Dict]:
        """
        Enrich multiple IOCs in batch.
        
        Args:
            iocs: List of IOC dictionaries
            source: Source of the IOCs
        
        Returns:
            List of enriched IOCs
        """
        enriched = []
        for ioc in iocs:
            try:
                enriched_ioc = self.enrich_ioc(ioc, source)
                enriched.append(enriched_ioc)
            except Exception as e:
                logger.warning(f"Failed to enrich IOC {ioc.get('value', 'unknown')}: {e}")
                continue
        
        logger.info(f"Enriched {len(enriched)}/{len(iocs)} IOCs")
        return enriched
    
    def normalize_threat_type(self, threat_type: str) -> str:
        """
        Normalize threat type to standard taxonomy.
        
        Args:
            threat_type: Raw threat type string
        
        Returns:
            Normalized threat category
        """
        threat_type_lower = threat_type.lower()
        
        # Map common variations to standard categories
        type_mapping = {
            'ransomware': ThreatCategory.RANSOMWARE.value,
            'malware': ThreatCategory.MALWARE.value,
            'ddos': ThreatCategory.DDOS.value,
            'dos': ThreatCategory.DDOS.value,
            'denial of service': ThreatCategory.DDOS.value,
            'trojan': ThreatCategory.TROJAN.value,
            'spyware': ThreatCategory.SPYWARE.value,
            'worm': ThreatCategory.WORM.value,
            'mitm': ThreatCategory.MITM.value,
            'man in the middle': ThreatCategory.MITM.value,
            'sql injection': ThreatCategory.SQL_INJECTION.value,
            'sqli': ThreatCategory.SQL_INJECTION.value,
            'dns tunneling': ThreatCategory.DNS_TUNNELING.value,
            'ai attack': ThreatCategory.AI_DRIVEN.value,
            'ai-driven': ThreatCategory.AI_DRIVEN.value,
            'supply chain': ThreatCategory.SUPPLY_CHAIN.value,
            'zero day': ThreatCategory.ZERO_DAY.value,
            '0day': ThreatCategory.ZERO_DAY.value,
            'cryptojacking': ThreatCategory.CRYPTOJACKING.value,
            'coinminer': ThreatCategory.CRYPTOJACKING.value,
            'phishing': ThreatCategory.PHISHING.value,
            'apt': ThreatCategory.APT.value,
            'botnet': ThreatCategory.BOTNET.value,
            'keylogger': ThreatCategory.KEYLOGGER.value,
            'rootkit': ThreatCategory.ROOTKIT.value,
            'backdoor': ThreatCategory.BACKDOOR.value,
            'fileless': ThreatCategory.FILELESS.value,
            'polymorphic': ThreatCategory.POLYMORPHIC.value,
            'metamorphic': ThreatCategory.METAMORPHIC.value
        }
        
        # Check for exact match
        if threat_type_lower in type_mapping:
            return type_mapping[threat_type_lower]
        
        # Check for partial match
        for key, value in type_mapping.items():
            if key in threat_type_lower:
                return value
        
        # Default to unknown
        return ThreatCategory.UNKNOWN.value
    
    def generate_enrichment_report(self, enriched_iocs: List[Dict]) -> Dict:
        """
        Generate enrichment statistics report.
        
        Args:
            enriched_iocs: List of enriched IOCs
        
        Returns:
            Enrichment report
        """
        stats = self.threat_classifier.get_threat_statistics(enriched_iocs)
        
        # Additional statistics
        by_source = {}
        by_confidence_range = {
            'very_high': 0,  # >= 0.9
            'high': 0,  # 0.8-0.9
            'medium': 0,  # 0.6-0.8
            'low': 0  # < 0.6
        }
        
        for ioc in enriched_iocs:
            source = ioc.get('source', 'unknown')
            by_source[source] = by_source.get(source, 0) + 1
            
            confidence = ioc.get('classification_confidence', 0.0)
            if confidence >= 0.9:
                by_confidence_range['very_high'] += 1
            elif confidence >= 0.8:
                by_confidence_range['high'] += 1
            elif confidence >= 0.6:
                by_confidence_range['medium'] += 1
            else:
                by_confidence_range['low'] += 1
        
        report = {
            'timestamp': datetime.now(timezone.utc).isoformat().replace('+00:00', '') + 'Z',
            'total_iocs': len(enriched_iocs),
            'threat_statistics': stats,
            'by_source': by_source,
            'by_confidence_range': by_confidence_range,
            'top_categories': sorted(
                stats['by_category'].items(),
                key=lambda x: x[1],
                reverse=True
            )[:10]
        }
        
        return report


def main():
    """CLI entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description='Unified Threat Intelligence Enricher')
    parser.add_argument('--ioc', type=str,
                       help='Single IOC value to enrich')
    parser.add_argument('--file', type=str,
                       help='JSON file with IOCs to enrich')
    parser.add_argument('--source', type=str,
                       help='Source of the IOCs')
    
    args = parser.parse_args()
    
    enricher = UnifiedThreatEnricher()
    
    if args.ioc:
        ioc = {'value': args.ioc, 'type': 'unknown'}
        enriched = enricher.enrich_ioc(ioc, args.source)
        print(json.dumps(enriched, indent=2))
    elif args.file:
        with open(args.file, 'r') as f:
            iocs = json.load(f)
        
        enriched = enricher.enrich_batch(iocs, args.source)
        report = enricher.generate_enrichment_report(enriched)
        
        print("Enrichment Report:")
        print(json.dumps(report, indent=2))
        
        # Save enriched IOCs
        output_file = Path(args.file).with_suffix('.enriched.json')
        with open(output_file, 'w') as f:
            json.dump(enriched, f, indent=2)
        print(f"\nEnriched IOCs saved to: {output_file}")


if __name__ == '__main__':
    main()

