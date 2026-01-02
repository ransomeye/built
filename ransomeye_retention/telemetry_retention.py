# Path and File Name : /home/ransomeye/rebuild/ransomeye_retention/telemetry_retention.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Telemetry purge logic with PostgreSQL partition drop and signed purge ledger

"""
Telemetry Retention: Enforces telemetry data retention policy.
Purges old telemetry data from PostgreSQL partitions.
"""

import os
import json
from pathlib import Path
from typing import List, Dict, Optional
from datetime import datetime, timedelta

from .retention_parser import RetentionParser
import sys
from pathlib import Path
# Add project root to path for trust imports
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))
try:
    from ransomeye_trust.sign_tool import SignTool
except ImportError:
    # Fallback for development
    SignTool = None


class TelemetryRetention:
    """Enforces telemetry retention policy."""
    
    def __init__(self, retention_parser: RetentionParser, db_config: Dict, trust_dir: str = "/home/ransomeye/rebuild/ransomeye_trust"):
        self.retention_parser = retention_parser
        self.db_config = db_config
        self.trust_dir = Path(trust_dir)
        self.sign_tool = SignTool(str(self.trust_dir))
        self.ledger_path = Path("/home/ransomeye/rebuild/logs/retention_ledger.jsonl")
        self.ledger_path.parent.mkdir(parents=True, exist_ok=True)
    
    def _get_db_connection(self):
        """Get PostgreSQL connection (lazy import)."""
        # Lazy import psycopg2 to avoid module-level dependency
        try:
            import psycopg2
        except ImportError as e:
            raise ImportError(
                "psycopg2 is required for database operations. "
                "Install with: pip install psycopg2-binary, "
                "or set RANSOMEYE_ENABLE_DB=0 to disable DB features."
            ) from e
        
        return psycopg2.connect(
            host=self.db_config.get('host', os.environ.get('DB_HOST', 'localhost')),
            port=self.db_config.get('port', int(os.environ.get('DB_PORT', 5432))),
            database=self.db_config.get('database', os.environ.get('DB_NAME', 'ransomeye')),
            user=self.db_config.get('user', os.environ.get('DB_USER', 'gagan')),
            password=self.db_config.get('password', os.environ.get('DB_PASS', 'gagan'))
        )
    
    def _get_partitions_to_drop(self) -> List[Dict]:
        """Get list of partitions older than retention period."""
        # Lazy import psycopg2.sql
        try:
            from psycopg2 import sql
        except ImportError as e:
            raise ImportError(
                "psycopg2 is required for database operations. "
                "Install with: pip install psycopg2-binary, "
                "or set RANSOMEYE_ENABLE_DB=0 to disable DB features."
            ) from e
        
        cutoff_date = self.retention_parser.get_telemetry_cutoff_date()
        
        conn = self._get_db_connection()
        cursor = conn.cursor()
        
        try:
            # Query for partitions older than cutoff
            query = sql.SQL("""
                SELECT 
                    schemaname,
                    tablename,
                    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
                FROM pg_tables
                WHERE tablename LIKE 'telemetry_%'
                AND tablename < %s
                ORDER BY tablename
            """)
            
            # Format partition name for comparison (assuming format: telemetry_YYYY_MM)
            partition_prefix = f"telemetry_{cutoff_date.strftime('%Y_%m')}"
            
            cursor.execute(query, (partition_prefix,))
            partitions = []
            
            for row in cursor.fetchall():
                partitions.append({
                    'schema': row[0],
                    'table': row[1],
                    'size': row[2]
                })
            
            return partitions
        finally:
            cursor.close()
            conn.close()
    
    def _drop_partition(self, schema: str, table: str) -> bool:
        """Drop a partition table."""
        # Lazy import psycopg2.sql
        try:
            from psycopg2 import sql
        except ImportError as e:
            raise ImportError(
                "psycopg2 is required for database operations. "
                "Install with: pip install psycopg2-binary, "
                "or set RANSOMEYE_ENABLE_DB=0 to disable DB features."
            ) from e
        
        conn = self._get_db_connection()
        cursor = conn.cursor()
        
        try:
            drop_query = sql.SQL("DROP TABLE IF EXISTS {}.{} CASCADE").format(
                sql.Identifier(schema),
                sql.Identifier(table)
            )
            cursor.execute(drop_query)
            conn.commit()
            return True
        except Exception as e:
            conn.rollback()
            print(f"Error dropping partition {schema}.{table}: {e}")
            return False
        finally:
            cursor.close()
            conn.close()
    
    def _log_purge_event(self, partitions: List[Dict], reason: str) -> Path:
        """Log purge event to signed ledger."""
        event = {
            'timestamp': datetime.utcnow().isoformat(),
            'type': 'telemetry_purge',
            'reason': reason,
            'partitions': partitions,
            'retention_months': self.retention_parser.get_telemetry_retention_months(),
            'cutoff_date': self.retention_parser.get_telemetry_cutoff_date().isoformat()
        }
        
        # Append to ledger
        with open(self.ledger_path, 'a') as f:
            f.write(json.dumps(event) + '\n')
        
        # Sign ledger entry
        ledger_manifest_path = self.ledger_path.parent / f"{self.ledger_path.stem}_manifest.json"
        if not ledger_manifest_path.exists():
            # Create manifest for ledger
            metadata = {
                'timestamp': datetime.utcnow().isoformat(),
                'version': '1.0.0',
                'type': 'retention_ledger'
            }
            self.sign_tool.create_and_sign_manifest(self.ledger_path, metadata, 'reports')
        
        return self.ledger_path
    
    def purge_old_telemetry(self, dry_run: bool = False) -> Dict:
        """
        Purge telemetry data older than retention period.
        
        Args:
            dry_run: If True, only report what would be purged
        
        Returns:
            Dictionary with purge results
        """
        partitions = self._get_partitions_to_drop()
        
        if not partitions:
            return {
                'purged': False,
                'partitions': [],
                'reason': 'No partitions to purge'
            }
        
        if dry_run:
            return {
                'purged': False,
                'partitions': partitions,
                'reason': 'dry_run',
                'would_purge': True
            }
        
        # Drop partitions
        dropped = []
        failed = []
        
        for partition in partitions:
            if self._drop_partition(partition['schema'], partition['table']):
                dropped.append(partition)
            else:
                failed.append(partition)
        
        # Log purge event
        reason = 'retention_policy' if not dry_run else 'dry_run'
        self._log_purge_event(dropped, reason)
        
        return {
            'purged': True,
            'partitions_dropped': dropped,
            'partitions_failed': failed,
            'total_dropped': len(dropped),
            'total_failed': len(failed)
        }
    
    def purge_on_disk_pressure(self, current_usage_percent: float, target_percent: float = 70) -> Dict:
        """
        Purge telemetry data due to disk pressure.
        
        Args:
            current_usage_percent: Current disk usage percentage
            target_percent: Target usage percentage after purge
        
        Returns:
            Dictionary with purge results
        """
        # Calculate how much to free
        from .disk_monitor import DiskMonitor
        monitor = DiskMonitor(self.retention_parser)
        target_bytes = monitor.get_cleanup_target_size(current_usage_percent, target_percent)
        
        # Get partitions sorted by age (oldest first)
        partitions = self._get_partitions_to_drop()
        
        # Drop partitions until we've freed enough space
        dropped = []
        freed_bytes = 0
        
        for partition in partitions:
            if freed_bytes >= target_bytes:
                break
            
            if self._drop_partition(partition['schema'], partition['table']):
                dropped.append(partition)
                # Estimate freed space (would need actual size query)
                freed_bytes += 100 * 1024 * 1024  # Estimate 100MB per partition
        
        # Log purge event
        self._log_purge_event(dropped, 'disk_pressure')
        
        return {
            'purged': True,
            'partitions_dropped': dropped,
            'freed_bytes_estimated': freed_bytes,
            'reason': 'disk_pressure'
        }

