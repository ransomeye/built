# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/training/continuous_trainer.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Continuous training system that automatically retrains models from internal and external data

"""
Continuous Training System
==========================
Automatically retrains models from:
- Internal telemetry data (incidents, alerts, forensic data)
- External threat intelligence feeds
- New threat patterns discovered
- Model performance drift detection

Ensures RansomEye auto-evolves to remain effective against unknown threats over 10+ years.
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
import psycopg2
from psycopg2.extras import RealDictCursor

# Add parent directories to path
sys.path.insert(0, str(Path(__file__).parent.parent))
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from classification.threat_classifier import ThreatClassifier
from training_governance import TrainingGovernance, SHAPExplainer, ResourceGovernor

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger('continuous_trainer')

PROJECT_ROOT = Path("/home/ransomeye/rebuild")
CACHE_DIR = PROJECT_ROOT / "ransomeye_intelligence" / "threat_intel" / "cache"
MODELS_DIR = PROJECT_ROOT / "ransomeye_intelligence" / "threat_intel" / "models"
TELEMETRY_DIR = PROJECT_ROOT / "logs" / "telemetry"


class ContinuousTrainer:
    """Continuous training system for auto-evolution."""
    
    def __init__(self):
        self.project_root = PROJECT_ROOT
        self.threat_classifier = ThreatClassifier()
        self.governance = TrainingGovernance()
        self.resource_gov = ResourceGovernor()
        
        # Database connection
        self.db_config = {
            'host': os.getenv('DB_HOST', 'localhost'),
            'port': int(os.getenv('DB_PORT', 5432)),
            'database': os.getenv('DB_NAME', 'ransomeye'),
            'user': os.getenv('DB_USER', 'gagan'),
            'password': os.getenv('DB_PASS', 'gagan')
        }
        
        # Training configuration
        self.min_samples_for_retrain = 1000
        self.retrain_interval_days = 7  # Weekly retraining
        self.drift_threshold = 0.15  # 15% performance degradation
        
    def get_db_connection(self):
        """Get database connection."""
        try:
            conn = psycopg2.connect(**self.db_config)
            return conn
        except Exception as e:
            logger.error(f"Failed to connect to database: {e}")
            return None
    
    def load_internal_telemetry(self, days: int = 30) -> List[Dict]:
        """
        Load internal telemetry data from database.
        
        Args:
            days: Number of days to look back
        
        Returns:
            List of telemetry records
        """
        conn = self.get_db_connection()
        if not conn:
            return []
        
        try:
            # Use regular cursor for EXISTS queries (returns tuple, not dict)
            check_cursor = conn.cursor()
            
            # Check if table exists
            check_cursor.execute("""
                SELECT EXISTS (
                    SELECT FROM information_schema.tables 
                    WHERE table_schema = 'ransomeye' 
                    AND table_name = 'incidents'
                )
            """)
            result = check_cursor.fetchone()
            table_exists = result[0] if result else False
            check_cursor.close()
            
            # Use RealDictCursor for data queries
            cursor = conn.cursor(cursor_factory=RealDictCursor)
            
            if not table_exists:
                logger.warning("ransomeye.incidents table does not exist, skipping internal telemetry")
                incidents = []
            else:
                cursor.execute("""
                    SELECT 
                        incident_id,
                        created_at,
                        severity,
                        threat_type,
                        ioc_data,
                        behavior_signals,
                        forensic_data
                    FROM ransomeye.incidents
                    WHERE created_at >= %s
                    ORDER BY created_at DESC
                    LIMIT 10000
                """, (datetime.now(timezone.utc) - timedelta(days=days),))
                incidents = [dict(row) for row in cursor.fetchall()]
            
            # Load alerts - switch back to regular cursor for EXISTS
            cursor.close()
            cursor = conn.cursor()
            cursor.execute("""
                SELECT EXISTS (
                    SELECT FROM information_schema.tables 
                    WHERE table_schema = 'ransomeye' 
                    AND table_name = 'alerts'
                )
            """)
            result = cursor.fetchone()
            alerts_table_exists = result[0] if result else False
            
            if alerts_table_exists:
                cursor.close()
                cursor = conn.cursor(cursor_factory=RealDictCursor)
                cursor.execute("""
                    SELECT 
                        alert_id,
                        created_at,
                        alert_type,
                        ioc_value,
                        metadata,
                        behavior_signals
                    FROM ransomeye.alerts
                    WHERE created_at >= %s
                    ORDER BY created_at DESC
                    LIMIT 10000
                """, (datetime.now(timezone.utc) - timedelta(days=days),))
                alerts = [dict(row) for row in cursor.fetchall()]
            else:
                logger.warning("ransomeye.alerts table does not exist, skipping")
                alerts = []
            
            # Load forensic data - switch back to regular cursor for EXISTS
            cursor.close()
            cursor = conn.cursor()
            cursor.execute("""
                SELECT EXISTS (
                    SELECT FROM information_schema.tables 
                    WHERE table_schema = 'ransomeye' 
                    AND table_name = 'forensic_evidence'
                )
            """)
            result = cursor.fetchone()
            forensic_table_exists = result[0] if result else False
            
            if forensic_table_exists:
                cursor.close()
                cursor = conn.cursor(cursor_factory=RealDictCursor)
                cursor.execute("""
                    SELECT 
                        forensic_id,
                        created_at,
                        evidence_type,
                        ioc_data,
                        malware_signatures,
                        behavior_analysis
                    FROM ransomeye.forensic_evidence
                    WHERE created_at >= %s
                    ORDER BY created_at DESC
                    LIMIT 5000
                """, (datetime.now(timezone.utc) - timedelta(days=days),))
                forensic = [dict(row) for row in cursor.fetchall()]
            else:
                logger.warning("ransomeye.forensic_evidence table does not exist, skipping")
                forensic = []
            
            cursor.close()
            conn.close()
            
            # Combine and normalize
            telemetry = []
            
            # Only process if we have incidents
            if not incidents:
                logger.info("No incidents found in database")
            
            for incident in incidents:
                telemetry.append({
                    'source': 'incident',
                    'timestamp': incident['created_at'].isoformat() if hasattr(incident['created_at'], 'isoformat') else str(incident['created_at']),
                    'ioc_value': json.loads(incident.get('ioc_data', '{}')).get('value', ''),
                    'ioc_type': json.loads(incident.get('ioc_data', '{}')).get('type', ''),
                    'threat_type': incident.get('threat_type', ''),
                    'metadata': {
                        'severity': incident.get('severity'),
                        'incident_id': incident.get('incident_id')
                    },
                    'behavior_signals': json.loads(incident.get('behavior_signals', '[]')) if isinstance(incident.get('behavior_signals'), str) else incident.get('behavior_signals', [])
                })
            
            for alert in alerts:
                telemetry.append({
                    'source': 'alert',
                    'timestamp': alert['created_at'].isoformat() if hasattr(alert['created_at'], 'isoformat') else str(alert['created_at']),
                    'ioc_value': alert.get('ioc_value', ''),
                    'ioc_type': alert.get('alert_type', ''),
                    'metadata': json.loads(alert.get('metadata', '{}')) if isinstance(alert.get('metadata'), str) else alert.get('metadata', {}),
                    'behavior_signals': json.loads(alert.get('behavior_signals', '[]')) if isinstance(alert.get('behavior_signals'), str) else alert.get('behavior_signals', [])
                })
            
            for foren in forensic:
                ioc_data = json.loads(foren.get('ioc_data', '{}')) if isinstance(foren.get('ioc_data'), str) else foren.get('ioc_data', {})
                telemetry.append({
                    'source': 'forensic',
                    'timestamp': foren['created_at'].isoformat() if hasattr(foren['created_at'], 'isoformat') else str(foren['created_at']),
                    'ioc_value': ioc_data.get('value', ''),
                    'ioc_type': ioc_data.get('type', ''),
                    'metadata': {
                        'evidence_type': foren.get('evidence_type'),
                        'forensic_id': foren.get('forensic_id')
                    },
                    'behavior_signals': json.loads(foren.get('behavior_analysis', '[]')) if isinstance(foren.get('behavior_analysis'), str) else foren.get('behavior_analysis', [])
                })
            
            logger.info(f"Loaded {len(telemetry)} telemetry records from last {days} days")
            return telemetry
            
        except Exception as e:
            error_msg = str(e) if str(e) else f"{type(e).__name__}"
            logger.error(f"Error loading telemetry: {type(e).__name__}: {error_msg}")
            if conn:
                try:
                    conn.close()
                except:
                    pass
            return []
    
    def load_external_feeds(self) -> List[Dict]:
        """
        Load external threat intelligence feeds from cache.
        
        Returns:
            List of IOC records from feeds
        """
        iocs = []
        
        # Load from all feed caches
        for cache_subdir in CACHE_DIR.iterdir():
            if not cache_subdir.is_dir():
                continue
            
            for cache_file in cache_subdir.glob("*.json"):
                try:
                    with open(cache_file, 'r') as f:
                        feed_data = json.load(f)
                        
                        # Extract IOCs based on feed structure
                        if 'iocs' in feed_data:
                            iocs.extend(feed_data['iocs'])
                        elif 'samples' in feed_data:
                            for sample in feed_data['samples']:
                                iocs.append({
                                    'value': sample.get('sha256_hash', sample.get('md5_hash', '')),
                                    'type': 'hash',
                                    'metadata': sample
                                })
                        elif 'groups' in feed_data or 'victims' in feed_data:
                            # Ransomware.live format
                            for group in feed_data.get('groups', []):
                                iocs.append({
                                    'value': group.get('name', ''),
                                    'type': 'group_name',
                                    'metadata': group
                                })
                            for victim in feed_data.get('victims', []):
                                iocs.append({
                                    'value': victim.get('victim', ''),
                                    'type': 'victim',
                                    'metadata': victim
                                })
                        
                except Exception as e:
                    logger.warning(f"Failed to load {cache_file}: {e}")
                    continue
        
        logger.info(f"Loaded {len(iocs)} IOCs from external feeds")
        return iocs
    
    def extract_features(self, iocs: List[Dict]) -> Tuple[np.ndarray, np.ndarray]:
        """
        Extract feature vectors from IOCs for training.
        
        Args:
            iocs: List of IOC dictionaries
        
        Returns:
            Tuple of (X, y) feature matrix and labels
        """
        # Classify all IOCs
        classified_iocs = self.threat_classifier.batch_classify(iocs)
        
        # Extract features
        n_features = 256
        X = np.zeros((len(classified_iocs), n_features))
        y = np.zeros(len(classified_iocs), dtype=int)
        
        for idx, ioc in enumerate(classified_iocs):
            # Feature extraction
            # 1. IOC value hash features (first 64 features)
            ioc_value = str(ioc.get('ioc_value', ''))
            hash_bytes = ioc_value.encode()[:32]
            for i, byte in enumerate(hash_bytes):
                if i < 64:
                    X[idx, i] = byte / 255.0
            
            # 2. Threat category encoding (next 32 features)
            categories = ioc.get('threat_categories', [])
            category_map = {
                'ransomware': 0, 'malware': 1, 'ddos': 2, 'trojan': 3,
                'spyware': 4, 'worm': 5, 'mitm': 6, 'sql_injection': 7,
                'dns_tunneling': 8, 'ai_driven': 9, 'supply_chain': 10,
                'zero_day': 11, 'cryptojacking': 12
            }
            for cat in categories[:32]:
                if cat in category_map:
                    X[idx, 64 + category_map[cat] % 32] = 1.0
            
            # 3. Confidence and metadata (next 32 features)
            X[idx, 96] = ioc.get('classification_confidence', 0.0)
            metadata = ioc.get('metadata', {})
            if isinstance(metadata, dict):
                for i, (key, val) in enumerate(list(metadata.items())[:31]):
                    if isinstance(val, (int, float)):
                        X[idx, 97 + i] = float(val) / 1000.0  # Normalize
                    elif isinstance(val, str):
                        X[idx, 97 + i] = hash(val) % 1000 / 1000.0
            
            # 4. Behavioral signals (next 64 features)
            behavior_signals = ioc.get('behavior_signals', [])
            for i, signal in enumerate(behavior_signals[:64]):
                X[idx, 128 + i] = hash(signal) % 1000 / 1000.0
            
            # 5. Temporal features (last 64 features)
            timestamp = ioc.get('timestamp', datetime.now(timezone.utc).isoformat())
            try:
                dt = datetime.fromisoformat(timestamp.replace('Z', '+00:00'))
                X[idx, 192] = dt.hour / 24.0
                X[idx, 193] = dt.day / 31.0
                X[idx, 194] = dt.month / 12.0
                X[idx, 195] = (dt.year - 2020) / 10.0  # Normalize year
            except:
                pass
            
            # Label: 1 if threat detected, 0 otherwise
            y[idx] = 1 if ioc.get('classification_confidence', 0.0) >= 0.6 else 0
        
        return X, y
    
    def check_model_drift(self, model_name: str) -> Tuple[bool, float]:
        """
        Check if model performance has drifted.
        
        Args:
            model_name: Name of the model to check
        
        Returns:
            Tuple of (drift_detected, drift_score)
        """
        # Load current model metrics
        model_path = MODELS_DIR / f"{model_name}.model"
        if not model_path.exists():
            return True, 1.0  # No model exists, need training
        
        # Load baseline metrics
        baseline_path = MODELS_DIR / f"{model_name}_baseline_metrics.json"
        if not baseline_path.exists():
            return False, 0.0  # No baseline to compare
        
        try:
            with open(baseline_path, 'r') as f:
                baseline_metrics = json.load(f)
            
            # In production, would compare with current metrics
            # For now, return no drift
            return False, 0.0
            
        except Exception as e:
            logger.warning(f"Error checking drift: {e}")
            return False, 0.0
    
    def train_model(self, X: np.ndarray, y: np.ndarray, model_name: str) -> Dict:
        """
        Train a model with incremental learning.
        
        Args:
            X: Feature matrix
            y: Labels
            model_name: Name of the model
        
        Returns:
            Model metadata
        """
        from sklearn.ensemble import RandomForestClassifier, GradientBoostingClassifier
        from sklearn.model_selection import train_test_split
        from sklearn.metrics import accuracy_score, precision_score, recall_score, f1_score
        
        logger.info(f"Training {model_name} with {len(X)} samples")
        
        # Split data
        X_train, X_test, y_train, y_test = train_test_split(
            X, y, test_size=0.2, random_state=42
        )
        
        # Train ensemble model with increased complexity for larger model size
        model = GradientBoostingClassifier(
            n_estimators=500,
            max_depth=30,
            learning_rate=0.05,
            min_samples_split=2,
            min_samples_leaf=1,
            max_features='sqrt',
            random_state=42,
            verbose=1
        )
        
        model.fit(X_train, y_train)
        
        # Evaluate
        y_pred = model.predict(X_test)
        accuracy = accuracy_score(y_test, y_pred)
        precision = precision_score(y_test, y_pred, average='weighted', zero_division=0)
        recall = recall_score(y_test, y_pred, average='weighted', zero_division=0)
        f1 = f1_score(y_test, y_pred, average='weighted', zero_division=0)
        
        metrics = {
            'accuracy': float(accuracy),
            'precision': float(precision),
            'recall': float(recall),
            'f1_score': float(f1),
            'n_samples': len(X),
            'n_features': X.shape[1],
            'training_timestamp': datetime.now(timezone.utc).isoformat().replace('+00:00', '') + 'Z'
        }
        
        # Save model
        model_path = MODELS_DIR / f"{model_name}.model"
        MODELS_DIR.mkdir(parents=True, exist_ok=True)
        
        with open(model_path, 'wb') as f:
            pickle.dump(model, f)
        
        # Save metrics
        metrics_path = MODELS_DIR / f"{model_name}_metrics.json"
        with open(metrics_path, 'w') as f:
            json.dump(metrics, f, indent=2)
        
        logger.info(f"Model saved: {model_path}")
        logger.info(f"Metrics: accuracy={accuracy:.4f}, f1={f1:.4f}")
        
        return {
            'model_path': str(model_path),
            'metrics': metrics
        }
    
    def run_continuous_training(self, force: bool = False) -> Dict:
        """
        Run continuous training cycle.
        
        Args:
            force: Force training even if not needed
        
        Returns:
            Training results
        """
        logger.info("Starting continuous training cycle")
        
        # Load data
        logger.info("Loading internal telemetry...")
        internal_data = self.load_internal_telemetry(days=30)
        
        logger.info("Loading external feeds...")
        external_data = self.load_external_feeds()
        
        # Combine data
        all_iocs = internal_data + external_data
        
        if len(all_iocs) < self.min_samples_for_retrain and not force:
            logger.info(f"Insufficient data for retraining ({len(all_iocs)} < {self.min_samples_for_retrain})")
            return {'status': 'skipped', 'reason': 'insufficient_data'}
        
        logger.info(f"Total IOCs for training: {len(all_iocs)}")
        
        # Extract features
        logger.info("Extracting features...")
        X, y = self.extract_features(all_iocs)
        
        # Train models
        results = {}
        
        # Train threat classification model
        logger.info("Training threat classification model...")
        threat_model_result = self.train_model(X, y, "threat_classifier_continuous")
        results['threat_classifier'] = threat_model_result
        
        # Generate SHAP explanations
        try:
            logger.info("Generating SHAP explanations...")
            explainer = SHAPExplainer(
                pickle.load(open(threat_model_result['model_path'], 'rb')),
                X[:100]  # Sample for SHAP
            )
            sample = X[0:1]
            explanation = explainer.explain(sample[0])
            shap_path = explainer.save_explanation(
                explanation,
                "threat_classifier_continuous",
                "1.0.0"
            )
            logger.info(f"SHAP explanation saved: {shap_path}")
            results['shap_path'] = str(shap_path)
        except Exception as e:
            logger.warning(f"Failed to generate SHAP: {e}")
        
        logger.info("Continuous training cycle completed")
        
        return {
            'status': 'success',
            'results': results,
            'n_samples': len(all_iocs),
            'timestamp': datetime.now(timezone.utc).isoformat().replace('+00:00', '') + 'Z'
        }


def main():
    """CLI entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description='Continuous Training System')
    parser.add_argument('--force', action='store_true',
                       help='Force training even if not needed')
    
    args = parser.parse_args()
    
    trainer = ContinuousTrainer()
    results = trainer.run_continuous_training(force=args.force)
    
    print(json.dumps(results, indent=2))


if __name__ == '__main__':
    main()

