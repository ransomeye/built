# Path and File Name : /home/ransomeye/rebuild/core/install_state/__init__.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Install state module initialization

"""
RansomEye Install State Module

This module provides cryptographically-signed installation state tracking
for fail-closed security enforcement. It ensures that database services
only start when explicitly enabled and properly configured.
"""

from .finalize_install_state import finalize_install_state, verify_install_state
from .state_signer import sign_state_file, verify_state_signature

__all__ = [
    'finalize_install_state',
    'verify_install_state',
    'sign_state_file',
    'verify_state_signature'
]

__version__ = "1.0.0"

