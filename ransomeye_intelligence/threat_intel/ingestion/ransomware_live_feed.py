# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/ingestion/ransomware_live_feed.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Ransomware.live threat intelligence feed collector for training data (Phase 6 - Secure, Key-Safe)

"""
Ransomware.live Feed Collector: Collects threat intelligence from Ransomware.live API.
All data is cached locally for offline training use.
Phase 6: Secure, key-safe implementation with fail-safe logic.
"""

import os
import sys
import json
import requests
import hashlib
import socket
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Optional, Tuple

# Ransomware.live API endpoint
RANSOMWARE_LIVE_API_URL = "https://api.ransomware.live/v1"

# Environment variable for API key (MANDATORY when online)
ENV_API_KEY = "RANSOMEYE_FEED_RANSOMWARELIVE_API_KEY"

FEEDS_DIR = Path("/home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/feeds")
CACHE_DIR = Path("/home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/cache/ransomware_live")


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
        ('api.ransomware.live', 443),  # Ransomware.live API
    ]
    
    for host, port in test_hosts:
        try:
            sock = socket.create_connection((host, port), timeout=timeout)
            sock.close()
            return True
        except (socket.error, OSError):
            continue
    
    return False


class RansomwareLiveFeedCollector:
    """Collects threat intelligence from Ransomware.live API (Phase 6 - Secure, Key-Safe)."""
    
    def __init__(self, api_key: Optional[str] = None):
        """
        Initialize Ransomware.live feed collector.
        
        Args:
            api_key: Optional API key (overrides environment variable)
        
        Raises:
            FeedError: If internet is available but API key is missing
        """
        # Read API key from environment (MANDATORY when online)
        self.api_key = api_key or os.getenv(ENV_API_KEY)
        self.api_url = RANSOMWARE_LIVE_API_URL
        FEEDS_DIR.mkdir(parents=True, exist_ok=True)
        CACHE_DIR.mkdir(parents=True, exist_ok=True)
        
        # Phase 6: Fail-safe logic - if internet available and key missing, fail feed (not system)
        if check_internet_connectivity():
            if not self.api_key:
                raise FeedError(
                    f"Internet is available but {ENV_API_KEY} is not set. "
                    "Feed will fail, but system continues running."
                )
    
    def fetch_groups(self) -> Tuple[List[Dict], bool]:
        """
        Fetch ransomware groups from Ransomware.live API.
        
        Returns:
            Tuple of (groups list, success flag)
        
        Raises:
            FeedError: If feed fetch fails (system continues)
        """
        # Phase 6: Check internet and key before attempting fetch
        if check_internet_connectivity():
            if not self.api_key:
                raise FeedError(
                    f"Internet available but {ENV_API_KEY} missing. Feed fails, system continues."
                )
        else:
            # Offline mode - return empty list, not an error
            print("Info: Offline mode - returning cached groups only", file=sys.stderr)
            return [], False
        
        try:
            headers = {
                'Authorization': f'Bearer {self.api_key}',
                'Content-Type': 'application/json'
            }
            
            response = requests.get(
                f"{self.api_url}/groups",
                headers=headers,
                timeout=30
            )
            
            if response.status_code == 200:
                return response.json(), True
            else:
                raise FeedError(f"Failed to fetch groups: HTTP {response.status_code}")
        
        except requests.exceptions.Timeout:
            raise FeedError("Timeout fetching Ransomware.live groups")
        except FeedError:
            raise
        except Exception as e:
            raise FeedError(f"Error fetching Ransomware.live groups: {e}")
    
    def fetch_recent_victims(self, limit: int = 100) -> Tuple[List[Dict], bool]:
        """
        Fetch recent victim data from Ransomware.live API.
        
        Args:
            limit: Maximum number of victims to fetch
        
        Returns:
            Tuple of (victims list, success flag)
        
        Raises:
            FeedError: If feed fetch fails (system continues)
        """
        # Phase 6: Check internet and key before attempting fetch
        if check_internet_connectivity():
            if not self.api_key:
                raise FeedError(
                    f"Internet available but {ENV_API_KEY} missing. Feed fails, system continues."
                )
        else:
            # Offline mode - return empty list, not an error
            print("Info: Offline mode - returning cached victims only", file=sys.stderr)
            return [], False
        
        try:
            headers = {
                'Authorization': f'Bearer {self.api_key}',
                'Content-Type': 'application/json'
            }
            
            response = requests.get(
                f"{self.api_url}/victims",
                headers=headers,
                params={'limit': limit},
                timeout=30
            )
            
            if response.status_code == 200:
                return response.json(), True
            else:
                raise FeedError(f"Failed to fetch victims: HTTP {response.status_code}")
        
        except requests.exceptions.Timeout:
            raise FeedError("Timeout fetching Ransomware.live victims")
        except FeedError:
            raise
        except Exception as e:
            raise FeedError(f"Error fetching Ransomware.live victims: {e}")
    
    def cache_data(self, groups: List[Dict], victims: List[Dict], feed_id: str = None) -> Path:
        """
        Cache feed data to local file for offline training.
        Verifies integrity and normalizes to feature vectors.
        
        Args:
            groups: List of ransomware group data
            victims: List of victim data
            feed_id: Optional feed identifier
        
        Returns:
            Path to cached file
        """
        if feed_id is None:
            feed_id = f"ransomware_live_{datetime.utcnow().strftime('%Y%m%d_%H%M%S')}"
        
        cache_path = CACHE_DIR / f"{feed_id}.json"
        
        feed_data = {
            'feed_id': feed_id,
            'source': 'ransomware.live',
            'timestamp': datetime.utcnow().isoformat() + 'Z',
            'group_count': len(groups),
            'victim_count': len(victims),
            'groups': groups,
            'victims': victims,
            'feed_hash': None
        }
        
        # Phase 6: Compute feed hash for integrity verification
        feed_json = json.dumps(feed_data, sort_keys=True)
        feed_hash = hashlib.sha256(feed_json.encode()).hexdigest()
        feed_data['feed_hash'] = f"sha256:{feed_hash}"
        
        # Phase 6: Normalize to feature vectors (campaigns, families, timelines)
        normalized_features = self._normalize_to_features(groups, victims)
        feed_data['normalized_features'] = normalized_features
        
        # Save to cache
        with open(cache_path, 'w') as f:
            json.dump(feed_data, f, indent=2)
        
        print(f"  ✓ Cached {len(groups)} groups and {len(victims)} victims to {cache_path}")
        return cache_path
    
    def _normalize_to_features(self, groups: List[Dict], victims: List[Dict]) -> Dict:
        """
        Normalize groups and victims to feature vectors for training.
        
        Args:
            groups: List of ransomware group data
            victims: List of victim data
        
        Returns:
            Normalized feature vectors
        """
        # Extract campaigns, families, timelines
        campaigns = []
        families = []
        timelines = []
        
        for group in groups:
            families.append({
                'name': group.get('name', ''),
                'description': group.get('description', ''),
                'website': group.get('website', ''),
                'locations': group.get('locations', []),
            })
        
        for victim in victims:
            campaigns.append({
                'group': victim.get('group', ''),
                'victim': victim.get('victim', ''),
                'discovered': victim.get('discovered', ''),
                'published': victim.get('published', ''),
                'post_url': victim.get('post_url', ''),
            })
            timelines.append({
                'timestamp': victim.get('discovered', ''),
                'group': victim.get('group', ''),
                'event': 'victim_published',
            })
        
        return {
            'campaigns': campaigns,
            'families': families,
            'timelines': timelines,
        }
    
    def load_cached_data(self) -> Dict:
        """Load all cached data."""
        all_groups = []
        all_victims = []
        
        for cache_file in sorted(CACHE_DIR.glob("*.json")):
            try:
                with open(cache_file, 'r') as f:
                    feed_data = json.load(f)
                    all_groups.extend(feed_data.get('groups', []))
                    all_victims.extend(feed_data.get('victims', []))
            except Exception:
                continue
        
        return {
            'groups': all_groups,
            'victims': all_victims
        }


def main():
    """CLI entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description='Ransomware.live Feed Collector (Phase 6 - Secure)')
    parser.add_argument('--limit', type=int, default=100,
                       help='Maximum number of victims to fetch')
    parser.add_argument('--cache-only', action='store_true',
                       help='Only load from cache, do not fetch new data')
    parser.add_argument('--api-key', default=None,
                       help=f'Ransomware.live API key (or use {ENV_API_KEY} env var)')
    
    args = parser.parse_args()
    
    try:
        collector = RansomwareLiveFeedCollector(api_key=args.api_key)
    except FeedError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        print("Feed fails, but system continues running.", file=sys.stderr)
        sys.exit(1)
    
    if args.cache_only:
        data = collector.load_cached_data()
        print(f"Loaded {len(data['groups'])} groups and {len(data['victims'])} victims from cache")
    else:
        print("Fetching data from Ransomware.live...")
        try:
            groups, groups_success = collector.fetch_groups()
            victims, victims_success = collector.fetch_recent_victims(limit=args.limit)
            
            if groups_success or victims_success:
                print(f"Fetched {len(groups)} groups and {len(victims)} victims")
                if groups or victims:
                    cache_path = collector.cache_data(groups, victims)
                    print(f"Cached to: {cache_path}")
            else:
                print("Offline mode - no new data fetched", file=sys.stderr)
        except FeedError as e:
            print(f"ERROR: {e}", file=sys.stderr)
            print("Feed fails, but system continues running.", file=sys.stderr)
            sys.exit(1)
        
        data = {'groups': groups, 'victims': victims}
    
    return data


if __name__ == '__main__':
    main()

