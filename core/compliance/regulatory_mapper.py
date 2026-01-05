# Path and File Name : /home/ransomeye/rebuild/core/compliance/regulatory_mapper.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Regulatory mapper - automated mapping of internal controls to regulations with evidence auto-linking

"""
Regulatory Mapper (PROMPT-62 Phase 2)

Automated mapping:
- Internal controls → regulations
- Evidence → controls

Initial mappings:
- ISO 27001
- SOC 2 (Type II)
- NIST 800-53
- GDPR (technical controls only)
- RBI Cyber Security Framework (India)

Rules:
- Mapping is data-driven
- Evidence auto-linked
- No manual spreadsheets
"""

import os
import sys
import json
import psycopg2
from pathlib import Path
from datetime import datetime, timezone
from typing import Dict, List, Optional, Set
import logging

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger('regulatory_mapper')

# Database configuration
DB_NAME = os.environ.get("DB_NAME", "ransomeye")
DB_USER = os.environ.get("DB_USER", "gagan")
DB_HOST = os.environ.get("DB_HOST", "localhost")
DB_PASSWORD = os.environ.get("DB_PASS", "gagan")
DB_PORT = int(os.environ.get("DB_PORT", "5432"))


class RegulatoryMapper:
    """Regulatory mapping engine."""
    
    # Control-to-regulation mappings
    CONTROL_MAPPINGS = {
        # ISO 27001
        'A.9.1.1': {
            'iso27001': 'A.9.1.1 - Access control policy',
            'soc2': 'CC6.1 - Logical and physical access controls',
            'nist80053': 'AC-1 - Access Control Policy',
            'gdpr': 'Article 32 - Security of processing',
            'rbi': 'Control 4.1 - Access Management'
        },
        'A.9.2.1': {
            'iso27001': 'A.9.2.1 - User registration and de-registration',
            'soc2': 'CC6.1 - Logical access controls',
            'nist80053': 'AC-2 - Account Management',
            'gdpr': 'Article 32 - Security of processing',
            'rbi': 'Control 4.1 - Access Management'
        },
        'A.12.4.1': {
            'iso27001': 'A.12.4.1 - Event logging',
            'soc2': 'CC7.2 - System monitoring',
            'nist80053': 'AU-2 - Audit Events',
            'gdpr': 'Article 30 - Records of processing activities',
            'rbi': 'Control 5.1 - Logging and Monitoring'
        },
        'A.12.6.1': {
            'iso27001': 'A.12.6.1 - Management of technical vulnerabilities',
            'soc2': 'CC7.4 - System monitoring',
            'nist80053': 'SI-2 - Flaw Remediation',
            'gdpr': 'Article 32 - Security of processing',
            'rbi': 'Control 6.1 - Vulnerability Management'
        },
        'A.18.1.1': {
            'iso27001': 'A.18.1.1 - Identification of applicable legislation',
            'soc2': 'CC1.1 - Control environment',
            'nist80053': 'PM-1 - Information Security Program Plan',
            'gdpr': 'Article 30 - Records of processing activities',
            'rbi': 'Control 1.1 - Governance Framework'
        },
        
        # SOC 2
        'CC6.1': {
            'iso27001': 'A.9.1.1 - Access control policy',
            'soc2': 'CC6.1 - Logical and physical access controls',
            'nist80053': 'AC-1 - Access Control Policy',
            'gdpr': 'Article 32 - Security of processing',
            'rbi': 'Control 4.1 - Access Management'
        },
        'CC7.2': {
            'iso27001': 'A.12.4.1 - Event logging',
            'soc2': 'CC7.2 - System monitoring',
            'nist80053': 'AU-2 - Audit Events',
            'gdpr': 'Article 30 - Records of processing activities',
            'rbi': 'Control 5.1 - Logging and Monitoring'
        },
        
        # NIST 800-53
        'AC-1': {
            'iso27001': 'A.9.1.1 - Access control policy',
            'soc2': 'CC6.1 - Logical and physical access controls',
            'nist80053': 'AC-1 - Access Control Policy',
            'gdpr': 'Article 32 - Security of processing',
            'rbi': 'Control 4.1 - Access Management'
        },
        'AU-2': {
            'iso27001': 'A.12.4.1 - Event logging',
            'soc2': 'CC7.2 - System monitoring',
            'nist80053': 'AU-2 - Audit Events',
            'gdpr': 'Article 30 - Records of processing activities',
            'rbi': 'Control 5.1 - Logging and Monitoring'
        },
        'SI-2': {
            'iso27001': 'A.12.6.1 - Management of technical vulnerabilities',
            'soc2': 'CC7.4 - System monitoring',
            'nist80053': 'SI-2 - Flaw Remediation',
            'gdpr': 'Article 32 - Security of processing',
            'rbi': 'Control 6.1 - Vulnerability Management'
        },
        
        # GDPR
        'GDPR-32': {
            'iso27001': 'A.12.6.1 - Management of technical vulnerabilities',
            'soc2': 'CC7.4 - System monitoring',
            'nist80053': 'SI-2 - Flaw Remediation',
            'gdpr': 'Article 32 - Security of processing',
            'rbi': 'Control 6.1 - Vulnerability Management'
        },
        'GDPR-30': {
            'iso27001': 'A.12.4.1 - Event logging',
            'soc2': 'CC7.2 - System monitoring',
            'nist80053': 'AU-2 - Audit Events',
            'gdpr': 'Article 30 - Records of processing activities',
            'rbi': 'Control 5.1 - Logging and Monitoring'
        },
        
        # RBI
        'RBI-4.1': {
            'iso27001': 'A.9.1.1 - Access control policy',
            'soc2': 'CC6.1 - Logical and physical access controls',
            'nist80053': 'AC-1 - Access Control Policy',
            'gdpr': 'Article 32 - Security of processing',
            'rbi': 'Control 4.1 - Access Management'
        },
        'RBI-5.1': {
            'iso27001': 'A.12.4.1 - Event logging',
            'soc2': 'CC7.2 - System monitoring',
            'nist80053': 'AU-2 - Audit Events',
            'gdpr': 'Article 30 - Records of processing activities',
            'rbi': 'Control 5.1 - Logging and Monitoring'
        }
    }
    
    # Internal controls
    INTERNAL_CONTROLS = {
        'access_control': {
            'description': 'Access control policy and enforcement',
            'evidence_sources': ['immutable_audit_log', 'components', 'agents'],
            'mappings': ['A.9.1.1', 'CC6.1', 'AC-1', 'GDPR-32', 'RBI-4.1']
        },
        'audit_logging': {
            'description': 'Immutable audit logging with chain hashing',
            'evidence_sources': ['immutable_audit_log'],
            'mappings': ['A.12.4.1', 'CC7.2', 'AU-2', 'GDPR-30', 'RBI-5.1']
        },
        'vulnerability_management': {
            'description': 'Technical vulnerability management',
            'evidence_sources': ['verifier_results', 'drift_snapshot'],
            'mappings': ['A.12.6.1', 'CC7.4', 'SI-2', 'GDPR-32', 'RBI-6.1']
        },
        'data_encryption': {
            'description': 'Data encryption at rest and in transit',
            'evidence_sources': ['immutable_audit_log', 'model_registry'],
            'mappings': ['A.10.1.1', 'CC6.7', 'SC-28', 'GDPR-32', 'RBI-3.1']
        },
        'change_control': {
            'description': 'Change control and management',
            'evidence_sources': ['immutable_audit_log', 'drift_snapshot'],
            'mappings': ['A.12.5.1', 'CC7.3', 'CM-3', 'GDPR-32', 'RBI-7.1']
        }
    }
    
    def __init__(self):
        """Initialize regulatory mapper."""
        self.conn = None
        
    def connect_db(self) -> bool:
        """Connect to database."""
        try:
            self.conn = psycopg2.connect(
                host=DB_HOST,
                port=DB_PORT,
                database=DB_NAME,
                user=DB_USER,
                password=DB_PASSWORD
            )
            logger.info("✓ Connected to database")
            return True
        except Exception as e:
            logger.error(f"✗ Failed to connect to database: {e}")
            return False
    
    def get_control_evidence(self, control_id: str) -> List[Dict]:
        """Get evidence for a control."""
        if not self.conn:
            return []
        
        control = self.INTERNAL_CONTROLS.get(control_id)
        if not control:
            return []
        
        evidence = []
        cursor = self.conn.cursor()
        
        try:
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Get evidence from audit log
            if 'immutable_audit_log' in control['evidence_sources']:
                cursor.execute("""
                    SELECT 
                        audit_id, created_at, action, object_type,
                        payload_sha256, chain_hash_sha256
                    FROM immutable_audit_log
                    WHERE action LIKE %s
                    ORDER BY created_at DESC
                    LIMIT 10
                """, (f'%{control_id}%',))
                
                for row in cursor.fetchall():
                    evidence.append({
                        'source': 'immutable_audit_log',
                        'audit_id': str(row[0]),
                        'created_at': row[1].isoformat() if row[1] else None,
                        'action': row[2],
                        'object_type': row[3],
                        'payload_hash': row[4].hex() if row[4] else None,
                        'chain_hash': row[5].hex() if row[5] else None
                    })
            
            # Get evidence from verifier results
            if 'verifier_results' in control['evidence_sources']:
                verifier_path = Path("/var/log/ransomeye/verifier_results.json")
                if verifier_path.exists():
                    try:
                        with open(verifier_path, 'r') as f:
                            verifier_data = json.load(f)
                        evidence.append({
                            'source': 'verifier_results',
                            'timestamp': verifier_data.get('timestamp'),
                            'overall_healthy': verifier_data.get('overall_healthy'),
                            'checks': list(verifier_data.get('checks', {}).keys())
                        })
                    except Exception as e:
                        logger.warning(f"⚠ Failed to read verifier results: {e}")
            
        except Exception as e:
            logger.error(f"✗ Failed to get control evidence: {e}")
        finally:
            cursor.close()
        
        return evidence
    
    def map_control_to_regulations(self, control_id: str) -> Dict:
        """Map internal control to regulations."""
        control = self.INTERNAL_CONTROLS.get(control_id)
        if not control:
            return {}
        
        mappings = {}
        for mapping_id in control['mappings']:
            mapping = self.CONTROL_MAPPINGS.get(mapping_id, {})
            if mapping:
                for reg_type, reg_control in mapping.items():
                    if reg_type not in mappings:
                        mappings[reg_type] = []
                    mappings[reg_type].append({
                        'control_id': mapping_id,
                        'control_name': reg_control
                    })
        
        return mappings
    
    def generate_regulatory_report(self, regulation: Optional[str] = None) -> Dict:
        """Generate regulatory compliance report."""
        report = {
            'generated_at': datetime.now(timezone.utc).isoformat(),
            'regulation': regulation or 'all',
            'controls': {}
        }
        
        for control_id, control_info in self.INTERNAL_CONTROLS.items():
            # Get mappings
            mappings = self.map_control_to_regulations(control_id)
            
            # Filter by regulation if specified
            if regulation and regulation not in mappings:
                continue
            
            # Get evidence
            evidence = self.get_control_evidence(control_id)
            
            report['controls'][control_id] = {
                'description': control_info['description'],
                'mappings': mappings,
                'evidence_count': len(evidence),
                'evidence': evidence[:5]  # Limit to 5 samples
            }
        
        return report
    
    def save_mapping(self, output_path: Path):
        """Save regulatory mapping to file."""
        mapping_data = {
            'generated_at': datetime.now(timezone.utc).isoformat(),
            'control_mappings': self.CONTROL_MAPPINGS,
            'internal_controls': self.INTERNAL_CONTROLS,
            'regulations': {
                'iso27001': 'ISO/IEC 27001:2022',
                'soc2': 'SOC 2 Type II',
                'nist80053': 'NIST SP 800-53 Rev. 5',
                'gdpr': 'GDPR (General Data Protection Regulation)',
                'rbi': 'RBI Cyber Security Framework (India)'
            }
        }
        
        with open(output_path, 'w') as f:
            json.dump(mapping_data, f, indent=2)
        
        logger.info(f"✓ Regulatory mapping saved: {output_path}")
    
    def run(self, regulation: Optional[str] = None, output_path: Optional[Path] = None) -> bool:
        """Run regulatory mapping."""
        logger.info("=" * 80)
        logger.info("Regulatory Mapper (PROMPT-62 Phase 2)")
        logger.info("=" * 80)
        
        # Connect to database
        if not self.connect_db():
            logger.error("FAIL-CLOSED: Database connection failed")
            return False
        
        # Generate report
        report = self.generate_regulatory_report(regulation)
        
        # Save report
        if output_path is None:
            output_dir = Path("/var/lib/ransomeye/compliance")
            output_dir.mkdir(parents=True, exist_ok=True)
            output_path = output_dir / f"regulatory_report_{datetime.now(timezone.utc).strftime('%Y%m%d%H%M%S')}.json"
        
        with open(output_path, 'w') as f:
            json.dump(report, f, indent=2)
        
        logger.info(f"✓ Regulatory report generated: {output_path}")
        logger.info(f"  Controls mapped: {len(report['controls'])}")
        
        # Save mapping
        mapping_path = output_path.parent / "regulatory_mapping.json"
        self.save_mapping(mapping_path)
        
        return True


def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description='Regulatory Mapper')
    parser.add_argument('--regulation', help='Filter by regulation (iso27001, soc2, nist80053, gdpr, rbi)')
    parser.add_argument('--output', type=Path, help='Output path for report')
    
    args = parser.parse_args()
    
    mapper = RegulatoryMapper()
    success = mapper.run(args.regulation, args.output)
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()

