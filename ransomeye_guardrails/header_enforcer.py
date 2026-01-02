# Path and File Name : /home/ransomeye/rebuild/ransomeye_guardrails/header_enforcer.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Validates mandatory file headers across all project files

"""
Header enforcer: validates mandatory file headers.
Every file must start with the required header format.
"""

import os
from pathlib import Path
from typing import List, Dict, Optional
import re

from .fail_closed import fail_closed


class HeaderEnforcer:
    """Enforces mandatory file headers across the project."""
    
    MANDATORY_HEADER_PATTERN = re.compile(
        r'^#\s*Path and File Name\s*:\s*(.+?)\n'
        r'#\s*Author:\s*(.+?)\n'
        r'#\s*Details of functionality of this file:\s*(.+?)\n',
        re.MULTILINE
    )
    
    REQUIRED_HEADER_LINES = [
        r'#\s*Path and File Name\s*:',
        r'#\s*Author:\s*nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU',
        r'#\s*Details of functionality of this file:'
    ]
    
    EXCLUDED_PATTERNS = [
        '__pycache__',
        '.git',
        'node_modules',
        'venv',
        '.venv',
        'dist',
        'build',
        '.pytest_cache',
        '.mypy_cache',
        'rules.yaml',
        'retention.txt',
        'key_hierarchy.json',
        '.env.example',
        '.md',
    ]
    
    REQUIRED_EXTENSIONS = [
        '.py', '.yaml', '.yml', '.json', '.sh', '.service', '.tsx', '.ts', '.js', '.jsx'
    ]
    
    def __init__(self, project_root: str = "/home/ransomeye/rebuild"):
        self.project_root = Path(project_root).resolve()
        self.violations: List[Dict] = []
    
    def _should_check_file(self, file_path: Path) -> bool:
        """Determine if file should be checked for header."""
        # Check extension
        if file_path.suffix not in self.REQUIRED_EXTENSIONS:
            return False
        
        # Check excluded patterns
        rel_path = str(file_path.relative_to(self.project_root))
        for pattern in self.EXCLUDED_PATTERNS:
            if pattern in rel_path:
                return False
        
        return True
    
    def _validate_header(self, content: str, file_path: Path) -> Optional[Dict]:
        """Validate file header format."""
        # Check first 10 lines for header
        lines = content.split('\n')[:10]
        header_text = '\n'.join(lines)
        
        # Check for all required header components
        missing_components = []
        
        if not re.search(self.REQUIRED_HEADER_LINES[0], header_text, re.IGNORECASE):
            missing_components.append("Path and File Name")
        
        if not re.search(self.REQUIRED_HEADER_LINES[1], header_text):
            missing_components.append("Author (must be nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU)")
        
        if not re.search(self.REQUIRED_HEADER_LINES[2], header_text, re.IGNORECASE):
            missing_components.append("Details of functionality")
        
        if missing_components:
            return {
                'file_path': str(file_path),
                'missing_components': missing_components,
                'header_found': header_text[:200]
            }
        
        return None
    
    def check_file(self, file_path: Path) -> Optional[Dict]:
        """Check a single file for header compliance."""
        if not self._should_check_file(file_path):
            return None
        
        try:
            with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
        except Exception:
            # Skip binary or unreadable files
            return None
        
        return self._validate_header(content, file_path)
    
    def check_directory(self, directory: Optional[Path] = None) -> List[Dict]:
        """Check all files in directory tree."""
        if directory is None:
            directory = self.project_root
        
        directory = Path(directory).resolve()
        violations = []
        
        for root, dirs, files in os.walk(directory):
            # Filter excluded directories
            dirs[:] = [d for d in dirs if d not in self.EXCLUDED_PATTERNS]
            
            for file in files:
                file_path = Path(root) / file
                violation = self.check_file(file_path)
                if violation:
                    violations.append(violation)
        
        self.violations = violations
        return violations
    
    def report_violations(self) -> None:
        """Report violations and fail-closed if any found."""
        if not self.violations:
            print("✓ All files have mandatory headers.")
            return
        
        print(f"\n{'='*80}")
        print(f"HEADER VIOLATIONS DETECTED: {len(self.violations)}")
        print(f"{'='*80}\n")
        
        for violation in self.violations:
            print(f"File: {violation['file_path']}")
            print(f"  Missing: {', '.join(violation['missing_components'])}")
            print(f"  Found header (first 200 chars):")
            print(f"    {violation['header_found']}")
            print()
        
        # Fail-closed
        fail_closed(
            "MANDATORY_HEADER_MISSING",
            f"Found {len(self.violations)} file(s) missing mandatory headers. Build cannot proceed.",
            file_path=None
        )


def main():
    """CLI entry point for header enforcer."""
    import argparse
    
    parser = argparse.ArgumentParser(description='RansomEye Header Enforcer')
    parser.add_argument('--project-root', default='/home/ransomeye/rebuild',
                       help='Project root directory')
    parser.add_argument('--directory', default=None,
                       help='Specific directory to check (default: entire project)')
    
    args = parser.parse_args()
    
    enforcer = HeaderEnforcer(args.project_root)
    
    if args.directory:
        violations = enforcer.check_directory(Path(args.directory))
    else:
        violations = enforcer.check_directory()
    
    enforcer.report_violations()


if __name__ == '__main__':
    main()

