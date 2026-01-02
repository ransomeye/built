# Path and File Name : /home/ransomeye/rebuild/ransomeye_posture_engine/engine/normalizer.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Normalizes telemetry signals into posture facts

"""
Signal Normalizer

Normalizes telemetry from different sources into standardized posture facts:
- Host hardening
- Auth hygiene
- Network exposure
- Drift detection
"""

import logging
from datetime import datetime
from typing import List, Dict, Any, Optional
from dataclasses import dataclass, field
from enum import Enum

from .telemetry_ingester import TelemetryEvent

logger = logging.getLogger("ransomeye_posture_engine.normalizer")

class PostureCategory(Enum):
    """Posture fact categories."""
    HOST_HARDENING = "host_hardening"
    AUTH_HYGIENE = "auth_hygiene"
    NETWORK_EXPOSURE = "network_exposure"
    DRIFT_DETECTION = "drift_detection"

@dataclass
class PostureFact:
    """Normalized posture fact."""
    fact_id: str
    host_id: Optional[str]
    category: PostureCategory
    timestamp: datetime
    fact_type: str
    fact_data: Dict[str, Any]
    confidence: float  # 0.0 to 1.0
    source_event_ids: List[str] = field(default_factory=list)

class SignalNormalizer:
    """Normalizes telemetry signals into posture facts."""
    
    def __init__(self):
        self.fact_counter = 0
    
    def normalize(self, events: List[TelemetryEvent]) -> List[PostureFact]:
        """
        Normalize telemetry events into posture facts.
        
        Args:
            events: List of telemetry events to normalize
        
        Returns:
            List of normalized posture facts
        """
        facts = []
        
        # Group events by host
        events_by_host: Dict[Optional[str], List[TelemetryEvent]] = {}
        for event in events:
            if event.host_id not in events_by_host:
                events_by_host[event.host_id] = []
            events_by_host[event.host_id].append(event)
        
        # Process each host
        for host_id, host_events in events_by_host.items():
            host_facts = self._normalize_host_events(host_id, host_events)
            facts.extend(host_facts)
        
        logger.info(f"Normalized {len(events)} events into {len(facts)} posture facts")
        return facts
    
    def _normalize_host_events(self, host_id: Optional[str], 
                               events: List[TelemetryEvent]) -> List[PostureFact]:
        """Normalize events for a single host."""
        facts = []
        
        # Extract host hardening facts
        hardening_facts = self._extract_host_hardening(host_id, events)
        facts.extend(hardening_facts)
        
        # Extract auth hygiene facts
        auth_facts = self._extract_auth_hygiene(host_id, events)
        facts.extend(auth_facts)
        
        # Extract network exposure facts
        network_facts = self._extract_network_exposure(host_id, events)
        facts.extend(network_facts)
        
        # Extract drift detection facts
        drift_facts = self._extract_drift_detection(host_id, events)
        facts.extend(drift_facts)
        
        return facts
    
    def _extract_host_hardening(self, host_id: Optional[str], 
                                events: List[TelemetryEvent]) -> List[PostureFact]:
        """Extract host hardening posture facts."""
        facts = []
        
        # Process Linux/Windows agent events for hardening indicators
        for event in events:
            if event.producer_type not in ['linux_agent', 'windows_agent']:
                continue
            
            event_data = event.data
            
            # Check for process execution patterns (potential privilege escalation)
            if event.event_type == 'process_exec' and event_data:
                process_name = event_data.get('process_name', '')
                user = event_data.get('user', '')
                
                # Detect root/admin execution
                if user in ['root', 'SYSTEM', 'Administrator']:
                    fact = PostureFact(
                        fact_id=f"hardening_{self._next_fact_id()}",
                        host_id=host_id,
                        category=PostureCategory.HOST_HARDENING,
                        timestamp=event.timestamp,
                        fact_type="privileged_execution",
                        fact_data={
                            "process": process_name,
                            "user": user,
                            "risk_level": "high" if user == 'root' else "medium",
                        },
                        confidence=0.9,
                        source_event_ids=[event.event_id],
                    )
                    facts.append(fact)
            
            # Check for file system changes (potential hardening violations)
            if event.event_type == 'file_modify' and event_data:
                file_path = event_data.get('path', '')
                # Critical system files
                critical_paths = ['/etc/passwd', '/etc/shadow', '/etc/sudoers',
                                 'C:\\Windows\\System32', 'C:\\Windows\\SysWOW64']
                if any(cp in file_path for cp in critical_paths):
                    fact = PostureFact(
                        fact_id=f"hardening_{self._next_fact_id()}",
                        host_id=host_id,
                        category=PostureCategory.HOST_HARDENING,
                        timestamp=event.timestamp,
                        fact_type="critical_file_modification",
                        fact_data={
                            "file_path": file_path,
                            "risk_level": "high",
                        },
                        confidence=0.85,
                        source_event_ids=[event.event_id],
                    )
                    facts.append(fact)
        
        return facts
    
    def _extract_auth_hygiene(self, host_id: Optional[str], 
                              events: List[TelemetryEvent]) -> List[PostureFact]:
        """Extract authentication hygiene posture facts."""
        facts = []
        
        failed_auth_count = 0
        successful_auth_count = 0
        auth_events = []
        
        for event in events:
            if event.producer_type not in ['linux_agent', 'windows_agent']:
                continue
            
            event_data = event.data
            
            # Track authentication events
            if event.event_type in ['auth_success', 'auth_failure', 'login', 'logout']:
                auth_events.append(event)
                if 'failure' in event.event_type or 'fail' in event.event_type.lower():
                    failed_auth_count += 1
                else:
                    successful_auth_count += 1
        
        # Generate auth hygiene facts
        if auth_events:
            total_auth = failed_auth_count + successful_auth_count
            failure_rate = failed_auth_count / total_auth if total_auth > 0 else 0.0
            
            if failure_rate > 0.1:  # More than 10% failure rate
                fact = PostureFact(
                    fact_id=f"auth_{self._next_fact_id()}",
                    host_id=host_id,
                    category=PostureCategory.AUTH_HYGIENE,
                    timestamp=auth_events[-1].timestamp if auth_events else datetime.utcnow(),
                    fact_type="high_auth_failure_rate",
                    fact_data={
                        "failure_rate": failure_rate,
                        "failed_attempts": failed_auth_count,
                        "total_attempts": total_auth,
                        "risk_level": "high" if failure_rate > 0.3 else "medium",
                    },
                    confidence=0.8,
                    source_event_ids=[e.event_id for e in auth_events],
                )
                facts.append(fact)
        
        return facts
    
    def _extract_network_exposure(self, host_id: Optional[str], 
                                  events: List[TelemetryEvent]) -> List[PostureFact]:
        """Extract network exposure posture facts."""
        facts = []
        
        # Process DPI probe events
        dpi_events = [e for e in events if e.producer_type == 'dpi_probe']
        
        exposed_ports = set()
        external_connections = []
        
        for event in dpi_events:
            event_data = event.data
            
            if event.event_type == 'network_flow':
                src_ip = event_data.get('src_ip', '')
                dst_ip = event_data.get('dst_ip', '')
                dst_port = event_data.get('dst_port')
                
                # Check if connection is to external IP
                if self._is_external_ip(dst_ip):
                    exposed_ports.add(dst_port)
                    external_connections.append({
                        "src_ip": src_ip,
                        "dst_ip": dst_ip,
                        "dst_port": dst_port,
                        "protocol": event_data.get('protocol', ''),
                    })
        
        # Generate network exposure facts
        if exposed_ports:
            fact = PostureFact(
                fact_id=f"network_{self._next_fact_id()}",
                host_id=host_id,
                category=PostureCategory.NETWORK_EXPOSURE,
                timestamp=events[-1].timestamp if events else datetime.utcnow(),
                fact_type="exposed_ports",
                fact_data={
                    "exposed_ports": list(exposed_ports),
                    "external_connections": len(external_connections),
                    "risk_level": "high" if len(exposed_ports) > 5 else "medium",
                },
                confidence=0.9,
                source_event_ids=[e.event_id for e in dpi_events],
            )
            facts.append(fact)
        
        # Process agent network events
        agent_events = [e for e in events if e.producer_type in ['linux_agent', 'windows_agent']]
        for event in agent_events:
            if event.event_type == 'network_connection':
                event_data = event.data
                remote_addr = event_data.get('remote_address', '')
                
                if self._is_external_ip(remote_addr):
                    fact = PostureFact(
                        fact_id=f"network_{self._next_fact_id()}",
                        host_id=host_id,
                        category=PostureCategory.NETWORK_EXPOSURE,
                        timestamp=event.timestamp,
                        fact_type="outbound_external_connection",
                        fact_data={
                            "remote_address": remote_addr,
                            "protocol": event_data.get('protocol', ''),
                            "risk_level": "medium",
                        },
                        confidence=0.85,
                        source_event_ids=[event.event_id],
                    )
                    facts.append(fact)
        
        return facts
    
    def _extract_drift_detection(self, host_id: Optional[str], 
                                 events: List[TelemetryEvent]) -> List[PostureFact]:
        """Extract configuration drift detection facts."""
        facts = []
        
        # Group events by type and detect changes
        config_changes = {}
        
        for event in events:
            if event.producer_type not in ['linux_agent', 'windows_agent']:
                continue
            
            # Track configuration changes
            if event.event_type in ['file_modify', 'registry_modify']:
                event_data = event.data
                config_key = event_data.get('path', event_data.get('key_path', ''))
                
                if config_key not in config_changes:
                    config_changes[config_key] = []
                config_changes[config_key].append(event)
        
        # Generate drift facts for frequently changed configs
        for config_key, change_events in config_changes.items():
            if len(change_events) > 3:  # More than 3 changes indicates drift
                fact = PostureFact(
                    fact_id=f"drift_{self._next_fact_id()}",
                    host_id=host_id,
                    category=PostureCategory.DRIFT_DETECTION,
                    timestamp=change_events[-1].timestamp,
                    fact_type="configuration_drift",
                    fact_data={
                        "config_key": config_key,
                        "change_count": len(change_events),
                        "risk_level": "medium",
                    },
                    confidence=0.75,
                    source_event_ids=[e.event_id for e in change_events],
                )
                facts.append(fact)
        
        return facts
    
    def _is_external_ip(self, ip: str) -> bool:
        """Check if IP is external (not private)."""
        if not ip:
            return False
        
        # Simple check for private IP ranges
        private_ranges = [
            '10.', '172.16.', '172.17.', '172.18.', '172.19.',
            '172.20.', '172.21.', '172.22.', '172.23.', '172.24.',
            '172.25.', '172.26.', '172.27.', '172.28.', '172.29.',
            '172.30.', '172.31.', '192.168.', '127.', '169.254.'
        ]
        
        return not any(ip.startswith(pr) for pr in private_ranges)
    
    def _next_fact_id(self) -> str:
        """Generate next fact ID."""
        self.fact_counter += 1
        return f"{self.fact_counter:08d}"

