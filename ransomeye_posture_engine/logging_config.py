# Path and File Name : /home/ransomeye/rebuild/ransomeye_posture_engine/logging_config.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Logging configuration for posture engine

"""
Logging configuration for Posture Engine.
"""

import logging
import sys
from pathlib import Path
import os

def setup_logging():
    """Configure logging for posture engine."""
    
    log_level = os.environ.get("POSTURE_LOG_LEVEL", "INFO").upper()
    
    # Create log directory
    log_dir = Path(os.environ.get("POSTURE_AUDIT_LOG_DIR", 
        "/home/ransomeye/rebuild/logs/posture_engine"))
    log_dir.mkdir(parents=True, exist_ok=True)
    
    # Configure root logger
    logger = logging.getLogger("ransomeye_posture_engine")
    logger.setLevel(getattr(logging, log_level, logging.INFO))
    
    # Remove existing handlers
    logger.handlers.clear()
    
    # Console handler
    console_handler = logging.StreamHandler(sys.stdout)
    console_handler.setLevel(logging.INFO)
    console_formatter = logging.Formatter(
        '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )
    console_handler.setFormatter(console_formatter)
    logger.addHandler(console_handler)
    
    # File handler
    file_handler = logging.FileHandler(log_dir / "posture_engine.log")
    file_handler.setLevel(logging.DEBUG)
    file_formatter = logging.Formatter(
        '%(asctime)s - %(name)s - %(levelname)s - [%(filename)s:%(lineno)d] - %(message)s'
    )
    file_handler.setFormatter(file_formatter)
    logger.addHandler(file_handler)
    
    # Audit log handler (separate file for audit trail)
    audit_handler = logging.FileHandler(log_dir / "audit.log")
    audit_handler.setLevel(logging.INFO)
    audit_formatter = logging.Formatter(
        '%(asctime)s - AUDIT - %(message)s'
    )
    audit_handler.setFormatter(audit_formatter)
    
    audit_logger = logging.getLogger("ransomeye_posture_engine.audit")
    audit_logger.addHandler(audit_handler)
    audit_logger.setLevel(logging.INFO)
    audit_logger.propagate = False
    
    return logger

