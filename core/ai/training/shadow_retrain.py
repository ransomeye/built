# Path and File Name : /home/ransomeye/rebuild/core/ai/training/shadow_retrain.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Shadow retraining - executes shadow retraining using delta-only data, produces candidate model versions without activation

"""
Shadow Retraining (PROMPT-61 Phase 2)

Executes shadow retraining using delta-only data:
- Uses threat_intel_delta data only
- Produces candidate model versions
- Generates metrics + SHAP baselines
- Does NOT activate models

Training artifacts signed and stored under /var/lib/ransomeye/models/candidates/
Registry entries marked state = CANDIDATE
"""

import os
import sys
import json
import hashlib
import pickle
import psycopg2
from pathlib import Path
from datetime import datetime, timezone
from typing import Dict, List, Optional, Tuple
import logging
import numpy as np
from sklearn.ensemble import RandomForestClassifier
from sklearn.model_selection import train_test_split
from sklearn.metrics import accuracy_score, precision_score, recall_score, f1_score
import shap

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger('shadow_retrain')

# Database configuration
DB_NAME = os.environ.get("DB_NAME", "ransomeye")
DB_USER = os.environ.get("DB_USER", "gagan")
DB_HOST = os.environ.get("DB_HOST", "localhost")
DB_PASSWORD = os.environ.get("DB_PASS", "gagan")
DB_PORT = int(os.environ.get("DB_PORT", "5432"))

# Model directory
CANDIDATES_DIR = Path("/var/lib/ransomeye/models/candidates")
CANDIDATES_DIR.mkdir(parents=True, exist_ok=True)


class ShadowRetrainer:
    """Shadow retraining engine."""
    
    def __init__(self):
        """Initialize shadow retrainer."""
        self.conn = None
        self.candidate_models = []
        
    def connect_db(self) -> bool:
        """Connect to database."""
        try:
            self.conn = psycopg2.connect(
                host=DB_HOST,
                port=DB_PORT,
                database=DB_NAME,
                user=DB_USER,
                password=DB_PASSWORD
            )
            logger.info("✓ Connected to database")
            return True
        except Exception as e:
            logger.error(f"✗ Failed to connect to database: {e}")
            return False
    
    def load_delta_data(self) -> Tuple[Optional[np.ndarray], Optional[np.ndarray]]:
        """Load delta-only data for training."""
        if not self.conn:
            return None, None
        
        cursor = self.conn.cursor()
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Get all deltas
            cursor.execute("""
                SELECT 
                    delta_type, ioc_type, ioc_value, source,
                    old_value, new_value, created_at
                FROM threat_intel_delta
                ORDER BY created_at DESC
            """)
            
            deltas = []
            labels = []
            
            for row in cursor.fetchall():
                delta_type = row[0]
                ioc_type = row[1]
                ioc_value = row[2]
                source = row[3]
                old_value = row[4]
                new_value = row[5]
                
                # Extract features from delta
                features = self.extract_features(delta_type, ioc_type, ioc_value, source, old_value, new_value)
                if features is not None:
                    deltas.append(features)
                    # Label: 1 if new IOC or significant change, 0 otherwise
                    label = 1 if delta_type in ['new_ioc', 'ioc_mutation'] else 0
                    labels.append(label)
            
            if len(deltas) == 0:
                logger.warning("⚠ No delta data available for training")
                return None, None
            
            X = np.array(deltas)
            y = np.array(labels)
            
            logger.info(f"✓ Loaded {len(X)} delta samples for training")
            return X, y
        except Exception as e:
            logger.error(f"✗ Failed to load delta data: {e}")
            return None, None
        finally:
            cursor.close()
    
    def extract_features(self, delta_type: str, ioc_type: str, ioc_value: str, 
                        source: str, old_value: Dict, new_value: Dict) -> Optional[List[float]]:
        """Extract features from delta record."""
        try:
            features = []
            
            # Delta type encoding
            delta_type_map = {
                'new_ioc': 1.0,
                'ioc_mutation': 2.0,
                'confidence_shift': 3.0,
                'ttp_pattern': 4.0
            }
            features.append(delta_type_map.get(delta_type, 0.0))
            
            # IOC type encoding
            ioc_type_map = {
                'ip': 1.0,
                'domain': 2.0,
                'hash': 3.0,
                'url': 4.0,
                'email': 5.0
            }
            features.append(ioc_type_map.get(ioc_type.lower(), 0.0))
            
            # Source encoding (hash-based)
            source_hash = hash(source) % 1000 / 1000.0
            features.append(source_hash)
            
            # Confidence change
            old_conf = old_value.get('confidence', 0.0) if old_value else 0.0
            new_conf = new_value.get('confidence', 0.0) if new_value else 0.0
            features.append(abs(new_conf - old_conf))
            
            # Correlation count change
            old_corr = old_value.get('correlated_count', 0) if old_value else 0
            new_corr = new_value.get('correlated_count', 0) if new_value else 0
            features.append(abs(new_corr - old_corr))
            
            # Tag count
            old_tags = len(old_value.get('tags', [])) if old_value else 0
            new_tags = len(new_value.get('tags', [])) if new_value else 0
            features.append(new_tags)
            features.append(abs(new_tags - old_tags))
            
            return features
        except Exception as e:
            logger.warning(f"⚠ Failed to extract features: {e}")
            return None
    
    def train_candidate_model(self, X: np.ndarray, y: np.ndarray, model_name: str) -> Optional[Dict]:
        """Train candidate model."""
        try:
            # Split data
            if len(X) < 10:
                logger.warning(f"⚠ Insufficient data for training {model_name} (need at least 10 samples)")
                return None
            
            X_train, X_test, y_train, y_test = train_test_split(
                X, y, test_size=0.2, random_state=42, stratify=y if len(np.unique(y)) > 1 else None
            )
            
            # Train model
            model = RandomForestClassifier(
                n_estimators=100,
                max_depth=10,
                random_state=42,
                n_jobs=-1
            )
            model.fit(X_train, y_train)
            
            # Evaluate
            y_pred = model.predict(X_test)
            accuracy = accuracy_score(y_test, y_pred)
            precision = precision_score(y_test, y_pred, zero_division=0)
            recall = recall_score(y_test, y_pred, zero_division=0)
            f1 = f1_score(y_test, y_pred, zero_division=0)
            
            # Generate SHAP baseline
            explainer = shap.TreeExplainer(model)
            shap_values = explainer.shap_values(X_train[:100])  # Sample for SHAP
            
            # Save model
            model_path = CANDIDATES_DIR / f"{model_name}_candidate.pkl"
            with open(model_path, 'wb') as f:
                pickle.dump(model, f)
            
            # Compute model hash
            with open(model_path, 'rb') as f:
                model_bytes = f.read()
            model_hash = hashlib.sha256(model_bytes).hexdigest()
            
            # Save SHAP baseline
            shap_path = CANDIDATES_DIR / f"{model_name}_candidate_shap.pkl"
            with open(shap_path, 'wb') as f:
                pickle.dump(shap_values, f)
            
            # Create metadata
            metadata = {
                'model_name': model_name,
                'version': f"candidate-{datetime.now(timezone.utc).strftime('%Y%m%d%H%M%S')}",
                'state': 'CANDIDATE',
                'model_path': str(model_path),
                'shap_path': str(shap_path),
                'model_hash': model_hash,
                'metrics': {
                    'accuracy': float(accuracy),
                    'precision': float(precision),
                    'recall': float(recall),
                    'f1_score': float(f1)
                },
                'training_samples': len(X_train),
                'test_samples': len(X_test),
                'created_at': datetime.now(timezone.utc).isoformat()
            }
            
            metadata_path = CANDIDATES_DIR / f"{model_name}_candidate_metadata.json"
            with open(metadata_path, 'w') as f:
                json.dump(metadata, f, indent=2)
            
            logger.info(f"✓ Trained candidate model: {model_name}")
            logger.info(f"  Accuracy: {accuracy:.4f}, F1: {f1:.4f}")
            
            return metadata
        except Exception as e:
            logger.error(f"✗ Failed to train candidate model {model_name}: {e}")
            return None
    
    def register_candidate(self, metadata: Dict) -> bool:
        """Register candidate model in registry."""
        if not self.conn:
            return False
        
        cursor = self.conn.cursor()
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Check if model exists in registry
            cursor.execute("""
                SELECT model_id FROM model_registry
                WHERE model_name = %s
            """, (metadata['model_name'],))
            
            model_row = cursor.fetchone()
            if not model_row:
                # Create model entry
                cursor.execute("""
                    INSERT INTO model_registry (model_name, model_task, description, is_active)
                    VALUES (%s, 'classification', %s, false)
                    RETURNING model_id
                """, (
                    metadata['model_name'],
                    f"Shadow retrained candidate model - {metadata['version']}"
                ))
                model_id = cursor.fetchone()[0]
            else:
                model_id = model_row[0]
            
            # Register candidate version
            cursor.execute("""
                INSERT INTO model_versions (
                    model_id, version, artifact_type, artifact_uri,
                    artifact_sha256, trained_on, shap_enabled,
                    shap_artifact_uri, metadata_json
                )
                VALUES (%s, %s, 'pkl', %s, %s, %s, true, %s, %s::jsonb)
            """, (
                model_id,
                metadata['version'],
                metadata['model_path'],
                bytes.fromhex(metadata['model_hash']),
                'threat_intel_delta',
                metadata['shap_path'],
                json.dumps(metadata)
            ))
            
            self.conn.commit()
            logger.info(f"✓ Registered candidate model: {metadata['model_name']} v{metadata['version']}")
            return True
        except Exception as e:
            logger.error(f"✗ Failed to register candidate: {e}")
            self.conn.rollback()
            return False
        finally:
            cursor.close()
    
    def run(self) -> bool:
        """Run shadow retraining."""
        logger.info("=" * 80)
        logger.info("Shadow Retraining (PROMPT-61 Phase 2)")
        logger.info("=" * 80)
        
        # Connect to database
        if not self.connect_db():
            logger.error("FAIL-CLOSED: Database connection failed")
            return False
        
        # Load delta data
        X, y = self.load_delta_data()
        if X is None or y is None:
            logger.warning("⚠ No delta data available - skipping shadow retraining")
            return True  # Not a failure, just no data
        
        # Train candidate models
        model_names = ['threat_delta_classifier', 'ioc_mutation_detector']
        for model_name in model_names:
            metadata = self.train_candidate_model(X, y, model_name)
            if metadata:
                self.register_candidate(metadata)
                self.candidate_models.append(metadata)
        
        if len(self.candidate_models) == 0:
            logger.warning("⚠ No candidate models trained")
            return True
        
        logger.info(f"✓ Shadow retraining complete: {len(self.candidate_models)} candidates")
        return True


def main():
    """Main entry point."""
    retrainer = ShadowRetrainer()
    success = retrainer.run()
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()

