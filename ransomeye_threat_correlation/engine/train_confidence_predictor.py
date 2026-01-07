# Path and File Name : /home/ransomeye/rebuild/ransomeye_threat_correlation/engine/train_confidence_predictor.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Train confidence predictor for threat correlation

import os
import sys
import json
import pickle
import hashlib
import numpy as np
from pathlib import Path
from datetime import datetime
from sklearn.ensemble import GradientBoostingRegressor
from sklearn.model_selection import train_test_split
from sklearn.metrics import mean_squared_error, r2_score
import warnings
warnings.filterwarnings('ignore')

RANDOM_SEED = 42
np.random.seed(RANDOM_SEED)

def generate_correlation_training_data(n_samples=50000, n_features=128):
    """Generate synthetic threat correlation training data."""
    # Features: entity similarity, temporal proximity, IOC overlap, etc.
    X = np.random.rand(n_samples, n_features)
    
    # Confidence scores (0-1): higher for strong correlations
    y = (
        0.3 * X[:, 0] +  # Entity similarity
        0.25 * X[:, 1] +  # Temporal proximity
        0.2 * X[:, 2] +   # IOC overlap
        0.15 * X[:, 3] +  # Attack pattern similarity
        0.1 * np.random.rand(n_samples)  # Noise
    )
    y = np.clip(y, 0.0, 1.0)
    
    return X, y

def main():
    models_dir = Path("/home/ransomeye/rebuild/ransomeye_threat_correlation/models")
    models_dir.mkdir(parents=True, exist_ok=True)
    
    print("Training threat correlation confidence predictor...")
    
    # Generate training data
    X, y = generate_correlation_training_data(n_samples=50000, n_features=128)
    
    # Split data
    X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=0.2, random_state=RANDOM_SEED
    )
    
    # Train model
    model = GradientBoostingRegressor(
        n_estimators=100,
        max_depth=5,
        learning_rate=0.1,
        random_state=RANDOM_SEED
    )
    model.fit(X_train, y_train)
    
    # Evaluate
    y_pred = model.predict(X_test)
    mse = mean_squared_error(y_test, y_pred)
    r2 = r2_score(y_test, y_pred)
    
    print(f"  MSE: {mse:.4f}")
    print(f"  R²: {r2:.4f}")
    
    # Save model
    model_path = models_dir / "confidence_predictor.model"
    with open(model_path, 'wb') as f:
        pickle.dump(model, f)
    
    # Compute hash
    with open(model_path, 'rb') as f:
        model_data = f.read()
    model_hash = hashlib.sha256(model_data).hexdigest()
    
    # Create metadata
    metadata = {
        'model_name': 'confidence_predictor',
        'model_version': '1.0.0',
        'model_hash': model_hash,
        'model_size_bytes': len(model_data),
        'trained_on': datetime.utcnow().isoformat() + 'Z',
        'model_type': 'GradientBoostingRegressor',
        'mse': float(mse),
        'r2_score': float(r2),
        'n_features': 128,
        'n_samples': 50000
    }
    
    metadata_path = models_dir / "confidence_predictor_metadata.json"
    with open(metadata_path, 'w') as f:
        json.dump(metadata, f, indent=2)
    
    print(f"✓ Model saved: {model_path}")
    print(f"  Hash: {model_hash}")
    print("✓ Training complete")

if __name__ == '__main__':
    main()
