# Path and File Name : /home/ransomeye/rebuild/ransomeye_posture_engine/config.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Configuration management using environment variables only

"""
Configuration management for Posture Engine.
All configuration via environment variables - ZERO hardcoding.
"""

import os
from pathlib import Path
from typing import Optional
from dataclasses import dataclass

@dataclass
class Config:
    """Posture Engine configuration."""
    
    # Database configuration
    db_host: str
    db_port: int
    db_name: str
    db_user: str
    db_pass: str
    
    # Policy directories
    cis_benchmarks_dir: Path
    stig_profiles_dir: Path
    custom_policies_dir: Path
    
    # Output directories
    output_dir: Path
    audit_log_dir: Path
    
    # Evaluation settings
    evaluation_interval_seconds: int
    drift_detection_window_hours: int
    
    # Signing configuration (Ed25519 ONLY)
    signing_key_path: Optional[Path]
    trust_store_path: Optional[Path]  # Path to trust store with Ed25519 public keys
    
    # API configuration (for telemetry subscription)
    core_api_url: str
    core_api_port: int
    
    @classmethod
    def from_env(cls) -> "Config":
        """Load configuration from environment variables."""
        
        # Database
        db_host = os.environ.get("DB_HOST", "localhost")
        db_port = int(os.environ.get("DB_PORT", "5432"))
        db_name = os.environ.get("DB_NAME", "ransomeye")
        db_user = os.environ.get("DB_USER", "gagan")
        db_pass = os.environ.get("DB_PASS", "gagan")
        
        # Policy directories
        # Runtime-safe default: /opt/ransomeye (never default to build-time tree)
        base_dir = Path(os.environ.get("RANSOMEYE_ROOT", "/opt/ransomeye"))
        cis_benchmarks_dir = Path(os.environ.get("CIS_BENCHMARKS_DIR", 
            str(base_dir / "ransomeye_posture_engine" / "policies" / "cis")))
        stig_profiles_dir = Path(os.environ.get("STIG_PROFILES_DIR",
            str(base_dir / "ransomeye_posture_engine" / "policies" / "stig")))
        custom_policies_dir = Path(os.environ.get("CUSTOM_POLICIES_DIR",
            str(base_dir / "ransomeye_posture_engine" / "policies" / "custom")))
        
        # Output directories
        output_dir = Path(os.environ.get("POSTURE_OUTPUT_DIR",
            str(base_dir / "ransomeye_posture_engine" / "output")))
        audit_log_dir = Path(os.environ.get("POSTURE_AUDIT_LOG_DIR",
            str(base_dir / "logs" / "posture_engine")))
        
        # Evaluation settings
        evaluation_interval_seconds = int(os.environ.get("POSTURE_EVAL_INTERVAL_SEC", "3600"))
        drift_detection_window_hours = int(os.environ.get("POSTURE_DRIFT_WINDOW_HOURS", "24"))
        
        # Signing (Ed25519 ONLY)
        signing_key_path_str = os.environ.get("POSTURE_SIGNING_KEY_PATH")
        signing_key_path = Path(signing_key_path_str) if signing_key_path_str else None
        
        trust_store_path_str = os.environ.get("POSTURE_TRUST_STORE_PATH",
            str(base_dir / "ransomeye_posture_engine" / "trust_store" / "public_keys.json"))
        trust_store_path = Path(trust_store_path_str)
        
        # API configuration
        core_api_url = os.environ.get("CORE_API_URL", "https://localhost")
        core_api_port = int(os.environ.get("CORE_API_PORT", "8443"))
        
        # Create directories
        for dir_path in [cis_benchmarks_dir, stig_profiles_dir, custom_policies_dir, 
                        output_dir, audit_log_dir]:
            dir_path.mkdir(parents=True, exist_ok=True)
        
        return cls(
            db_host=db_host,
            db_port=db_port,
            db_name=db_name,
            db_user=db_user,
            db_pass=db_pass,
            cis_benchmarks_dir=cis_benchmarks_dir,
            stig_profiles_dir=stig_profiles_dir,
            custom_policies_dir=custom_policies_dir,
            output_dir=output_dir,
            audit_log_dir=audit_log_dir,
            evaluation_interval_seconds=evaluation_interval_seconds,
            drift_detection_window_hours=drift_detection_window_hours,
            signing_key_path=signing_key_path,
            trust_store_path=trust_store_path,
            core_api_url=core_api_url,
            core_api_port=core_api_port,
        )

