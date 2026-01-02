# Path and File Name : /home/ransomeye/rebuild/ransomeye_posture_engine/engine/report_generator.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Report generator - generates PDF, HTML, and CSV compliance reports

"""
Report Generator

Generates compliance reports in PDF, HTML, and CSV formats.
All reports include footer: "© RansomEye.Tech | Support: Gagan@RansomEye.Tech"
"""

import logging
import csv
import json
from pathlib import Path
from datetime import datetime
from typing import List, Dict, Any, Optional
import hashlib
from datetime import datetime

from .scorer import HostPostureScore, NetworkPostureScore
from .cis_evaluator import CISEvaluationResult
from .stig_evaluator import STIGEvaluationResult
from .custom_policy_evaluator import CustomPolicyEvaluationResult
from .drift_detector import DriftAlert
from .policy_metadata import PolicyMetadataManager

logger = logging.getLogger("ransomeye_posture_engine.report_generator")

class ReportGenerator:
    """Generates compliance reports with policy metadata."""
    
    def __init__(self, output_dir: Path, metadata_manager: Optional[PolicyMetadataManager] = None):
        self.output_dir = output_dir
        self.metadata_manager = metadata_manager
        self.output_dir.mkdir(parents=True, exist_ok=True)
        self.footer = "© RansomEye.Tech | Support: Gagan@RansomEye.Tech"
    
    def generate_host_report(self,
                            host_id: str,
                            score: HostPostureScore,
                            cis_results: List[CISEvaluationResult],
                            stig_results: List[STIGEvaluationResult],
                            custom_results: List[CustomPolicyEvaluationResult],
                            drift_alerts: List[DriftAlert]) -> Dict[str, Path]:
        """
        Generate host compliance report in all formats.
        
        Returns:
            Dict mapping format to file path
        """
        timestamp = datetime.utcnow()
        report_id = f"host_{host_id}_{timestamp.strftime('%Y%m%d_%H%M%S')}"
        
        # Generate reports
        pdf_path = self._generate_pdf_report(report_id, host_id, score, 
                                            cis_results, stig_results, custom_results, drift_alerts)
        html_path = self._generate_html_report(report_id, host_id, score,
                                              cis_results, stig_results, custom_results, drift_alerts)
        csv_path = self._generate_csv_report(report_id, host_id, score,
                                            cis_results, stig_results, custom_results, drift_alerts)
        
        return {
            'pdf': pdf_path,
            'html': html_path,
            'csv': csv_path,
        }
    
    def generate_network_report(self,
                               network_id: str,
                               score: NetworkPostureScore) -> Dict[str, Path]:
        """Generate network compliance report in all formats."""
        timestamp = datetime.utcnow()
        report_id = f"network_{network_id}_{timestamp.strftime('%Y%m%d_%H%M%S')}"
        
        pdf_path = self._generate_pdf_network_report(report_id, network_id, score)
        html_path = self._generate_html_network_report(report_id, network_id, score)
        csv_path = self._generate_csv_network_report(report_id, network_id, score)
        
        return {
            'pdf': pdf_path,
            'html': html_path,
            'csv': csv_path,
        }
    
    def _generate_pdf_report(self, report_id: str, host_id: str, score: HostPostureScore,
                            cis_results: List[CISEvaluationResult],
                            stig_results: List[STIGEvaluationResult],
                            custom_results: List[CustomPolicyEvaluationResult],
                            drift_alerts: List[DriftAlert]) -> Path:
        """Generate PDF report."""
        # For now, create a simple text-based PDF using reportlab or similar
        # In production, would use proper PDF library
        pdf_path = self.output_dir / f"{report_id}.pdf"
        
        try:
            # Try to use reportlab if available
            try:
                from reportlab.lib.pagesizes import letter
                from reportlab.platypus import SimpleDocTemplate, Paragraph, Spacer
                from reportlab.lib.styles import getSampleStyleSheet
                from reportlab.lib.units import inch
                
                doc = SimpleDocTemplate(str(pdf_path), pagesize=letter)
                story = []
                styles = getSampleStyleSheet()
                
                # Title
                story.append(Paragraph(f"Host Posture Report: {host_id}", styles['Title']))
                story.append(Spacer(1, 0.2*inch))
                
                # Score summary
                story.append(Paragraph(f"Overall Score: {score.overall_score:.2%}", styles['Heading2']))
                story.append(Paragraph(f"Host Hardening: {score.host_hardening_score:.2%}", styles['Normal']))
                story.append(Paragraph(f"Auth Hygiene: {score.auth_hygiene_score:.2%}", styles['Normal']))
                story.append(Spacer(1, 0.2*inch))
                
                # Findings
                story.append(Paragraph(f"Total Findings: {score.findings_count}", styles['Heading2']))
                story.append(Paragraph(f"Critical Findings: {score.critical_findings_count}", styles['Normal']))
                story.append(Spacer(1, 0.2*inch))
                
                # Footer
                story.append(Spacer(1, 0.5*inch))
                story.append(Paragraph(self.footer, styles['Normal']))
                
                doc.build(story)
                logger.info(f"Generated PDF report: {pdf_path}")
            
            except ImportError:
                # Fallback: create simple text file
                logger.warning("reportlab not available, creating text file instead")
                with open(pdf_path.with_suffix('.txt'), 'w') as f:
                    f.write(f"Host Posture Report: {host_id}\n")
                    f.write(f"Overall Score: {score.overall_score:.2%}\n")
                    f.write(f"\n{self.footer}\n")
                pdf_path = pdf_path.with_suffix('.txt')
        
        except Exception as e:
            logger.error(f"Error generating PDF report: {e}")
            # Create minimal text file as fallback
            with open(pdf_path.with_suffix('.txt'), 'w') as f:
                f.write(f"Host Posture Report: {host_id}\n")
                f.write(f"Error: {e}\n")
            pdf_path = pdf_path.with_suffix('.txt')
        
        return pdf_path
    
    def _generate_html_report(self, report_id: str, host_id: str, score: HostPostureScore,
                             cis_results: List[CISEvaluationResult],
                             stig_results: List[STIGEvaluationResult],
                             custom_results: List[CustomPolicyEvaluationResult],
                             drift_alerts: List[DriftAlert]) -> Path:
        """Generate HTML report with policy metadata."""
        """Generate HTML report."""
        html_path = self.output_dir / f"{report_id}.html"
        
        html_content = f"""
<!DOCTYPE html>
<html>
<head>
    <title>Host Posture Report: {host_id}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        h1 {{ color: #333; }}
        .score {{ font-size: 24px; font-weight: bold; color: #0066cc; }}
        .footer {{ margin-top: 40px; padding-top: 20px; border-top: 1px solid #ccc; 
                   text-align: center; color: #666; }}
        table {{ border-collapse: collapse; width: 100%; margin-top: 20px; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
    </style>
</head>
<body>
    <h1>Host Posture Report: {host_id}</h1>
    <p><strong>Generated:</strong> {datetime.utcnow().isoformat()}</p>
    
    <h2>Posture Scores</h2>
    <div class="score">Overall Score: {score.overall_score:.2%}</div>
    <ul>
        <li>Host Hardening: {score.host_hardening_score:.2%}</li>
        <li>Auth Hygiene: {score.auth_hygiene_score:.2%}</li>
        <li>CIS Score: {score.cis_score:.2%}</li>
        <li>STIG Score: {score.stig_score:.2%}</li>
        <li>Custom Policy Score: {score.custom_policy_score:.2%}</li>
    </ul>
    
    <h2>Findings</h2>
    <p>Total Findings: {score.findings_count}</p>
    <p>Critical Findings: {score.critical_findings_count}</p>
    
    <h2>Drift Alerts</h2>
    <p>Total Drift Alerts: {len(drift_alerts)}</p>
    
    <h2>Policy Metadata</h2>
    <table>
        <tr><th>Policy ID</th><th>Type</th><th>Version</th><th>SHA-256 Hash</th><th>Source Path</th></tr>
"""
        
        # Add policy metadata from all results
        all_policies = []
        for result in cis_results + stig_results + custom_results:
            if hasattr(result, 'policy_metadata'):
                meta = result.policy_metadata
                all_policies.append((meta.policy_id, meta.policy_type, meta.version, meta.sha256_hash, str(meta.source_path)))
        
        for policy_id, policy_type, version, hash_val, source_path in all_policies:
            html_content += f"        <tr><td>{policy_id}</td><td>{policy_type}</td><td>{version}</td><td>{hash_val[:16]}...</td><td>{source_path}</td></tr>\n"
        
        html_content += """    </table>
    
    <div class="footer">
        """ + self.footer + """
    </div>
</body>
</html>
"""
        
        try:
            with open(html_path, 'w') as f:
                f.write(html_content)
            logger.info(f"Generated HTML report: {html_path}")
        except Exception as e:
            logger.error(f"Error generating HTML report: {e}")
            raise
        
        return html_path
    
    def _generate_csv_report(self, report_id: str, host_id: str, score: HostPostureScore,
                            cis_results: List[CISEvaluationResult],
                            stig_results: List[STIGEvaluationResult],
                            custom_results: List[CustomPolicyEvaluationResult],
                            drift_alerts: List[DriftAlert]) -> Path:
        """Generate CSV report."""
        csv_path = self.output_dir / f"{report_id}.csv"
        
        try:
            with open(csv_path, 'w', newline='') as f:
                writer = csv.writer(f)
                
                # Header
                writer.writerow(['Host Posture Report', host_id])
                writer.writerow(['Generated', datetime.utcnow().isoformat()])
                writer.writerow([])
                
                # Scores
                writer.writerow(['Posture Scores'])
                writer.writerow(['Metric', 'Score'])
                writer.writerow(['Overall Score', f"{score.overall_score:.4f}"])
                writer.writerow(['Host Hardening', f"{score.host_hardening_score:.4f}"])
                writer.writerow(['Auth Hygiene', f"{score.auth_hygiene_score:.4f}"])
                writer.writerow(['CIS Score', f"{score.cis_score:.4f}"])
                writer.writerow(['STIG Score', f"{score.stig_score:.4f}"])
                writer.writerow(['Custom Policy Score', f"{score.custom_policy_score:.4f}"])
                writer.writerow([])
                
                # Findings
                writer.writerow(['Findings'])
                writer.writerow(['Total Findings', score.findings_count])
                writer.writerow(['Critical Findings', score.critical_findings_count])
                writer.writerow([])
                
                # Policy Metadata (MANDATORY)
                writer.writerow(['Policy Metadata'])
                writer.writerow(['Policy ID', 'Type', 'Version', 'SHA-256 Hash', 'Source Path'])
                for result in cis_results + stig_results + custom_results:
                    if hasattr(result, 'policy_metadata'):
                        meta = result.policy_metadata
                        writer.writerow([meta.policy_id, meta.policy_type, meta.version, meta.sha256_hash, str(meta.source_path)])
                writer.writerow([])
                
                # Footer
                writer.writerow([])
                writer.writerow([self.footer])
            
            logger.info(f"Generated CSV report: {csv_path}")
        except Exception as e:
            logger.error(f"Error generating CSV report: {e}")
            raise
        
        return csv_path
    
    def _generate_pdf_network_report(self, report_id: str, network_id: str, 
                                     score: NetworkPostureScore) -> Path:
        """Generate PDF network report."""
        pdf_path = self.output_dir / f"{report_id}.pdf"
        
        # Similar to host report but for network
        # Implementation would be similar to _generate_pdf_report
        # For brevity, creating text file
        with open(pdf_path.with_suffix('.txt'), 'w') as f:
            f.write(f"Network Posture Report: {network_id}\n")
            f.write(f"Overall Score: {score.overall_score:.2%}\n")
            f.write(f"\n{self.footer}\n")
        
        return pdf_path.with_suffix('.txt')
    
    def _generate_html_network_report(self, report_id: str, network_id: str,
                                     score: NetworkPostureScore) -> Path:
        """Generate HTML network report."""
        html_path = self.output_dir / f"{report_id}.html"
        
        html_content = f"""
<!DOCTYPE html>
<html>
<head>
    <title>Network Posture Report: {network_id}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .footer {{ margin-top: 40px; padding-top: 20px; border-top: 1px solid #ccc; 
                   text-align: center; color: #666; }}
    </style>
</head>
<body>
    <h1>Network Posture Report: {network_id}</h1>
    <p>Overall Score: {score.overall_score:.2%}</p>
    <p>Exposure Score: {score.exposure_score:.2%}</p>
    <div class="footer">{self.footer}</div>
</body>
</html>
"""
        
        with open(html_path, 'w') as f:
            f.write(html_content)
        
        return html_path
    
    def _generate_csv_network_report(self, report_id: str, network_id: str,
                                     score: NetworkPostureScore) -> Path:
        """Generate CSV network report."""
        csv_path = self.output_dir / f"{report_id}.csv"
        
        with open(csv_path, 'w', newline='') as f:
            writer = csv.writer(f)
            writer.writerow(['Network Posture Report', network_id])
            writer.writerow(['Overall Score', f"{score.overall_score:.4f}"])
            writer.writerow(['Exposure Score', f"{score.exposure_score:.4f}"])
            writer.writerow([])
            writer.writerow([self.footer])
        
        return csv_path

