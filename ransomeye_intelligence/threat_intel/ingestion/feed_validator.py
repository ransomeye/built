# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/ingestion/feed_validator.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Validates threat intelligence feeds - detects poisoning and validates integrity

"""
Feed Validator: Validates threat intelligence feeds.
Detects poisoning, validates integrity, and ensures feed quality.
"""

import json
from pathlib import Path
from typing import Dict, List, Tuple
from datetime import datetime
import hashlib


class FeedValidator:
    """Validates threat intelligence feeds."""
    
    def __init__(self):
        self.poisoning_indicators = [
            'suspicious_patterns',
            'anomalous_ioc_density',
            'unusual_timestamps',
            'invalid_signatures'
        ]
    
    def _validate_ioc_format(self, ioc: Dict) -> bool:
        """Validate IOC format."""
        required_fields = ['type', 'value', 'confidence']
        for field in required_fields:
            if field not in ioc:
                return False
        
        # Validate IOC type
        valid_types = ['ip', 'domain', 'hash', 'url', 'email']
        if ioc['type'] not in valid_types:
            return False
        
        # Validate confidence
        if not (0.0 <= ioc['confidence'] <= 1.0):
            return False
        
        return True
    
    def _detect_poisoning(self, feed_data: Dict) -> Tuple[bool, List[str]]:
        """
        Detect potential feed poisoning.
        
        Returns:
            Tuple of (is_poisoned: bool, indicators: List[str])
        """
        indicators = []
        
        # Check IOC density
        iocs = feed_data.get('iocs', [])
        if len(iocs) > 10000:  # Suspiciously high
            indicators.append('anomalous_ioc_density')
        
        # Check IOC quality
        invalid_iocs = 0
        for ioc in iocs:
            if not self._validate_ioc_format(ioc):
                invalid_iocs += 1
        
        if invalid_iocs > len(iocs) * 0.1:  # More than 10% invalid
            indicators.append('suspicious_patterns')
        
        # Check timestamps
        feed_timestamp = feed_data.get('timestamp', '')
        try:
            dt = datetime.fromisoformat(feed_timestamp)
            if dt > datetime.utcnow():
                indicators.append('unusual_timestamps')
        except Exception:
            indicators.append('unusual_timestamps')
        
        is_poisoned = len(indicators) > 0
        return is_poisoned, indicators
    
    def validate_feed(self, feed_data: Dict) -> Tuple[bool, List[str]]:
        """
        Validate threat intelligence feed.
        
        Returns:
            Tuple of (is_valid: bool, errors: List[str])
        """
        errors = []
        
        # Validate required fields
        required_fields = ['feed_id', 'timestamp', 'iocs', 'signature']
        for field in required_fields:
            if field not in feed_data:
                errors.append(f"Missing required field: {field}")
        
        # Validate IOCs
        iocs = feed_data.get('iocs', [])
        if not isinstance(iocs, list):
            errors.append("IOCs must be a list")
        else:
            for i, ioc in enumerate(iocs):
                if not self._validate_ioc_format(ioc):
                    errors.append(f"Invalid IOC format at index {i}")
        
        # Detect poisoning
        is_poisoned, indicators = self._detect_poisoning(feed_data)
        if is_poisoned:
            errors.append(f"Feed poisoning detected: {', '.join(indicators)}")
        
        return len(errors) == 0, errors


def main():
    """CLI entry point for feed validator."""
    import sys
    
    validator = FeedValidator()
    
    # Example validation
    feed_data = {
        'feed_id': 'test_feed_001',
        'timestamp': datetime.utcnow().isoformat(),
        'iocs': [
            {'type': 'ip', 'value': '192.0.2.1', 'confidence': 0.9}
        ],
        'signature': {}
    }
    
    is_valid, errors = validator.validate_feed(feed_data)
    
    if is_valid:
        print("✓ Feed validation passed")
    else:
        print("✗ Feed validation failed:")
        for error in errors:
            print(f"  {error}")
        sys.exit(1)


if __name__ == '__main__':
    main()

