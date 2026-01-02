# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/ingestion/additional_sources.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Additional threat intelligence feed sources for training data

"""
Additional Threat Intelligence Sources: Collectors for additional threat intelligence feeds.
Extensible framework for adding new feed sources.
"""

import os
import sys
import json
import subprocess
import hashlib
import requests
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Optional

FEEDS_DIR = Path("/home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/feeds")
CACHE_DIR = Path("/home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/cache")


class ThreatIntelligenceFeed:
    """Base class for threat intelligence feed collectors."""
    
    def __init__(self, source_name: str, cache_subdir: str):
        self.source_name = source_name
        self.cache_dir = CACHE_DIR / cache_subdir
        self.cache_dir.mkdir(parents=True, exist_ok=True)
        FEEDS_DIR.mkdir(parents=True, exist_ok=True)
    
    def fetch(self) -> Optional[Dict]:
        """Fetch feed data. Must be implemented by subclasses."""
        raise NotImplementedError
    
    def parse(self, data: Dict) -> List[Dict]:
        """Parse feed data into IOCs. Must be implemented by subclasses."""
        raise NotImplementedError
    
    def cache(self, data: Dict, feed_id: str = None) -> Path:
        """Cache feed data."""
        if feed_id is None:
            feed_id = f"{self.source_name}_{datetime.utcnow().strftime('%Y%m%d_%H%M%S')}"
        
        cache_path = self.cache_dir / f"{feed_id}.json"
        
        iocs = self.parse(data)
        
        feed_data = {
            'feed_id': feed_id,
            'source': self.source_name,
            'timestamp': datetime.utcnow().isoformat() + 'Z',
            'ioc_count': len(iocs),
            'iocs': iocs,
            'raw_data': data,
            'feed_hash': None
        }
        
        # Compute feed hash
        feed_json = json.dumps(feed_data, sort_keys=True)
        feed_hash = hashlib.sha256(feed_json.encode()).hexdigest()
        feed_data['feed_hash'] = f"sha256:{feed_hash}"
        
        # Save to cache
        with open(cache_path, 'w') as f:
            json.dump(feed_data, f, indent=2)
        
        return cache_path
    
    def load_cached(self) -> List[Dict]:
        """Load all cached IOCs."""
        all_iocs = []
        for cache_file in sorted(self.cache_dir.glob("*.json")):
            try:
                with open(cache_file, 'r') as f:
                    feed_data = json.load(f)
                    all_iocs.extend(feed_data.get('iocs', []))
            except Exception:
                continue
        return all_iocs


class AlienVaultOTXFeed(ThreatIntelligenceFeed):
    """AlienVault OTX threat intelligence feed."""
    
    def __init__(self, api_key: Optional[str] = None):
        super().__init__('alienvault_otx', 'alienvault_otx')
        self.api_key = api_key or os.getenv('OTX_KEY', '')
        self.api_url = 'https://otx.alienvault.com/api/v1'
    
    def fetch(self) -> Optional[Dict]:
        """Fetch pulses from OTX."""
        if not self.api_key:
            return None
        
        try:
            headers = {'X-OTX-API-KEY': self.api_key}
            response = requests.get(
                f"{self.api_url}/pulses/subscribed",
                headers=headers,
                timeout=30
            )
            
            if response.status_code == 200:
                return response.json()
            return None
        except Exception as e:
            print(f"Warning: OTX fetch error: {e}", file=sys.stderr)
            return None
    
    def parse(self, data: Dict) -> List[Dict]:
        """Parse OTX pulses into IOCs."""
        iocs = []
        pulses = data.get('results', [])
        
        for pulse in pulses:
            indicators = pulse.get('indicators', [])
            for indicator in indicators:
                ioc = {
                    'type': indicator.get('type', ''),
                    'value': indicator.get('indicator', ''),
                    'title': pulse.get('name', ''),
                    'description': pulse.get('description', ''),
                    'created': pulse.get('created', ''),
                    'modified': pulse.get('modified', ''),
                    'source': 'alienvault_otx'
                }
                iocs.append(ioc)
        
        return iocs


class VirusTotalFeed(ThreatIntelligenceFeed):
    """VirusTotal threat intelligence feed (requires API key)."""
    
    def __init__(self, api_key: Optional[str] = None):
        super().__init__('virustotal', 'virustotal')
        self.api_key = api_key or os.getenv('VIRUSTOTAL_KEY', '')
        self.api_url = 'https://www.virustotal.com/api/v3'
    
    def fetch(self) -> Optional[Dict]:
        """Fetch recent intelligence from VirusTotal."""
        if not self.api_key:
            return None
        
        try:
            headers = {'X-Apikey': self.api_key}
            response = requests.get(
                f"{self.api_url}/intelligence/search",
                headers=headers,
                params={'query': 'ransomware', 'limit': 100},
                timeout=30
            )
            
            if response.status_code == 200:
                return response.json()
            return None
        except Exception as e:
            print(f"Warning: VirusTotal fetch error: {e}", file=sys.stderr)
            return None
    
    def parse(self, data: Dict) -> List[Dict]:
        """Parse VirusTotal data into IOCs."""
        iocs = []
        results = data.get('data', [])
        
        for result in results:
            ioc = {
                'type': 'hash',
                'value': result.get('id', ''),
                'attributes': result.get('attributes', {}),
                'source': 'virustotal'
            }
            iocs.append(ioc)
        
        return iocs


class AbuseCHURLhausFeed(ThreatIntelligenceFeed):
    """Abuse.ch URLhaus feed (no API key required)."""
    
    def __init__(self):
        super().__init__('urlhaus', 'urlhaus')
        self.api_url = 'https://urlhaus.abuse.ch/downloads'
    
    def fetch(self) -> Optional[Dict]:
        """Fetch URLhaus feed."""
        try:
            # Fetch CSV feed
            response = requests.get(
                f"{self.api_url}/csv_online/",
                timeout=60
            )
            
            if response.status_code == 200:
                return {'csv_data': response.text}
            return None
        except Exception as e:
            print(f"Warning: URLhaus fetch error: {e}", file=sys.stderr)
            return None
    
    def parse(self, data: Dict) -> List[Dict]:
        """Parse URLhaus CSV into IOCs."""
        iocs = []
        csv_data = data.get('csv_data', '')
        
        for line in csv_data.split('\n')[9:]:  # Skip header
            if not line.strip():
                continue
            
            parts = line.split(',')
            if len(parts) >= 3:
                ioc = {
                    'type': 'url',
                    'value': parts[2].strip('"'),
                    'status': parts[1].strip('"'),
                    'date_added': parts[0].strip('"'),
                    'source': 'urlhaus'
                }
                iocs.append(ioc)
        
        return iocs


class ThreatFoxFeed(ThreatIntelligenceFeed):
    """Abuse.ch ThreatFox feed (no API key required)."""
    
    def __init__(self):
        super().__init__('threatfox', 'threatfox')
        self.api_url = 'https://threatfox.abuse.ch/api/v1'
    
    def fetch(self) -> Optional[Dict]:
        """Fetch ThreatFox IOCs."""
        try:
            response = requests.post(
                self.api_url,
                json={'query': 'get_iocs', 'days': 1},
                timeout=30
            )
            
            if response.status_code == 200:
                return response.json()
            return None
        except Exception as e:
            print(f"Warning: ThreatFox fetch error: {e}", file=sys.stderr)
            return None
    
    def parse(self, data: Dict) -> List[Dict]:
        """Parse ThreatFox data into IOCs."""
        iocs = []
        ioc_list = data.get('data', [])
        
        for ioc_data in ioc_list:
            ioc = {
                'type': ioc_data.get('ioc_type', ''),
                'value': ioc_data.get('ioc', ''),
                'threat_type': ioc_data.get('threat_type', ''),
                'malware': ioc_data.get('malware', ''),
                'first_seen': ioc_data.get('first_seen', ''),
                'source': 'threatfox'
            }
            iocs.append(ioc)
        
        return iocs


def get_all_feed_collectors() -> List[ThreatIntelligenceFeed]:
    """Get all available feed collectors."""
    collectors = [
        AbuseCHURLhausFeed(),
        ThreatFoxFeed(),
    ]
    
    # Add optional feeds if API keys are available
    otx_key = os.getenv('OTX_KEY', '')
    if otx_key:
        collectors.append(AlienVaultOTXFeed(api_key=otx_key))
    
    vt_key = os.getenv('VIRUSTOTAL_KEY', '')
    if vt_key:
        collectors.append(VirusTotalFeed(api_key=vt_key))
    
    return collectors


def main():
    """Test additional feed collectors."""
    import argparse
    
    parser = argparse.ArgumentParser(description='Additional Threat Intelligence Feed Collectors')
    parser.add_argument('--source', choices=['urlhaus', 'threatfox', 'otx', 'virustotal', 'all'],
                       default='all', help='Feed source to fetch')
    
    args = parser.parse_args()
    
    collectors = get_all_feed_collectors()
    
    for collector in collectors:
        if args.source != 'all' and collector.source_name != args.source:
            continue
        
        print(f"Fetching {collector.source_name}...")
        data = collector.fetch()
        if data:
            cache_path = collector.cache(data)
            iocs = collector.parse(data)
            print(f"  ✓ Cached {len(iocs)} IOCs to {cache_path}")
        else:
            print(f"  ✗ Failed to fetch {collector.source_name}")


if __name__ == '__main__':
    main()

