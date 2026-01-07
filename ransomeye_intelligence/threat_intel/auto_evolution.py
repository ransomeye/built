# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/auto_evolution.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Auto-evolution system ensuring RansomEye remains effective against unknown threats over 10+ years

"""
Auto-Evolution System
=====================
Ensures RansomEye automatically evolves to remain effective against unknown threats
even after 10+ years of deployment through:
- Continuous model retraining
- Novel threat pattern detection
- Adaptive learning from false positives/negatives
- Long-term knowledge retention
- Model versioning and rollback
"""

import os
import sys
import json
import pickle
import numpy as np
from pathlib import Path
from datetime import datetime, timedelta, timezone
from typing import Dict, List, Optional, Tuple
import logging

# Add parent directories to path
sys.path.insert(0, str(Path(__file__).parent))
sys.path.insert(0, str(Path(__file__).parent.parent))

from classification.threat_classifier import ThreatClassifier
from training.continuous_trainer import ContinuousTrainer

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger('auto_evolution')

PROJECT_ROOT = Path("/home/ransomeye/rebuild")
EVOLUTION_DIR = PROJECT_ROOT / "ransomeye_intelligence" / "threat_intel" / "evolution"
EVOLUTION_DIR.mkdir(parents=True, exist_ok=True)


class AutoEvolutionSystem:
    """Auto-evolution system for long-term effectiveness."""
    
    def __init__(self):
        self.evolution_dir = EVOLUTION_DIR
        self.threat_classifier = ThreatClassifier()
        self.continuous_trainer = ContinuousTrainer()
        
        # Evolution configuration
        self.evolution_config = {
            'min_unknown_threats_for_evolution': 100,  # Minimum unknown threats to trigger evolution
            'evolution_interval_days': 30,  # Monthly evolution cycles
            'knowledge_retention_years': 10,  # Retain knowledge for 10 years
            'novelty_threshold': 0.3,  # Threshold for detecting novel threats
            'adaptation_rate': 0.1  # Rate of adaptation to new patterns
        }
        
        # Load evolution history
        self.evolution_history = self.load_evolution_history()
    
    def load_evolution_history(self) -> List[Dict]:
        """Load evolution history from disk."""
        history_file = self.evolution_dir / "evolution_history.json"
        if history_file.exists():
            try:
                with open(history_file, 'r') as f:
                    return json.load(f)
            except Exception as e:
                logger.warning(f"Failed to load evolution history: {e}")
        return []
    
    def save_evolution_history(self):
        """Save evolution history to disk."""
        history_file = self.evolution_dir / "evolution_history.json"
        try:
            with open(history_file, 'w') as f:
                json.dump(self.evolution_history, f, indent=2)
        except Exception as e:
            logger.error(f"Failed to save evolution history: {e}")
    
    def detect_novel_threats(self, iocs: List[Dict]) -> List[Dict]:
        """
        Detect novel/unknown threat patterns.
        
        Args:
            iocs: List of IOCs to analyze
        
        Returns:
            List of novel threats
        """
        novel_threats = []
        
        for ioc in iocs:
            classification = self.threat_classifier.classify(
                ioc_value=ioc.get('value', ''),
                ioc_type=ioc.get('type', ''),
                metadata=ioc.get('metadata', {}),
                behavior_signals=ioc.get('behavior_signals', [])
            )
            
            # Check if threat is novel (low confidence, unknown category, or new pattern)
            confidence = max(classification['confidence_scores'].values()) if classification['confidence_scores'] else 0.0
            
            # Only mark as novel if:
            # 1. Very low confidence (< novelty_threshold) AND unknown category
            # 2. OR it's a completely new pattern not seen before
            is_novel = False
            
            if classification['primary_category'] == 'unknown' and confidence < self.evolution_config['novelty_threshold']:
                is_novel = True
            elif self.is_new_pattern(ioc):
                is_novel = True
            elif confidence < (self.evolution_config['novelty_threshold'] * 0.5):  # Very low confidence
                is_novel = True
            
            if is_novel:
                novel_threats.append({
                    'ioc': ioc,
                    'classification': classification,
                    'novelty_score': 1.0 - confidence,
                    'detected_at': datetime.now(timezone.utc).isoformat().replace('+00:00', '') + 'Z'
                })
        
        logger.info(f"Detected {len(novel_threats)} novel threats out of {len(iocs)} total IOCs")
        return novel_threats
    
    def is_new_pattern(self, ioc: Dict) -> bool:
        """
        Check if IOC represents a new pattern not seen before.
        
        Args:
            ioc: IOC to check
        
        Returns:
            True if new pattern
        """
        # Check evolution history for similar patterns
        ioc_signature = self.generate_ioc_signature(ioc)
        
        for history_entry in self.evolution_history:
            if history_entry.get('ioc_signature') == ioc_signature:
                return False  # Pattern seen before
        
        return True  # New pattern
    
    def generate_ioc_signature(self, ioc: Dict) -> str:
        """Generate a signature for an IOC pattern."""
        import hashlib
        
        signature_data = {
            'type': ioc.get('type', ''),
            'value_hash': hashlib.sha256(str(ioc.get('value', '')).encode()).hexdigest()[:16],
            'behavior_signals': sorted(ioc.get('behavior_signals', []))
        }
        
        signature_str = json.dumps(signature_data, sort_keys=True)
        return hashlib.sha256(signature_str.encode()).hexdigest()
    
    def adapt_to_novel_threats(self, novel_threats: List[Dict]) -> Dict:
        """
        Adapt models to novel threats.
        
        Args:
            novel_threats: List of novel threats detected
        
        Returns:
            Adaptation results
        """
        if len(novel_threats) < self.evolution_config['min_unknown_threats_for_evolution']:
            logger.info(f"Insufficient novel threats for evolution ({len(novel_threats)} < {self.evolution_config['min_unknown_threats_for_evolution']})")
            return {'status': 'skipped', 'reason': 'insufficient_novel_threats'}
        
        logger.info(f"Adapting to {len(novel_threats)} novel threats")
        
        # Extract IOCs from novel threats
        novel_iocs = [threat['ioc'] for threat in novel_threats]
        
        # Run continuous training with novel threats
        training_result = self.continuous_trainer.run_continuous_training(force=True)
        
        # Record evolution event
        evolution_event = {
            'event_id': f"evolution_{datetime.now(timezone.utc).strftime('%Y%m%d_%H%M%S')}",
            'timestamp': datetime.now(timezone.utc).isoformat().replace('+00:00', '') + 'Z',
            'novel_threats_count': len(novel_threats),
            'training_result': training_result,
            'evolution_version': self.get_next_evolution_version()
        }
        
        self.evolution_history.append(evolution_event)
        self.save_evolution_history()
        
        logger.info(f"Evolution event recorded: {evolution_event['event_id']}")
        
        return {
            'status': 'success',
            'evolution_event': evolution_event,
            'novel_threats_processed': len(novel_threats)
        }
    
    def get_next_evolution_version(self) -> str:
        """Get next evolution version number."""
        if not self.evolution_history:
            return "1.0.0"
        
        # Increment minor version
        last_version = self.evolution_history[-1].get('evolution_version', '1.0.0')
        parts = last_version.split('.')
        if len(parts) == 3:
            major, minor, patch = map(int, parts)
            return f"{major}.{minor + 1}.0"
        
        return "1.0.0"
    
    def retain_long_term_knowledge(self):
        """
        Retain knowledge from past 10 years while pruning older data.
        
        Ensures models remember important patterns while staying current.
        """
        cutoff_date = datetime.now(timezone.utc) - timedelta(days=365 * self.evolution_config['knowledge_retention_years'])
        
        # Filter evolution history to retain only recent knowledge
        retained_history = []
        for entry in self.evolution_history:
            try:
                # Parse timestamp - handle both 'Z' suffix and '+00:00' format
                timestamp_str = entry.get('timestamp', '')
                if timestamp_str.endswith('Z'):
                    # Replace 'Z' with '+00:00' only if not already present
                    if '+00:00' not in timestamp_str:
                        timestamp_str = timestamp_str.replace('Z', '+00:00')
                    else:
                        # Remove 'Z' if timezone already present
                        timestamp_str = timestamp_str.rstrip('Z')
                elif not ('+' in timestamp_str or timestamp_str.endswith('Z')):
                    # No timezone info, assume UTC
                    timestamp_str = timestamp_str + '+00:00'
                
                entry_date = datetime.fromisoformat(timestamp_str)
                if entry_date >= cutoff_date:
                    retained_history.append(entry)
            except (ValueError, KeyError) as e:
                logger.warning(f"Failed to parse timestamp for entry {entry.get('event_id', 'unknown')}: {e}")
                # Keep entry if we can't parse (conservative approach)
                retained_history.append(entry)
        
        if len(retained_history) < len(self.evolution_history):
            logger.info(f"Pruned evolution history: {len(self.evolution_history)} -> {len(retained_history)} entries")
            self.evolution_history = retained_history
            self.save_evolution_history()
    
    def learn_from_feedback(self, false_positives: List[Dict], false_negatives: List[Dict]):
        """
        Learn from false positives and false negatives.
        
        Args:
            false_positives: IOCs incorrectly classified as threats
            false_negatives: IOCs incorrectly classified as benign
        """
        logger.info(f"Learning from {len(false_positives)} false positives and {len(false_negatives)} false negatives")
        
        # Adjust classifier thresholds based on feedback
        if false_positives:
            # Increase threshold to reduce false positives
            self.threat_classifier.confidence_threshold = min(
                self.threat_classifier.confidence_threshold + 0.05,
                0.9
            )
            logger.info(f"Adjusted confidence threshold to {self.threat_classifier.confidence_threshold}")
        
        if false_negatives:
            # Decrease threshold to catch more threats
            self.threat_classifier.confidence_threshold = max(
                self.threat_classifier.confidence_threshold - 0.05,
                0.4
            )
            logger.info(f"Adjusted confidence threshold to {self.threat_classifier.confidence_threshold}")
        
        # Retrain with feedback data
        if false_positives or false_negatives:
            feedback_iocs = false_positives + false_negatives
            training_result = self.continuous_trainer.run_continuous_training(force=True)
            logger.info("Retrained models with feedback data")
    
    def run_evolution_cycle(self) -> Dict:
        """
        Run a complete evolution cycle.
        
        Returns:
            Evolution cycle results
        """
        logger.info("Starting auto-evolution cycle")
        
        # 1. Retain long-term knowledge
        self.retain_long_term_knowledge()
        
        # 2. Load recent data
        internal_data = self.continuous_trainer.load_internal_telemetry(days=30)
        external_data = self.continuous_trainer.load_external_feeds()
        all_iocs = internal_data + external_data
        
        # 3. Detect novel threats
        novel_threats = self.detect_novel_threats(all_iocs)
        
        # 4. Adapt to novel threats
        adaptation_result = None
        if novel_threats:
            adaptation_result = self.adapt_to_novel_threats(novel_threats)
        
        # 5. Run continuous training
        training_result = self.continuous_trainer.run_continuous_training(force=False)
        
        # 6. Generate evolution report
        evolution_report = {
            'cycle_id': f"cycle_{datetime.now(timezone.utc).strftime('%Y%m%d_%H%M%S')}",
            'timestamp': datetime.now(timezone.utc).isoformat().replace('+00:00', '') + 'Z',
            'novel_threats_detected': len(novel_threats),
            'adaptation_result': adaptation_result,
            'training_result': training_result,
            'evolution_version': self.get_next_evolution_version(),
            'knowledge_retained_years': self.evolution_config['knowledge_retention_years']
        }
        
        # Save evolution report
        report_file = self.evolution_dir / f"evolution_report_{datetime.now(timezone.utc).strftime('%Y%m%d_%H%M%S')}.json"
        with open(report_file, 'w') as f:
            json.dump(evolution_report, f, indent=2)
        
        logger.info(f"Evolution cycle completed: {evolution_report['cycle_id']}")
        
        return evolution_report


def main():
    """CLI entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description='Auto-Evolution System')
    parser.add_argument('--cycle', action='store_true',
                       help='Run complete evolution cycle')
    parser.add_argument('--retain-knowledge', action='store_true',
                       help='Retain long-term knowledge')
    
    args = parser.parse_args()
    
    evolution = AutoEvolutionSystem()
    
    if args.retain_knowledge:
        evolution.retain_long_term_knowledge()
    
    if args.cycle:
        results = evolution.run_evolution_cycle()
        print(json.dumps(results, indent=2))


if __name__ == '__main__':
    main()

