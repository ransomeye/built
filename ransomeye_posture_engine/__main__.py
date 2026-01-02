# Path and File Name : /home/ransomeye/rebuild/ransomeye_posture_engine/__main__.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Module entry point enabling python3 -m ransomeye_posture_engine invocation

"""
Module entry point for python3 -m ransomeye_posture_engine.
Enables deterministic invocation without relying on 'python' command.
"""

import sys
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from ransomeye_posture_engine.service_main import main

if __name__ == '__main__':
    sys.exit(main())

