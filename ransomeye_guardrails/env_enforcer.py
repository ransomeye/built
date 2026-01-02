# Path and File Name : /home/ransomeye/rebuild/ransomeye_guardrails/env_enforcer.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Enforces ENV-only configuration - no hardcoded values allowed

"""
ENV enforcer: ensures all configuration is environment-driven.
No hardcoded IPs, ports, paths, or secrets allowed (except localhost).
"""

import os
import re
import ast
from pathlib import Path
from typing import List, Dict, Optional, Set
import yaml

from .fail_closed import fail_closed


class EnvEnforcer:
    """Enforces ENV-only configuration policy."""
    
    # Patterns that indicate hardcoded configuration
    HARDCODED_PATTERNS = [
        # IP addresses (excluding localhost)
        (r'\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b',
         'Hardcoded IPv4 address (use ENV variable)',
         r'127\.0\.0\.1|localhost|0\.0\.0\.0'),
        
        # Port numbers in common contexts
        (r'(?:port|PORT)\s*[=:]\s*["\']?\d{4,5}["\']?',
         'Hardcoded port number (use ENV variable)',
         None),
        
        # Database connection strings
        (r'(?:postgresql|postgres|mysql|mongodb)://[^\s\'"`]+',
         'Hardcoded database connection string (use ENV variables)',
         None),
        
        # File paths (excluding project root)
        (r'["\'](/usr|/var|/etc|/opt|/tmp|/home/[^/]+(?!ransomeye/rebuild))[^\'"`]*["\']',
         'Hardcoded external path (use ENV variable)',
         None),
    ]
    
    # ENV variable access patterns (allowed)
    ENV_ACCESS_PATTERNS = [
        r'os\.environ\.get\(',
        r'os\.getenv\(',
        r'process\.env\.',
        r'\$\{[A-Z_][A-Z0-9_]*\}',
        r'\$[A-Z_][A-Z0-9_]*',
    ]
    
    def __init__(self, project_root: str = "/home/ransomeye/rebuild"):
        self.project_root = Path(project_root).resolve()
        self.violations: List[Dict] = []
        self.required_env_vars = self._load_required_env_vars()
    
    def _load_required_env_vars(self) -> Set[str]:
        """Load list of required ENV variables from project documentation."""
        # These are the mandatory ENV variables from the project spec
        return {
            'DB_HOST', 'DB_PORT', 'DB_NAME', 'DB_USER', 'DB_PASS',
            'MODEL_DIR', 'PDF_ENGINE_PATH', 'OUTPUT_DIR', 'CORE_API_PORT',
            'CHAIN_OUTPUT_PATH', 'SUMMARY_EXPORT_PATH', 'FRONTEND_PORT',
            'BACKEND_API_PORT', 'RETENTION_YEARS', 'DB_ENCRYPTION_KEY_PATH',
            'CAPTURE_IFACE', 'COMPLIANCE_CONFIG', 'ASSISTANT_TOPK',
            'ASSISTANT_MAX_TOKENS', 'ASSISTANT_DATA_DIR', 'ENABLE_EBPF',
            'ENABLE_KERNEL_DRIVER', 'UPDATE_BUNDLE_DIR',
            'MISP_URL', 'MISP_KEY', 'OTX_URL', 'OTX_KEY', 'TALOS_URL',
            'TALOS_KEY', 'THREATFOX_URL', 'THREATFOX_KEY'
        }
    
    def _check_file_for_hardcoded_values(self, file_path: Path) -> List[Dict]:
        """Check a file for hardcoded configuration values."""
        violations = []
        
        try:
            with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
        except Exception:
            return violations
        
        lines = content.split('\n')
        
        for pattern, description, exception in self.HARDCODED_PATTERNS:
            regex = re.compile(pattern, re.IGNORECASE | re.MULTILINE)
            
            for match in regex.finditer(content):
                matched_text = match.group(0)
                
                # Check exception
                if exception:
                    exception_regex = re.compile(exception, re.IGNORECASE)
                    if exception_regex.search(matched_text):
                        continue
                
                # Check if it's in a comment
                line_num = content[:match.start()].count('\n') + 1
                line_content = lines[line_num - 1] if line_num <= len(lines) else ''
                
                # Skip if in comment (simple check)
                if '#' in line_content and match.start() - content[:match.start()].rfind('\n') > line_content.find('#'):
                    continue
                
                # Check if nearby ENV access exists (might be acceptable)
                start = max(0, match.start() - 100)
                end = min(len(content), match.end() + 100)
                context = content[start:end]
                
                has_env_access = any(re.search(pat, context, re.IGNORECASE) for pat in self.ENV_ACCESS_PATTERNS)
                
                violations.append({
                    'file_path': str(file_path),
                    'line': line_num,
                    'description': description,
                    'matched_text': matched_text[:50],
                    'has_env_context': has_env_access
                })
        
        return violations
    
    def _check_ast_for_hardcoded_strings(self, file_path: Path) -> List[Dict]:
        """Use AST to find hardcoded string literals in Python files."""
        if file_path.suffix != '.py':
            return []
        
        violations = []
        
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                tree = ast.parse(f.read(), filename=str(file_path))
        except Exception:
            return violations
        
        # Look for string literals that look like IPs, URLs, or paths
        for node in ast.walk(tree):
            if isinstance(node, ast.Str) or (isinstance(node, ast.Constant) and isinstance(node.value, str)):
                value = node.value if isinstance(node, ast.Str) else node.value
                
                # Check for IP address
                ip_pattern = re.compile(r'\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b')
                if ip_pattern.search(value) and 'localhost' not in value and '127.0.0.1' not in value:
                    violations.append({
                        'file_path': str(file_path),
                        'line': node.lineno if hasattr(node, 'lineno') else 0,
                        'description': 'Hardcoded IP address in string literal (use ENV variable)',
                        'matched_text': value[:50]
                    })
                
                # Check for URLs
                url_pattern = re.compile(r'(?:https?|ftp)://(?!localhost|127\.0\.0\.1)[^\s]+')
                if url_pattern.search(value):
                    violations.append({
                        'file_path': str(file_path),
                        'line': node.lineno if hasattr(node, 'lineno') else 0,
                        'description': 'Hardcoded URL in string literal (use ENV variable)',
                        'matched_text': value[:50]
                    })
        
        return violations
    
    def check_file(self, file_path: Path) -> List[Dict]:
        """Check a single file for ENV violations."""
        violations = []
        
        # Skip excluded files
        rel_path = str(file_path.relative_to(self.project_root))
        if any(excluded in rel_path for excluded in ['__pycache__', '.git', 'node_modules', 'venv', '.venv']):
            return violations
        
        violations.extend(self._check_file_for_hardcoded_values(file_path))
        violations.extend(self._check_ast_for_hardcoded_strings(file_path))
        
        return violations
    
    def check_directory(self, directory: Optional[Path] = None) -> List[Dict]:
        """Check all files in directory tree."""
        if directory is None:
            directory = self.project_root
        
        directory = Path(directory).resolve()
        all_violations = []
        
        for root, dirs, files in os.walk(directory):
            dirs[:] = [d for d in dirs if d not in ['__pycache__', '.git', 'node_modules', 'venv', '.venv']]
            
            for file in files:
                if file.endswith(('.py', '.yaml', '.yml', '.sh', '.ts', '.tsx', '.js', '.jsx')):
                    file_path = Path(root) / file
                    violations = self.check_file(file_path)
                    all_violations.extend(violations)
        
        self.violations = all_violations
        return all_violations
    
    def report_violations(self) -> None:
        """Report violations and fail-closed if any found."""
        if not self.violations:
            print("✓ No ENV configuration violations detected.")
            return
        
        print(f"\n{'='*80}")
        print(f"ENV CONFIGURATION VIOLATIONS: {len(self.violations)}")
        print(f"{'='*80}\n")
        
        for violation in self.violations:
            print(f"File: {violation['file_path']}")
            print(f"  Line: {violation['line']}")
            print(f"  Description: {violation['description']}")
            print(f"  Matched: {violation['matched_text']}")
            print()
        
        # Fail-closed
        fail_closed(
            "ENV_CONFIGURATION_VIOLATION",
            f"Found {len(self.violations)} ENV configuration violation(s). All config must use ENV variables. Build cannot proceed.",
            file_path=None
        )


def main():
    """CLI entry point for ENV enforcer."""
    import argparse
    
    parser = argparse.ArgumentParser(description='RansomEye ENV Enforcer')
    parser.add_argument('--project-root', default='/home/ransomeye/rebuild',
                       help='Project root directory')
    parser.add_argument('--directory', default=None,
                       help='Specific directory to check (default: entire project)')
    
    args = parser.parse_args()
    
    enforcer = EnvEnforcer(args.project_root)
    
    if args.directory:
        violations = enforcer.check_directory(Path(args.directory))
    else:
        violations = enforcer.check_directory()
    
    enforcer.report_violations()


if __name__ == '__main__':
    main()

