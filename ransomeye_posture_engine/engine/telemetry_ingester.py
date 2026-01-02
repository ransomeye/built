# Path and File Name : /home/ransomeye/rebuild/ransomeye_posture_engine/engine/telemetry_ingester.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Telemetry ingestion from Linux Agent, Windows Agent, and DPI Probe with Ed25519 signature verification

"""
Telemetry Ingester

Subscribes to telemetry events from:
- Linux Agent
- Windows Agent  
- DPI Probe

Database is UNTRUSTED - every telemetry record MUST be verified with Ed25519.
Fail-closed on missing or invalid signatures.
"""

import asyncio
import logging
from datetime import datetime, timedelta
from typing import List, Dict, Any, Optional
import asyncpg
from dataclasses import dataclass

from ..config import Config
from .signature_verifier import SignatureVerifier, SignatureVerificationError

logger = logging.getLogger("ransomeye_posture_engine.ingester")

@dataclass
class TelemetryEvent:
    """Normalized telemetry event with verified signature."""
    event_id: str
    producer_type: str  # "linux_agent", "windows_agent", "dpi_probe"
    host_id: Optional[str]
    timestamp: datetime
    event_type: str
    data: Dict[str, Any]
    signature_verified: bool  # Explicitly verified, not from database

class TelemetryIngester:
    """Ingests telemetry from agents and DPI probe with Ed25519 verification."""
    
    def __init__(self, config: Config, signature_verifier: SignatureVerifier):
        self.config = config
        self.signature_verifier = signature_verifier
        self.db_pool: Optional[asyncpg.Pool] = None
        self.last_poll_time: Optional[datetime] = None
        
    async def initialize(self):
        """Initialize database connection pool."""
        try:
            self.db_pool = await asyncpg.create_pool(
                host=self.config.db_host,
                port=self.config.db_port,
                database=self.config.db_name,
                user=self.config.db_user,
                password=self.config.db_pass,
                min_size=2,
                max_size=10,
            )
            logger.info("Database connection pool initialized")
        except Exception as e:
            logger.error(f"Failed to initialize database pool: {e}")
            raise
    
    async def close(self):
        """Close database connection pool."""
        if self.db_pool:
            await self.db_pool.close()
            logger.info("Database connection pool closed")
    
    async def fetch_new_telemetry(self, since: Optional[datetime] = None) -> List[TelemetryEvent]:
        """
        Fetch new telemetry events from database and verify Ed25519 signatures.
        
        Database is UNTRUSTED - every record is verified explicitly.
        Fail-closed on missing or invalid signatures.
        
        Args:
            since: Only fetch events after this timestamp. If None, uses last poll time.
        
        Returns:
            List of verified telemetry events.
        
        Raises:
            SignatureVerificationError: If any signature verification fails
        """
        if not self.db_pool:
            raise RuntimeError("Database pool not initialized")
        
        # Determine query start time
        if since:
            query_since = since
        elif self.last_poll_time:
            query_since = self.last_poll_time
        else:
            # First poll - get last hour
            query_since = datetime.utcnow() - timedelta(hours=1)
        
        events = []
        verification_failures = []
        
        try:
            async with self.db_pool.acquire() as conn:
                # Query telemetry table - get signature fields
                # Database is UNTRUSTED - we verify signatures ourselves
                query = """
                    SELECT 
                        event_id,
                        producer_id,
                        producer_type,
                        host_id,
                        timestamp,
                        event_type,
                        event_data,
                        signature,
                        signature_algorithm,
                        signature_valid
                    FROM telemetry_events
                    WHERE timestamp > $1
                        AND producer_type IN ('linux_agent', 'windows_agent', 'dpi_probe')
                    ORDER BY timestamp ASC
                """
                
                rows = await conn.fetch(query, query_since)
                
                for row in rows:
                    event_id = row['event_id']
                    producer_id = row['producer_id']
                    producer_type = row['producer_type']
                    
                    # Parse event data
                    event_data = row['event_data']
                    if isinstance(event_data, str):
                        import json
                        event_data = json.loads(event_data)
                    
                    # Verify Ed25519 signature - database is UNTRUSTED
                    signature = row.get('signature')
                    signature_algorithm = row.get('signature_algorithm')
                    
                    try:
                        # Explicit verification - ignore database's signature_valid flag
                        self.signature_verifier.verify_from_database_record(
                            event_id=event_id,
                            event_data=event_data,
                            signature=signature,
                            producer_id=producer_id,
                            signature_algorithm=signature_algorithm,
                            signature_valid=row.get('signature_valid')  # IGNORED - we verify ourselves
                        )
                        
                        # Signature verified - create event
                        event = TelemetryEvent(
                            event_id=event_id,
                            producer_type=producer_type,
                            host_id=row.get('host_id'),
                            timestamp=row['timestamp'],
                            event_type=row['event_type'],
                            data=event_data,
                            signature_verified=True,
                        )
                        events.append(event)
                    
                    except SignatureVerificationError as e:
                        # Fail-closed: log and track failure
                        logger.error(f"Ed25519 signature verification failed for event {event_id}: {e}")
                        verification_failures.append((event_id, str(e)))
                        # Continue processing other events, but track failures
                
                logger.info(f"Fetched {len(rows)} telemetry events, verified {len(events)}, failed {len(verification_failures)}")
                
                # Fail-closed: if any verification failed, raise error
                if verification_failures:
                    error_msg = f"Ed25519 signature verification failed for {len(verification_failures)} events (FAIL-CLOSED)"
                    logger.error(error_msg)
                    raise SignatureVerificationError(error_msg)
                
        except SignatureVerificationError:
            # Re-raise signature verification errors
            raise
        except Exception as e:
            logger.error(f"Error fetching telemetry: {e}")
            # Fail-closed: re-raise on database errors
            raise
        
        # Update last poll time
        self.last_poll_time = datetime.utcnow()
        
        return events
    
    async def get_host_telemetry(self, host_id: str, 
                                start_time: datetime, 
                                end_time: datetime) -> List[TelemetryEvent]:
        """
        Get telemetry for a specific host within a time range with Ed25519 verification.
        
        Database is UNTRUSTED - every record is verified explicitly.
        
        Args:
            host_id: Host identifier
            start_time: Start of time range
            end_time: End of time range
        
        Returns:
            List of verified telemetry events for the host.
        
        Raises:
            SignatureVerificationError: If any signature verification fails
        """
        if not self.db_pool:
            raise RuntimeError("Database pool not initialized")
        
        events = []
        verification_failures = []
        
        try:
            async with self.db_pool.acquire() as conn:
                query = """
                    SELECT 
                        event_id,
                        producer_id,
                        producer_type,
                        host_id,
                        timestamp,
                        event_type,
                        event_data,
                        signature,
                        signature_algorithm,
                        signature_valid
                    FROM telemetry_events
                    WHERE host_id = $1
                        AND timestamp >= $2
                        AND timestamp <= $3
                    ORDER BY timestamp ASC
                """
                
                rows = await conn.fetch(query, host_id, start_time, end_time)
                
                for row in rows:
                    event_id = row['event_id']
                    producer_id = row['producer_id']
                    
                    # Parse event data
                    event_data = row['event_data']
                    if isinstance(event_data, str):
                        import json
                        event_data = json.loads(event_data)
                    
                    # Verify Ed25519 signature - database is UNTRUSTED
                    signature = row.get('signature')
                    signature_algorithm = row.get('signature_algorithm')
                    
                    try:
                        # Explicit verification
                        self.signature_verifier.verify_from_database_record(
                            event_id=event_id,
                            event_data=event_data,
                            signature=signature,
                            producer_id=producer_id,
                            signature_algorithm=signature_algorithm,
                            signature_valid=row.get('signature_valid')  # IGNORED
                        )
                        
                        event = TelemetryEvent(
                            event_id=event_id,
                            producer_type=row['producer_type'],
                            host_id=row['host_id'],
                            timestamp=row['timestamp'],
                            event_type=row['event_type'],
                            data=event_data,
                            signature_verified=True,
                        )
                        events.append(event)
                    
                    except SignatureVerificationError as e:
                        logger.error(f"Ed25519 signature verification failed for event {event_id}: {e}")
                        verification_failures.append((event_id, str(e)))
                
                # Fail-closed: if any verification failed, raise error
                if verification_failures:
                    error_msg = f"Ed25519 signature verification failed for {len(verification_failures)} events (FAIL-CLOSED)"
                    logger.error(error_msg)
                    raise SignatureVerificationError(error_msg)
        
        except SignatureVerificationError:
            raise
        except Exception as e:
            logger.error(f"Error fetching host telemetry: {e}")
            raise
        
        return events
