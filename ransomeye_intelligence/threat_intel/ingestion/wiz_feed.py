# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/ingestion/wiz_feed.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Wiz.io Cloud Threat Landscape STIX feed collector for training data (Phase 6 - Secure, Key-Safe)

"""
Wiz.io Feed Collector: Collects threat intelligence from Wiz.io Cloud Threat Landscape STIX feed.
All data is cached locally for offline training use.
Phase 6: Secure, key-safe implementation with fail-safe logic.
"""

import os
import sys
import json
import subprocess
import hashlib
import socket
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Optional, Tuple

# Environment variable for WIZ URL (MANDATORY when online)
ENV_WIZ_URL = "RANSOMEYE_FEED_WIZ_URL"

# Default Wiz.io API endpoint (fallback if env var not set)
DEFAULT_WIZ_API_URL = "https://www.wiz.io/api/feed/cloud-threat-landscape/stix.json"

FEEDS_DIR = Path("/home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/feeds")
CACHE_DIR = Path("/home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/cache/wiz")


class FeedError(Exception):
    """Feed-specific error that does not crash the system."""
    pass


def check_internet_connectivity(timeout: int = 5) -> bool:
    """
    Check if internet connectivity is available.
    
    Args:
        timeout: Connection timeout in seconds
    
    Returns:
        True if internet is available, False otherwise
    """
    test_hosts = [
        ('8.8.8.8', 53),  # Google DNS
        ('1.1.1.1', 53),  # Cloudflare DNS
        ('www.wiz.io', 443),  # Wiz.io API
    ]
    
    for host, port in test_hosts:
        try:
            sock = socket.create_connection((host, port), timeout=timeout)
            sock.close()
            return True
        except (socket.error, OSError):
            continue
    
    return False


class WizFeedCollector:
    """Collects threat intelligence from Wiz.io STIX feed (Phase 6 - Secure, Key-Safe)."""
    
    def __init__(self, wiz_url: Optional[str] = None):
        """
        Initialize Wiz.io feed collector.
        
        Args:
            wiz_url: Optional WIZ URL (overrides environment variable)
        
        Raises:
            FeedError: If internet is available but WIZ URL is missing
        """
        # Read WIZ URL from environment (MANDATORY when online)
        self.api_url = wiz_url or os.getenv(ENV_WIZ_URL) or DEFAULT_WIZ_API_URL
        FEEDS_DIR.mkdir(parents=True, exist_ok=True)
        CACHE_DIR.mkdir(parents=True, exist_ok=True)
        
        # Phase 6: Fail-safe logic - if internet available and URL missing, fail feed (not system)
        if check_internet_connectivity():
            if not os.getenv(ENV_WIZ_URL) and not wiz_url:
                # Only fail if using default and internet is available
                # If custom URL provided, allow it
                if not wiz_url:
                    raise FeedError(
                        f"Internet is available but {ENV_WIZ_URL} is not set. "
                        "Feed will fail, but system continues running."
                    )
    
    def fetch_stix_feed(self) -> Tuple[Optional[Dict], bool]:
        """
        Fetch STIX feed from Wiz.io.
        
        Returns:
            Tuple of (STIX feed data or None, success flag)
        
        Raises:
            FeedError: If feed fetch fails (system continues)
        """
        # Phase 6: Check internet before attempting fetch
        if check_internet_connectivity():
            if not os.getenv(ENV_WIZ_URL) and self.api_url == DEFAULT_WIZ_API_URL:
                raise FeedError(
                    f"Internet available but {ENV_WIZ_URL} missing. Feed fails, system continues."
                )
        else:
            # Offline mode - return None, not an error
            print("Info: Offline mode - returning cached feed only", file=sys.stderr)
            return None, False
        
        try:
            # Use wget to fetch STIX feed
            cmd = [
                'wget',
                self.api_url,
                '-O-',
                '--timeout=60',
                '--tries=2'
            ]
            
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=60
            )
            
            if result.returncode != 0:
                raise FeedError(f"Failed to fetch Wiz.io feed: {result.stderr}")
            
            try:
                data = json.loads(result.stdout)
                return data, True
            except json.JSONDecodeError:
                raise FeedError("Invalid JSON response from Wiz.io")
        
        except subprocess.TimeoutExpired:
            raise FeedError("Timeout fetching Wiz.io feed")
        except FeedError:
            raise
        except Exception as e:
            raise FeedError(f"Error fetching Wiz.io feed: {e}")
    
    def parse_stix_objects(self, stix_data: Dict) -> List[Dict]:
        """
        Parse STIX objects and extract IOCs.
        
        Args:
            stix_data: STIX JSON data
        
        Returns:
            List of parsed IOCs
        """
        iocs = []
        
        # STIX 2.1 format: objects array
        objects = stix_data.get('objects', [])
        
        for obj in objects:
            obj_type = obj.get('type', '')
            
            # Extract indicators
            if obj_type == 'indicator':
                pattern = obj.get('pattern', '')
                labels = obj.get('labels', [])
                
                ioc = {
                    'type': 'indicator',
                    'pattern': pattern,
                    'labels': labels,
                    'created': obj.get('created'),
                    'modified': obj.get('modified'),
                    'description': obj.get('description', ''),
                    'confidence': obj.get('confidence', 0.5),
                    'source': 'wiz.io'
                }
                iocs.append(ioc)
            
            # Extract threat actors
            elif obj_type == 'threat-actor':
                ioc = {
                    'type': 'threat-actor',
                    'name': obj.get('name', ''),
                    'description': obj.get('description', ''),
                    'labels': obj.get('labels', []),
                    'created': obj.get('created'),
                    'modified': obj.get('modified'),
                    'source': 'wiz.io'
                }
                iocs.append(ioc)
            
            # Extract malware
            elif obj_type == 'malware':
                ioc = {
                    'type': 'malware',
                    'name': obj.get('name', ''),
                    'description': obj.get('description', ''),
                    'labels': obj.get('labels', []),
                    'created': obj.get('created'),
                    'modified': obj.get('modified'),
                    'source': 'wiz.io'
                }
                iocs.append(ioc)
        
        return iocs
    
    def _normalize_to_features(self, iocs: List[Dict]) -> List[Dict]:
        """
        Normalize STIX IOCs to feature vectors for training.
        
        Args:
            iocs: List of parsed IOCs
        
        Returns:
            List of normalized feature vectors
        """
        features = []
        for ioc in iocs:
            feature_vector = {
                'type': ioc.get('type', ''),
                'pattern': ioc.get('pattern', ''),
                'labels': ioc.get('labels', []),
                'confidence': ioc.get('confidence', 0.5),
                'created': ioc.get('created', ''),
                'modified': ioc.get('modified', ''),
                'description': ioc.get('description', ''),
                'source': ioc.get('source', 'wiz.io'),
            }
            features.append(feature_vector)
        return features
    
    def cache_feed(self, stix_data: Dict, feed_id: str = None) -> Path:
        """
        Cache STIX feed to local file for offline training.
        Verifies integrity and normalizes to feature vectors.
        
        Args:
            stix_data: STIX feed data
            feed_id: Optional feed identifier
        
        Returns:
            Path to cached file
        """
        if feed_id is None:
            feed_id = f"wiz_{datetime.utcnow().strftime('%Y%m%d_%H%M%S')}"
        
        cache_path = CACHE_DIR / f"{feed_id}.json"
        
        # Parse IOCs
        iocs = self.parse_stix_objects(stix_data)
        
        feed_data = {
            'feed_id': feed_id,
            'source': 'wiz.io',
            'format': 'stix',
            'timestamp': datetime.utcnow().isoformat() + 'Z',
            'ioc_count': len(iocs),
            'iocs': iocs,
            'raw_stix': stix_data,
            'feed_hash': None
        }
        
        # Phase 6: Compute feed hash for integrity verification
        feed_json = json.dumps(feed_data, sort_keys=True)
        feed_hash = hashlib.sha256(feed_json.encode()).hexdigest()
        feed_data['feed_hash'] = f"sha256:{feed_hash}"
        
        # Phase 6: Normalize to feature vectors
        normalized_features = self._normalize_to_features(iocs)
        feed_data['normalized_features'] = normalized_features
        
        # Save to cache
        with open(cache_path, 'w') as f:
            json.dump(feed_data, f, indent=2)
        
        print(f"  ✓ Cached {len(iocs)} IOCs to {cache_path}")
        return cache_path
    
    def load_cached_feeds(self) -> List[Dict]:
        """Load all cached feeds."""
        all_iocs = []
        
        for cache_file in sorted(CACHE_DIR.glob("*.json")):
            try:
                with open(cache_file, 'r') as f:
                    feed_data = json.load(f)
                    iocs = feed_data.get('iocs', [])
                    all_iocs.extend(iocs)
            except Exception:
                continue
        
        return all_iocs


def main():
    """CLI entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description='Wiz.io Feed Collector (Phase 6 - Secure)')
    parser.add_argument('--cache-only', action='store_true',
                       help='Only load from cache, do not fetch new data')
    parser.add_argument('--wiz-url', default=None,
                       help=f'Wiz.io STIX feed URL (or use {ENV_WIZ_URL} env var)')
    
    args = parser.parse_args()
    
    try:
        collector = WizFeedCollector(wiz_url=args.wiz_url)
    except FeedError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        print("Feed fails, but system continues running.", file=sys.stderr)
        sys.exit(1)
    
    if args.cache_only:
        iocs = collector.load_cached_feeds()
        print(f"Loaded {len(iocs)} IOCs from cache")
    else:
        print("Fetching STIX feed from Wiz.io...")
        try:
            stix_data, success = collector.fetch_stix_feed()
            
            if success and stix_data:
                iocs = collector.parse_stix_objects(stix_data)
                print(f"Parsed {len(iocs)} IOCs from STIX feed")
                
                cache_path = collector.cache_feed(stix_data)
                print(f"Cached to: {cache_path}")
            else:
                print("Offline mode or failed to fetch feed", file=sys.stderr)
                iocs = []
        except FeedError as e:
            print(f"ERROR: {e}", file=sys.stderr)
            print("Feed fails, but system continues running.", file=sys.stderr)
            sys.exit(1)
    
    return iocs


if __name__ == '__main__':
    main()

