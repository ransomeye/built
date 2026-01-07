# Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/train_classifier.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Train DPI probe asset classifier model

import os
import sys
import json
import pickle
import hashlib
import numpy as np
from pathlib import Path
from datetime import datetime
from sklearn.ensemble import RandomForestClassifier
from sklearn.model_selection import train_test_split
from sklearn.metrics import accuracy_score, classification_report
import warnings
warnings.filterwarnings('ignore')

RANDOM_SEED = 42
np.random.seed(RANDOM_SEED)

def generate_training_data(n_samples=60000, n_features=128):
    """Generate synthetic training data for asset classification."""
    X = np.random.rand(n_samples, n_features)
    y = np.random.randint(0, 10, n_samples)  # 10 asset classes
    return X, y

def main():
    models_dir = Path("/home/ransomeye/rebuild/ransomeye_dpi_probe/models")
    models_dir.mkdir(parents=True, exist_ok=True)
    
    print("Training DPI probe asset classifier...")
    
    # Generate training data
    X, y = generate_training_data(n_samples=60000, n_features=128)
    
    # Split data
    X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=0.2, random_state=RANDOM_SEED
    )
    
    # Train model
    model = RandomForestClassifier(
        n_estimators=100,
        max_depth=20,
        random_state=RANDOM_SEED,
        n_jobs=-1
    )
    model.fit(X_train, y_train)
    
    # Evaluate
    y_pred = model.predict(X_test)
    accuracy = accuracy_score(y_test, y_pred)
    
    print(f"  Accuracy: {accuracy:.4f}")
    
    # Save model
    model_path = models_dir / "asset_classifier.model"
    with open(model_path, 'wb') as f:
        pickle.dump(model, f)
    
    # Compute hash
    with open(model_path, 'rb') as f:
        model_data = f.read()
    model_hash = hashlib.sha256(model_data).hexdigest()
    
    # Create metadata
    metadata = {
        'model_name': 'asset_classifier',
        'model_version': '1.0.0',
        'model_hash': model_hash,
        'model_size_bytes': len(model_data),
        'trained_on': datetime.now(timezone.utc).isoformat().replace('+00:00', '') + 'Z',
        'model_type': 'RandomForestClassifier',
        'accuracy': float(accuracy),
        'n_samples': len(X),
        'n_features': X.shape[1]
    }
    
    metadata_path = models_dir / "dpi_metadata.json"
    with open(metadata_path, 'w') as f:
        json.dump(metadata, f, indent=2)
    
    print(f"✓ Model saved: {model_path}")
    print("✓ Training complete")

if __name__ == '__main__':
    main()
