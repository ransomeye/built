# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/ingestion/fetch_all_feeds.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Fetches all threat intelligence feeds and caches them for offline training

"""
Unified Feed Fetcher: Fetches all threat intelligence feeds and caches them locally.
Run this script periodically to update cached feeds for offline training.
"""

import os
import sys
from pathlib import Path
from datetime import datetime

# Add current directory to path
sys.path.insert(0, str(Path(__file__).parent))

from malwarebazaar_feed import MalwareBazaarFeedCollector
from wiz_feed import WizFeedCollector
from ransomware_live_feed import RansomwareLiveFeedCollector


def fetch_all_feeds(use_cache: bool = False):
    """
    Fetch all threat intelligence feeds.
    
    Args:
        use_cache: If True, only load from cache, don't fetch new data
    """
    print("=" * 80)
    print("RansomEye Threat Intelligence Feed Fetcher")
    print("=" * 80)
    print()
    print(f"Mode: {'Cache-only' if use_cache else 'Fetch and cache'}")
    print()
    
    results = {
        'malwarebazaar': {'samples': 0, 'cached': False},
        'wiz': {'iocs': 0, 'cached': False},
        'ransomware_live': {'groups': 0, 'victims': 0, 'cached': False}
    }
    
    # 1. MalwareBazaar
    print("1. MalwareBazaar Feed...")
    try:
        mb_collector = MalwareBazaarFeedCollector()
        if use_cache:
            samples = mb_collector.load_cached_samples()
            print(f"   ✓ Loaded {len(samples)} samples from cache")
        else:
            print("   Fetching recent samples...")
            samples = mb_collector.fetch_recent_samples(limit=100)
            if samples:
                cache_path = mb_collector.cache_samples(samples)
                print(f"   ✓ Cached {len(samples)} samples to {cache_path}")
            else:
                # Try loading from cache if fetch failed
                samples = mb_collector.load_cached_samples()
                print(f"   ⚠ Fetch failed, loaded {len(samples)} samples from cache")
        
        results['malwarebazaar']['samples'] = len(samples)
        results['malwarebazaar']['cached'] = True
    except Exception as e:
        print(f"   ✗ Error: {e}")
    
    print()
    
    # 2. Wiz.io
    print("2. Wiz.io Cloud Threat Landscape Feed...")
    try:
        wiz_collector = WizFeedCollector()
        if use_cache:
            iocs = wiz_collector.load_cached_feeds()
            print(f"   ✓ Loaded {len(iocs)} IOCs from cache")
        else:
            print("   Fetching STIX feed...")
            stix_data = wiz_collector.fetch_stix_feed()
            if stix_data:
                iocs = wiz_collector.parse_stix_objects(stix_data)
                cache_path = wiz_collector.cache_feed(stix_data)
                print(f"   ✓ Cached {len(iocs)} IOCs to {cache_path}")
            else:
                # Try loading from cache if fetch failed
                iocs = wiz_collector.load_cached_feeds()
                print(f"   ⚠ Fetch failed, loaded {len(iocs)} IOCs from cache")
        
        results['wiz']['iocs'] = len(iocs)
        results['wiz']['cached'] = True
    except Exception as e:
        print(f"   ✗ Error: {e}")
    
    print()
    
    # 3. Ransomware.live
    print("3. Ransomware.live Feed...")
    try:
        rl_collector = RansomwareLiveFeedCollector()
        if use_cache:
            data = rl_collector.load_cached_data()
            print(f"   ✓ Loaded {len(data['groups'])} groups and {len(data['victims'])} victims from cache")
        else:
            print("   Fetching groups and victims...")
            groups = rl_collector.fetch_groups()
            victims = rl_collector.fetch_recent_victims(limit=100)
            if groups or victims:
                cache_path = rl_collector.cache_data(groups, victims)
                print(f"   ✓ Cached {len(groups)} groups and {len(victims)} victims to {cache_path}")
            else:
                # Try loading from cache if fetch failed
                data = rl_collector.load_cached_data()
                print(f"   ⚠ Fetch failed, loaded {len(data['groups'])} groups and {len(data['victims'])} victims from cache")
                groups = data['groups']
                victims = data['victims']
        
        results['ransomware_live']['groups'] = len(groups) if not use_cache else len(data.get('groups', []))
        results['ransomware_live']['victims'] = len(victims) if not use_cache else len(data.get('victims', []))
        results['ransomware_live']['cached'] = True
    except Exception as e:
        print(f"   ✗ Error: {e}")
    
    print()
    print("=" * 80)
    print("Feed Fetch Summary")
    print("=" * 80)
    print(f"MalwareBazaar: {results['malwarebazaar']['samples']} samples")
    print(f"Wiz.io: {results['wiz']['iocs']} IOCs")
    print(f"Ransomware.live: {results['ransomware_live']['groups']} groups, {results['ransomware_live']['victims']} victims")
    print()
    print("✓ All feeds processed")
    print()
    print("Next steps:")
    print("  1. Run enhance_training_with_feeds.py to generate enhanced training data")
    print("  2. Run train_baseline_models.py with --use-feeds flag to train with enhanced data")
    print()


def main():
    """CLI entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description='Fetch all threat intelligence feeds')
    parser.add_argument('--cache-only', action='store_true',
                       help='Only load from cache, do not fetch new data')
    parser.add_argument('--malwarebazaar-key', default=None,
                       help='MalwareBazaar API key (or use MALWARBAZAAR_AUTH_KEY env var)')
    parser.add_argument('--ransomware-live-key', default=None,
                       help='Ransomware.live API key (or use RANSOMWARE_LIVE_API_KEY env var)')
    
    args = parser.parse_args()
    
    # Set environment variables if provided
    if args.malwarebazaar_key:
        os.environ['MALWARBAZAAR_AUTH_KEY'] = args.malwarebazaar_key
    if args.ransomware_live_key:
        os.environ['RANSOMWARE_LIVE_API_KEY'] = args.ransomware_live_key
    
    fetch_all_feeds(use_cache=args.cache_only)


if __name__ == '__main__':
    main()

