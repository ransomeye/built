# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/__init__.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Package initialization for RansomEye Intelligence System

"""
RansomEye Intelligence System Package
Provides Day-1 operational AI models, threat intelligence, and LLM RAG knowledge.
"""

__version__ = "1.0.0"

# Systemd launcher contract:
# /opt/ransomeye/bin/ransomeye-intelligence executes `from ransomeye_intelligence import main`
from .service_main import main  # noqa: F401

