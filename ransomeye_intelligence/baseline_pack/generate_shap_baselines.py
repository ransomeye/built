# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack/generate_shap_baselines.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Generates SHAP baseline values for trained models

"""
SHAP Baseline Generator: Generates SHAP baseline values for trained models.
Ensures SHAP explainability is available from Day 1.
"""

import sys
import json
import pickle
import numpy as np
from pathlib import Path
from datetime import datetime
from typing import Dict, List
import warnings
warnings.filterwarnings('ignore')

try:
    import shap
except ImportError:
    print("ERROR: SHAP library not installed. Install with: pip install shap", file=sys.stderr)
    sys.exit(1)

BASELINE_PACK_DIR = Path("/home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack")
MODELS_DIR = BASELINE_PACK_DIR / "models"
SHAP_DIR = BASELINE_PACK_DIR / "shap"


def generate_shap_baseline_for_classifier(model, model_name: str, n_features: int, n_samples: int = 1000) -> Dict:
    """
    Generate SHAP baseline for classifier model.
    
    Returns:
        Dictionary with SHAP baseline statistics
    """
    # Generate background data
    background_data = np.random.randn(n_samples, n_features)
    
    # Create SHAP explainer
    explainer = shap.TreeExplainer(model)
    
    # Compute SHAP values for background data
    shap_values = explainer.shap_values(background_data)
    
    # Handle multi-class case
    if isinstance(shap_values, list):
        # Multi-class: average across classes
        shap_values_avg = np.mean([np.abs(sv) for sv in shap_values], axis=0)
    else:
        shap_values_avg = np.abs(shap_values)
    
    # Compute statistics
    mean_absolute_shap = np.mean(shap_values_avg, axis=0).tolist()
    std_absolute_shap = np.std(shap_values_avg, axis=0).tolist()
    
    # Feature importance ranking
    feature_importance = np.mean(shap_values_avg, axis=0)
    feature_importance_rank = np.argsort(feature_importance)[::-1].tolist()
    
    # Expected base value
    expected_base_value = float(np.mean(explainer.expected_value))
    if isinstance(expected_base_value, np.ndarray):
        expected_base_value = float(expected_base_value[0])
    
    return {
        'mean_absolute_shap': mean_absolute_shap[:10],  # Top 10 features
        'std_absolute_shap': std_absolute_shap[:10],
        'feature_importance_rank': feature_importance_rank[:10],
        'expected_base_value': expected_base_value
    }


def generate_shap_baseline_for_anomaly(model, model_name: str, n_features: int, n_samples: int = 1000) -> Dict:
    """
    Generate SHAP baseline for anomaly detection model.
    """
    # Generate background data
    background_data = np.random.randn(n_samples, n_features)
    
    # Create SHAP explainer
    explainer = shap.TreeExplainer(model)
    
    # Compute SHAP values
    shap_values = explainer.shap_values(background_data)
    shap_values_abs = np.abs(shap_values)
    
    # Compute statistics
    mean_absolute_shap = np.mean(shap_values_abs, axis=0).tolist()
    std_absolute_shap = np.std(shap_values_abs, axis=0).tolist()
    
    # Feature importance ranking
    feature_importance = np.mean(shap_values_abs, axis=0)
    feature_importance_rank = np.argsort(feature_importance)[::-1].tolist()
    
    # Expected base value
    expected_base_value = float(np.mean(explainer.expected_value))
    
    return {
        'mean_absolute_shap': mean_absolute_shap[:10],
        'std_absolute_shap': std_absolute_shap[:10],
        'feature_importance_rank': feature_importance_rank[:10],
        'expected_base_value': expected_base_value
    }


def main():
    """Generate SHAP baselines for all models."""
    print("=" * 80)
    print("RansomEye Baseline Intelligence Pack - SHAP Baseline Generation")
    print("=" * 80)
    print()
    
    # Ensure SHAP directory exists
    SHAP_DIR.mkdir(parents=True, exist_ok=True)
    
    # Load models
    models = {}
    
    # Load ransomware behavior model
    model_path = MODELS_DIR / "ransomware_behavior.model"
    if not model_path.exists():
        print(f"ERROR: Model not found: {model_path}", file=sys.stderr)
        print("Run train_baseline_models.py first", file=sys.stderr)
        sys.exit(1)
    
    with open(model_path, 'rb') as f:
        models['ransomware_behavior.model'] = pickle.load(f)
    
    print("  ✓ Loaded ransomware_behavior.model")
    
    # Load anomaly baseline model
    model_path = MODELS_DIR / "anomaly_baseline.model"
    with open(model_path, 'rb') as f:
        models['anomaly_baseline.model'] = pickle.load(f)
    
    print("  ✓ Loaded anomaly_baseline.model")
    
    # Load confidence calibration model
    model_path = MODELS_DIR / "confidence_calibration.model"
    with open(model_path, 'rb') as f:
        models['confidence_calibration.model'] = pickle.load(f)
    
    print("  ✓ Loaded confidence_calibration.model")
    print()
    
    # Generate SHAP baselines
    shap_baselines = {
        'baseline_shap_version': '1.0.0',
        'generated': datetime.utcnow().isoformat() + 'Z',
        'models': {}
    }
    
    # Generate for ransomware behavior model
    print("Generating SHAP baseline for ransomware_behavior.model...")
    shap_baseline = generate_shap_baseline_for_classifier(
        models['ransomware_behavior.model'],
        'ransomware_behavior.model',
        n_features=256,
        n_samples=1000
    )
    
    # Add validation thresholds
    shap_baseline['validation_thresholds'] = {
        'min_shap_sum': -1.0,
        'max_shap_sum': 1.0,
        'min_feature_contribution': 0.01,
        'max_feature_contribution': 0.5
    }
    
    shap_baselines['models']['ransomware_behavior.model'] = shap_baseline
    print("  ✓ Generated SHAP baseline")
    print()
    
    # Generate for anomaly baseline model
    print("Generating SHAP baseline for anomaly_baseline.model...")
    shap_baseline = generate_shap_baseline_for_anomaly(
        models['anomaly_baseline.model'],
        'anomaly_baseline.model',
        n_features=128,
        n_samples=1000
    )
    
    shap_baseline['validation_thresholds'] = {
        'min_shap_sum': -0.5,
        'max_shap_sum': 0.5,
        'min_feature_contribution': 0.005,
        'max_feature_contribution': 0.3
    }
    
    shap_baselines['models']['anomaly_baseline.model'] = shap_baseline
    print("  ✓ Generated SHAP baseline")
    print()
    
    # Generate for confidence calibration model
    print("Generating SHAP baseline for confidence_calibration.model...")
    # Extract base estimator from calibrated model
    calibrated_model = models['confidence_calibration.model']
    # Access base estimator - in sklearn, calibrated models have calibrated_classifiers_ list
    if hasattr(calibrated_model, 'calibrated_classifiers_'):
        base_model = calibrated_model.calibrated_classifiers_[0].estimator
    elif hasattr(calibrated_model, 'base_estimator'):
        base_model = calibrated_model.base_estimator
    else:
        # Fallback: use the calibrated model directly (SHAP can work with it)
        base_model = calibrated_model
    
    shap_baseline = generate_shap_baseline_for_classifier(
        base_model,
        'confidence_calibration.model',
        n_features=64,
        n_samples=1000
    )
    
    shap_baseline['validation_thresholds'] = {
        'min_shap_sum': -0.8,
        'max_shap_sum': 0.8,
        'min_feature_contribution': 0.01,
        'max_feature_contribution': 0.4
    }
    
    shap_baselines['models']['confidence_calibration.model'] = shap_baseline
    print("  ✓ Generated SHAP baseline")
    print()
    
    # Save SHAP baselines
    shap_path = SHAP_DIR / "baseline_shap_values.json"
    with open(shap_path, 'w') as f:
        json.dump(shap_baselines, f, indent=2)
    
    print(f"  ✓ Saved SHAP baselines: {shap_path}")
    print()
    
    print("=" * 80)
    print("✓ SHAP baseline generation complete")
    print("=" * 80)


if __name__ == '__main__':
    main()

