# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack/incremental_update.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Incremental learning pipeline for updating models with new data

"""
Incremental Learning Pipeline: Updates models with new labeled data.
Supports continual learning for ransomware behavior and anomaly detection models.
"""

import os
import sys
import json
import pickle
import hashlib
import numpy as np
from pathlib import Path
from datetime import datetime
from typing import Dict, Tuple, List, Optional
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


def load_existing_model(model_name: str) -> Tuple[object, Dict]:
    """Load existing model and metadata."""
    model_path = MODELS_DIR / f"{model_name}.model"
    metadata_path = MODELS_DIR / f"{model_name}_metadata.json"
    
    if not model_path.exists():
        raise FileNotFoundError(f"Model not found: {model_path}")
    
    # Load model
    with open(model_path, 'rb') as f:
        model = pickle.load(f)
    
    # Load metadata
    metadata = {}
    if metadata_path.exists():
        with open(metadata_path, 'r') as f:
            metadata = json.load(f)
    
    return model, metadata


def incremental_update_ransomware_model(
    model: RandomForestClassifier,
    X_new: np.ndarray,
    y_new: np.ndarray,
    n_estimators_increment: int = 10
) -> RandomForestClassifier:
    """
    Incrementally update ransomware behavior model with new data.
    
    Args:
        model: Existing RandomForest model
        X_new: New feature data
        y_new: New labels
        n_estimators_increment: Number of new trees to add
    
    Returns:
        Updated model
    """
    # Get existing trees
    existing_estimators = list(model.estimators_)
    
    # Train new trees on new data
    new_estimators = []
    for _ in range(n_estimators_increment):
        # Sample from new data
        n_samples = min(len(X_new), 1000)
        indices = np.random.choice(len(X_new), n_samples, replace=False)
        X_sample = X_new[indices]
        y_sample = y_new[indices]
        
        # Train new tree
        from sklearn.tree import DecisionTreeClassifier
        tree = DecisionTreeClassifier(
            max_depth=20,
            min_samples_split=5,
            min_samples_leaf=2,
            random_state=RANDOM_SEED
        )
        tree.fit(X_sample, y_sample)
        new_estimators.append(tree)
    
    # Combine estimators
    all_estimators = existing_estimators + new_estimators
    
    # Create new model with combined estimators
    updated_model = RandomForestClassifier(
        n_estimators=len(all_estimators),
        max_depth=20,
        min_samples_split=5,
        min_samples_leaf=2,
        random_state=RANDOM_SEED,
        n_jobs=-1
    )
    
    # Set estimators directly
    updated_model.estimators_ = all_estimators
    updated_model.classes_ = model.classes_
    updated_model.n_classes_ = model.n_classes_
    updated_model.n_features_ = model.n_features_
    updated_model.n_outputs_ = model.n_outputs_
    
    return updated_model


def incremental_update_anomaly_model(
    model: IsolationForest,
    X_new: np.ndarray,
    n_estimators_increment: int = 10
) -> IsolationForest:
    """
    Incrementally update anomaly detection model with new data.
    
    Args:
        model: Existing IsolationForest model
        X_new: New feature data
        n_estimators_increment: Number of new trees to add
    
    Returns:
        Updated model
    """
    # Get existing estimators
    existing_estimators = list(model.estimators_)
    
    # Train new trees on new data
    new_estimators = []
    for _ in range(n_estimators_increment):
        # Sample from new data
        n_samples = min(len(X_new), 1000)
        indices = np.random.choice(len(X_new), n_samples, replace=False)
        X_sample = X_new[indices]
        
        # Train new tree
        from sklearn.tree import ExtraTreeRegressor
        tree = ExtraTreeRegressor(
            max_depth=model.max_depth_,
            random_state=RANDOM_SEED
        )
        tree.fit(X_sample, np.zeros(len(X_sample)))
        new_estimators.append(tree)
    
    # Combine estimators
    all_estimators = existing_estimators + new_estimators
    
    # Create new model
    updated_model = IsolationForest(
        contamination=model.contamination,
        n_estimators=len(all_estimators),
        random_state=RANDOM_SEED,
        n_jobs=-1
    )
    
    # Set estimators
    updated_model.estimators_ = all_estimators
    updated_model.estimators_features_ = model.estimators_features_
    updated_model.max_samples_ = model.max_samples_
    updated_model.n_features_ = model.n_features_
    updated_model.n_features_in_ = model.n_features_in_
    
    return updated_model


def save_updated_model(model, model_path: Path, metadata: Dict, version_increment: str = "0.0.1") -> str:
    """Save updated model and return hash."""
    # Save model
    with open(model_path, 'wb') as f:
        pickle.dump(model, f)
    
    # Compute hash
    model_hash = compute_file_hash(model_path)
    
    # Update metadata
    if 'version' in metadata:
        # Increment version
        version_parts = metadata['version'].split('.')
        if len(version_parts) == 3:
            patch = int(version_parts[2]) + 1
            new_version = f"{version_parts[0]}.{version_parts[1]}.{patch}"
        else:
            new_version = version_increment
    else:
        new_version = version_increment
    
    metadata['version'] = new_version
    metadata['hash'] = f"sha256:{model_hash}"
    metadata['updated_on'] = datetime.utcnow().isoformat() + 'Z'
    metadata['incremental_update'] = True
    
    # Save metadata
    metadata_path = model_path.parent / f"{model_path.stem}_metadata.json"
    with open(metadata_path, 'w') as f:
        json.dump(metadata, f, indent=2)
    
    return model_hash


def compute_file_hash(file_path: Path) -> str:
    """Compute SHA-256 hash of file."""
    sha256 = hashlib.sha256()
    with open(file_path, 'rb') as f:
        for chunk in iter(lambda: f.read(4096), b''):
            sha256.update(chunk)
    return sha256.hexdigest()


def main():
    """Main incremental learning pipeline."""
    import argparse
    
    parser = argparse.ArgumentParser(description='RansomEye Incremental Learning Pipeline')
    parser.add_argument('--model', required=True, choices=['ransomware_behavior', 'anomaly_baseline'],
                       help='Model to update')
    parser.add_argument('--data', required=True, type=Path,
                       help='Path to new training data (numpy .npz file with X and y keys)')
    parser.add_argument('--n-estimators', type=int, default=10,
                       help='Number of new estimators to add')
    
    args = parser.parse_args()
    
    print("=" * 80)
    print("RansomEye Baseline Intelligence Pack - Incremental Learning")
    print("=" * 80)
    print()
    print(f"Updating model: {args.model}")
    print(f"New data: {args.data}")
    print()
    
    # Load existing model
    print(f"Loading existing model: {args.model}...")
    model, metadata = load_existing_model(f"{args.model}.model")
    print(f"  ✓ Model loaded (version: {metadata.get('version', 'unknown')})")
    print()
    
    # Load new data
    print(f"Loading new data from {args.data}...")
    data = np.load(args.data)
    if args.model == 'ransomware_behavior':
        X_new = data['X']
        y_new = data['y']
        print(f"  ✓ Loaded {len(X_new)} samples with {X_new.shape[1]} features")
        
        # Update model
        print("Updating model with new data...")
        updated_model = incremental_update_ransomware_model(
            model, X_new, y_new, n_estimators_increment=args.n_estimators
        )
        
        # Evaluate
        X_test, _, y_test, _ = train_test_split(
            X_new, y_new, test_size=0.2, random_state=RANDOM_SEED
        )
        y_pred = updated_model.predict(X_test)
        accuracy = accuracy_score(y_test, y_pred)
        print(f"  ✓ Updated model accuracy on new data: {accuracy:.4f}")
    else:
        X_new = data['X']
        print(f"  ✓ Loaded {len(X_new)} samples with {X_new.shape[1]} features")
        
        # Update model
        print("Updating model with new data...")
        updated_model = incremental_update_anomaly_model(
            model, X_new, n_estimators_increment=args.n_estimators
        )
        print(f"  ✓ Model updated with {len(updated_model.estimators_)} estimators")
    
    print()
    
    # Save updated model
    model_path = MODELS_DIR / f"{args.model}.model"
    model_hash = save_updated_model(updated_model, model_path, metadata)
    
    print(f"  ✓ Updated model saved: {model_path}")
    print(f"    New version: {metadata['version']}")
    print(f"    Hash: sha256:{model_hash}")
    print()
    
    print("=" * 80)
    print("✓ Incremental learning complete")
    print("=" * 80)


if __name__ == '__main__':
    main()

