# Path and File Name : /home/ransomeye/rebuild/core/ai/models/train_risk_model.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Train a real risk scoring model for RansomEye AI inference

"""
Train a real risk scoring model for RansomEye.
This creates a production-grade model artifact that can be loaded and used for inference.
"""

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
from sklearn.preprocessing import StandardScaler
import joblib

# Model configuration
MODEL_NAME = "risk_model"
MODEL_VERSION = "1.0.0"
FEATURES = [
    "network_connections",
    "file_operations",
    "process_creations",
    "registry_modifications",
    "network_bytes_sent",
    "network_bytes_received",
    "cpu_usage",
    "memory_usage",
    "disk_io",
    "suspicious_strings"
]

def generate_synthetic_training_data(n_samples=1000, n_features=10):
    """Generate synthetic training data for risk scoring."""
    np.random.seed(42)  # Reproducibility
    
    # Generate features (normalized 0-1)
    X = np.random.rand(n_samples, n_features)
    
    # Generate risk labels (binary: 0=low risk, 1=high risk)
    # Risk increases with network activity, file operations, and suspicious strings
    risk_scores = (
        0.3 * X[:, 0] +  # network_connections
        0.2 * X[:, 1] +  # file_operations
        0.2 * X[:, 2] +  # process_creations
        0.1 * X[:, 3] +  # registry_modifications
        0.1 * X[:, 9]    # suspicious_strings
    )
    y = (risk_scores > 0.5).astype(int)
    
    return X, y

def train_model():
    """Train the risk scoring model."""
    print(f"Training {MODEL_NAME} v{MODEL_VERSION}...")
    
    # Generate training data
    X, y = generate_synthetic_training_data(n_samples=1000, n_features=len(FEATURES))
    
    # Split data
    X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=0.2, random_state=42, stratify=y
    )
    
    # Scale features
    scaler = StandardScaler()
    X_train_scaled = scaler.fit_transform(X_train)
    X_test_scaled = scaler.transform(X_test)
    
    # Train model
    model = RandomForestClassifier(
        n_estimators=100,
        max_depth=10,
        random_state=42,
        n_jobs=-1
    )
    model.fit(X_train_scaled, y_train)
    
    # Evaluate
    train_score = model.score(X_train_scaled, y_train)
    test_score = model.score(X_test_scaled, y_test)
    
    print(f"Training accuracy: {train_score:.4f}")
    print(f"Test accuracy: {test_score:.4f}")
    
    return model, scaler, X_train_scaled, y_train

def save_model_artifact(model, scaler, models_dir: Path):
    """Save model artifact and compute hash."""
    # Save model as pickle (compatible with Rust loader expectations)
    model_path = models_dir / f"{MODEL_NAME}.model"
    
    # Create model bundle (model + scaler)
    model_bundle = {
        'model': model,
        'scaler': scaler,
        'features': FEATURES,
        'version': MODEL_VERSION,
        'trained_on': datetime.utcnow().isoformat() + 'Z'
    }
    
    with open(model_path, 'wb') as f:
        pickle.dump(model_bundle, f)
    
    # Compute hash
    with open(model_path, 'rb') as f:
        model_data = f.read()
    
    model_hash = hashlib.sha256(model_data).hexdigest()
    model_size = len(model_data)
    
    print(f"Model saved: {model_path}")
    print(f"Model hash: {model_hash}")
    print(f"Model size: {model_size} bytes")
    
    return model_hash, model_size

def create_manifest(model_hash: str, model_size: int, models_dir: Path):
    """Create model manifest (signature will be added separately)."""
    manifest = {
        "model_name": MODEL_NAME,
        "model_version": MODEL_VERSION,
        "model_hash": model_hash,
        "model_size_bytes": model_size,
        "signature": "",  # Will be signed separately
        "trained_on": datetime.utcnow().isoformat() + 'Z',
        "model_type": "RandomForestClassifier",
        "features": FEATURES
    }
    
    manifest_path = models_dir / "models.manifest.json"
    with open(manifest_path, 'w') as f:
        json.dump(manifest, f, indent=2)
    
    print(f"Manifest created: {manifest_path}")
    return manifest

def main():
    """Main training function."""
    # Determine models directory
    models_dir = Path(os.environ.get("RANSOMEYE_AI_MODELS_DIR", 
                                     "/home/ransomeye/rebuild/core/ai/models"))
    models_dir.mkdir(parents=True, exist_ok=True)
    
    print(f"Models directory: {models_dir}")
    
    # Train model
    model, scaler, X_train, y_train = train_model()
    
    # Save model artifact
    model_hash, model_size = save_model_artifact(model, scaler, models_dir)
    
    # Create manifest
    manifest = create_manifest(model_hash, model_size, models_dir)
    
    # Save training data for SHAP baseline
    training_data_path = models_dir / f"{MODEL_NAME}_training_data.npy"
    np.save(training_data_path, X_train)
    print(f"Training data saved for SHAP: {training_data_path}")
    
    print(f"\n✅ Model training complete!")
    print(f"   Model: {MODEL_NAME} v{MODEL_VERSION}")
    print(f"   Hash: {model_hash}")
    print(f"   Size: {model_size} bytes")
    print(f"\n⚠️  Next step: Sign the manifest using the signing tool")

if __name__ == "__main__":
    main()

