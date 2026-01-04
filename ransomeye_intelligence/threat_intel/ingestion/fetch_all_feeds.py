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

from malwarebazaar_feed import MalwareBazaarFeedCollector, FeedError
from wiz_feed import WizFeedCollector
from ransomware_live_feed import RansomwareLiveFeedCollector
try:
    from additional_sources import get_all_feed_collectors
except ImportError:
    get_all_feed_collectors = None


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
    
    # Track additional sources
    additional_sources_results = {}
    
    # 1. MalwareBazaar
    print("1. MalwareBazaar Feed...")
    try:
        mb_collector = MalwareBazaarFeedCollector()
        if use_cache:
            samples = mb_collector.load_cached_samples()
            print(f"   ✓ Loaded {len(samples)} samples from cache")
        else:
            print("   Fetching recent samples...")
            samples, success = mb_collector.fetch_recent_samples(limit=100)
            if success and samples:
                cache_path = mb_collector.cache_samples(samples)
                print(f"   ✓ Cached {len(samples)} samples to {cache_path}")
            else:
                # Try loading from cache if fetch failed
                samples = mb_collector.load_cached_samples()
                print(f"   ⚠ Fetch failed, loaded {len(samples)} samples from cache")
        
        results['malwarebazaar']['samples'] = len(samples)
        results['malwarebazaar']['cached'] = True
    except FeedError as e:
        print(f"   ⚠ Feed disabled or misconfigured: {e}")
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
            stix_data, success = wiz_collector.fetch_stix_feed()
            if success and stix_data:
                iocs = wiz_collector.parse_stix_objects(stix_data)
                cache_path = wiz_collector.cache_feed(stix_data)
                print(f"   ✓ Cached {len(iocs)} IOCs to {cache_path}")
            else:
                # Try loading from cache if fetch failed
                iocs = wiz_collector.load_cached_feeds()
                print(f"   ⚠ Fetch failed, loaded {len(iocs)} IOCs from cache")
        
        results['wiz']['iocs'] = len(iocs)
        results['wiz']['cached'] = True
    except FeedError as e:
        print(f"   ⚠ Feed disabled or misconfigured: {e}")
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
            groups = data['groups']
            victims = data['victims']
        else:
            print("   Fetching groups and victims...")
            groups, groups_success = rl_collector.fetch_groups()
            victims, victims_success = rl_collector.fetch_recent_victims(limit=100)
            if (groups_success or victims_success) and (groups or victims):
                cache_path = rl_collector.cache_data(groups, victims)
                print(f"   ✓ Cached {len(groups)} groups and {len(victims)} victims to {cache_path}")
            else:
                # Try loading from cache if fetch failed
                data = rl_collector.load_cached_data()
                print(f"   ⚠ Fetch failed, loaded {len(data['groups'])} groups and {len(data['victims'])} victims from cache")
                groups = data['groups']
                victims = data['victims']
        
        results['ransomware_live']['groups'] = len(groups)
        results['ransomware_live']['victims'] = len(victims)
        results['ransomware_live']['cached'] = True
    except FeedError as e:
        print(f"   ⚠ Feed disabled or misconfigured: {e}")
    except Exception as e:
        print(f"   ✗ Error: {e}")
    
    print()
    
    # 4. Additional feed sources (URLhaus, ThreatFox, OTX, VirusTotal, etc.)
    if get_all_feed_collectors:
        print("4. Additional Feed Sources...")
        try:
            additional_collectors = get_all_feed_collectors()
            for collector in additional_collectors:
                source_name = collector.source_name
                print(f"   {source_name}...")
                try:
                    if use_cache:
                        iocs = collector.load_cached()
                        print(f"      ✓ Loaded {len(iocs)} IOCs from cache")
                        additional_sources_results[source_name] = {'iocs': len(iocs), 'cached': True}
                    else:
                        data = collector.fetch()
                        if data:
                            cache_path = collector.cache(data)
                            iocs = collector.parse(data)
                            print(f"      ✓ Cached {len(iocs)} IOCs to {cache_path}")
                            additional_sources_results[source_name] = {'iocs': len(iocs), 'cached': True}
                        else:
                            # Try loading from cache if fetch failed
                            iocs = collector.load_cached()
                            if iocs:
                                print(f"      ⚠ Fetch failed, loaded {len(iocs)} IOCs from cache")
                                additional_sources_results[source_name] = {'iocs': len(iocs), 'cached': True}
                            else:
                                print(f"      ✗ Failed to fetch and no cached data available")
                                additional_sources_results[source_name] = {'iocs': 0, 'cached': False}
                except Exception as e:
                    print(f"      ✗ Error: {e}")
                    additional_sources_results[source_name] = {'iocs': 0, 'cached': False}
        except Exception as e:
            print(f"   ✗ Error loading additional sources: {e}")
    else:
        print("4. Additional Feed Sources...")
        print("   ⚠ Additional sources module not available")
    
    print()
    print("=" * 80)
    print("Feed Fetch Summary")
    print("=" * 80)
    print(f"MalwareBazaar: {results['malwarebazaar']['samples']} samples")
    print(f"Wiz.io: {results['wiz']['iocs']} IOCs")
    print(f"Ransomware.live: {results['ransomware_live']['groups']} groups, {results['ransomware_live']['victims']} victims")
    
    if additional_sources_results:
        print("\nAdditional Sources:")
        for source_name, source_results in additional_sources_results.items():
            print(f"  {source_name}: {source_results['iocs']} IOCs")
    
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
                       help='MalwareBazaar API key (or use RANSOMEYE_FEED_MALWAREBAZAAR_API_KEY env var)')
    parser.add_argument('--ransomware-live-key', default=None,
                       help='Ransomware.live API key (or use RANSOMEYE_FEED_RANSOMWARELIVE_API_KEY env var)')
    
    args = parser.parse_args()
    
    # Set environment variables if provided (use correct env var names)
    if args.malwarebazaar_key:
        os.environ['RANSOMEYE_FEED_MALWAREBAZAAR_API_KEY'] = args.malwarebazaar_key
    if args.ransomware_live_key:
        os.environ['RANSOMEYE_FEED_RANSOMWARELIVE_API_KEY'] = args.ransomware_live_key
    
    fetch_all_feeds(use_cache=args.cache_only)


if __name__ == '__main__':
    main()

