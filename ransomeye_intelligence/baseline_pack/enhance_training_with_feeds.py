# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack/enhance_training_with_feeds.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Enhances training data generation with threat intelligence feeds

"""
Enhanced Training Data Generator: Uses threat intelligence feeds to enhance synthetic training data.
Combines synthetic data with real threat intelligence patterns for more realistic training.
"""

import os
import sys
import json
import numpy as np
from pathlib import Path
from typing import Tuple, Dict, List
from datetime import datetime

# Add threat intel to path
sys.path.insert(0, str(Path(__file__).parent.parent / "threat_intel" / "ingestion"))

from malwarebazaar_feed import MalwareBazaarFeedCollector
from wiz_feed import WizFeedCollector
from ransomware_live_feed import RansomwareLiveFeedCollector

BASELINE_PACK_DIR = Path("/home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack")
RANDOM_SEED = 42
np.random.seed(RANDOM_SEED)


def extract_features_from_malwarebazaar(samples: List[Dict]) -> np.ndarray:
    """
    Extract features from MalwareBazaar samples.
    
    Args:
        samples: List of MalwareBazaar sample data
    
    Returns:
        Feature matrix
    """
    features = []
    
    for sample in samples:
        # Extract features from sample metadata
        feature_vector = np.zeros(256)  # Match training feature count
        
        # File type features (0-50)
        file_type = sample.get('file_type', '') or ''
        file_type_lower = file_type.lower() if isinstance(file_type, str) else ''
        if 'pe' in file_type_lower or 'exe' in file_type_lower:
            feature_vector[0:10] = np.random.exponential(2.0, 10)
        elif 'dll' in file_type_lower:
            feature_vector[10:20] = np.random.exponential(1.5, 10)
        
        # Signature features (51-100)
        signature = sample.get('signature', '') or ''
        signature_lower = signature.lower() if isinstance(signature, str) else ''
        if 'ransomware' in signature_lower or 'trojan' in signature_lower:
            feature_vector[51:100] = np.random.poisson(10, 49)
        
        # File size features (101-150)
        file_size = sample.get('file_size', 0)
        if file_size > 0:
            normalized_size = min(1.0, file_size / 10_000_000)  # Normalize to 10MB
            feature_vector[101:150] = normalized_size * np.random.gamma(2, 2, 49)
        
        # Timestamp features (151-200)
        first_seen = sample.get('first_seen', '')
        if first_seen:
            # Use recency as feature
            feature_vector[151:200] = np.random.beta(2, 5, 49) * 10
        
        # Tag features (201-256)
        tags = sample.get('tags', [])
        if tags:
            tag_score = len(tags) / 10.0  # Normalize tag count
            feature_vector[201:256] = tag_score * np.random.exponential(1.5, 55)
        
        features.append(feature_vector)
    
    return np.array(features) if features else np.zeros((0, 256))


def extract_features_from_wiz(iocs: List[Dict]) -> np.ndarray:
    """
    Extract features from Wiz.io IOCs.
    
    Args:
        iocs: List of Wiz.io IOC data
    
    Returns:
        Feature matrix
    """
    features = []
    
    for ioc in iocs:
        feature_vector = np.zeros(256)
        
        # Indicator pattern features (0-50)
        pattern = ioc.get('pattern', '') or ''
        pattern_lower = pattern.lower() if isinstance(pattern, str) else ''
        if 'ipv4' in pattern_lower:
            feature_vector[0:10] = np.random.exponential(2.0, 10)
        elif 'domain' in pattern_lower:
            feature_vector[10:20] = np.random.exponential(1.5, 10)
        
        # Label features (51-100)
        labels = ioc.get('labels', [])
        labels_str = str(labels).lower() if labels else ''
        if 'malicious' in labels_str:
            feature_vector[51:100] = np.random.poisson(10, 49)
        
        # Confidence features (101-150)
        confidence = ioc.get('confidence', 0.5)
        feature_vector[101:150] = confidence * np.random.gamma(2, 2, 49)
        
        features.append(feature_vector)
    
    return np.array(features) if features else np.zeros((0, 256))


def extract_features_from_ransomware_live(data: Dict) -> np.ndarray:
    """
    Extract features from Ransomware.live data.
    
    Args:
        data: Ransomware.live feed data
    
    Returns:
        Feature matrix
    """
    features = []
    
    # Extract from groups
    for group in data.get('groups', []):
        feature_vector = np.zeros(256)
        
        # Group name features (0-50)
        name = group.get('name', '') or ''
        name_lower = name.lower() if isinstance(name, str) else ''
        if name_lower:
            name_hash = hash(name_lower) % 1000 / 1000.0
            feature_vector[0:50] = name_hash * np.random.exponential(2.0, 50)
        
        # Victim count features (51-100)
        victim_count = group.get('victim_count', 0) or 0
        normalized_count = min(1.0, victim_count / 1000.0)
        feature_vector[51:100] = normalized_count * np.random.poisson(10, 49)
        
        features.append(feature_vector)
    
    # Extract from victims
    for victim in data.get('victims', []):
        feature_vector = np.zeros(256)
        
        # Victim name features (0-50)
        name = victim.get('name', '') or ''
        name_lower = name.lower() if isinstance(name, str) else ''
        if name_lower:
            name_hash = hash(name_lower) % 1000 / 1000.0
            feature_vector[0:50] = name_hash * np.random.exponential(2.0, 50)
        
        # Date features (51-100)
        date = victim.get('discovered', '') or ''
        if date:
            feature_vector[51:100] = np.random.poisson(5, 49)
        
        features.append(feature_vector)
    
    return np.array(features) if features else np.zeros((0, 256))


def generate_enhanced_ransomware_data(
    n_samples: int = 100000,
    n_features: int = 256,
    use_feeds: bool = True
) -> Tuple[np.ndarray, np.ndarray]:
    """
    Generate enhanced ransomware behavior data using threat intelligence feeds.
    
    Args:
        n_samples: Number of samples to generate
        n_features: Number of features per sample
        use_feeds: Whether to use threat intelligence feeds
    
    Returns:
        Tuple of (features, labels)
    """
    # Start with synthetic data
    from train_baseline_models import generate_synthetic_ransomware_data
    X, y = generate_synthetic_ransomware_data(n_samples=n_samples, n_features=n_features)
    
    if not use_feeds:
        return X, y
    
    # Enhance with threat intelligence feeds
    print("Enhancing training data with threat intelligence feeds...")
    
    # Load MalwareBazaar samples
    mb_collector = MalwareBazaarFeedCollector()
    mb_samples = mb_collector.load_cached_samples()
    if mb_samples:
        print(f"  ✓ Loaded {len(mb_samples)} MalwareBazaar samples")
        mb_features = extract_features_from_malwarebazaar(mb_samples)
        if len(mb_features) > 0:
            # Blend with synthetic data
            n_blend = min(len(mb_features), n_samples // 10)
            blend_indices = np.random.choice(len(mb_features), n_blend, replace=False)
            synthetic_indices = np.random.choice(n_samples, n_blend, replace=False)
            X[synthetic_indices] = 0.7 * X[synthetic_indices] + 0.3 * mb_features[blend_indices]
            y[synthetic_indices] = 1  # Mark as ransomware
    
    # Load Wiz.io IOCs
    wiz_collector = WizFeedCollector()
    wiz_iocs = wiz_collector.load_cached_feeds()
    if wiz_iocs:
        print(f"  ✓ Loaded {len(wiz_iocs)} Wiz.io IOCs")
        wiz_features = extract_features_from_wiz(wiz_iocs)
        if len(wiz_features) > 0:
            n_blend = min(len(wiz_features), n_samples // 20)
            blend_indices = np.random.choice(len(wiz_features), n_blend, replace=False)
            synthetic_indices = np.random.choice(n_samples, n_blend, replace=False)
            X[synthetic_indices] = 0.8 * X[synthetic_indices] + 0.2 * wiz_features[blend_indices]
    
    # Load Ransomware.live data
    rl_collector = RansomwareLiveFeedCollector()
    rl_data = rl_collector.load_cached_data()
    if rl_data.get('groups') or rl_data.get('victims'):
        print(f"  ✓ Loaded {len(rl_data.get('groups', []))} groups and {len(rl_data.get('victims', []))} victims")
        rl_features = extract_features_from_ransomware_live(rl_data)
        if len(rl_features) > 0:
            n_blend = min(len(rl_features), n_samples // 20)
            blend_indices = np.random.choice(len(rl_features), n_blend, replace=False)
            synthetic_indices = np.random.choice(n_samples, n_blend, replace=False)
            X[synthetic_indices] = 0.8 * X[synthetic_indices] + 0.2 * rl_features[blend_indices]
            y[synthetic_indices] = 1  # Mark as ransomware
    
    print("  ✓ Training data enhanced with threat intelligence")
    return X, y


def main():
    """Test enhanced training data generation."""
    print("=" * 80)
    print("Enhanced Training Data Generator with Threat Intelligence Feeds")
    print("=" * 80)
    print()
    
    # Generate enhanced data
    X, y = generate_enhanced_ransomware_data(n_samples=1000, use_feeds=True)
    
    print(f"Generated {len(X)} samples with {X.shape[1]} features")
    print(f"Ransomware samples: {np.sum(y == 1)}")
    print(f"Suspicious samples: {np.sum(y == 2)}")
    print(f"Benign samples: {np.sum(y == 0)}")
    print()
    print("✓ Enhanced training data generation complete")


if __name__ == '__main__':
    main()

