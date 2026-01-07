# Path and File Name : /home/ransomeye/rebuild/ransomeye_threat_intel_engine/engine/train_trust_scoring.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Train trust scoring and clustering models for threat intelligence

import os
import sys
import json
import pickle
import hashlib
import numpy as np
from pathlib import Path
from datetime import datetime
from sklearn.ensemble import GradientBoostingRegressor
from sklearn.cluster import DBSCAN
from sklearn.model_selection import train_test_split
from sklearn.metrics import mean_squared_error, r2_score
import warnings
warnings.filterwarnings('ignore')

RANDOM_SEED = 42
np.random.seed(RANDOM_SEED)

def generate_trust_scoring_training_data(n_samples=40000, n_features=96):
    """Generate synthetic trust scoring training data."""
    # Features: source reputation, IOC quality, recency, verification, etc.
    X = np.random.rand(n_samples, n_features)
    
    # Trust scores (0-1): higher for reliable sources
    y = (
        0.3 * X[:, 0] +  # Source reputation
        0.25 * X[:, 1] +  # IOC quality
        0.2 * X[:, 2] +   # Recency
        0.15 * X[:, 3] +  # Verification status
        0.1 * np.random.rand(n_samples)  # Noise
    )
    y = np.clip(y, 0.0, 1.0)
    
    return X, y

def main():
    models_dir = Path("/home/ransomeye/rebuild/ransomeye_threat_intel_engine/models")
    models_dir.mkdir(parents=True, exist_ok=True)
    
    print("Training threat intel trust scoring model...")
    
    # Generate training data
    X, y = generate_trust_scoring_training_data(n_samples=40000, n_features=96)
    
    # Split data
    X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=0.2, random_state=RANDOM_SEED
    )
    
    # Train trust scoring model
    trust_model = GradientBoostingRegressor(
        n_estimators=100,
        max_depth=5,
        learning_rate=0.1,
        random_state=RANDOM_SEED
    )
    trust_model.fit(X_train, y_train)
    
    # Evaluate
    y_pred = trust_model.predict(X_test)
    mse = mean_squared_error(y_test, y_pred)
    r2 = r2_score(y_test, y_pred)
    
    print(f"  MSE: {mse:.4f}")
    print(f"  R²: {r2:.4f}")
    
    # Save trust scoring model
    trust_model_path = models_dir / "trust_scorer.model"
    with open(trust_model_path, 'wb') as f:
        pickle.dump(trust_model, f)
    
    # Compute hash
    with open(trust_model_path, 'rb') as f:
        trust_model_data = f.read()
    trust_model_hash = hashlib.sha256(trust_model_data).hexdigest()
    
    # Train clustering model
    print("Training IOC clustering model...")
    cluster_model = DBSCAN(eps=0.5, min_samples=5)
    cluster_labels = cluster_model.fit_predict(X_train[:10000])  # Sample for clustering
    
    cluster_model_path = models_dir / "ioc_clusterer.model"
    with open(cluster_model_path, 'wb') as f:
        pickle.dump(cluster_model, f)
    
    with open(cluster_model_path, 'rb') as f:
        cluster_model_data = f.read()
    cluster_model_hash = hashlib.sha256(cluster_model_data).hexdigest()
    
    # Create metadata
    metadata = {
        'trust_scorer': {
            'model_name': 'trust_scorer',
            'model_version': '1.0.0',
            'model_hash': trust_model_hash,
            'model_size_bytes': len(trust_model_data),
            'trained_on': datetime.utcnow().isoformat() + 'Z',
            'model_type': 'GradientBoostingRegressor',
            'mse': float(mse),
            'r2_score': float(r2)
        },
        'ioc_clusterer': {
            'model_name': 'ioc_clusterer',
            'model_version': '1.0.0',
            'model_hash': cluster_model_hash,
            'model_size_bytes': len(cluster_model_data),
            'trained_on': datetime.utcnow().isoformat() + 'Z',
            'model_type': 'DBSCAN'
        }
    }
    
    metadata_path = models_dir / "trust_scoring_metadata.json"
    with open(metadata_path, 'w') as f:
        json.dump(metadata, f, indent=2)
    
    print(f"✓ Trust scorer saved: {trust_model_path}")
    print(f"✓ IOC clusterer saved: {cluster_model_path}")
    print("✓ Training complete")

if __name__ == '__main__':
    main()
