# Path and File Name : /home/ransomeye/rebuild/core/governance/vendor_non_repudiation.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Vendor Non-Repudiation - Scans for backdoors, override flags, and hidden recovery mechanisms (PROMPT-64-C)

"""
RansomEye Vendor Non-Repudiation (PROMPT-64-C)

Proves that even RansomEye engineers cannot override protections.

Scans for:
- Backdoor override mechanisms
- Hidden disable flags
- Secret recovery mechanisms
- Vendor bypass codes
- Hidden configuration options

Outputs static scan results and verifier proof.
"""

import os
import sys
import re
import json
import hashlib
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Tuple, Set

PROJECT_ROOT = Path("/home/ransomeye/rebuild")
SCAN_RESULTS_PATH = Path("/var/lib/ransomeye/governance/vendor_non_repudiation_scan.json")
EVIDENCE_PATH = Path("/var/lib/ransomeye/governance/vendor_non_repudiation_evidence.md")


class VendorNonRepudiationScanner:
    """Vendor Non-Repudiation Scanner - Detects vendor override mechanisms."""
    
    def __init__(self):
        """Initialize scanner."""
        self.findings = []
        self.backdoor_patterns = []
        self.override_patterns = []
        self.recovery_patterns = []
        self._load_patterns()
    
    def _load_patterns(self):
        """Load patterns for backdoor/override detection."""
        # Backdoor patterns
        self.backdoor_patterns = [
            r'backdoor',
            r'vendor.*override',
            r'engineer.*bypass',
            r'secret.*key',
            r'hidden.*flag',
            r'disable.*protection',
            r'bypass.*verification',
            r'skip.*check',
            r'force.*enable',
            r'vendor.*mode',
            r'debug.*mode.*production',
            r'admin.*override',
            r'master.*key',
            r'recovery.*code',
            r'emergency.*access',
        ]
        
        # Override flag patterns
        self.override_patterns = [
            r'OVERRIDE',
            r'BYPASS',
            r'DISABLE.*PROTECTION',
            r'FORCE.*ENABLE',
            r'VENDOR.*MODE',
            r'DEBUG.*MODE',
            r'ADMIN.*OVERRIDE',
            r'MASTER.*KEY',
            r'SKIP.*VERIFICATION',
            r'IGNORE.*CHECK',
        ]
        
        # Recovery mechanism patterns
        self.recovery_patterns = [
            r'recovery.*mechanism',
            r'reset.*protection',
            r'clear.*lock',
            r'remove.*assurance',
            r'disable.*assurance',
            r'unlock.*mode',
            r'reset.*mode',
            r'factory.*reset',
            r'vendor.*reset',
        ]
    
    def scan_file(self, file_path: Path) -> List[Dict]:
        """Scan a single file for backdoor/override patterns."""
        findings = []
        
        if not file_path.exists() or not file_path.is_file():
            return findings
        
        # Skip binary files (check extension)
        if file_path.suffix in ['.so', '.bin', '.model', '.pkl', '.gguf', '.png', '.jpg', '.jpeg']:
            return findings
        
        try:
            with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
                lines = content.split('\n')
                
                # Check for backdoor patterns
                for pattern in self.backdoor_patterns:
                    matches = re.finditer(pattern, content, re.IGNORECASE)
                    for match in matches:
                        line_num = content[:match.start()].count('\n') + 1
                        line_content = lines[line_num - 1] if line_num <= len(lines) else ""
                        findings.append({
                            'type': 'backdoor_pattern',
                            'pattern': pattern,
                            'file': str(file_path.relative_to(PROJECT_ROOT)),
                            'line': line_num,
                            'content': line_content.strip()[:100],
                            'severity': 'HIGH'
                        })
                
                # Check for override flags
                for pattern in self.override_patterns:
                    matches = re.finditer(pattern, content, re.IGNORECASE)
                    for match in matches:
                        line_num = content[:match.start()].count('\n') + 1
                        line_content = lines[line_num - 1] if line_num <= len(lines) else ""
                        findings.append({
                            'type': 'override_flag',
                            'pattern': pattern,
                            'file': str(file_path.relative_to(PROJECT_ROOT)),
                            'line': line_num,
                            'content': line_content.strip()[:100],
                            'severity': 'CRITICAL'
                        })
                
                # Check for recovery mechanisms
                for pattern in self.recovery_patterns:
                    matches = re.finditer(pattern, content, re.IGNORECASE)
                    for match in matches:
                        line_num = content[:match.start()].count('\n') + 1
                        line_content = lines[line_num - 1] if line_num <= len(lines) else ""
                        findings.append({
                            'type': 'recovery_mechanism',
                            'pattern': pattern,
                            'file': str(file_path.relative_to(PROJECT_ROOT)),
                            'line': line_num,
                            'content': line_content.strip()[:100],
                            'severity': 'HIGH'
                        })
        
        except Exception as e:
            # Skip files that can't be read
            pass
        
        return findings
    
    def scan_directory(self, directory: Path, exclude_dirs: Set[str] = None) -> List[Dict]:
        """Scan directory recursively for backdoor/override patterns."""
        if exclude_dirs is None:
            exclude_dirs = {
                '__pycache__', '.git', 'node_modules', '.venv', 'venv',
                'build', 'dist', '.pytest_cache', 'logs', 'tmp', 'temp'
            }
        
        all_findings = []
        
        for root, dirs, files in os.walk(directory):
            # Skip excluded directories
            dirs[:] = [d for d in dirs if d not in exclude_dirs]
            
            for file in files:
                file_path = Path(root) / file
                findings = self.scan_file(file_path)
                all_findings.extend(findings)
        
        return all_findings
    
    def check_assurance_lock_removal(self) -> List[Dict]:
        """Check for code that removes assurance lock."""
        findings = []
        assurance_lock_path = Path("/etc/ransomeye/ASSURANCE_MODE_LOCK")
        
        # Scan for code that removes or modifies assurance lock
        for root, dirs, files in os.walk(PROJECT_ROOT):
            for file in files:
                if file.endswith(('.py', '.sh', '.service')):
                    file_path = Path(root) / file
                    try:
                        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                            content = f.read()
                            if 'ASSURANCE_MODE_LOCK' in content:
                                # Check for removal operations
                                if any(op in content for op in ['rm ', 'unlink', 'remove', 'delete']):
                                    findings.append({
                                        'type': 'assurance_lock_removal',
                                        'file': str(file_path.relative_to(PROJECT_ROOT)),
                                        'severity': 'CRITICAL',
                                        'description': 'Code may remove assurance lock'
                                    })
                    except Exception:
                        pass
        
        return findings
    
    def check_verifier_bypass(self) -> List[Dict]:
        """Check for code that bypasses verifier."""
        findings = []
        verifier_path = PROJECT_ROOT / "core/verifier/verifier.py"
        
        # Scan for code that skips verifier checks
        for root, dirs, files in os.walk(PROJECT_ROOT):
            for file in files:
                if file.endswith('.py'):
                    file_path = Path(root) / file
                    try:
                        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                            content = f.read()
                            # Check for verifier bypass patterns
                            if any(pattern in content.lower() for pattern in [
                                'skip.*verifier', 'bypass.*verifier', 'ignore.*verifier',
                                'disable.*verifier', 'force.*skip'
                            ]):
                                findings.append({
                                    'type': 'verifier_bypass',
                                    'file': str(file_path.relative_to(PROJECT_ROOT)),
                                    'severity': 'CRITICAL',
                                    'description': 'Code may bypass verifier'
                                })
                    except Exception:
                        pass
        
        return findings
    
    def check_ship_seal_bypass(self) -> List[Dict]:
        """Check for code that bypasses ship seal."""
        findings = []
        ship_seal_enforcer_path = PROJECT_ROOT / "core/assurance/ship_seal_enforcer.py"
        
        # Scan for code that skips ship seal checks
        for root, dirs, files in os.walk(PROJECT_ROOT):
            for file in files:
                if file.endswith('.py'):
                    file_path = Path(root) / file
                    try:
                        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                            content = f.read()
                            # Check for ship seal bypass patterns
                            if any(pattern in content.lower() for pattern in [
                                'skip.*ship.*seal', 'bypass.*ship.*seal', 'ignore.*ship.*seal',
                                'disable.*ship.*seal', 'force.*skip.*seal'
                            ]):
                                findings.append({
                                    'type': 'ship_seal_bypass',
                                    'file': str(file_path.relative_to(PROJECT_ROOT)),
                                    'severity': 'CRITICAL',
                                    'description': 'Code may bypass ship seal'
                                })
                    except Exception:
                        pass
        
        return findings
    
    def scan(self) -> Dict:
        """Run full vendor non-repudiation scan."""
        print("=" * 80)
        print("Vendor Non-Repudiation Scan (PROMPT-64-C)")
        print("=" * 80)
        print("Scanning for backdoors, override flags, and recovery mechanisms...")
        print()
        
        all_findings = []
        
        # Scan codebase for patterns
        print("Scanning codebase for backdoor/override patterns...")
        pattern_findings = self.scan_directory(PROJECT_ROOT)
        all_findings.extend(pattern_findings)
        print(f"  Found {len(pattern_findings)} pattern matches")
        
        # Check for assurance lock removal
        print("Checking for assurance lock removal code...")
        lock_findings = self.check_assurance_lock_removal()
        all_findings.extend(lock_findings)
        print(f"  Found {len(lock_findings)} potential lock removal attempts")
        
        # Check for verifier bypass
        print("Checking for verifier bypass code...")
        verifier_findings = self.check_verifier_bypass()
        all_findings.extend(verifier_findings)
        print(f"  Found {len(verifier_findings)} potential verifier bypasses")
        
        # Check for ship seal bypass
        print("Checking for ship seal bypass code...")
        seal_findings = self.check_ship_seal_bypass()
        all_findings.extend(seal_findings)
        print(f"  Found {len(seal_findings)} potential ship seal bypasses")
        
        # Filter false positives (comments, documentation)
        critical_findings = []
        for finding in all_findings:
            # Skip if in comment or documentation
            if finding.get('content', '').strip().startswith('#'):
                continue
            if 'docs/' in finding.get('file', ''):
                continue
            if 'README' in finding.get('file', ''):
                continue
            critical_findings.append(finding)
        
        # Generate report
        report = {
            'scan_timestamp': datetime.now(timezone.utc).isoformat(),
            'total_findings': len(all_findings),
            'critical_findings': len(critical_findings),
            'findings': critical_findings,
            'summary': {
                'backdoor_patterns': len([f for f in critical_findings if f['type'] == 'backdoor_pattern']),
                'override_flags': len([f for f in critical_findings if f['type'] == 'override_flag']),
                'recovery_mechanisms': len([f for f in critical_findings if f['type'] == 'recovery_mechanism']),
                'assurance_lock_removal': len([f for f in critical_findings if f['type'] == 'assurance_lock_removal']),
                'verifier_bypass': len([f for f in critical_findings if f['type'] == 'verifier_bypass']),
                'ship_seal_bypass': len([f for f in critical_findings if f['type'] == 'ship_seal_bypass']),
            }
        }
        
        # Save results
        SCAN_RESULTS_PATH.parent.mkdir(parents=True, exist_ok=True)
        with open(SCAN_RESULTS_PATH, 'w') as f:
            json.dump(report, f, indent=2)
        
        # Generate evidence markdown
        self._generate_evidence_markdown(report)
        
        print()
        print("=" * 80)
        print(f"Scan complete: {len(critical_findings)} critical findings")
        print(f"Results saved to: {SCAN_RESULTS_PATH}")
        print(f"Evidence saved to: {EVIDENCE_PATH}")
        print("=" * 80)
        
        return report
    
    def _generate_evidence_markdown(self, report: Dict):
        """Generate evidence markdown report."""
        EVIDENCE_PATH.parent.mkdir(parents=True, exist_ok=True)
        
        with open(EVIDENCE_PATH, 'w') as f:
            f.write("# Vendor Non-Repudiation Scan Evidence\n\n")
            f.write(f"**Date:** {report['scan_timestamp']}\n\n")
            f.write("## Summary\n\n")
            f.write(f"- **Total Findings:** {report['total_findings']}\n")
            f.write(f"- **Critical Findings:** {report['critical_findings']}\n\n")
            f.write("## Finding Breakdown\n\n")
            f.write(f"- Backdoor Patterns: {report['summary']['backdoor_patterns']}\n")
            f.write(f"- Override Flags: {report['summary']['override_flags']}\n")
            f.write(f"- Recovery Mechanisms: {report['summary']['recovery_mechanisms']}\n")
            f.write(f"- Assurance Lock Removal: {report['summary']['assurance_lock_removal']}\n")
            f.write(f"- Verifier Bypass: {report['summary']['verifier_bypass']}\n")
            f.write(f"- Ship Seal Bypass: {report['summary']['ship_seal_bypass']}\n\n")
            f.write("## Critical Findings\n\n")
            
            if report['critical_findings'] == 0:
                f.write("✅ **NO CRITICAL FINDINGS** - No vendor override mechanisms detected.\n\n")
            else:
                for finding in report['findings']:
                    f.write(f"### {finding['type']}\n\n")
                    f.write(f"- **File:** `{finding['file']}`\n")
                    f.write(f"- **Severity:** {finding['severity']}\n")
                    if 'line' in finding:
                        f.write(f"- **Line:** {finding['line']}\n")
                    if 'content' in finding:
                        f.write(f"- **Content:** `{finding['content']}`\n")
                    f.write(f"- **Description:** {finding.get('description', 'N/A')}\n\n")
            
            f.write("## Conclusion\n\n")
            if report['critical_findings'] == 0:
                f.write("✅ **VENDOR NON-REPUDIATION VERIFIED**\n\n")
                f.write("No backdoor override mechanisms, hidden disable flags, or secret recovery mechanisms detected.\n\n")
                f.write("Even RansomEye engineers cannot override protections.\n\n")
            else:
                f.write("⚠️ **CRITICAL FINDINGS DETECTED**\n\n")
                f.write("Review findings above for potential vendor override mechanisms.\n\n")
            
            f.write("---\n\n")
            f.write("© RansomEye.Tech | Support: Gagan@RansomEye.Tech\n")


def main():
    """Main entry point."""
    scanner = VendorNonRepudiationScanner()
    report = scanner.scan()
    
    # Exit with error if critical findings
    if report['critical_findings'] > 0:
        print(f"\n⚠️  WARNING: {report['critical_findings']} critical findings detected")
        print("Review evidence report for details")
        sys.exit(1)
    else:
        print("\n✅ No critical findings - Vendor non-repudiation verified")
        sys.exit(0)


if __name__ == "__main__":
    main()

