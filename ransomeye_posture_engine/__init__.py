# Path and File Name : /home/ransomeye/rebuild/ransomeye_posture_engine/__init__.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Package initialization for RansomEye Posture Engine

"""
RansomEye Posture Engine

Host & Network Management Posture (HNMP) evaluation engine.
Ingests telemetry from Linux Agent, Windows Agent, and DPI Probe.
Evaluates against CIS Benchmarks, STIG profiles, and custom YAML policies.
Produces advisory-only posture scores and compliance reports.
"""

__version__ = "1.0.0"
__author__ = "nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU"

# Systemd launcher contract:
# /opt/ransomeye/bin/ransomeye-posture_engine executes `from ransomeye_posture_engine import main`
from .service_main import main  # noqa: F401

