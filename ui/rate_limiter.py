# Path and File Name : /home/ransomeye/rebuild/ui/rate_limiter.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: Token-based rate limiter for share endpoint - sliding window per token

"""
Rate Limiter for Share Endpoint:
- Per-token sliding window rate limiting
- Configurable via environment variables
- Fail-closed on abuse (429 Too Many Requests)
- Thread-safe for concurrent access
"""

import os
import time
import threading
from typing import Dict, Optional
from collections import deque
from datetime import datetime, timezone

logger = None  # Will be initialized on first use


class TokenRateLimiter:
    """
    Per-token rate limiter using sliding window algorithm.
    
    Thread-safe implementation for concurrent Flask requests.
    """
    
    def __init__(self, requests_per_minute: int = 60, burst_allowance: int = 10):
        """
        Initialize rate limiter.
        
        Args:
            requests_per_minute: Maximum requests per minute per token
            burst_allowance: Additional burst requests allowed (beyond per-minute limit)
        """
        self.requests_per_minute = requests_per_minute
        self.burst_allowance = burst_allowance
        self.max_requests = requests_per_minute + burst_allowance
        
        # Per-token sliding windows: {token: deque of timestamps}
        self._windows: Dict[str, deque] = {}
        self._lock = threading.Lock()
        
        # Cleanup thread for old entries (runs every 5 minutes)
        self._cleanup_interval = 300  # 5 minutes
        self._last_cleanup = time.time()
    
    def is_allowed(self, token: str) -> tuple[bool, Optional[str]]:
        """
        Check if request is allowed for token.
        
        Args:
            token: Share token to check
            
        Returns:
            Tuple of (is_allowed: bool, reason: Optional[str])
            - If allowed: (True, None)
            - If rate-limited: (False, "rate_limit_exceeded")
        """
        current_time = time.time()
        window_start = current_time - 60.0  # 1 minute window
        
        with self._lock:
            # Get or create window for token
            if token not in self._windows:
                self._windows[token] = deque()
            
            window = self._windows[token]
            
            # Remove timestamps outside 1-minute window
            while window and window[0] < window_start:
                window.popleft()
            
            # Check if within limits
            if len(window) >= self.max_requests:
                # Rate limit exceeded
                return False, "rate_limit_exceeded"
            
            # Add current request timestamp
            window.append(current_time)
            
            # Periodic cleanup of old tokens (every 5 minutes)
            if current_time - self._last_cleanup > self._cleanup_interval:
                self._cleanup_old_tokens(current_time)
                self._last_cleanup = current_time
            
            return True, None
    
    def _cleanup_old_tokens(self, current_time: float):
        """
        Remove tokens with no recent activity (older than 1 hour).
        
        Args:
            current_time: Current timestamp
        """
        cutoff_time = current_time - 3600.0  # 1 hour
        
        tokens_to_remove = []
        for token, window in self._windows.items():
            # Remove old timestamps
            while window and window[0] < cutoff_time:
                window.popleft()
            
            # Remove token if window is empty
            if not window:
                tokens_to_remove.append(token)
        
        for token in tokens_to_remove:
            del self._windows[token]
    
    def get_stats(self, token: str) -> Dict:
        """
        Get rate limit statistics for a token (for debugging/monitoring).
        
        Args:
            token: Share token
            
        Returns:
            Dict with request count and window info
        """
        current_time = time.time()
        window_start = current_time - 60.0
        
        with self._lock:
            if token not in self._windows:
                return {
                    "token": token[:16] + "...",
                    "requests_in_window": 0,
                    "max_requests": self.max_requests,
                    "rate_limited": False
                }
            
            window = self._windows[token]
            
            # Remove old timestamps
            while window and window[0] < window_start:
                window.popleft()
            
            return {
                "token": token[:16] + "...",
                "requests_in_window": len(window),
                "max_requests": self.max_requests,
                "rate_limited": len(window) >= self.max_requests
            }


# Global rate limiter instance (initialized on first use)
_rate_limiter: Optional[TokenRateLimiter] = None
_limiter_lock = threading.Lock()


def get_rate_limiter() -> TokenRateLimiter:
    """
    Get or create global rate limiter instance.
    
    Returns:
        TokenRateLimiter instance
    """
    global _rate_limiter
    
    if _rate_limiter is None:
        with _limiter_lock:
            if _rate_limiter is None:
                # Read configuration from environment
                requests_per_minute = int(os.environ.get(
                    'RANSOMEYE_SHARE_RATE_LIMIT_PER_MINUTE', '60'
                ))
                burst_allowance = int(os.environ.get(
                    'RANSOMEYE_SHARE_RATE_LIMIT_BURST', '10'
                ))
                
                # Validate limits
                if requests_per_minute < 1:
                    requests_per_minute = 60
                if burst_allowance < 0:
                    burst_allowance = 10
                
                _rate_limiter = TokenRateLimiter(
                    requests_per_minute=requests_per_minute,
                    burst_allowance=burst_allowance
                )
                
                # Initialize logger if not already done
                global logger
                if logger is None:
                    import logging
                    logger = logging.getLogger(__name__)
                
                logger.info(
                    f"Rate limiter initialized: {requests_per_minute} req/min + "
                    f"{burst_allowance} burst = {requests_per_minute + burst_allowance} max"
                )
    
    return _rate_limiter

