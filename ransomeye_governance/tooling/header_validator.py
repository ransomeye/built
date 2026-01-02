# Path: /home/ransomeye/rebuild/ransomeye_governance/tooling/header_validator.py
# Author: RansomEye Core Team
# Purpose: Validates mandatory file headers across all project files - enforces header presence and format

"""
Header Validator: Enforces mandatory file headers.

Every file (.rs .c .cpp .py .sh .yaml .json .toml .service) must start with:
- Path: <absolute repo path>
- Author: RansomEye Core Team
- Purpose: <explicit purpose>
"""

import os
import sys
from pathlib import Path
from typing import List, Dict, Optional
import re

# Required header components
REQUIRED_HEADER_PATTERNS = {
    'path': re.compile(r'^#\s*Path\s*:\s*(.+)$', re.IGNORECASE),
    'author': re.compile(r'^#\s*Author\s*:\s*RansomEye Core Team$', re.IGNORECASE),
    'purpose': re.compile(r'^#\s*Purpose\s*:\s*(.+)$', re.IGNORECASE),
}

# File extensions that require headers
REQUIRED_EXTENSIONS = {
    '.rs', '.c', '.cpp', '.py', '.sh', '.yaml', '.yml', '.json', '.toml', '.service'
}

# Excluded patterns (directories/files to skip)
EXCLUDED_PATTERNS = [
    '__pycache__',
    '.git',
    'node_modules',
    'venv',
    '.venv',
    'target',
    'dist',
    'build',
    '.pytest_cache',
    '.mypy_cache',
    'Cargo.lock',  # Generated file
    'package-lock.json',  # Generated file
    '.md',  # Markdown files excluded per spec
]


class HeaderValidator:
    """Validates mandatory file headers."""
    
    def __init__(self, project_root: str = "/home/ransomeye/rebuild"):
        self.project_root = Path(project_root).resolve()
        self.violations: List[Dict] = []
    
    def _should_check_file(self, file_path: Path) -> bool:
        """Determine if file should be checked."""
        # Check extension
        if file_path.suffix not in REQUIRED_EXTENSIONS:
            return False
        
        # Check excluded patterns
        rel_path = str(file_path.relative_to(self.project_root))
        for pattern in EXCLUDED_PATTERNS:
            if pattern in rel_path:
                return False
        
        return True
    
    def _validate_header(self, content: str, file_path: Path) -> Optional[Dict]:
        """Validate file header format."""
        lines = content.split('\n')[:20]  # Check first 20 lines
        
        found_path = False
        found_author = False
        found_purpose = False
        
        missing_components = []
        
        for line in lines:
            # Check for Path
            if REQUIRED_HEADER_PATTERNS['path'].match(line.strip()):
                found_path = True
            # Check for Author
            if REQUIRED_HEADER_PATTERNS['author'].match(line.strip()):
                found_author = True
            # Check for Purpose
            if REQUIRED_HEADER_PATTERNS['purpose'].match(line.strip()):
                found_purpose = True
        
        if not found_path:
            missing_components.append("Path")
        if not found_author:
            missing_components.append("Author (must be 'RansomEye Core Team')")
        if not found_purpose:
            missing_components.append("Purpose")
        
        if missing_components:
            return {
                'file_path': str(file_path),
                'missing_components': missing_components,
                'header_preview': '\n'.join(lines[:5])
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
            dirs[:] = [d for d in dirs if not any(pattern in d for pattern in EXCLUDED_PATTERNS)]
            
            for file in files:
                file_path = Path(root) / file
                violation = self.check_file(file_path)
                if violation:
                    violations.append(violation)
        
        self.violations = violations
        return violations
    
    def validate(self) -> bool:
        """Run full header validation."""
        print("=" * 80)
        print("RansomEye Header Validator")
        print("=" * 80)
        print()
        
        violations = self.check_directory()
        
        if violations:
            print(f"\n{'='*80}")
            print(f"HEADER VIOLATIONS DETECTED: {len(violations)}")
            print(f"{'='*80}\n")
            
            for violation in violations:
                print(f"File: {violation['file_path']}")
                print(f"  Missing: {', '.join(violation['missing_components'])}")
                print(f"  Preview:")
                for line in violation['header_preview'].split('\n')[:3]:
                    print(f"    {line}")
                print()
            
            return False
        else:
            print("\n✓ All files have mandatory headers.")
            return True
    
    def fail_closed(self, message: str):
        """Fail-closed: exit with error."""
        print(f"\n{'='*80}")
        print("HEADER VALIDATION FAILED")
        print(f"{'='*80}")
        print(message)
        print("\nBuild blocked due to missing mandatory headers.")
        sys.exit(1)


def main():
    """CLI entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description='RansomEye Header Validator')
    parser.add_argument('--project-root', default='/home/ransomeye/rebuild',
                       help='Project root directory')
    parser.add_argument('--directory', default=None,
                       help='Specific directory to check (default: entire project)')
    
    args = parser.parse_args()
    
    validator = HeaderValidator(args.project_root)
    
    if args.directory:
        violations = validator.check_directory(Path(args.directory))
    else:
        violations = validator.check_directory()
    
    is_valid = len(violations) == 0
    
    if not is_valid:
        validator.fail_closed(f"Found {len(violations)} file(s) missing mandatory headers.")
    
    print("\n✓ Header validation passed.")
    sys.exit(0)


if __name__ == '__main__':
    main()

