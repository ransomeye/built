# Path and File Name : /home/ransomeye/rebuild/ransomeye_db_core/__init__.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Package initialization and systemd entrypoint export for RansomEye DB Core service

"""
RansomEye DB Core Package

Systemd launcher contract:
- /opt/ransomeye/bin/ransomeye-db_core imports `main` from this package.
"""

from .service_main import main  # noqa: F401


