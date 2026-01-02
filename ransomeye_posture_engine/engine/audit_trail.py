# Path and File Name : /home/ransomeye/rebuild/ransomeye_posture_engine/engine/audit_trail.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Audit trail - maintains full audit log of all posture operations

"""
Audit Trail

Maintains full audit log of all posture operations.
Immutable audit records.
"""

import logging
import json
from pathlib import Path
from datetime import datetime
from typing import Dict, Any, List, Optional
from dataclasses import dataclass, asdict
from enum import Enum

logger = logging.getLogger("ransomeye_posture_engine.audit_trail")

class AuditAction(Enum):
    """Audit action types."""
    TELEMETRY_INGESTED = "telemetry_ingested"
    FACTS_NORMALIZED = "facts_normalized"
    CIS_EVALUATED = "cis_evaluated"
    STIG_EVALUATED = "stig_evaluated"
    CUSTOM_POLICY_EVALUATED = "custom_policy_evaluated"
    SCORE_CALCULATED = "score_calculated"
    DRIFT_DETECTED = "drift_detected"
    REPORT_GENERATED = "report_generated"
    OUTPUT_SIGNED = "output_signed"
    ERROR = "error"

@dataclass
class AuditRecord:
    """Audit record."""
    record_id: str
    timestamp: datetime
    action: AuditAction
    host_id: Optional[str]
    details: Dict[str, Any]
    success: bool
    error_message: Optional[str] = None

class AuditTrail:
    """Maintains audit trail."""
    
    def __init__(self, audit_log_dir: Path):
        self.audit_log_dir = audit_log_dir
        self.audit_log_dir.mkdir(parents=True, exist_ok=True)
        self.record_counter = 0
    
    def log(self, action: AuditAction, host_id: Optional[str] = None,
           details: Dict[str, Any] = None, success: bool = True,
           error_message: Optional[str] = None):
        """
        Log an audit event.
        
        Args:
            action: Audit action type
            host_id: Host identifier (if applicable)
            details: Additional details
            success: Whether operation succeeded
            error_message: Error message (if failed)
        """
        if details is None:
            details = {}
        
        record = AuditRecord(
            record_id=f"audit_{self._next_record_id()}",
            timestamp=datetime.utcnow(),
            action=action,
            host_id=host_id,
            details=details,
            success=success,
            error_message=error_message,
        )
        
        self._write_record(record)
        
        # Also log to Python logger
        if success:
            logger.info(f"AUDIT: {action.value} - {host_id or 'N/A'} - {details}")
        else:
            logger.error(f"AUDIT: {action.value} - {host_id or 'N/A'} - ERROR: {error_message}")
    
    def _write_record(self, record: AuditRecord):
        """Write audit record to file."""
        try:
            # Write to daily log file
            date_str = record.timestamp.strftime('%Y%m%d')
            log_file = self.audit_log_dir / f"audit_{date_str}.jsonl"
            
            # Convert to JSON-serializable format
            record_dict = {
                'record_id': record.record_id,
                'timestamp': record.timestamp.isoformat(),
                'action': record.action.value,
                'host_id': record.host_id,
                'details': record.details,
                'success': record.success,
                'error_message': record.error_message,
            }
            
            with open(log_file, 'a') as f:
                f.write(json.dumps(record_dict) + '\n')
        
        except Exception as e:
            logger.error(f"Error writing audit record: {e}")
            # Fail-closed: log to Python logger as fallback
            logger.critical(f"AUDIT RECORD FAILED: {record}")
    
    def _next_record_id(self) -> str:
        """Generate next record ID."""
        self.record_counter += 1
        return f"{self.record_counter:010d}"
    
    def query(self, start_time: datetime, end_time: datetime,
             action: Optional[AuditAction] = None,
             host_id: Optional[str] = None) -> List[AuditRecord]:
        """
        Query audit records.
        
        Args:
            start_time: Start of time range
            end_time: End of time range
            action: Filter by action (if provided)
            host_id: Filter by host (if provided)
        
        Returns:
            List of matching audit records
        """
        records = []
        
        # Iterate over date range
        current_date = start_time.date()
        end_date = end_time.date()
        
        while current_date <= end_date:
            date_str = current_date.strftime('%Y%m%d')
            log_file = self.audit_log_dir / f"audit_{date_str}.jsonl"
            
            if log_file.exists():
                try:
                    with open(log_file, 'r') as f:
                        for line in f:
                            record_dict = json.loads(line.strip())
                            record_time = datetime.fromisoformat(record_dict['timestamp'])
                            
                            # Check time range
                            if record_time < start_time or record_time > end_time:
                                continue
                            
                            # Check action filter
                            if action and record_dict['action'] != action.value:
                                continue
                            
                            # Check host filter
                            if host_id and record_dict.get('host_id') != host_id:
                                continue
                            
                            # Reconstruct record
                            record = AuditRecord(
                                record_id=record_dict['record_id'],
                                timestamp=record_time,
                                action=AuditAction(record_dict['action']),
                                host_id=record_dict.get('host_id'),
                                details=record_dict.get('details', {}),
                                success=record_dict.get('success', True),
                                error_message=record_dict.get('error_message'),
                            )
                            records.append(record)
                
                except Exception as e:
                    logger.error(f"Error reading audit log {log_file}: {e}")
            
            # Move to next date
            from datetime import timedelta
            current_date += timedelta(days=1)
        
        return records

