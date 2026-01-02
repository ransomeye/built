# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack/train_baseline_models.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Trains baseline intelligence models using synthetic + red-team data only

"""
Baseline Model Training: Trains baseline intelligence models.
Uses ONLY synthetic and red-team data. No customer data.
Deterministic training pipeline for reproducibility.
"""

import os
import sys
import json
import pickle
import hashlib
import numpy as np
from pathlib import Path
from datetime import datetime
from typing import Dict, Tuple, List
from sklearn.ensemble import RandomForestClassifier, IsolationForest
from sklearn.calibration import CalibratedClassifierCV
from sklearn.model_selection import train_test_split
from sklearn.metrics import accuracy_score, precision_score, recall_score, f1_score
import warnings
warnings.filterwarnings('ignore')

# Set random seed for reproducibility
RANDOM_SEED = 42
np.random.seed(RANDOM_SEED)

BASELINE_PACK_DIR = Path("/home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack")
MODELS_DIR = BASELINE_PACK_DIR / "models"


def generate_synthetic_ransomware_data(n_samples: int = 100000, n_features: int = 256) -> Tuple[np.ndarray, np.ndarray]:
    """
    Generate synthetic ransomware behavior data.
    
    Ransomware indicators:
    - High file entropy
    - Rapid file modifications
    - Network connections to suspicious IPs
    - Registry modifications
    - Process injection patterns
    """
    X = np.random.randn(n_samples, n_features)
    y = np.zeros(n_samples, dtype=int)
    
    # Generate ransomware samples (30% of data)
    n_ransomware = int(n_samples * 0.3)
    ransomware_indices = np.random.choice(n_samples, n_ransomware, replace=False)
    
    # Ransomware patterns: higher entropy, more file ops, suspicious network
    for idx in ransomware_indices:
        # High entropy features (features 0-50)
        X[idx, 0:50] += np.random.exponential(2.0, 50)
        # Rapid file modifications (features 51-100)
        X[idx, 51:100] += np.random.poisson(10, 49)
        # Suspicious network patterns (features 101-150)
        X[idx, 101:150] += np.random.gamma(2, 2, 49)
        # Registry modifications (features 151-200)
        X[idx, 151:200] += np.random.beta(2, 5, 49) * 10
        # Process injection (features 201-256)
        X[idx, 201:256] += np.random.exponential(1.5, 55)
        
        y[idx] = 1  # ransomware
    
    # Generate suspicious samples (20% of data)
    n_suspicious = int(n_samples * 0.2)
    remaining_indices = np.setdiff1d(np.arange(n_samples), ransomware_indices)
    suspicious_indices = np.random.choice(remaining_indices, n_suspicious, replace=False)
    
    for idx in suspicious_indices:
        # Moderate suspicious patterns
        X[idx, 0:50] += np.random.exponential(1.0, 50)
        X[idx, 51:100] += np.random.poisson(5, 49)
        y[idx] = 2  # suspicious
    
    # Remaining are benign (50%)
    # y remains 0 for benign
    
    return X, y


def generate_synthetic_anomaly_data(n_samples: int = 100000, n_features: int = 128) -> np.ndarray:
    """
    Generate synthetic anomaly detection baseline data.
    Normal operations with occasional anomalies.
    """
    X = np.random.randn(n_samples, n_features)
    
    # Generate anomalies (1% contamination)
    n_anomalies = int(n_samples * 0.01)
    anomaly_indices = np.random.choice(n_samples, n_anomalies, replace=False)
    
    for idx in anomaly_indices:
        # Anomalies have significantly different patterns
        X[idx] += np.random.randn(n_features) * 5
    
    return X


def train_ransomware_behavior_model(X: np.ndarray, y: np.ndarray) -> Tuple[RandomForestClassifier, Dict]:
    """
    Train ransomware behavior classifier.
    
    Returns:
        Tuple of (model, metrics)
    """
    # Split data
    X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=0.2, random_state=RANDOM_SEED, stratify=y
    )
    
    # Train Random Forest
    model = RandomForestClassifier(
        n_estimators=100,
        max_depth=20,
        min_samples_split=5,
        min_samples_leaf=2,
        random_state=RANDOM_SEED,
        n_jobs=-1
    )
    
    model.fit(X_train, y_train)
    
    # Evaluate
    y_pred = model.predict(X_test)
    accuracy = accuracy_score(y_test, y_pred)
    precision = precision_score(y_test, y_pred, average='weighted')
    recall = recall_score(y_test, y_pred, average='weighted')
    f1 = f1_score(y_test, y_pred, average='weighted')
    
    metrics = {
        'accuracy': float(accuracy),
        'precision': float(precision),
        'recall': float(recall),
        'f1_score': float(f1)
    }
    
    return model, metrics


def train_anomaly_baseline_model(X: np.ndarray) -> IsolationForest:
    """
    Train anomaly detection baseline model.
    
    Returns:
        Isolation Forest model
    """
    model = IsolationForest(
        contamination=0.01,
        random_state=RANDOM_SEED,
        n_estimators=100,
        n_jobs=-1
    )
    
    model.fit(X)
    
    return model


def train_confidence_calibration_model(X: np.ndarray, y: np.ndarray) -> Tuple[CalibratedClassifierCV, Dict]:
    """
    Train confidence calibration model.
    
    Returns:
        Tuple of (calibrated_model, metrics)
    """
    # Split data
    X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=0.2, random_state=RANDOM_SEED, stratify=y
    )
    
    # Base classifier
    base_model = RandomForestClassifier(
        n_estimators=50,
        max_depth=10,
        random_state=RANDOM_SEED,
        n_jobs=-1
    )
    
    # Calibrate using Platt scaling
    calibrated_model = CalibratedClassifierCV(
        base_model,
        method='sigmoid',
        cv=5
    )
    
    calibrated_model.fit(X_train, y_train)
    
    # Evaluate calibration
    y_pred_proba = calibrated_model.predict_proba(X_test)
    y_pred = calibrated_model.predict(X_test)
    accuracy = accuracy_score(y_test, y_pred)
    
    metrics = {
        'accuracy': float(accuracy),
        'calibration_method': 'platt_scaling'
    }
    
    return calibrated_model, metrics


def compute_file_hash(file_path: Path) -> str:
    """Compute SHA-256 hash of file."""
    sha256 = hashlib.sha256()
    with open(file_path, 'rb') as f:
        for chunk in iter(lambda: f.read(4096), b''):
            sha256.update(chunk)
    return sha256.hexdigest()


def save_model(model, model_path: Path, metadata: Dict) -> str:
    """Save model and return hash."""
    with open(model_path, 'wb') as f:
        pickle.dump(model, f)
    
    model_hash = compute_file_hash(model_path)
    return model_hash


def main():
    """Main training pipeline."""
    import argparse
    
    parser = argparse.ArgumentParser(description='RansomEye Baseline Model Training')
    parser.add_argument('--use-feeds', action='store_true',
                       help='Enhance training data with threat intelligence feeds')
    parser.add_argument('--feed-only', action='store_true',
                       help='Use only threat intelligence feeds (no synthetic data)')
    
    args = parser.parse_args()
    
    print("=" * 80)
    print("RansomEye Baseline Intelligence Pack - Model Training")
    print("=" * 80)
    print()
    
    if args.use_feeds:
        print("Training models using SYNTHETIC + RED-TEAM + THREAT INTELLIGENCE FEEDS")
    elif args.feed_only:
        print("Training models using THREAT INTELLIGENCE FEEDS only")
    else:
        print("Training models using SYNTHETIC + RED-TEAM data only")
    
    print("No customer data used")
    print()
    
    # Ensure models directory exists
    MODELS_DIR.mkdir(parents=True, exist_ok=True)
    
    # Training data hashes (for reproducibility tracking)
    training_data_hashes = {}
    
    # 1. Train Ransomware Behavior Model
    print("Training ransomware behavior classifier...")
    
    if args.use_feeds or args.feed_only:
        # Use enhanced data generator with threat intelligence feeds
        try:
            from enhance_training_with_feeds import generate_enhanced_ransomware_data
            X_ransomware, y_ransomware = generate_enhanced_ransomware_data(
                n_samples=100000,
                n_features=256,
                use_feeds=True
            )
            print("  ✓ Using enhanced training data with threat intelligence feeds")
        except Exception as e:
            print(f"  ⚠ Warning: Failed to load threat intelligence feeds: {e}")
            print("  Falling back to synthetic data only")
            X_ransomware, y_ransomware = generate_synthetic_ransomware_data(n_samples=100000, n_features=256)
    else:
        X_ransomware, y_ransomware = generate_synthetic_ransomware_data(n_samples=100000, n_features=256)
    
    model_ransomware, metrics_ransomware = train_ransomware_behavior_model(X_ransomware, y_ransomware)
    
    model_path = MODELS_DIR / "ransomware_behavior.model"
    model_hash = save_model(model_ransomware, model_path, metrics_ransomware)
    training_data_hashes['ransomware_behavior'] = hashlib.sha256(X_ransomware.tobytes()).hexdigest()
    
    print(f"  ✓ Model saved: {model_path}")
    print(f"    Accuracy: {metrics_ransomware['accuracy']:.4f}")
    print(f"    Precision: {metrics_ransomware['precision']:.4f}")
    print(f"    Recall: {metrics_ransomware['recall']:.4f}")
    print(f"    F1-Score: {metrics_ransomware['f1_score']:.4f}")
    print(f"    Hash: sha256:{model_hash}")
    print()
    
    # 2. Train Anomaly Baseline Model
    print("Training anomaly detection baseline...")
    X_anomaly = generate_synthetic_anomaly_data(n_samples=100000, n_features=128)
    model_anomaly = train_anomaly_baseline_model(X_anomaly)
    
    model_path = MODELS_DIR / "anomaly_baseline.model"
    model_hash = save_model(model_anomaly, model_path, {})
    training_data_hashes['anomaly_baseline'] = hashlib.sha256(X_anomaly.tobytes()).hexdigest()
    
    print(f"  ✓ Model saved: {model_path}")
    print(f"    Contamination: 0.01")
    print(f"    Hash: sha256:{model_hash}")
    print()
    
    # 3. Train Confidence Calibration Model
    print("Training confidence calibration model...")
    # Use subset of ransomware data for calibration
    X_calibration = X_ransomware[:50000, :64]  # Use first 64 features
    y_calibration = y_ransomware[:50000]
    model_calibration, metrics_calibration = train_confidence_calibration_model(X_calibration, y_calibration)
    
    model_path = MODELS_DIR / "confidence_calibration.model"
    model_hash = save_model(model_calibration, model_path, metrics_calibration)
    training_data_hashes['confidence_calibration'] = hashlib.sha256(X_calibration.tobytes()).hexdigest()
    
    print(f"  ✓ Model saved: {model_path}")
    print(f"    Accuracy: {metrics_calibration['accuracy']:.4f}")
    print(f"    Method: {metrics_calibration['calibration_method']}")
    print(f"    Hash: sha256:{model_hash}")
    print()
    
    # Update model manifest with real hashes
    print("Updating model manifest with real hashes...")
    manifest_path = MODELS_DIR / "model_manifest.json"
    
    with open(manifest_path, 'r') as f:
        manifest = json.load(f)
    
    # Update hashes
    for model_info in manifest['models']:
        model_name = model_info['name']
        if model_name == "ransomware_behavior.model":
            model_path = MODELS_DIR / model_name
            model_info['hash'] = f"sha256:{compute_file_hash(model_path)}"
            model_info['training_data_hash'] = f"sha256:{training_data_hashes['ransomware_behavior']}"
            model_info['accuracy'] = metrics_ransomware['accuracy']
            model_info['precision'] = metrics_ransomware['precision']
            model_info['recall'] = metrics_ransomware['recall']
            model_info['f1_score'] = metrics_ransomware['f1_score']
        elif model_name == "anomaly_baseline.model":
            model_path = MODELS_DIR / model_name
            model_info['hash'] = f"sha256:{compute_file_hash(model_path)}"
            model_info['training_data_hash'] = f"sha256:{training_data_hashes['anomaly_baseline']}"
        elif model_name == "confidence_calibration.model":
            model_path = MODELS_DIR / model_name
            model_info['hash'] = f"sha256:{compute_file_hash(model_path)}"
            model_info['training_data_hash'] = f"sha256:{training_data_hashes['confidence_calibration']}"
    
    # Update pack hash
    pack_hash = hashlib.sha256(json.dumps(manifest, sort_keys=True).encode()).hexdigest()
    manifest['pack_hash'] = f"sha256:{pack_hash}"
    manifest['created'] = datetime.utcnow().isoformat() + 'Z'
    manifest['trained_on'] = datetime.utcnow().isoformat() + 'Z'
    
    with open(manifest_path, 'w') as f:
        json.dump(manifest, f, indent=2)
    
    print(f"  ✓ Model manifest updated: {manifest_path}")
    print()
    
    print("=" * 80)
    print("✓ Baseline model training complete")
    print("=" * 80)
    print()
    print("Next steps:")
    print("  1. Generate SHAP baselines for trained models")
    print("  2. Generate cryptographic signatures")
    print("  3. Update training manifest with real hashes")
    print()


if __name__ == '__main__':
    main()

