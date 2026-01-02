# Path and File Name : /home/ransomeye/rebuild/ransomeye_installer/runtime/__init__.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Runtime deployment package initialization

"""
Runtime Deployment Package: Deploys RansomEye to /opt/ransomeye for production runtime.
"""

from .runtime_deployer import RuntimeDeployer

__all__ = ['RuntimeDeployer']

