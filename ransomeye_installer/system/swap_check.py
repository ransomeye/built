# Path and File Name : /home/ransomeye/rebuild/ransomeye_installer/system/swap_check.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Enforces swap space requirement - >=16GB or equal to RAM for Core/DPI

"""
Swap Check: Enforces swap space requirements.
Core and DPI require swap >= 16GB or equal to RAM (whichever is larger).
"""

import os
import subprocess
import sys
from typing import Dict, Tuple, Optional


class SwapCheck:
    """Validates swap space availability."""
    
    MIN_SWAP_GB = 16  # Minimum 16GB swap required
    SWAP_TO_RAM_RATIO = 1.0  # Swap should be at least equal to RAM
    
    def __init__(self):
        self.swap_info: Optional[Dict] = None
        self.ram_info: Optional[Dict] = None
        self._detect_swap()
        self._detect_ram()
    
    def _detect_swap(self) -> None:
        """Detect swap space."""
        try:
            # Read /proc/swaps
            swap_total_kb = 0
            swap_used_kb = 0
            
            if os.path.exists('/proc/swaps'):
                with open('/proc/swaps', 'r') as f:
                    lines = f.readlines()[1:]  # Skip header
                    for line in lines:
                        parts = line.split()
                        if len(parts) >= 3:
                            size_kb = int(parts[2])
                            swap_total_kb += size_kb
            
            # Also check /proc/meminfo for swap
            if os.path.exists('/proc/meminfo'):
                with open('/proc/meminfo', 'r') as f:
                    for line in f:
                        if line.startswith('SwapTotal:'):
                            swap_total_kb = int(line.split()[1])
                        elif line.startswith('SwapFree:'):
                            swap_free_kb = int(line.split()[1])
                            swap_used_kb = swap_total_kb - swap_free_kb
            
            self.swap_info = {
                'total_kb': swap_total_kb,
                'used_kb': swap_used_kb,
                'free_kb': swap_total_kb - swap_used_kb,
                'total_gb': swap_total_kb / (1024 * 1024),
                'used_gb': swap_used_kb / (1024 * 1024),
                'free_gb': (swap_total_kb - swap_used_kb) / (1024 * 1024)
            }
        except Exception:
            self.swap_info = {
                'total_kb': 0,
                'used_kb': 0,
                'free_kb': 0,
                'total_gb': 0,
                'used_gb': 0,
                'free_gb': 0
            }
    
    def _detect_ram(self) -> None:
        """Detect RAM size."""
        try:
            if os.path.exists('/proc/meminfo'):
                with open('/proc/meminfo', 'r') as f:
                    for line in f:
                        if line.startswith('MemTotal:'):
                            mem_total_kb = int(line.split()[1])
                            self.ram_info = {
                                'total_kb': mem_total_kb,
                                'total_gb': mem_total_kb / (1024 * 1024)
                            }
                            return
        except Exception:
            pass
        
        self.ram_info = {
            'total_kb': 0,
            'total_gb': 0
        }
    
    def check_swap(self) -> Tuple[bool, str, Dict]:
        """
        Check if swap meets requirements.
        
        Returns:
            Tuple of (meets_requirements: bool, message: str, info: Dict)
        """
        if self.swap_info is None or self.ram_info is None:
            return False, "Could not detect swap or RAM information", {}
        
        swap_gb = self.swap_info['total_gb']
        ram_gb = self.ram_info['total_gb']
        
        # Calculate required swap (max of MIN_SWAP_GB or RAM size)
        required_swap_gb = max(self.MIN_SWAP_GB, ram_gb * self.SWAP_TO_RAM_RATIO)
        
        if swap_gb < required_swap_gb:
            return False, f"Insufficient swap: {swap_gb:.2f}GB (required: {required_swap_gb:.2f}GB - max of {self.MIN_SWAP_GB}GB or {ram_gb:.2f}GB RAM)", {
                'swap_gb': swap_gb,
                'ram_gb': ram_gb,
                'required_gb': required_swap_gb
            }
        
        return True, f"Swap space adequate: {swap_gb:.2f}GB (required: {required_swap_gb:.2f}GB)", {
            'swap_gb': swap_gb,
            'ram_gb': ram_gb,
            'required_gb': required_swap_gb
        }
    
    def get_swap_info(self) -> Dict:
        """Get swap information."""
        return self.swap_info.copy() if self.swap_info else {}
    
    def get_ram_info(self) -> Dict:
        """Get RAM information."""
        return self.ram_info.copy() if self.ram_info else {}


def main():
    """CLI entry point for swap check."""
    checker = SwapCheck()
    meets_requirements, message, info = checker.check_swap()
    
    print(f"RAM: {checker.get_ram_info().get('total_gb', 0):.2f}GB")
    print(f"Swap: {checker.get_swap_info().get('total_gb', 0):.2f}GB")
    print(f"Status: {message}")
    
    if not meets_requirements:
        sys.exit(1)
    
    sys.exit(0)


if __name__ == '__main__':
    main()

