# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/init_threat_intel.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Initialize threat intelligence database and load cached feeds (PROMPT-45)

"""
Threat Intelligence Initialization Script (PROMPT-45):
- Creates threat_intel database table if it doesn't exist
- Loads cached threat intelligence feeds into database
- Normalizes and correlates IOCs
- Fails-closed if initialization fails
"""

import os
import sys
import json
import psycopg2
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Optional
import logging
import hashlib

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent))

from normalization.ontology import ThreatIntelligenceOntology, IOCType
from fusion.correlation import ThreatIntelligenceCorrelator
from fusion.confidence import ThreatIntelligenceConfidence

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger('init_threat_intel')

CACHE_DIR = Path("/home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/cache")
# Database connection - use peer authentication (same user as postgres)
DB_NAME = os.environ.get("DB_NAME", "ransomeye")
DB_USER = os.environ.get("DB_USER", "gagan")
DB_HOST = os.environ.get("DB_HOST", "localhost")
DB_PASSWORD = os.environ.get("DB_PASS", "gagan")
DB_PORT = os.environ.get("DB_PORT", "5432")


def create_threat_intel_table(conn):
    """Verify threat_intel table exists (created by admin)."""
    cursor = conn.cursor()
    try:
        cursor.execute("SET search_path = ransomeye, public;")
        cursor.execute("""
            SELECT EXISTS (
                SELECT FROM information_schema.tables 
                WHERE table_schema = 'ransomeye' 
                AND table_name = 'threat_intel'
            );
        """)
        exists = cursor.fetchone()[0]
        if not exists:
            logger.error("✗ threat_intel table does not exist. Please create it first.")
            return False
        logger.info("✓ Threat intel table verified")
        return True
    except Exception as e:
        logger.error(f"✗ Failed to create threat_intel table: {e}")
        conn.rollback()
        return False
    finally:
        cursor.close()


def load_cached_feeds() -> Dict:
    """Load all cached feed data."""
    feed_data = {
        'malwarebazaar': [],
        'ransomware_live': {'groups': [], 'victims': []},
        'wiz': [],
        'threatfox': [],
        'urlhaus': []
    }
    
    # Load MalwareBazaar samples
    mb_cache = CACHE_DIR / "malwarebazaar"
    if mb_cache.exists():
        for cache_file in mb_cache.glob("*.json"):
            try:
                with open(cache_file, 'r') as f:
                    data = json.load(f)
                    if isinstance(data, list):
                        feed_data['malwarebazaar'].extend(data)
                    elif isinstance(data, dict) and 'samples' in data:
                        feed_data['malwarebazaar'].extend(data['samples'])
            except Exception as e:
                logger.warning(f"Failed to load {cache_file}: {e}")
    
    # Load Ransomware.live data
    rl_cache = CACHE_DIR / "ransomware_live"
    if rl_cache.exists():
        for cache_file in rl_cache.glob("*.json"):
            try:
                with open(cache_file, 'r') as f:
                    data = json.load(f)
                    if isinstance(data, dict):
                        feed_data['ransomware_live']['groups'].extend(data.get('groups', []))
                        feed_data['ransomware_live']['victims'].extend(data.get('victims', []))
            except Exception as e:
                logger.warning(f"Failed to load {cache_file}: {e}")
    
    # Load WIZ STIX data
    wiz_cache = CACHE_DIR / "wiz"
    if wiz_cache.exists():
        for cache_file in wiz_cache.glob("*.json"):
            try:
                with open(cache_file, 'r') as f:
                    data = json.load(f)
                    if isinstance(data, dict) and 'objects' in data:
                        # STIX format
                        feed_data['wiz'].extend(data['objects'])
                    elif isinstance(data, list):
                        feed_data['wiz'].extend(data)
            except Exception as e:
                logger.warning(f"Failed to load {cache_file}: {e}")
    
    # Load ThreatFox data
    tf_cache = CACHE_DIR / "threatfox"
    if tf_cache.exists():
        for cache_file in tf_cache.glob("*.json"):
            try:
                with open(cache_file, 'r') as f:
                    data = json.load(f)
                    if isinstance(data, list):
                        feed_data['threatfox'].extend(data)
                    elif isinstance(data, dict) and 'iocs' in data:
                        feed_data['threatfox'].extend(data['iocs'])
            except Exception as e:
                logger.warning(f"Failed to load {cache_file}: {e}")
    
    # Load URLhaus data
    uh_cache = CACHE_DIR / "urlhaus"
    if uh_cache.exists():
        for cache_file in uh_cache.glob("*.json"):
            try:
                with open(cache_file, 'r') as f:
                    data = json.load(f)
                    if isinstance(data, list):
                        feed_data['urlhaus'].extend(data)
                    elif isinstance(data, dict) and 'urls' in data:
                        feed_data['urlhaus'].extend(data['urls'])
            except Exception as e:
                logger.warning(f"Failed to load {cache_file}: {e}")
    
    return feed_data


def extract_iocs_from_feeds(feed_data: Dict, ontology: ThreatIntelligenceOntology) -> List[Dict]:
    """Extract and normalize IOCs from feed data."""
    iocs = []
    
    # Extract from MalwareBazaar
    for sample in feed_data.get('malwarebazaar', []):
        if isinstance(sample, dict):
            # Extract hash IOCs
            for hash_type in ['sha256_hash', 'md5_hash', 'sha1_hash']:
                if hash_type in sample and sample[hash_type]:
                    ioc = {
                        'type': 'hash',
                        'value': sample[hash_type],
                        'source': 'malwarebazaar',
                        'confidence': 0.8,
                        'tags': sample.get('tags', []),
                        'metadata': sample
                    }
                    iocs.append(ontology.normalize_ioc(ioc))
    
    # Extract from Ransomware.live
    for group in feed_data.get('ransomware_live', {}).get('groups', []):
        if isinstance(group, dict):
            # Extract domain IOCs
            if 'name' in group:
                ioc = {
                    'type': 'domain',
                    'value': group['name'],
                    'source': 'ransomware_live',
                    'confidence': 0.9,
                    'tags': ['ransomware', 'group'],
                    'metadata': group
                }
                iocs.append(ontology.normalize_ioc(ioc))
    
    # Extract from Wiz STIX
    for obj in feed_data.get('wiz', []):
        if isinstance(obj, dict):
            obj_type = obj.get('type', '')
            if obj_type == 'indicator':
                pattern = obj.get('pattern', '')
                # Simple pattern extraction (STIX patterns are complex)
                if 'ipv4-addr:value' in pattern:
                    # Extract IP from pattern
                    import re
                    ip_match = re.search(r"ipv4-addr:value\s*=\s*'([^']+)'", pattern)
                    if ip_match:
                        ioc = {
                            'type': 'ip',
                            'value': ip_match.group(1),
                            'source': 'wiz',
                            'confidence': 0.85,
                            'tags': obj.get('labels', []),
                            'metadata': obj
                        }
                        iocs.append(ontology.normalize_ioc(ioc))
    
    # Extract from ThreatFox
    for ioc_data in feed_data.get('threatfox', []):
        if isinstance(ioc_data, dict):
            ioc_type = ioc_data.get('ioc_type', '').lower()
            ioc_value = ioc_data.get('ioc_value', '')
            if ioc_type and ioc_value:
                ioc = {
                    'type': ioc_type,
                    'value': ioc_value,
                    'source': 'threatfox',
                    'confidence': 0.75,
                    'tags': ioc_data.get('threat_type', []),
                    'metadata': ioc_data
                }
                iocs.append(ontology.normalize_ioc(ioc))
    
    # Extract from URLhaus
    for url_data in feed_data.get('urlhaus', []):
        if isinstance(url_data, dict):
            url = url_data.get('url', '')
            if url:
                ioc = {
                    'type': 'url',
                    'value': url,
                    'source': 'urlhaus',
                    'confidence': 0.8,
                    'tags': ['malware', 'url'],
                    'metadata': url_data
                }
                iocs.append(ontology.normalize_ioc(ioc))
    
    return iocs


def insert_iocs(conn, iocs: List[Dict], correlator: ThreatIntelligenceCorrelator, 
                 confidence_calc: ThreatIntelligenceConfidence):
    """Insert IOCs into database with correlation and confidence scoring."""
    cursor = conn.cursor()
    inserted_count = 0
    updated_count = 0
    
    try:
        for ioc in iocs:
            # Correlate IOC
            correlated = correlator.correlate_ioc(ioc)
            
            # Calculate confidence
            final_confidence = confidence_calc.calculate_correlated_confidence(correlated)
            
            # Prepare data
            ioc_type = ioc.get('type', 'unknown')
            ioc_value = ioc.get('value', '')
            source = ioc.get('source', 'unknown')
            tags = ioc.get('tags', [])
            if isinstance(tags, str):
                tags = [tags]
            metadata = json.dumps(ioc.get('metadata', {}))
            
            # Try to insert or update
            cursor.execute("SET search_path = ransomeye, public;")
            cursor.execute("""
                INSERT INTO threat_intel (
                    ioc_type, ioc_value, source, confidence, 
                    tags, metadata, correlated_count, 
                    correlated_confidence, correlated_sources, 
                    first_seen, last_seen, updated_at
                )
                VALUES (%s, %s, %s, %s, %s, %s::jsonb, %s, %s, %s, now(), now(), now())
                ON CONFLICT (ioc_type, ioc_value, source) 
                DO UPDATE SET
                    confidence = GREATEST(threat_intel.confidence, EXCLUDED.confidence),
                    correlated_count = EXCLUDED.correlated_count,
                    correlated_confidence = EXCLUDED.correlated_confidence,
                    correlated_sources = EXCLUDED.correlated_sources,
                    last_seen = now(),
                    updated_at = now()
                RETURNING (xmax = 0) AS inserted
            """, (
                ioc_type, ioc_value, source, float(final_confidence),
                tags, metadata, correlated.get('correlation_count', 0),
                float(correlated.get('correlated_confidence', final_confidence)),
                correlated.get('sources', [source]), 
            ))
            
            result = cursor.fetchone()
            if result and result[0]:
                inserted_count += 1
                correlator.index_ioc(ioc)  # Index for future correlation
            else:
                updated_count += 1
        
        conn.commit()
        logger.info(f"✓ Inserted {inserted_count} new IOCs, updated {updated_count} existing IOCs")
        return inserted_count + updated_count
    except Exception as e:
        logger.error(f"✗ Failed to insert IOCs: {e}")
        conn.rollback()
        return 0
    finally:
        cursor.close()


def main():
    """Main initialization function."""
    logger.info("=" * 80)
    logger.info("RansomEye Threat Intelligence Initialization (PROMPT-45)")
    logger.info("=" * 80)
    
    # Step 1: Connect to database
    try:
        conn = psycopg2.connect(
            host=DB_HOST,
            port=DB_PORT,
            database=DB_NAME,
            user=DB_USER,
            password=DB_PASSWORD
        )
        logger.info("✓ Connected to database")
    except Exception as e:
        logger.error(f"✗ Failed to connect to database: {e}")
        logger.error("FAIL-CLOSED: Threat intelligence initialization failed")
        sys.exit(1)
    
    try:
        # Step 2: Create table
        if not create_threat_intel_table(conn):
            logger.error("FAIL-CLOSED: Failed to create threat_intel table")
            sys.exit(1)
        
        # Step 3: Load cached feeds
        logger.info("Loading cached threat intelligence feeds...")
        feed_data = load_cached_feeds()
        
        total_feeds = (
            len(feed_data['malwarebazaar']) +
            len(feed_data['ransomware_live']['groups']) +
            len(feed_data['ransomware_live']['victims']) +
            len(feed_data['wiz']) +
            len(feed_data['threatfox']) +
            len(feed_data['urlhaus'])
        )
        
        if total_feeds == 0:
            logger.warning("⚠ No cached feeds found. Threat intel will be empty.")
            logger.warning("Run fetch_all_feeds.py to fetch feeds first.")
        else:
            logger.info(f"✓ Loaded {total_feeds} feed items from cache")
        
        # Step 4: Extract and normalize IOCs
        logger.info("Extracting and normalizing IOCs...")
        ontology = ThreatIntelligenceOntology()
        iocs = extract_iocs_from_feeds(feed_data, ontology)
        logger.info(f"✓ Extracted {len(iocs)} IOCs")
        
        # Step 5: Correlate and score IOCs
        logger.info("Correlating and scoring IOCs...")
        correlator = ThreatIntelligenceCorrelator()
        confidence_calc = ThreatIntelligenceConfidence()
        
        # Index all IOCs for correlation
        for ioc in iocs:
            correlator.index_ioc(ioc)
        
        # Step 6: Insert into database
        logger.info("Inserting IOCs into database...")
        total_inserted = insert_iocs(conn, iocs, correlator, confidence_calc)
        
        if total_inserted == 0 and total_feeds > 0:
            logger.error("FAIL-CLOSED: Failed to insert any IOCs despite having feed data")
            sys.exit(1)
        
        # Step 7: Verify
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")
        cursor.execute("SELECT COUNT(*) FROM threat_intel")
        count = cursor.fetchone()[0]
        cursor.close()
        
        logger.info("=" * 80)
        logger.info(f"✓ Threat intelligence initialization complete")
        logger.info(f"  Total IOCs in database: {count}")
        logger.info("=" * 80)
        
        if count == 0:
            logger.warning("⚠ Threat intel table is empty. System will operate without threat intel.")
            logger.warning("This is acceptable for offline operation, but threat intel features will be disabled.")
        
    except Exception as e:
        logger.error(f"✗ Initialization failed: {e}")
        logger.error("FAIL-CLOSED: Threat intelligence initialization failed")
        sys.exit(1)
    finally:
        conn.close()


if __name__ == '__main__':
    main()

