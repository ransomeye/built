# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/ingestion/auto_feed_fetcher.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Automatic threat intelligence feed fetcher with retry logic and internet detection

"""
Automatic Feed Fetcher: Automatically fetches threat intelligence feeds daily with hourly retries.
Checks for internet availability before attempting fetches.
"""

import os
import sys
import subprocess
import socket
import time
import json
from pathlib import Path
from datetime import datetime, timedelta
from typing import Dict, List, Optional
import logging

# Ensure log directory exists
log_dir = Path('/var/log/ransomeye')
log_dir.mkdir(parents=True, exist_ok=True)

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('/var/log/ransomeye/feed_fetcher.log') if log_dir.exists() else logging.StreamHandler(sys.stdout),
        logging.StreamHandler(sys.stdout)
    ]
)
logger = logging.getLogger('feed_fetcher')

# Add current directory to path
sys.path.insert(0, str(Path(__file__).parent))

from malwarebazaar_feed import MalwareBazaarFeedCollector, FeedError as MBFeedError
from wiz_feed import WizFeedCollector, FeedError as WizFeedError
from ransomware_live_feed import RansomwareLiveFeedCollector, FeedError as RLFeedError
try:
    from additional_sources import get_all_feed_collectors
except ImportError:
    get_all_feed_collectors = None

# State file to track last successful fetch
STATE_FILE = Path("/var/lib/ransomeye/feed_fetcher_state.json")
STATE_FILE.parent.mkdir(parents=True, exist_ok=True)

# Retry configuration
MAX_RETRIES = 24  # Retry every hour for 24 hours
RETRY_INTERVAL = 3600  # 1 hour in seconds


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
        ('abuse.ch', 443),  # MalwareBazaar
    ]
    
    for host, port in test_hosts:
        try:
            sock = socket.create_connection((host, port), timeout=timeout)
            sock.close()
            logger.debug(f"Internet connectivity confirmed via {host}:{port}")
            return True
        except (socket.error, OSError):
            continue
    
    logger.warning("No internet connectivity detected")
    return False


def load_state() -> Dict:
    """Load fetcher state from file."""
    if STATE_FILE.exists():
        try:
            with open(STATE_FILE, 'r') as f:
                return json.load(f)
        except Exception as e:
            logger.warning(f"Failed to load state: {e}")
    
    return {
        'last_successful_fetch': None,
        'last_attempt': None,
        'retry_count': 0,
        'feeds_status': {}
    }


def save_state(state: Dict):
    """Save fetcher state to file."""
    try:
        with open(STATE_FILE, 'w') as f:
            json.dump(state, f, indent=2)
    except Exception as e:
        logger.error(f"Failed to save state: {e}")


def should_fetch(state: Dict) -> bool:
    """
    Determine if feeds should be fetched.
    
    Returns:
        True if fetch should be attempted
    """
    last_success = state.get('last_successful_fetch')
    
    # If never fetched, always try
    if not last_success:
        return True
    
    # Parse last successful fetch time
    try:
        last_time = datetime.fromisoformat(last_success.replace('Z', '+00:00'))
        time_since = datetime.now(last_time.tzinfo) - last_time
        
        # Fetch if more than 24 hours since last success
        if time_since.total_seconds() > 86400:  # 24 hours
            return True
        
        # Also fetch if retry count indicates we should retry
        retry_count = state.get('retry_count', 0)
        if retry_count < MAX_RETRIES:
            last_attempt = state.get('last_attempt')
            if last_attempt:
                try:
                    attempt_time = datetime.fromisoformat(last_attempt.replace('Z', '+00:00'))
                    time_since_attempt = datetime.now(attempt_time.tzinfo) - attempt_time
                    
                    # Retry if more than 1 hour since last attempt
                    if time_since_attempt.total_seconds() >= RETRY_INTERVAL:
                        return True
                except Exception:
                    pass
        
        return False
    except Exception as e:
        logger.warning(f"Error parsing last fetch time: {e}")
        return True


def fetch_feeds_with_retry() -> Dict:
    """
    Fetch all feeds with retry logic.
    
    Returns:
        Dictionary with fetch results
    """
    state = load_state()
    
    # Check if we should fetch
    if not should_fetch(state):
        logger.info("Skipping fetch - too soon since last successful fetch")
        return state.get('feeds_status', {})
    
    # Check internet connectivity
    if not check_internet_connectivity():
        logger.warning("No internet connectivity - skipping fetch")
        state['retry_count'] = state.get('retry_count', 0) + 1
        state['last_attempt'] = datetime.utcnow().isoformat() + 'Z'
        save_state(state)
        return {}
    
    logger.info("Starting automatic feed fetch...")
    state['last_attempt'] = datetime.utcnow().isoformat() + 'Z'
    
    results = {
        'malwarebazaar': {'success': False, 'samples': 0},
        'wiz': {'success': False, 'iocs': 0},
        'ransomware_live': {'success': False, 'groups': 0, 'victims': 0}
    }
    
    all_success = True
    
    # 1. MalwareBazaar
    try:
        logger.info("Fetching MalwareBazaar feed...")
        mb_collector = MalwareBazaarFeedCollector()
        samples, success = mb_collector.fetch_recent_samples(limit=100)
        if success and samples:
            mb_collector.cache_samples(samples)
            results['malwarebazaar']['success'] = True
            results['malwarebazaar']['samples'] = len(samples)
            logger.info(f"✓ MalwareBazaar: {len(samples)} samples cached")
        else:
            # Try loading from cache
            samples = mb_collector.load_cached_samples()
            if samples:
                results['malwarebazaar']['samples'] = len(samples)
                logger.warning(f"⚠ MalwareBazaar fetch failed, using {len(samples)} cached samples")
    except MBFeedError as e:
        logger.error(f"MalwareBazaar feed error (system continues): {e}")
        all_success = False
        # Try loading from cache
        try:
            mb_collector = MalwareBazaarFeedCollector()
            samples = mb_collector.load_cached_samples()
            if samples:
                results['malwarebazaar']['samples'] = len(samples)
                logger.info(f"Using {len(samples)} cached MalwareBazaar samples")
            else:
                logger.warning("✗ MalwareBazaar: No cached samples available")
        except Exception as cache_error:
            logger.warning(f"Failed to load cached MalwareBazaar samples: {cache_error}")
    except Exception as e:
        logger.error(f"✗ MalwareBazaar error: {e}")
        all_success = False
    
    # 2. Wiz.io
    try:
        logger.info("Fetching Wiz.io feed...")
        wiz_collector = WizFeedCollector()
        stix_data, success = wiz_collector.fetch_stix_feed()
        if success and stix_data:
            iocs = wiz_collector.parse_stix_objects(stix_data)
            wiz_collector.cache_feed(stix_data)
            results['wiz']['success'] = True
            results['wiz']['iocs'] = len(iocs)
            logger.info(f"✓ Wiz.io: {len(iocs)} IOCs cached")
        else:
            # Try loading from cache
            iocs = wiz_collector.load_cached_feeds()
            if iocs:
                results['wiz']['iocs'] = len(iocs)
                logger.warning(f"⚠ Wiz.io fetch failed, using {len(iocs)} cached IOCs")
            else:
                logger.warning("✗ Wiz.io: No cached IOCs available")
    except WizFeedError as e:
        logger.error(f"Wiz.io feed error (system continues): {e}")
        all_success = False
        # Try loading from cache
        try:
            wiz_collector = WizFeedCollector()
            iocs = wiz_collector.load_cached_feeds()
            if iocs:
                results['wiz']['iocs'] = len(iocs)
                logger.info(f"Using {len(iocs)} cached Wiz.io IOCs")
        except Exception:
            pass
    except Exception as e:
        logger.error(f"✗ Wiz.io error: {e}")
        all_success = False
    
    # 3. Ransomware.live
    try:
        logger.info("Fetching Ransomware.live feed...")
        rl_collector = RansomwareLiveFeedCollector()
        groups, groups_success = rl_collector.fetch_groups()
        victims, victims_success = rl_collector.fetch_recent_victims(limit=100)
        
        if (groups_success or victims_success) and (groups or victims):
            rl_collector.cache_data(groups, victims)
            results['ransomware_live']['success'] = True
            results['ransomware_live']['groups'] = len(groups)
            results['ransomware_live']['victims'] = len(victims)
            logger.info(f"✓ Ransomware.live: {len(groups)} groups, {len(victims)} victims cached")
        else:
            # Try loading from cache
            cached_data = rl_collector.load_cached_data()
            if cached_data['groups'] or cached_data['victims']:
                results['ransomware_live']['groups'] = len(cached_data['groups'])
                results['ransomware_live']['victims'] = len(cached_data['victims'])
                logger.warning(f"⚠ Ransomware.live fetch failed, using cached data")
            else:
                logger.warning("✗ Ransomware.live: No cached data available")
    except RLFeedError as e:
        logger.error(f"Ransomware.live feed error (system continues): {e}")
        all_success = False
        # Try loading from cache
        try:
            rl_collector = RansomwareLiveFeedCollector()
            cached_data = rl_collector.load_cached_data()
            if cached_data['groups'] or cached_data['victims']:
                results['ransomware_live']['groups'] = len(cached_data['groups'])
                results['ransomware_live']['victims'] = len(cached_data['victims'])
                logger.info(f"Using cached Ransomware.live data")
        except Exception:
            pass
    except Exception as e:
        logger.error(f"✗ Ransomware.live error: {e}")
        all_success = False
    
    # 4. Additional feed sources (optional)
    try:
        logger.info("Fetching additional feed sources...")
        additional_collectors = get_all_feed_collectors()
        for collector in additional_collectors:
            try:
                logger.info(f"  Fetching {collector.source_name}...")
                data = collector.fetch()
                if data:
                    cache_path = collector.cache(data)
                    iocs = collector.parse(data)
                    if collector.source_name not in results:
                        results[collector.source_name] = {'success': False, 'iocs': 0}
                    results[collector.source_name]['success'] = True
                    results[collector.source_name]['iocs'] = len(iocs)
                    logger.info(f"  ✓ {collector.source_name}: {len(iocs)} IOCs cached")
                else:
                    # Try loading from cache
                    iocs = collector.load_cached()
                    if iocs:
                        if collector.source_name not in results:
                            results[collector.source_name] = {'success': False, 'iocs': 0}
                        results[collector.source_name]['iocs'] = len(iocs)
                        logger.warning(f"  ⚠ {collector.source_name} fetch failed, using {len(iocs)} cached IOCs")
            except Exception as e:
                logger.warning(f"  ✗ {collector.source_name} error: {e}")
    except Exception as e:
        logger.warning(f"Additional sources error: {e}")
    
    # Update state
    if all_success:
        state['last_successful_fetch'] = datetime.utcnow().isoformat() + 'Z'
        state['retry_count'] = 0
        logger.info("✓ All feeds fetched successfully")
    else:
        state['retry_count'] = state.get('retry_count', 0) + 1
        logger.warning(f"⚠ Some feeds failed (retry count: {state['retry_count']}/{MAX_RETRIES})")
    
    state['feeds_status'] = results
    save_state(state)
    
    return results


def main():
    """Main entry point for automatic feed fetcher."""
    try:
        results = fetch_feeds_with_retry()
        
        # Log summary
        logger.info("=" * 80)
        logger.info("Feed Fetch Summary")
        logger.info("=" * 80)
        logger.info(f"MalwareBazaar: {results.get('malwarebazaar', {}).get('samples', 0)} samples")
        logger.info(f"Wiz.io: {results.get('wiz', {}).get('iocs', 0)} IOCs")
        logger.info(f"Ransomware.live: {results.get('ransomware_live', {}).get('groups', 0)} groups, {results.get('ransomware_live', {}).get('victims', 0)} victims")
        
        return 0 if any(r.get('success', False) for r in results.values()) else 1
    
    except Exception as e:
        logger.error(f"Fatal error in feed fetcher: {e}", exc_info=True)
        return 1


if __name__ == '__main__':
    sys.exit(main())

