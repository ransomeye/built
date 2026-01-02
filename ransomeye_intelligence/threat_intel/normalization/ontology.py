# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/normalization/ontology.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Threat intelligence ontology - standardizes IOC types and attributes

"""
Threat Intelligence Ontology: Standardizes IOC types and attributes.
Ensures consistent representation across all threat intelligence sources.
"""

from typing import Dict, List, Optional
from enum import Enum


class IOCType(Enum):
    """IOC type enumeration."""
    IP = "ip"
    DOMAIN = "domain"
    HASH = "hash"
    URL = "url"
    EMAIL = "email"
    FILE_PATH = "file_path"
    REGISTRY_KEY = "registry_key"
    MUTEX = "mutex"
    CMD_LINE = "cmd_line"


class ThreatIntelligenceOntology:
    """Threat intelligence ontology."""
    
    IOC_TYPE_MAPPING = {
        'ip': IOCType.IP,
        'domain': IOCType.DOMAIN,
        'hash': IOCType.HASH,
        'url': IOCType.URL,
        'email': IOCType.EMAIL,
        'file_path': IOCType.FILE_PATH,
        'registry_key': IOCType.REGISTRY_KEY,
        'mutex': IOCType.MUTEX,
        'cmd_line': IOCType.CMD_LINE
    }
    
    IOC_ATTRIBUTES = {
        IOCType.IP: ['value', 'confidence', 'first_seen', 'last_seen', 'source', 'tags'],
        IOCType.DOMAIN: ['value', 'confidence', 'first_seen', 'last_seen', 'source', 'tags'],
        IOCType.HASH: ['value', 'type', 'confidence', 'first_seen', 'last_seen', 'source', 'tags'],
        IOCType.URL: ['value', 'confidence', 'first_seen', 'last_seen', 'source', 'tags'],
        IOCType.EMAIL: ['value', 'confidence', 'first_seen', 'last_seen', 'source', 'tags'],
        IOCType.FILE_PATH: ['value', 'confidence', 'first_seen', 'last_seen', 'source', 'tags'],
        IOCType.REGISTRY_KEY: ['value', 'confidence', 'first_seen', 'last_seen', 'source', 'tags'],
        IOCType.MUTEX: ['value', 'confidence', 'first_seen', 'last_seen', 'source', 'tags'],
        IOCType.CMD_LINE: ['value', 'confidence', 'first_seen', 'last_seen', 'source', 'tags']
    }
    
    def normalize_ioc(self, ioc: Dict) -> Dict:
        """
        Normalize IOC to standard format.
        
        Args:
            ioc: IOC dictionary
        
        Returns:
            Normalized IOC dictionary
        """
        # Extract type
        ioc_type_str = ioc.get('type', '').lower()
        if ioc_type_str not in self.IOC_TYPE_MAPPING:
            raise ValueError(f"Invalid IOC type: {ioc_type_str}")
        
        ioc_type = self.IOC_TYPE_MAPPING[ioc_type_str]
        
        # Build normalized IOC
        normalized = {
            'type': ioc_type.value,
            'value': ioc.get('value', ''),
            'confidence': float(ioc.get('confidence', 0.0)),
            'first_seen': ioc.get('first_seen', ''),
            'last_seen': ioc.get('last_seen', ''),
            'source': ioc.get('source', ''),
            'tags': ioc.get('tags', [])
        }
        
        # Add type-specific attributes
        if ioc_type == IOCType.HASH:
            normalized['hash_type'] = ioc.get('hash_type', 'sha256')
        
        return normalized
    
    def validate_ioc(self, ioc: Dict) -> bool:
        """Validate IOC format."""
        try:
            normalized = self.normalize_ioc(ioc)
            
            # Check required attributes
            ioc_type = IOCType(normalized['type'])
            required_attrs = self.IOC_ATTRIBUTES[ioc_type]
            
            for attr in required_attrs:
                if attr not in normalized:
                    return False
            
            return True
        except Exception:
            return False


def main():
    """CLI entry point for ontology."""
    ontology = ThreatIntelligenceOntology()
    
    # Example normalization
    ioc = {
        'type': 'ip',
        'value': '192.0.2.1',
        'confidence': 0.9,
        'source': 'test_feed'
    }
    
    normalized = ontology.normalize_ioc(ioc)
    print(f"Normalized IOC: {normalized}")


if __name__ == '__main__':
    main()

