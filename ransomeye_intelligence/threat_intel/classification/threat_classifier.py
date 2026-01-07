# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/classification/threat_classifier.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Comprehensive threat classification system covering all cyber threat types

"""
Comprehensive Threat Classification System
==========================================
Classifies and normalizes all cyber threat types including:
- Malware (all variants)
- Ransomware
- DDoS attacks
- Trojans
- Spyware
- Worms
- Man-in-the-Middle (MitM)
- SQL Injection
- DNS Tunneling
- AI-Driven Attacks
- Supply Chain Attacks
- Zero-Day Exploits
- Cryptojacking
- And more...
"""

import os
import sys
import json
import re
from pathlib import Path
from datetime import datetime, timezone
from typing import Dict, List, Optional, Set, Tuple
from enum import Enum

# Threat type taxonomy
class ThreatCategory(Enum):
    """Primary threat categories."""
    MALWARE = "malware"
    RANSOMWARE = "ransomware"
    DDOS = "ddos"
    TROJAN = "trojan"
    SPYWARE = "spyware"
    WORM = "worm"
    MITM = "mitm"
    SQL_INJECTION = "sql_injection"
    DNS_TUNNELING = "dns_tunneling"
    AI_DRIVEN = "ai_driven"
    SUPPLY_CHAIN = "supply_chain"
    ZERO_DAY = "zero_day"
    CRYPTOJACKING = "cryptojacking"
    PHISHING = "phishing"
    APT = "apt"
    BOTNET = "botnet"
    KEYLOGGER = "keylogger"
    ROOTKIT = "rootkit"
    BACKDOOR = "backdoor"
    FILELESS = "fileless"
    POLYMORPHIC = "polymorphic"
    METAMORPHIC = "metamorphic"
    UNKNOWN = "unknown"

# Threat type patterns and indicators
THREAT_PATTERNS = {
    ThreatCategory.RANSOMWARE: {
        'keywords': ['ransom', 'encrypt', 'decrypt', 'bitcoin', 'payment', 'locker', 'crypto'],
        'file_extensions': ['.encrypted', '.locked', '.crypto', '.vault'],
        'behavior': ['file_encryption', 'ransom_note', 'payment_demand']
    },
    ThreatCategory.DDOS: {
        'keywords': ['ddos', 'distributed denial', 'flood', 'syn flood', 'udp flood'],
        'network_patterns': ['high_bandwidth', 'connection_exhaustion', 'resource_exhaustion'],
        'behavior': ['traffic_amplification', 'botnet_coordination']
    },
    ThreatCategory.TROJAN: {
        'keywords': ['trojan', 'trojanized', 'backdoor', 'remote access'],
        'behavior': ['unauthorized_access', 'command_control', 'data_exfiltration']
    },
    ThreatCategory.SPYWARE: {
        'keywords': ['spyware', 'keylogger', 'screen_capture', 'data_collection'],
        'behavior': ['keystroke_logging', 'screen_recording', 'data_exfiltration']
    },
    ThreatCategory.WORM: {
        'keywords': ['worm', 'self_replicating', 'network_propagation'],
        'behavior': ['self_replication', 'network_scanning', 'lateral_movement']
    },
    ThreatCategory.MITM: {
        'keywords': ['mitm', 'man in the middle', 'ssl_strip', 'arp_spoofing'],
        'behavior': ['traffic_interception', 'certificate_manipulation', 'session_hijacking']
    },
    ThreatCategory.SQL_INJECTION: {
        'keywords': ['sql injection', 'sqli', 'database attack'],
        'patterns': [r"('|(\\')|(;)|(--)|(/\*)|(\*/)|(\+)|(\|)|(\&)|(\^)|(\()|(\))|(\[)|(\])|(\{)|(\})|(\')|(\")|(\;)|(\-)|(\*)|(\+)|(\|)|(\&)|(\^))"],
        'behavior': ['database_query_manipulation', 'unauthorized_data_access']
    },
    ThreatCategory.DNS_TUNNELING: {
        'keywords': ['dns tunneling', 'dns exfiltration', 'dns covert channel'],
        'behavior': ['dns_query_anomaly', 'high_dns_volume', 'unusual_dns_patterns']
    },
    ThreatCategory.AI_DRIVEN: {
        'keywords': ['ai attack', 'ml evasion', 'adversarial', 'deepfake', 'ai-generated'],
        'behavior': ['model_evasion', 'adversarial_examples', 'ai_manipulation']
    },
    ThreatCategory.SUPPLY_CHAIN: {
        'keywords': ['supply chain', 'dependency attack', 'typosquatting', 'compromised package'],
        'behavior': ['package_compromise', 'dependency_poisoning', 'build_system_attack']
    },
    ThreatCategory.ZERO_DAY: {
        'keywords': ['zero day', '0day', 'unknown exploit', 'unpatched'],
        'behavior': ['unknown_vulnerability', 'no_signature_match', 'novel_technique']
    },
    ThreatCategory.CRYPTOJACKING: {
        'keywords': ['cryptojacking', 'coinminer', 'mining', 'cryptocurrency mining'],
        'behavior': ['cpu_intensive', 'mining_pool_connection', 'resource_theft']
    },
    ThreatCategory.PHISHING: {
        'keywords': ['phishing', 'spear phishing', 'credential theft'],
        'behavior': ['credential_harvesting', 'social_engineering', 'fake_login']
    },
    ThreatCategory.APT: {
        'keywords': ['apt', 'advanced persistent threat', 'nation state'],
        'behavior': ['long_term_persistence', 'sophisticated_evasion', 'targeted_attack']
    },
    ThreatCategory.BOTNET: {
        'keywords': ['botnet', 'command and control', 'c2', 'zombie'],
        'behavior': ['c2_communication', 'coordinated_activity', 'network_propagation']
    },
    ThreatCategory.FILELESS: {
        'keywords': ['fileless', 'memory only', 'powershell', 'wmi'],
        'behavior': ['memory_execution', 'no_file_artifacts', 'script_based']
    },
    ThreatCategory.POLYMORPHIC: {
        'keywords': ['polymorphic', 'code mutation', 'self-modifying'],
        'behavior': ['code_mutation', 'signature_evasion', 'variant_generation']
    }
}

class ThreatClassifier:
    """Comprehensive threat classifier for all cyber threat types."""
    
    def __init__(self):
        self.classification_cache = {}
        self.confidence_threshold = 0.6
        
    def classify(self, 
                 ioc_value: str,
                 ioc_type: str = None,
                 metadata: Dict = None,
                 behavior_signals: List[str] = None) -> Dict:
        """
        Classify a threat IOC into one or more threat categories.
        
        Args:
            ioc_value: The IOC value (hash, IP, domain, URL, etc.)
            ioc_type: Type of IOC (hash, ip, domain, url, etc.)
            metadata: Additional metadata about the IOC
            behavior_signals: List of behavioral indicators
        
        Returns:
            Dictionary with classification results
        """
        metadata = metadata or {}
        behavior_signals = behavior_signals or []
        
        # Combine all text for analysis
        text_to_analyze = f"{ioc_value} {metadata.get('description', '')} {metadata.get('name', '')} {' '.join(behavior_signals)}".lower()
        
        classifications = []
        confidence_scores = {}
        
        # Check each threat category
        for category, patterns in THREAT_PATTERNS.items():
            score = 0.0
            matches = []
            
            # Check keywords
            for keyword in patterns.get('keywords', []):
                if keyword.lower() in text_to_analyze:
                    score += 0.3
                    matches.append(f"keyword:{keyword}")
            
            # Check regex patterns
            for pattern in patterns.get('patterns', []):
                if re.search(pattern, text_to_analyze, re.IGNORECASE):
                    score += 0.4
                    matches.append(f"pattern:{pattern}")
            
            # Check behavioral indicators
            for behavior in patterns.get('behavior', []):
                if behavior.lower() in text_to_analyze:
                    score += 0.5
                    matches.append(f"behavior:{behavior}")
            
            # Check file extensions
            for ext in patterns.get('file_extensions', []):
                if ext.lower() in ioc_value.lower():
                    score += 0.3
                    matches.append(f"extension:{ext}")
            
            # Normalize score
            score = min(score, 1.0)
            
            if score >= self.confidence_threshold:
                confidence_scores[category.value] = score
                classifications.append({
                    'category': category.value,
                    'confidence': score,
                    'matches': matches
                })
        
        # Sort by confidence
        classifications.sort(key=lambda x: x['confidence'], reverse=True)
        
        # Primary classification (highest confidence)
        primary_category = classifications[0]['category'] if classifications else ThreatCategory.UNKNOWN.value
        
        return {
            'ioc_value': ioc_value,
            'ioc_type': ioc_type,
            'primary_category': primary_category,
            'all_categories': [c['category'] for c in classifications],
            'confidence_scores': confidence_scores,
            'classifications': classifications,
            'timestamp': datetime.now(timezone.utc).isoformat().replace('+00:00', '') + 'Z'
        }
    
    def enrich_ioc(self, ioc: Dict) -> Dict:
        """
        Enrich an IOC with threat classification.
        
        Args:
            ioc: IOC dictionary with at least 'value' field
        
        Returns:
            Enriched IOC with classification
        """
        classification = self.classify(
            ioc_value=ioc.get('value', ''),
            ioc_type=ioc.get('type', ''),
            metadata=ioc.get('metadata', {}),
            behavior_signals=ioc.get('behavior_signals', [])
        )
        
        # Merge classification into IOC
        enriched_ioc = ioc.copy()
        enriched_ioc['threat_classification'] = classification
        enriched_ioc['threat_category'] = classification['primary_category']
        enriched_ioc['threat_categories'] = classification['all_categories']
        enriched_ioc['classification_confidence'] = max(classification['confidence_scores'].values()) if classification['confidence_scores'] else 0.0
        
        return enriched_ioc
    
    def batch_classify(self, iocs: List[Dict]) -> List[Dict]:
        """
        Classify multiple IOCs in batch.
        
        Args:
            iocs: List of IOC dictionaries
        
        Returns:
            List of enriched IOCs
        """
        return [self.enrich_ioc(ioc) for ioc in iocs]
    
    def get_threat_statistics(self, iocs: List[Dict]) -> Dict:
        """
        Generate statistics about threat distribution.
        
        Args:
            iocs: List of classified IOCs
        
        Returns:
            Statistics dictionary
        """
        stats = {
            'total_iocs': len(iocs),
            'by_category': {},
            'by_confidence': {
                'high': 0,  # >= 0.8
                'medium': 0,  # 0.6-0.8
                'low': 0  # < 0.6
            }
        }
        
        for ioc in iocs:
            category = ioc.get('threat_category', 'unknown')
            stats['by_category'][category] = stats['by_category'].get(category, 0) + 1
            
            confidence = ioc.get('classification_confidence', 0.0)
            if confidence >= 0.8:
                stats['by_confidence']['high'] += 1
            elif confidence >= 0.6:
                stats['by_confidence']['medium'] += 1
            else:
                stats['by_confidence']['low'] += 1
        
        return stats


def main():
    """Test threat classifier."""
    classifier = ThreatClassifier()
    
    # Test IOCs
    test_iocs = [
        {
            'value': 'malware.exe.encrypted',
            'type': 'filename',
            'metadata': {'description': 'Encrypted file with ransom note'},
            'behavior_signals': ['file_encryption', 'ransom_note']
        },
        {
            'value': '192.168.1.100',
            'type': 'ip',
            'metadata': {'description': 'High bandwidth DDoS attack source'},
            'behavior_signals': ['high_bandwidth', 'connection_exhaustion']
        },
        {
            'value': "'; DROP TABLE users; --",
            'type': 'payload',
            'metadata': {'description': 'SQL injection attempt'},
            'behavior_signals': ['database_query_manipulation']
        }
    ]
    
    print("Testing Threat Classifier")
    print("=" * 80)
    
    for ioc in test_iocs:
        result = classifier.classify(
            ioc_value=ioc['value'],
            ioc_type=ioc['type'],
            metadata=ioc['metadata'],
            behavior_signals=ioc['behavior_signals']
        )
        print(f"\nIOC: {ioc['value']}")
        print(f"Primary Category: {result['primary_category']}")
        print(f"All Categories: {result['all_categories']}")
        print(f"Confidence Scores: {result['confidence_scores']}")
    
    # Batch classification
    enriched = classifier.batch_classify(test_iocs)
    stats = classifier.get_threat_statistics(enriched)
    
    print("\n" + "=" * 80)
    print("Threat Statistics:")
    print(json.dumps(stats, indent=2))


if __name__ == '__main__':
    main()

