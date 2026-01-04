#!/usr/bin/env python3
# Path and File Name: /home/ransomeye/rebuild/register_models.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: Register all trained models into database model registry

"""
Model Registry Registration Script
Registers all trained models into ransomeye.model_registry and ransomeye.model_versions tables.
"""

import os
import sys
import json
import hashlib
from pathlib import Path
from datetime import datetime
from typing import Dict, Optional

try:
    import psycopg2
    from psycopg2.extras import Json
except ImportError:
    print("ERROR: psycopg2 not installed. Install with: pip install psycopg2-binary", file=sys.stderr)
    sys.exit(1)

# Database connection
DB_HOST = os.environ.get("DB_HOST", "localhost")
DB_PORT = int(os.environ.get("DB_PORT", "5432"))
DB_NAME = os.environ.get("DB_NAME", "ransomeye")
DB_USER = os.environ.get("DB_USER", "gagan")
DB_PASS = os.environ.get("DB_PASS", "gagan")

# Model paths
BASELINE_PACK_DIR = Path("/home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack")
CORE_AI_MODELS_DIR = Path("/home/ransomeye/rebuild/core/ai/models")


def hex_to_bytea(hex_str: str) -> bytes:
    """Convert hex string (with or without 'sha256:' prefix) to bytes."""
    hex_str = hex_str.replace("sha256:", "").strip()
    return bytes.fromhex(hex_str)


def get_model_task_type(model_type: str) -> str:
    """Map model type to model_task_type enum."""
    mapping = {
        "behavior_classifier": "classification",
        "anomaly_detector": "anomaly_detection",
        "calibration": "classification",
        "RandomForestClassifier": "classification",
    }
    return mapping.get(model_type, "other")


def register_baseline_models(conn):
    """Register baseline pack models."""
    manifest_path = BASELINE_PACK_DIR / "models" / "model_manifest.json"
    
    if not manifest_path.exists():
        print(f"ERROR: Manifest not found: {manifest_path}")
        return False
    
    with open(manifest_path, 'r') as f:
        manifest = json.load(f)
    
    cur = conn.cursor()
    
    for model_info in manifest.get("models", []):
        model_name = model_info["name"].replace(".model", "")
        model_type = model_info.get("type", "other")
        version = model_info.get("version", "1.0.0")
        artifact_hash_hex = model_info.get("hash", "").replace("sha256:", "")
        training_data_hash_hex = model_info.get("training_data_hash", "").replace("sha256:", "")
        shap_file = model_info.get("shap_file", "")
        shap_required = model_info.get("shap_required", True)
        
        # Model file path
        model_path = BASELINE_PACK_DIR / "models" / model_info["name"]
        artifact_uri = str(model_path)
        
        # SHAP artifact path
        shap_artifact_uri = None
        shap_artifact_hash = None
        if shap_file:
            shap_path = BASELINE_PACK_DIR / shap_file
            if shap_path.exists():
                shap_artifact_uri = str(shap_path)
                # Compute SHAP file hash
                with open(shap_path, 'rb') as f:
                    shap_artifact_hash = hashlib.sha256(f.read()).digest()
        
        # Hyperparameters
        hyperparameters = {
            "algorithm": model_info.get("algorithm", ""),
            "features": model_info.get("features", 0),
        }
        if "contamination" in model_info:
            hyperparameters["contamination"] = model_info["contamination"]
        if "accuracy" in model_info:
            hyperparameters["accuracy"] = model_info["accuracy"]
        
        # Metadata
        metadata = {
            "model_name": model_name,
            "model_type": model_type,
            "version": version,
            "trained_on": manifest.get("trained_on", ""),
            "training_methodology": manifest.get("training_methodology", ""),
        }
        if "accuracy" in model_info:
            metadata["accuracy"] = model_info["accuracy"]
        if "precision" in model_info:
            metadata["precision"] = model_info["precision"]
        if "recall" in model_info:
            metadata["recall"] = model_info["recall"]
        if "f1_score" in model_info:
            metadata["f1_score"] = model_info["f1_score"]
        
        try:
            # Insert into model_registry
            cur.execute("""
                INSERT INTO ransomeye.model_registry (model_name, model_task, description, is_active)
                VALUES (%s, %s, %s, %s)
                ON CONFLICT (model_name) DO UPDATE
                SET model_task = EXCLUDED.model_task,
                    description = EXCLUDED.description,
                    updated_at = now()
                RETURNING model_id
            """, (
                model_name,
                get_model_task_type(model_type),
                f"{model_type} model for {model_name}",
                True
            ))
            
            model_id = cur.fetchone()[0]
            print(f"✓ Registered model: {model_name} (ID: {model_id})")
            
            # Insert into model_versions
            artifact_hash_bytes = hex_to_bytea(artifact_hash_hex)
            training_data_hash_bytes = hex_to_bytea(training_data_hash_hex) if training_data_hash_hex else None
            
            cur.execute("""
                INSERT INTO ransomeye.model_versions (
                    model_id, version, artifact_type, artifact_uri, artifact_sha256,
                    trained_on, training_data_hash, hyperparameters,
                    shap_enabled, shap_artifact_uri, shap_artifact_sha256,
                    metadata_json
                )
                VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s)
                ON CONFLICT (model_id, version) DO UPDATE
                SET artifact_uri = EXCLUDED.artifact_uri,
                    artifact_sha256 = EXCLUDED.artifact_sha256,
                    trained_on = EXCLUDED.trained_on,
                    training_data_hash = EXCLUDED.training_data_hash,
                    hyperparameters = EXCLUDED.hyperparameters,
                    shap_enabled = EXCLUDED.shap_enabled,
                    shap_artifact_uri = EXCLUDED.shap_artifact_uri,
                    shap_artifact_sha256 = EXCLUDED.shap_artifact_sha256,
                    metadata_json = EXCLUDED.metadata_json
            """, (
                model_id,
                version,
                "pkl",
                artifact_uri,
                artifact_hash_bytes,
                manifest.get("trained_on", ""),
                training_data_hash_bytes,
                Json(hyperparameters),
                shap_required,
                shap_artifact_uri,
                shap_artifact_hash,
                Json(metadata)
            ))
            
            print(f"  ✓ Registered version: {version}")
            
        except Exception as e:
            print(f"ERROR: Failed to register {model_name}: {e}")
            conn.rollback()
            return False
    
    conn.commit()
    return True


def register_risk_model(conn):
    """Register risk_model from core/ai/models."""
    manifest_path = CORE_AI_MODELS_DIR / "models.manifest.json"
    
    if not manifest_path.exists():
        print(f"ERROR: Manifest not found: {manifest_path}")
        return False
    
    with open(manifest_path, 'r') as f:
        manifest = json.load(f)
    
    model_name = manifest.get("model_name", "risk_model")
    version = manifest.get("model_version", "1.0.0")
    model_type = manifest.get("model_type", "RandomForestClassifier")
    artifact_hash_hex = manifest.get("model_hash", "")
    trained_on = manifest.get("trained_on", "")
    features = manifest.get("features", [])
    
    model_path = CORE_AI_MODELS_DIR / "risk_model.model"
    artifact_uri = str(model_path)
    
    # Hyperparameters
    hyperparameters = {
        "model_type": model_type,
        "features": features,
        "n_features": len(features)
    }
    
    # Metadata
    metadata = {
        "model_name": model_name,
        "model_type": model_type,
        "version": version,
        "trained_on": trained_on,
        "features": features,
        "model_size_bytes": manifest.get("model_size_bytes", 0)
    }
    
    cur = conn.cursor()
    
    try:
        # Insert into model_registry
        cur.execute("""
            INSERT INTO ransomeye.model_registry (model_name, model_task, description, is_active)
            VALUES (%s, %s, %s, %s)
            ON CONFLICT (model_name) DO UPDATE
            SET model_task = EXCLUDED.model_task,
                description = EXCLUDED.description,
                updated_at = now()
            RETURNING model_id
        """, (
            model_name,
            get_model_task_type(model_type),
            f"{model_type} model for risk scoring",
            True
        ))
        
        model_id = cur.fetchone()[0]
        print(f"✓ Registered model: {model_name} (ID: {model_id})")
        
        # Insert into model_versions
        artifact_hash_bytes = hex_to_bytea(artifact_hash_hex)
        
        cur.execute("""
            INSERT INTO ransomeye.model_versions (
                model_id, version, artifact_type, artifact_uri, artifact_sha256,
                trained_on, hyperparameters, shap_enabled, metadata_json
            )
            VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s)
            ON CONFLICT (model_id, version) DO UPDATE
            SET artifact_uri = EXCLUDED.artifact_uri,
                artifact_sha256 = EXCLUDED.artifact_sha256,
                trained_on = EXCLUDED.trained_on,
                hyperparameters = EXCLUDED.hyperparameters,
                shap_enabled = EXCLUDED.shap_enabled,
                metadata_json = EXCLUDED.metadata_json
        """, (
            model_id,
            version,
            "pkl",
            artifact_uri,
            artifact_hash_bytes,
            trained_on,
            Json(hyperparameters),
            True,  # SHAP required
            Json(metadata)
        ))
        
        print(f"  ✓ Registered version: {version}")
        conn.commit()
        return True
        
    except Exception as e:
        print(f"ERROR: Failed to register {model_name}: {e}")
        conn.rollback()
        return False


def main():
    """Main registration function."""
    print("=" * 80)
    print("RansomEye Model Registry Registration")
    print("=" * 80)
    print()
    
    # Connect to database
    try:
        conn = psycopg2.connect(
            host=DB_HOST,
            port=DB_PORT,
            database=DB_NAME,
            user=DB_USER,
            password=DB_PASS
        )
        print(f"✓ Connected to database: {DB_NAME}")
    except Exception as e:
        print(f"ERROR: Failed to connect to database: {e}")
        sys.exit(1)
    
    try:
        # Register baseline models
        print("\nRegistering baseline pack models...")
        if not register_baseline_models(conn):
            print("ERROR: Failed to register baseline models")
            sys.exit(1)
        
        # Register risk model
        print("\nRegistering risk model...")
        if not register_risk_model(conn):
            print("ERROR: Failed to register risk model")
            sys.exit(1)
        
        print("\n" + "=" * 80)
        print("✓ All models registered successfully")
        print("=" * 80)
        
    finally:
        conn.close()


if __name__ == "__main__":
    main()

