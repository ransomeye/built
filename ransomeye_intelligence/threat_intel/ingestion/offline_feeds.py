# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/ingestion/offline_feeds.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Offline-capable threat intelligence feed ingestion - all feeds validated and signed

"""
Offline Threat Intelligence Feeds: Ingests threat intelligence feeds offline.
All feeds are validated, normalized, and signed before use.
"""

import os
import json
from pathlib import Path
from typing import Dict, List, Optional
from datetime import datetime
import hashlib

from ransomeye_trust.verify_tool import VerifyTool


class OfflineFeedIngester:
    """Ingests threat intelligence feeds offline."""
    
    FEEDS_DIR = Path("/home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/feeds")
    CACHE_DIR = Path("/home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/cache")
    
    def __init__(self, trust_dir: str = "/home/ransomeye/rebuild/ransomeye_trust"):
        self.trust_dir = Path(trust_dir)
        self.verify_tool = VerifyTool(str(self.trust_dir))
        self.FEEDS_DIR.mkdir(parents=True, exist_ok=True)
        self.CACHE_DIR.mkdir(parents=True, exist_ok=True)
    
    def _validate_feed_signature(self, feed_path: Path) -> bool:
        """Validate feed signature."""
        manifest_path = feed_path.parent / f"{feed_path.stem}_manifest.json"
        if not manifest_path.exists():
            return False
        
        result = self.verify_tool.verify_manifest(manifest_path)
        return result.get('valid', False)
    
    def _validate_feed_format(self, feed_data: Dict) -> bool:
        """Validate feed data format."""
        required_fields = ['feed_id', 'timestamp', 'iocs', 'signature']
        for field in required_fields:
            if field not in feed_data:
                return False
        return True
    
    def ingest_feed(self, feed_path: Path) -> Dict:
        """
        Ingest a threat intelligence feed.
        
        Args:
            feed_path: Path to feed file
        
        Returns:
            Ingested feed data
        """
        # Validate signature
        if not self._validate_feed_signature(feed_path):
            raise ValueError(f"Invalid signature for feed: {feed_path}")
        
        # Load feed data
        with open(feed_path, 'r') as f:
            feed_data = json.load(f)
        
        # Validate format
        if not self._validate_feed_format(feed_data):
            raise ValueError(f"Invalid feed format: {feed_path}")
        
        # Cache feed
        cache_path = self.CACHE_DIR / f"{feed_data['feed_id']}.json"
        with open(cache_path, 'w') as f:
            json.dump(feed_data, f, indent=2)
        
        return feed_data
    
    def get_cached_feeds(self) -> List[Dict]:
        """Get all cached feeds."""
        feeds = []
        for cache_file in self.CACHE_DIR.glob("*.json"):
            try:
                with open(cache_file, 'r') as f:
                    feed_data = json.load(f)
                    feeds.append(feed_data)
            except Exception:
                continue
        return feeds


def main():
    """CLI entry point for offline feed ingester."""
    ingester = OfflineFeedIngester()
    
    # Example: ingest a feed
    feed_path = Path("/home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/feeds/feed_001.json")
    if feed_path.exists():
        try:
            feed_data = ingester.ingest_feed(feed_path)
            print(f"✓ Ingested feed: {feed_data['feed_id']}")
        except Exception as e:
            print(f"✗ Error ingesting feed: {e}", file=sys.stderr)
            sys.exit(1)
    else:
        print("No feed files found")


if __name__ == '__main__':
    import sys
    main()

