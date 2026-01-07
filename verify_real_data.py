# Path and File Name : /home/ransomeye/rebuild/verify_real_data.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Verification script to check DPI Probe and Linux Agent for synthetic/dummy data

"""
RansomEye Data Verification Script
==================================
This script verifies that DPI Probe and Linux Agent collect REAL data,
not synthetic, dummy, or placeholder data.

Checks:
1. DPI Probe - packet capture sources
2. Linux Agent - process monitoring sources
3. Any hardcoded test/synthetic data generation
"""

import os
import re
import json
from pathlib import Path
from typing import List, Dict, Tuple

# Synthetic/dummy data patterns
SYNTHETIC_PATTERNS = [
    r'/usr/bin/test',
    r'test\s+--arg',
    r'synthetic',
    r'dummy',
    r'fake',
    r'placeholder',
    r'mock',
    r'generate.*test',
    r'hardcoded.*test',
    r'1234.*event_count',  # Synthetic PID generation
]

# Real data source patterns (GOOD)
REAL_DATA_PATTERNS = [
    r'/proc/',
    r'pcap::',
    r'Capture::',
    r'Device::',
    r'next_packet',
    r'read_dir\("/proc"\)',
    r'read_to_string\("/proc',
    r'inotify',
    r'auditd',
    r'ebpf',
    r'syscall',
]

class DataVerifier:
    def __init__(self, root_path: str):
        self.root_path = Path(root_path)
        self.issues: List[Dict] = []
        self.findings: List[Dict] = []
        
    def check_file(self, file_path: Path, module_name: str) -> List[Dict]:
        """Check a single file for synthetic data issues."""
        issues = []
        
        try:
            with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
                lines = content.split('\n')
                
            # Check for synthetic patterns
            for line_num, line in enumerate(lines, 1):
                for pattern in SYNTHETIC_PATTERNS:
                    if re.search(pattern, line, re.IGNORECASE):
                        # Skip if in comments or tests
                        if not self._is_test_or_comment(line):
                            issues.append({
                                'file': str(file_path.relative_to(self.root_path)),
                                'line': line_num,
                                'pattern': pattern,
                                'content': line.strip()[:100],
                                'severity': 'HIGH' if 'test' in pattern.lower() else 'MEDIUM',
                            })
                            
        except Exception as e:
            issues.append({
                'file': str(file_path.relative_to(self.root_path)),
                'line': 0,
                'error': str(e),
                'severity': 'LOW',
            })
            
        return issues
    
    def _is_test_or_comment(self, line: str) -> bool:
        """Check if line is a test file or comment."""
        stripped = line.strip()
        return (
            stripped.startswith('//') or
            stripped.startswith('#') or
            stripped.startswith('*') or
            'test' in line.lower() and ('fn test_' in line or 'def test_' in line or 'TEST' in line)
        )
    
    def check_dpi_probe(self) -> Dict:
        """Verify DPI Probe data sources."""
        print("\n" + "="*80)
        print("VERIFYING DPI PROBE DATA SOURCES")
        print("="*80)
        
        dpi_path = self.root_path / 'edge' / 'dpi'
        if not dpi_path.exists():
            return {'status': 'ERROR', 'message': 'DPI Probe directory not found'}
        
        findings = {
            'module': 'DPI Probe',
            'status': 'UNKNOWN',
            'data_sources': [],
            'issues': [],
            'verification': {},
        }
        
        # Check capture.rs files
        capture_files = list(dpi_path.rglob('**/capture.rs'))
        capture_files.extend(list(dpi_path.rglob('**/main.rs')))
        
        real_capture_found = False
        synthetic_found = False
        
        for cap_file in capture_files:
            issues = self.check_file(cap_file, 'DPI Probe')
            findings['issues'].extend(issues)
            
            with open(cap_file, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
                
                # Check for real packet capture
                if any(re.search(pattern, content, re.IGNORECASE) for pattern in REAL_DATA_PATTERNS):
                    real_capture_found = True
                    findings['data_sources'].append({
                        'file': str(cap_file.relative_to(self.root_path)),
                        'type': 'REAL_PACKET_CAPTURE',
                        'method': self._identify_capture_method(content),
                    })
                
                # Check for synthetic generation
                if any(re.search(pattern, content, re.IGNORECASE) for pattern in SYNTHETIC_PATTERNS):
                    if not self._is_test_file(cap_file):
                        synthetic_found = True
        
        findings['verification'] = {
            'real_capture_detected': real_capture_found,
            'synthetic_data_detected': synthetic_found,
            'uses_pcap': 'pcap::' in str(findings['data_sources']),
            'uses_af_packet': 'AF_PACKET' in str(findings['data_sources']),
        }
        
        if real_capture_found and not synthetic_found:
            findings['status'] = 'PASS - Real data only'
        elif real_capture_found and synthetic_found:
            findings['status'] = 'WARNING - Real data but synthetic code present'
        elif not real_capture_found:
            findings['status'] = 'FAIL - No real data sources detected'
        else:
            findings['status'] = 'UNKNOWN'
        
        return findings
    
    def check_linux_agent(self) -> Dict:
        """Verify Linux Agent data sources."""
        print("\n" + "="*80)
        print("VERIFYING LINUX AGENT DATA SOURCES")
        print("="*80)
        
        agent_path = self.root_path / 'edge' / 'agent' / 'linux'
        if not agent_path.exists():
            return {'status': 'ERROR', 'message': 'Linux Agent directory not found'}
        
        findings = {
            'module': 'Linux Agent',
            'status': 'UNKNOWN',
            'data_sources': [],
            'issues': [],
            'verification': {},
        }
        
        # Check main.rs files (entry points)
        main_files = list(agent_path.rglob('**/main.rs'))
        
        # Check process monitoring files
        process_files = list(agent_path.rglob('**/process.rs'))
        process_files.extend(list(agent_path.rglob('**/telemetry.rs')))
        
        real_monitoring_found = False
        synthetic_found = False
        synthetic_in_main = False
        
        # Check process monitoring (should be real)
        for proc_file in process_files:
            issues = self.check_file(proc_file, 'Linux Agent')
            findings['issues'].extend(issues)
            
            with open(proc_file, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
                
                # Check for real /proc monitoring
                if '/proc/' in content:
                    real_monitoring_found = True
                    findings['data_sources'].append({
                        'file': str(proc_file.relative_to(self.root_path)),
                        'type': 'REAL_PROC_MONITORING',
                        'method': 'procfs',
                    })
        
        # Check main.rs for synthetic data generation
        for main_file in main_files:
            issues = self.check_file(main_file, 'Linux Agent')
            findings['issues'].extend(issues)
            
            with open(main_file, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
                lines = content.split('\n')
                
                # Look for synthetic event generation in main loop
                for line_num, line in enumerate(lines, 1):
                    if '/usr/bin/test' in line and 'record_exec' in line:
                        synthetic_in_main = True
                        synthetic_found = True
                        findings['issues'].append({
                            'file': str(main_file.relative_to(self.root_path)),
                            'line': line_num,
                            'pattern': 'SYNTHETIC_EVENT_GENERATION',
                            'content': line.strip()[:100],
                            'severity': 'CRITICAL',
                            'description': 'Main loop generates synthetic process events instead of monitoring real processes',
                        })
        
        findings['verification'] = {
            'real_monitoring_detected': real_monitoring_found,
            'synthetic_data_detected': synthetic_found,
            'synthetic_in_main_loop': synthetic_in_main,
            'uses_procfs': real_monitoring_found,
            'uses_ebpf': 'ebpf' in str(findings['data_sources']).lower(),
            'uses_auditd': 'auditd' in str(findings['data_sources']).lower(),
        }
        
        if real_monitoring_found and not synthetic_in_main:
            findings['status'] = 'PASS - Real data only'
        elif real_monitoring_found and synthetic_in_main:
            findings['status'] = 'CRITICAL - Real monitoring exists but main loop generates synthetic data'
        elif not real_monitoring_found:
            findings['status'] = 'FAIL - No real data sources detected'
        else:
            findings['status'] = 'UNKNOWN'
        
        return findings
    
    def _identify_capture_method(self, content: str) -> str:
        """Identify packet capture method from code."""
        if 'pcap::' in content or 'libpcap' in content.lower():
            return 'libpcap'
        elif 'AF_PACKET' in content:
            return 'AF_PACKET'
        elif 'Capture::' in content:
            return 'pcap_rust'
        else:
            return 'unknown'
    
    def _is_test_file(self, file_path: Path) -> bool:
        """Check if file is a test file."""
        return 'test' in file_path.name.lower() or '/tests/' in str(file_path)
    
    def generate_report(self, dpi_findings: Dict, agent_findings: Dict) -> str:
        """Generate verification report."""
        report = []
        report.append("="*80)
        report.append("RANSOMEYE DATA VERIFICATION REPORT")
        report.append("="*80)
        report.append("")
        report.append("Purpose: Verify that DPI Probe and Linux Agent collect REAL data,")
        report.append("         not synthetic, dummy, or placeholder data.")
        report.append("")
        report.append("="*80)
        report.append("EXECUTIVE SUMMARY")
        report.append("="*80)
        report.append("")
        
        # DPI Probe Summary
        report.append(f"DPI Probe Status: {dpi_findings.get('status', 'UNKNOWN')}")
        if dpi_findings.get('verification', {}).get('real_capture_detected'):
            report.append("  ✓ Real packet capture detected")
        else:
            report.append("  ✗ No real packet capture detected")
        if dpi_findings.get('verification', {}).get('synthetic_data_detected'):
            report.append("  ⚠ Synthetic data generation code found")
        report.append("")
        
        # Linux Agent Summary
        report.append(f"Linux Agent Status: {agent_findings.get('status', 'UNKNOWN')}")
        if agent_findings.get('verification', {}).get('real_monitoring_detected'):
            report.append("  ✓ Real process monitoring detected")
        else:
            report.append("  ✗ No real process monitoring detected")
        if agent_findings.get('verification', {}).get('synthetic_in_main_loop'):
            report.append("  ✗ CRITICAL: Main loop generates synthetic events")
        report.append("")
        
        # Detailed Findings
        report.append("="*80)
        report.append("DETAILED FINDINGS - DPI PROBE")
        report.append("="*80)
        report.append("")
        report.append(f"Status: {dpi_findings.get('status', 'UNKNOWN')}")
        report.append("")
        report.append("Data Sources:")
        for source in dpi_findings.get('data_sources', []):
            report.append(f"  - {source['file']}: {source['type']} ({source['method']})")
        report.append("")
        
        if dpi_findings.get('issues'):
            report.append("Issues Found:")
            for issue in dpi_findings['issues'][:10]:  # Limit to first 10
                report.append(f"  Line {issue.get('line', '?')}: {issue.get('pattern', 'unknown')}")
                report.append(f"    {issue.get('content', '')[:80]}")
        else:
            report.append("No issues found.")
        report.append("")
        
        report.append("="*80)
        report.append("DETAILED FINDINGS - LINUX AGENT")
        report.append("="*80)
        report.append("")
        report.append(f"Status: {agent_findings.get('status', 'UNKNOWN')}")
        report.append("")
        report.append("Data Sources:")
        for source in agent_findings.get('data_sources', []):
            report.append(f"  - {source['file']}: {source['type']} ({source.get('method', 'unknown')})")
        report.append("")
        
        if agent_findings.get('issues'):
            report.append("Issues Found:")
            for issue in agent_findings['issues']:
                severity = issue.get('severity', 'UNKNOWN')
                report.append(f"  [{severity}] Line {issue.get('line', '?')}: {issue.get('pattern', 'unknown')}")
                report.append(f"    File: {issue.get('file', 'unknown')}")
                report.append(f"    {issue.get('content', '')[:80]}")
                if 'description' in issue:
                    report.append(f"    Description: {issue['description']}")
        else:
            report.append("No issues found.")
        report.append("")
        
        # Recommendations
        report.append("="*80)
        report.append("RECOMMENDATIONS")
        report.append("="*80)
        report.append("")
        
        if agent_findings.get('verification', {}).get('synthetic_in_main_loop'):
            report.append("CRITICAL: Linux Agent main loop generates synthetic events.")
            report.append("  Action Required:")
            report.append("  1. Replace synthetic event generation in main.rs with real process monitoring")
            report.append("  2. Use the ProcessMonitor from src/process.rs that reads from /proc")
            report.append("  3. Connect syscall monitoring (eBPF/auditd) to feed real events")
            report.append("")
        
        if dpi_findings.get('verification', {}).get('synthetic_data_detected'):
            report.append("WARNING: DPI Probe may contain synthetic data generation code.")
            report.append("  Action: Review and remove any test/synthetic data generation")
            report.append("")
        
        report.append("="*80)
        
        return "\n".join(report)


def main():
    root_path = os.path.dirname(os.path.abspath(__file__))
    verifier = DataVerifier(root_path)
    
    print("Starting RansomEye Data Verification...")
    print(f"Root path: {root_path}")
    
    # Check DPI Probe
    dpi_findings = verifier.check_dpi_probe()
    
    # Check Linux Agent
    agent_findings = verifier.check_linux_agent()
    
    # Generate report
    report = verifier.generate_report(dpi_findings, agent_findings)
    
    # Print report
    print(report)
    
    # Save report
    report_path = Path(root_path) / 'logs' / 'data_verification_report.txt'
    report_path.parent.mkdir(parents=True, exist_ok=True)
    with open(report_path, 'w') as f:
        f.write(report)
    
    # Save JSON findings
    json_path = Path(root_path) / 'logs' / 'data_verification_findings.json'
    with open(json_path, 'w') as f:
        json.dump({
            'dpi_probe': dpi_findings,
            'linux_agent': agent_findings,
        }, f, indent=2)
    
    print(f"\nReport saved to: {report_path}")
    print(f"JSON findings saved to: {json_path}")
    
    # Exit code based on findings
    if agent_findings.get('verification', {}).get('synthetic_in_main_loop'):
        print("\n❌ CRITICAL ISSUE FOUND: Synthetic data generation in Linux Agent main loop")
        return 1
    elif dpi_findings.get('status', '').startswith('FAIL') or agent_findings.get('status', '').startswith('FAIL'):
        print("\n❌ FAILURES DETECTED")
        return 1
    else:
        print("\n✓ Verification complete")
        return 0


if __name__ == '__main__':
    exit(main())

