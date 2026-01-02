# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/normalization/mapping.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Maps threat intelligence from various sources to standard ontology

"""
Threat Intelligence Mapping: Maps threat intelligence from various sources.
Converts external formats to RansomEye standard ontology.
"""

from typing import Dict, List
from .ontology import ThreatIntelligenceOntology, IOCType


class ThreatIntelligenceMapper:
    """Maps threat intelligence to standard format."""
    
    def __init__(self):
        self.ontology = ThreatIntelligenceOntology()
    
    def map_misp_ioc(self, misp_ioc: Dict) -> Dict:
        """Map MISP IOC to standard format."""
        # MISP format mapping
        ioc_type_map = {
            'ip-dst': IOCType.IP,
            'domain': IOCType.DOMAIN,
            'sha256': IOCType.HASH,
            'url': IOCType.URL,
            'email': IOCType.EMAIL
        }
        
        # Extract MISP attributes
        misp_type = misp_ioc.get('type', '')
        mapped_type = ioc_type_map.get(misp_type, None)
        
        if mapped_type is None:
            raise ValueError(f"Unsupported MISP type: {misp_type}")
        
        return {
            'type': mapped_type.value,
            'value': misp_ioc.get('value', ''),
            'confidence': misp_ioc.get('confidence', 0.5),
            'first_seen': misp_ioc.get('first_seen', ''),
            'last_seen': misp_ioc.get('last_seen', ''),
            'source': 'misp',
            'tags': misp_ioc.get('tags', [])
        }
    
    def map_otx_ioc(self, otx_ioc: Dict) -> Dict:
        """Map OTX IOC to standard format."""
        # OTX format mapping
        ioc_type_map = {
            'IPv4': IOCType.IP,
            'domain': IOCType.DOMAIN,
            'FileHash-SHA256': IOCType.HASH,
            'URL': IOCType.URL
        }
        
        # Extract OTX attributes
        otx_type = otx_ioc.get('type', '')
        mapped_type = ioc_type_map.get(otx_type, None)
        
        if mapped_type is None:
            raise ValueError(f"Unsupported OTX type: {otx_type}")
        
        return {
            'type': mapped_type.value,
            'value': otx_ioc.get('indicator', ''),
            'confidence': otx_ioc.get('pulse_count', 0) / 100.0,  # Normalize
            'first_seen': otx_ioc.get('created', ''),
            'last_seen': otx_ioc.get('modified', ''),
            'source': 'otx',
            'tags': otx_ioc.get('tags', [])
        }
    
    def map_stix_ioc(self, stix_ioc: Dict) -> Dict:
        """Map STIX IOC to standard format."""
        # STIX format mapping
        stix_type = stix_ioc.get('type', '')
        
        if stix_type == 'ipv4-addr':
            ioc_type = IOCType.IP
        elif stix_type == 'domain-name':
            ioc_type = IOCType.DOMAIN
        elif stix_type == 'file':
            ioc_type = IOCType.HASH
        elif stix_type == 'url':
            ioc_type = IOCType.URL
        else:
            raise ValueError(f"Unsupported STIX type: {stix_type}")
        
        return {
            'type': ioc_type.value,
            'value': stix_ioc.get('value', ''),
            'confidence': stix_ioc.get('confidence', 0.5),
            'first_seen': stix_ioc.get('created', ''),
            'last_seen': stix_ioc.get('modified', ''),
            'source': 'stix',
            'tags': stix_ioc.get('labels', [])
        }


def main():
    """CLI entry point for mapper."""
    mapper = ThreatIntelligenceMapper()
    
    # Example mapping
    misp_ioc = {
        'type': 'ip-dst',
        'value': '192.0.2.1',
        'confidence': 0.9,
        'tags': ['malware', 'ransomware']
    }
    
    mapped = mapper.map_misp_ioc(misp_ioc)
    print(f"Mapped IOC: {mapped}")


if __name__ == '__main__':
    main()

