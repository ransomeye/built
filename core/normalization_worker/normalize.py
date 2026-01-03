# Path and File Name : /home/ransomeye/rebuild/core/normalization_worker/normalize.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Minimal normalization worker - deterministic mapping from raw_events to normalized_events (PROMPT-39)

"""
Minimal Normalization Worker (PROMPT-39)

Purpose:
- Read unprocessed rows from ransomeye.raw_events
- Write corresponding rows into ransomeye.normalized_events
- Deterministic mapping only (no enrichment, correlation, or inference)
- Fail-closed on any error
- Idempotent (one raw event → one normalized event)
"""

import os
import sys
import time
import hashlib
import psycopg2
from psycopg2.extras import RealDictCursor
import json
import logging

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s [%(levelname)s] %(message)s'
)
logger = logging.getLogger(__name__)

def get_db_connection():
    """Get database connection from environment variables."""
    db_host = os.environ.get('DB_HOST', 'localhost')
    db_port = int(os.environ.get('DB_PORT', '5432'))
    db_name = os.environ.get('DB_NAME', 'ransomeye')
    db_user = os.environ.get('DB_USER', 'gagan')
    db_pass = os.environ.get('DB_PASS', 'gagan')
    
    conn = psycopg2.connect(
        host=db_host,
        port=db_port,
        database=db_name,
        user=db_user,
        password=db_pass
    )
    conn.set_session(autocommit=False)
    return conn

def get_or_create_normalization_component(conn):
    """Get or create normalization component for audit attribution."""
    cursor = conn.cursor()
    try:
        component_name = "ransomeye_normalization"
        instance_id = os.environ.get('HOSTNAME', 'localhost')
        
        # Try to find existing component
        cursor.execute("""
            SELECT component_id FROM ransomeye.components
            WHERE component_type = 'core_engine'::component_type
              AND component_name = %s
              AND (instance_id = %s OR (instance_id IS NULL AND %s IS NULL))
            LIMIT 1
        """, (component_name, instance_id, instance_id))
        
        row = cursor.fetchone()
        if row:
            component_id = row[0]
            # Update last_heartbeat_at
            cursor.execute("""
                UPDATE ransomeye.components 
                SET last_heartbeat_at = NOW() 
                WHERE component_id = %s
            """, (component_id,))
            conn.commit()
            return component_id
        
        # Create new component
        import uuid
        component_id = uuid.uuid4()
        cursor.execute("""
            INSERT INTO ransomeye.components 
                (component_id, component_type, component_name, instance_id, started_at, last_heartbeat_at)
            VALUES (%s, 'core_engine'::component_type, %s, %s, NOW(), NOW())
        """, (component_id, component_name, instance_id))
        conn.commit()
        return component_id
    finally:
        cursor.close()

def insert_immutable_audit_log(conn, actor_component_id, actor_agent_id, action, object_type, 
                               object_id, event_time, payload_json, payload_sha256):
    """Insert into immutable_audit_log (fail-closed)."""
    cursor = conn.cursor()
    try:
        # Get previous audit chain entry for hash chaining
        cursor.execute("""
            SELECT audit_id, chain_hash_sha256, payload_sha256
            FROM ransomeye.immutable_audit_log
            ORDER BY created_at DESC
            LIMIT 1
        """)
        
        row = cursor.fetchone()
        if row:
            prev_audit_id = row[0]
            prev_chain_hash = row[1]
            prev_payload_sha256 = row[2]
        else:
            prev_audit_id = None
            prev_chain_hash = bytes(32)  # 32 zero bytes
            prev_payload_sha256 = None
        
        # Compute chain hash: SHA256(prev_chain_hash || payload_sha256)
        chain_input = prev_chain_hash + payload_sha256
        chain_hash_sha256 = hashlib.sha256(chain_input).digest()
        
        # Insert audit log entry
        import uuid
        audit_id = uuid.uuid4()
        cursor.execute("""
            INSERT INTO ransomeye.immutable_audit_log (
                audit_id, actor_component_id, actor_agent_id, action, object_type, object_id, event_time,
                payload_json, payload_sha256, prev_audit_id, prev_payload_sha256, chain_hash_sha256, signature_status
            )
            VALUES (%s, %s, %s, %s, %s::text::trust_object_type, %s, %s, %s, %s, %s, %s, %s, 'unknown')
            RETURNING audit_id
        """, (
            audit_id,
            actor_component_id,
            actor_agent_id,
            action,
            object_type,
            object_id,
            event_time,
            json.dumps(payload_json),
            payload_sha256,
            prev_audit_id,
            prev_payload_sha256,
            chain_hash_sha256,
        ))
        
        result = cursor.fetchone()
        return result[0] if result else audit_id
    finally:
        cursor.close()

def compute_deterministic_key(raw_event_id, source_type, event_kind, observed_at_str):
    """Compute deterministic key from normalized fields (SHA-256, 32 bytes)."""
    key_data = f"{raw_event_id}|{source_type}|{event_kind}|{observed_at_str or ''}"
    return hashlib.sha256(key_data.encode('utf-8')).digest()

def extract_event_kind(payload_json):
    """Extract event_kind from payload (required field)."""
    if not payload_json:
        raise ValueError("payload_json is None or empty")
    
    # Try data.event_category first (most common)
    event_category = payload_json.get('data', {}).get('event_category')
    if event_category:
        return event_category
    
    # Fallback to event_type at root
    event_type = payload_json.get('event_type')
    if event_type:
        return event_type
    
    # Fallback to data.features.event_type
    features_event_type = payload_json.get('data', {}).get('features', {}).get('event_type')
    if features_event_type:
        return features_event_type
    
    # FAIL-CLOSED: event_kind is required
    raise ValueError("Cannot extract event_kind from payload - required field missing")

def normalize_event(raw_event):
    """Normalize a single raw_event into normalized_events format."""
    raw_event_id = raw_event['raw_event_id']
    source_type = raw_event['source_type']
    source_agent_id = raw_event['source_agent_id']
    source_component_id = raw_event['source_component_id']
    observed_at = raw_event['observed_at']
    payload_json = raw_event['payload_json']
    
    # Extract event_kind (required) - FAIL-CLOSED if missing
    try:
        event_kind = extract_event_kind(payload_json)
    except ValueError as e:
        logger.error(f"FAIL-CLOSED: Cannot normalize raw_event_id={raw_event_id}: {e}")
        raise
    
    # Extract optional event_subkind from payload
    event_subkind = None
    if payload_json:
        event_subkind = payload_json.get('data', {}).get('features', {}).get('event_type')
    
    # Extract severity from payload if present, otherwise default to 'info'
    severity = 'info'
    if payload_json:
        severity_from_payload = payload_json.get('data', {}).get('severity')
        if severity_from_payload:
            severity = severity_from_payload.lower()
    
    # Compute deterministic_key (required, 32 bytes SHA-256)
    observed_at_str = observed_at.isoformat() if observed_at else None
    deterministic_key = compute_deterministic_key(
        str(raw_event_id),
        source_type,
        event_kind,
        observed_at_str
    )
    
    # Build attributes JSONB from payload (preserve original structure)
    attributes = payload_json if payload_json else None
    
    return {
        'raw_event_id': raw_event_id,
        'observed_at': observed_at,
        'source_type': source_type,
        'source_agent_id': source_agent_id,
        'source_component_id': source_component_id,
        'event_kind': event_kind,
        'event_subkind': event_subkind,
        'severity': severity,
        'attributes': attributes,
        'deterministic_key': deterministic_key,
    }

def process_batch(conn, batch_size=100):
    """Process a batch of unprocessed raw_events."""
    cursor = conn.cursor(cursor_factory=RealDictCursor)
    
    try:
        # Find raw_events that don't have corresponding normalized_events (idempotency)
        cursor.execute("""
            SELECT re.raw_event_id, re.source_type, re.source_agent_id, re.source_component_id,
                   re.observed_at, re.payload_json
            FROM ransomeye.raw_events re
            WHERE NOT EXISTS (
                SELECT 1 FROM ransomeye.normalized_events ne
                WHERE ne.raw_event_id = re.raw_event_id
            )
            ORDER BY re.received_at ASC
            LIMIT %s
        """, (batch_size,))
        
        raw_events = cursor.fetchall()
        
        if not raw_events:
            return 0
        
        logger.info(f"Processing {len(raw_events)} raw_events for normalization")
        
        # PROMPT-40A: Get normalization component for audit attribution
        normalization_component_id = get_or_create_normalization_component(conn)
        
        normalized_count = 0
        error_count = 0
        
        for raw_event in raw_events:
            try:
                normalized = normalize_event(raw_event)
                
                # Insert into normalized_events
                cursor.execute("""
                    INSERT INTO ransomeye.normalized_events (
                        raw_event_id, observed_at, source_type, source_agent_id, source_component_id,
                        event_kind, event_subkind, severity, attributes, deterministic_key
                    )
                    VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s)
                    RETURNING normalized_event_id
                """, (
                    normalized['raw_event_id'],
                    normalized['observed_at'],
                    normalized['source_type'],
                    normalized['source_agent_id'],
                    normalized['source_component_id'],
                    normalized['event_kind'],
                    normalized['event_subkind'],
                    normalized['severity'],
                    json.dumps(normalized['attributes']) if normalized['attributes'] else None,
                    normalized['deterministic_key'],
                ))
                
                normalized_event_id = cursor.fetchone()[0]
                
                # PROMPT-40A: Audit NORMALIZED_EVENT_INSERT (after successful normalized_events INSERT)
                normalized_payload = {
                    "normalized_event_id": str(normalized_event_id),
                    "raw_event_id": str(normalized['raw_event_id']),
                    "source_type": normalized['source_type'],
                    "agent_id": str(normalized['source_agent_id']) if normalized['source_agent_id'] else None,
                    "event_kind": normalized['event_kind'],
                    "event_subkind": normalized['event_subkind'],
                    "severity": normalized['severity'],
                    "observed_at": normalized['observed_at'].isoformat() if normalized['observed_at'] else None,
                    "deterministic_key": normalized['deterministic_key'].hex() if isinstance(normalized['deterministic_key'], bytes) else normalized['deterministic_key']
                }
                normalized_payload_str = json.dumps(normalized_payload, sort_keys=True)
                normalized_payload_sha256 = hashlib.sha256(normalized_payload_str.encode()).digest()
                
                insert_immutable_audit_log(
                    conn,
                    normalization_component_id,
                    normalized['source_agent_id'],
                    "NORMALIZED_EVENT_INSERT",
                    "normalized_event",
                    normalized_event_id,
                    normalized['observed_at'],
                    normalized_payload,
                    normalized_payload_sha256,
                )
                
                normalized_count += 1
                
            except Exception as e:
                error_count += 1
                logger.error(f"FAIL-CLOSED: Failed to normalize raw_event_id={raw_event['raw_event_id']}: {e}")
                # Continue processing other events, but log the error
        
        # Commit batch
        conn.commit()
        
        if normalized_count > 0:
            logger.info(f"Normalized {normalized_count} events (errors: {error_count})")
        
        return normalized_count
        
    except Exception as e:
        conn.rollback()
        logger.error(f"FAIL-CLOSED: Batch processing failed: {e}")
        raise
    finally:
        cursor.close()

def main():
    """Main normalization worker loop."""
    logger.info("Normalization worker starting (PROMPT-39)")
    
    # Set search_path to ransomeye schema
    conn = get_db_connection()
    cursor = conn.cursor()
    cursor.execute("SET search_path = ransomeye, public;")
    conn.commit()
    cursor.close()
    
    batch_size = int(os.environ.get('NORMALIZATION_BATCH_SIZE', '100'))
    poll_interval = float(os.environ.get('NORMALIZATION_POLL_INTERVAL', '1.0'))
    
    try:
        while True:
            try:
                processed = process_batch(conn, batch_size)
                
                if processed == 0:
                    # No new events, sleep before next poll
                    time.sleep(poll_interval)
                else:
                    # Processed events, continue immediately for next batch
                    pass
                    
            except Exception as e:
                logger.error(f"Error in normalization loop: {e}")
                time.sleep(poll_interval)
                
    except KeyboardInterrupt:
        logger.info("Normalization worker stopping (SIGINT)")
    finally:
        conn.close()

if __name__ == '__main__':
    main()

