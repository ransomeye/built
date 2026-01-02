# Path and File Name : /home/ransomeye/rebuild/ransomeye_guardrails/scanner.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Static code scanner using regex and AST to detect guardrail violations

"""
Static code scanner for RansomEye guardrails.
Uses regex patterns and AST parsing to detect violations.
"""

import os
import re
import ast
import yaml
from pathlib import Path
from typing import List, Dict, Tuple, Optional, Set
from collections import defaultdict

from .fail_closed import fail_closed
from .rules_schema import validate_rules_file


class GuardrailScanner:
    """Static scanner that enforces guardrail rules."""
    
    def __init__(self, rules_path: str, project_root: str = "/home/ransomeye/rebuild"):
        self.project_root = Path(project_root).resolve()
        self.rules = self._load_rules(rules_path)
        self.violations: List[Dict] = []
        self._allowed_exceptions = self._parse_exceptions()
    
    def _load_rules(self, rules_path: str) -> Dict:
        """
        Load rules from YAML file.
        Validates rules.yaml structure BEFORE loading.
        Fails-closed if validation fails.
        """
        # CRITICAL: Validate rules.yaml schema BEFORE loading
        if not validate_rules_file(rules_path):
            # validate_rules_file will fail-closed on errors
            # This should never be reached, but included for safety
            fail_closed(
                "RULES_LOAD_FAILED",
                f"Rules file validation failed: {rules_path}",
                file_path=rules_path
            )
        
        # Load validated rules
        with open(rules_path, 'r') as f:
            rules = yaml.safe_load(f)
        
        if rules is None:
            fail_closed(
                "RULES_LOAD_FAILED",
                f"Rules file is empty or invalid: {rules_path}",
                file_path=rules_path
            )
        
        return rules
    
    def _parse_exceptions(self) -> Dict[str, Set[str]]:
        """Parse allowed exceptions from rules."""
        exceptions = defaultdict(set)
        for exception in self.rules.get('allowed_exceptions', []):
            for rule in exception.get('rules', []):
                for pattern in exception.get('file_patterns', []):
                    exceptions[rule].add(pattern)
        return exceptions
    
    def _is_excluded(self, file_path: Path) -> bool:
        """Check if file should be excluded from scanning."""
        exclude_dirs = self.rules.get('scan_config', {}).get('exclude_dirs', [])
        rel_path = file_path.relative_to(self.project_root)
        
        for part in rel_path.parts:
            if part in exclude_dirs:
                return True
        return False
    
    def _should_scan_file(self, file_path: Path) -> bool:
        """Determine if file should be scanned."""
        if self._is_excluded(file_path):
            return False
        
        extensions = self.rules.get('scan_config', {}).get('file_extensions', [])
        return file_path.suffix in extensions
    
    def _matches_exception(self, rule_name: str, file_path: Path) -> bool:
        """Check if file matches an exception pattern for a rule."""
        if rule_name not in self._allowed_exceptions:
            return False
        
        rel_path = str(file_path.relative_to(self.project_root))
        for pattern in self._allowed_exceptions[rule_name]:
            # Simple glob-like matching
            if self._glob_match(pattern, rel_path):
                return True
        return False
    
    def _glob_match(self, pattern: str, path: str) -> bool:
        """Simple glob matching for exception patterns."""
        import fnmatch
        return fnmatch.fnmatch(path, pattern)
    
    def _check_hardcoded_patterns(self, content: str, file_path: Path) -> List[Dict]:
        """Check for hardcoded patterns (IPs, URLs, secrets)."""
        violations = []
        
        for rule in self.rules.get('hardcoded_patterns', []):
            # Use 'regex' field (preferred) or fallback to 'pattern' (legacy)
            pattern = rule.get('regex', rule.get('pattern', ''))
            if not pattern:
                continue  # Skip invalid rules
            exception = rule.get('exception', '')
            
            # Compile regex
            regex = re.compile(pattern, re.IGNORECASE | re.MULTILINE)
            
            for match in regex.finditer(content):
                matched_text = match.group(0)
                
                # Check exception
                if exception:
                    exception_regex = re.compile(exception, re.IGNORECASE)
                    if exception_regex.search(matched_text):
                        continue
                
                line_num = content[:match.start()].count('\n') + 1
                violations.append({
                    'rule_name': rule['rule_name'],
                    'description': rule['description'],
                    'file_path': str(file_path),
                    'line': line_num,
                    'matched_text': matched_text[:50]  # Truncate for display
                })
        
        return violations
    
    def _check_path_patterns(self, content: str, file_path: Path) -> List[Dict]:
        """Check for paths outside project root."""
        violations = []
        
        for rule in self.rules.get('path_patterns', []):
            # Use 'regex' field (preferred) or fallback to 'pattern' (legacy)
            pattern = rule.get('regex', rule.get('pattern', ''))
            if not pattern:
                continue  # Skip invalid rules
            exception = rule.get('exception', '')
            
            regex = re.compile(pattern, re.IGNORECASE | re.MULTILINE)
            
            for match in regex.finditer(content):
                matched_text = match.group(0)
                
                if exception and exception in matched_text:
                    continue
                
                # Verify it's actually outside project root
                try:
                    abs_path = Path(matched_text).resolve()
                    if str(abs_path).startswith(str(self.project_root)):
                        continue
                except:
                    pass
                
                line_num = content[:match.start()].count('\n') + 1
                violations.append({
                    'rule_name': rule['rule_name'],
                    'description': rule['description'],
                    'file_path': str(file_path),
                    'line': line_num,
                    'matched_text': matched_text[:50]
                })
        
        return violations
    
    def _check_ml_patterns(self, content: str, file_path: Path) -> List[Dict]:
        """Check for ML-related violations."""
        violations = []
        
        for rule in self.rules.get('ml_patterns', []):
            # Use 'regex' field (preferred) or fallback to 'pattern' (legacy)
            pattern = rule.get('regex', rule.get('pattern', ''))
            if not pattern:
                continue  # Skip invalid rules
            regex = re.compile(pattern, re.IGNORECASE | re.MULTILINE)
            
            for match in regex.finditer(content):
                line_num = content[:match.start()].count('\n') + 1
                
                # Additional context check for SHAP
                if 'SHAP_REQUIRED' in rule['rule_name']:
                    # Check if SHAP is used in nearby context
                    start = max(0, match.start() - 200)
                    end = min(len(content), match.end() + 200)
                    context = content[start:end]
                    if 'shap' in context.lower() or 'explain' in context.lower():
                        continue
                
                violations.append({
                    'rule_name': rule['rule_name'],
                    'description': rule['description'],
                    'file_path': str(file_path),
                    'line': line_num,
                    'matched_text': match.group(0)[:50]
                })
        
        return violations
    
    def scan_file(self, file_path: Path) -> List[Dict]:
        """Scan a single file for violations."""
        violations = []
        
        if not self._should_scan_file(file_path):
            return violations
        
        try:
            with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
        except Exception as e:
            # Skip binary files or unreadable files
            return violations
        
        # Check all pattern types
        violations.extend(self._check_hardcoded_patterns(content, file_path))
        violations.extend(self._check_path_patterns(content, file_path))
        violations.extend(self._check_ml_patterns(content, file_path))
        
        # Filter out exceptions
        filtered_violations = []
        for violation in violations:
            if not self._matches_exception(violation['rule_name'], file_path):
                filtered_violations.append(violation)
        
        return filtered_violations
    
    def scan_directory(self, directory: Optional[Path] = None) -> List[Dict]:
        """Scan entire directory tree for violations."""
        if directory is None:
            directory = self.project_root
        
        directory = Path(directory).resolve()
        all_violations = []
        
        for root, dirs, files in os.walk(directory):
            # Filter excluded directories
            dirs[:] = [d for d in dirs if d not in self.rules.get('scan_config', {}).get('exclude_dirs', [])]
            
            for file in files:
                file_path = Path(root) / file
                violations = self.scan_file(file_path)
                all_violations.extend(violations)
        
        self.violations = all_violations
        return all_violations
    
    def report_violations(self) -> None:
        """Report violations and fail-closed if any found."""
        if not self.violations:
            print("✓ No guardrail violations detected.")
            return
        
        print(f"\n{'='*80}")
        print(f"GUARDRAIL VIOLATIONS DETECTED: {len(self.violations)}")
        print(f"{'='*80}\n")
        
        for violation in self.violations:
            print(f"Rule: {violation['rule_name']}")
            print(f"  File: {violation['file_path']}")
            print(f"  Line: {violation['line']}")
            print(f"  Description: {violation['description']}")
            print(f"  Matched: {violation.get('matched_text', 'N/A')}")
            print()
        
        # Fail-closed
        fail_closed(
            "STATIC_SCAN_VIOLATIONS",
            f"Found {len(self.violations)} guardrail violation(s). Build cannot proceed.",
            file_path=None
        )


def main():
    """CLI entry point for scanner."""
    import argparse
    
    parser = argparse.ArgumentParser(description='RansomEye Guardrail Scanner')
    parser.add_argument('--rules', default='/home/ransomeye/rebuild/ransomeye_guardrails/rules.yaml',
                       help='Path to rules YAML file')
    parser.add_argument('--project-root', default='/home/ransomeye/rebuild',
                       help='Project root directory')
    parser.add_argument('--directory', default=None,
                       help='Specific directory to scan (default: entire project)')
    
    args = parser.parse_args()
    
    scanner = GuardrailScanner(args.rules, args.project_root)
    
    if args.directory:
        violations = scanner.scan_directory(Path(args.directory))
    else:
        violations = scanner.scan_directory()
    
    scanner.report_violations()


if __name__ == '__main__':
    main()

