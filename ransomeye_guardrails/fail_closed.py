# Path and File Name : /home/ransomeye/rebuild/ransomeye_guardrails/fail_closed.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Central abort mechanism for guardrail violations - ensures fail-closed behavior

"""
Central fail-closed mechanism for RansomEye guardrails.
Any violation triggers immediate build/runtime failure.
"""

import sys
import logging
from typing import Optional
from datetime import datetime

logging.basicConfig(
    level=logging.ERROR,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


class GuardrailViolation(Exception):
    """Exception raised when a guardrail is violated."""
    
    def __init__(self, rule_name: str, violation_details: str, file_path: Optional[str] = None):
        self.rule_name = rule_name
        self.violation_details = violation_details
        self.file_path = file_path
        self.timestamp = datetime.utcnow().isoformat()
        message = f"[GUARDRAIL VIOLATION] {rule_name}: {violation_details}"
        if file_path:
            message += f" | File: {file_path}"
        super().__init__(message)


def fail_closed(rule_name: str, violation_details: str, file_path: Optional[str] = None, exit_code: int = 1) -> None:
    """
    Fail-closed mechanism: logs violation and exits immediately.
    
    Args:
        rule_name: Name of the violated guardrail rule
        violation_details: Detailed description of the violation
        file_path: Optional path to the file containing the violation
        exit_code: Exit code to use (default: 1)
    
    Raises:
        SystemExit: Always exits with the specified exit code
    """
    violation = GuardrailViolation(rule_name, violation_details, file_path)
    logger.error(str(violation))
    
    # Write to stderr for CI/CD visibility
    print(f"\n{'='*80}", file=sys.stderr)
    print(f"RANSOMEYE GUARDRAIL VIOLATION - BUILD FAILED", file=sys.stderr)
    print(f"{'='*80}", file=sys.stderr)
    print(f"Rule: {rule_name}", file=sys.stderr)
    print(f"Details: {violation_details}", file=sys.stderr)
    if file_path:
        print(f"File: {file_path}", file=sys.stderr)
    print(f"Timestamp: {violation.timestamp}", file=sys.stderr)
    print(f"{'='*80}\n", file=sys.stderr)
    
    sys.exit(exit_code)


def validate_and_fail(condition: bool, rule_name: str, violation_details: str, file_path: Optional[str] = None) -> None:
    """
    Validate a condition and fail-closed if False.
    
    Args:
        condition: Condition that must be True
        rule_name: Name of the guardrail rule
        violation_details: Description if condition is False
        file_path: Optional file path for context
    """
    if not condition:
        fail_closed(rule_name, violation_details, file_path)

