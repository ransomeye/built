# Path and File Name : /home/ransomeye/rebuild/ransomeye_posture_engine/engine/posture_daemon.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Main posture daemon - orchestrates all posture evaluation components

"""
Posture Daemon

Main daemon that orchestrates:
- Telemetry ingestion
- Signal normalization
- CIS/STIG/Custom policy evaluation
- Scoring
- Drift detection
- Report generation
- Output signing
- Audit trail
"""

import asyncio
import logging
from datetime import datetime, timedelta
from typing import Dict, List, Set

from ..config import Config
from .telemetry_ingester import TelemetryIngester
from .normalizer import SignalNormalizer
from .cis_evaluator import CISEvaluator
from .stig_evaluator import STIGEvaluator
from .custom_policy_evaluator import CustomPolicyEvaluator
from .scorer import PostureScorer
from .drift_detector import DriftDetector
from .report_generator import ReportGenerator
from .output_signer import OutputSigner
from .audit_trail import AuditTrail, AuditAction
from .signature_verifier import SignatureVerifier
from .policy_metadata import PolicyMetadataManager

logger = logging.getLogger("ransomeye_posture_engine.daemon")

class PostureDaemon:
    """Main posture evaluation daemon."""
    
    def __init__(self, config: Config):
        self.config = config
        self.running = False
        
        # Initialize policy metadata manager (MANDATORY for policy hash pinning)
        self.policy_metadata_manager = PolicyMetadataManager()
        
        # Initialize signature verifier (MANDATORY - database is UNTRUSTED)
        self.signature_verifier = SignatureVerifier(config.trust_store_path)
        
        # Initialize components
        self.ingester = TelemetryIngester(config, self.signature_verifier)
        self.normalizer = SignalNormalizer()
        self.cis_evaluator = CISEvaluator(config.cis_benchmarks_dir, self.policy_metadata_manager)
        self.stig_evaluator = STIGEvaluator(config.stig_profiles_dir, self.policy_metadata_manager)
        self.custom_evaluator = CustomPolicyEvaluator(config.custom_policies_dir, self.policy_metadata_manager)
        self.scorer = PostureScorer()
        self.drift_detector = DriftDetector(
            config.output_dir / "baselines",
            config.drift_detection_window_hours
        )
        self.report_generator = ReportGenerator(config.output_dir, self.policy_metadata_manager)
        self.signer = OutputSigner(config.signing_key_path, config.trust_store_path)
        self.audit_trail = AuditTrail(config.audit_log_dir)
        
        # Track processed hosts
        self.processed_hosts: Set[str] = set()
    
    async def start(self):
        """Start the daemon."""
        logger.info("Starting Posture Engine daemon")
        self.audit_trail.log(AuditAction.TELEMETRY_INGESTED, details={'action': 'daemon_start'})
        
        try:
            # Initialize database connection
            await self.ingester.initialize()
            self.audit_trail.log(AuditAction.TELEMETRY_INGESTED, details={'action': 'db_initialized'})
            
            self.running = True
            
            # Main evaluation loop
            while self.running:
                try:
                    await self._evaluation_cycle()
                    
                    # Wait for next evaluation interval
                    await asyncio.sleep(self.config.evaluation_interval_seconds)
                
                except Exception as e:
                    logger.error(f"Error in evaluation cycle: {e}")
                    self.audit_trail.log(
                        AuditAction.ERROR,
                        details={'error': str(e)},
                        success=False,
                        error_message=str(e)
                    )
                    # Continue running despite errors
                    await asyncio.sleep(60)  # Wait before retry
        
        except Exception as e:
            logger.error(f"Fatal error in daemon: {e}")
            self.audit_trail.log(
                AuditAction.ERROR,
                details={'error': str(e), 'fatal': True},
                success=False,
                error_message=str(e)
            )
            raise
    
    async def stop(self):
        """Stop the daemon."""
        logger.info("Stopping Posture Engine daemon")
        self.running = False
        
        try:
            await self.ingester.close()
            self.audit_trail.log(AuditAction.TELEMETRY_INGESTED, details={'action': 'daemon_stop'})
        except Exception as e:
            logger.error(f"Error stopping daemon: {e}")
    
    async def _evaluation_cycle(self):
        """Single evaluation cycle."""
        logger.info("Starting evaluation cycle")
        
        try:
            # Step 1: Fetch new telemetry
            events = await self.ingester.fetch_new_telemetry()
            if not events:
                logger.debug("No new telemetry events")
                return
            
            self.audit_trail.log(
                AuditAction.TELEMETRY_INGESTED,
                details={'event_count': len(events)}
            )
            
            # Step 2: Normalize signals into facts
            facts = self.normalizer.normalize(events)
            self.audit_trail.log(
                AuditAction.FACTS_NORMALIZED,
                details={'fact_count': len(facts)}
            )
            
            # Group facts by host
            facts_by_host: Dict[str, List] = {}
            for fact in facts:
                host_id = fact.host_id or "unknown"
                if host_id not in facts_by_host:
                    facts_by_host[host_id] = []
                facts_by_host[host_id].append(fact)
            
            # Process each host
            for host_id, host_facts in facts_by_host.items():
                await self._process_host(host_id, host_facts)
        
        except Exception as e:
            logger.error(f"Error in evaluation cycle: {e}")
            self.audit_trail.log(
                AuditAction.ERROR,
                details={'error': str(e), 'cycle': 'evaluation'},
                success=False,
                error_message=str(e)
            )
            raise
    
    async def _process_host(self, host_id: str, facts: List):
        """Process posture evaluation for a single host."""
        logger.info(f"Processing host: {host_id}")
        
        try:
            # Step 3: Evaluate against frameworks
            cis_results = self.cis_evaluator.evaluate(facts)
            self.audit_trail.log(
                AuditAction.CIS_EVALUATED,
                host_id=host_id,
                details={'result_count': len(cis_results)}
            )
            
            stig_results = self.stig_evaluator.evaluate(facts)
            self.audit_trail.log(
                AuditAction.STIG_EVALUATED,
                host_id=host_id,
                details={'result_count': len(stig_results)}
            )
            
            custom_results = self.custom_evaluator.evaluate(facts)
            self.audit_trail.log(
                AuditAction.CUSTOM_POLICY_EVALUATED,
                host_id=host_id,
                details={'result_count': len(custom_results)}
            )
            
            # Step 4: Calculate scores
            host_score = self.scorer.calculate_host_score(
                host_id, facts, cis_results, stig_results, custom_results
            )
            self.audit_trail.log(
                AuditAction.SCORE_CALCULATED,
                host_id=host_id,
                details={'overall_score': host_score.overall_score}
            )
            
            # Step 5: Detect drift
            drift_alerts = self.drift_detector.detect_host_drift(host_id, host_score)
            if drift_alerts:
                self.audit_trail.log(
                    AuditAction.DRIFT_DETECTED,
                    host_id=host_id,
                    details={'alert_count': len(drift_alerts)}
                )
            
            # Step 6: Generate reports
            report_paths = self.report_generator.generate_host_report(
                host_id, host_score, cis_results, stig_results, custom_results, drift_alerts
            )
            self.audit_trail.log(
                AuditAction.REPORT_GENERATED,
                host_id=host_id,
                details={'reports': list(report_paths.keys())}
            )
            
            # Step 7: Sign outputs
            for format_type, report_path in report_paths.items():
                try:
                    signature_metadata = self.signer.sign_file(report_path)
                    self.audit_trail.log(
                        AuditAction.OUTPUT_SIGNED,
                        host_id=host_id,
                        details={'file': str(report_path), 'signed': signature_metadata.get('signed', False)}
                    )
                except Exception as e:
                    logger.error(f"Error signing report {report_path}: {e}")
                    self.audit_trail.log(
                        AuditAction.ERROR,
                        host_id=host_id,
                        details={'error': str(e), 'file': str(report_path)},
                        success=False,
                        error_message=str(e)
                    )
            
            self.processed_hosts.add(host_id)
            logger.info(f"Completed processing host: {host_id}")
        
        except Exception as e:
            logger.error(f"Error processing host {host_id}: {e}")
            self.audit_trail.log(
                AuditAction.ERROR,
                host_id=host_id,
                details={'error': str(e), 'operation': 'host_processing'},
                success=False,
                error_message=str(e)
            )
            # Fail-closed: re-raise to stop processing this host
            raise

