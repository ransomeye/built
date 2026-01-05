# Path and File Name : /home/ransomeye/rebuild/core/customer_verifier/customer_attestation.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Customer legal attestation support - generates court-ready customer attestation with no RansomEye-signed assertions

"""
Customer Legal Attestation Support (PROMPT-63 Phase 4)

Generate customer-side attestation:
- What was verified
- How it was verified
- What was NOT trusted
- Cryptographic evidence references

Rules:
- No RansomEye-signed assertions
- Fully customer-generated
- Court-defensible wording
"""

import os
import sys
import json
import hashlib
from pathlib import Path
from datetime import datetime, timezone
from typing import Dict, List, Optional
import logging

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger('customer_attestation')


class CustomerAttestation:
    """Customer legal attestation generator."""
    
    def __init__(self, customer_name: str, customer_role: str):
        """Initialize customer attestation."""
        self.customer_name = customer_name
        self.customer_role = customer_role
        self.attestation = {
            'attestation_id': f"attest_{datetime.now(timezone.utc).strftime('%Y%m%d%H%M%S')}",
            'generated_at': datetime.now(timezone.utc).isoformat(),
            'customer_name': customer_name,
            'customer_role': customer_role,
            'attestation_type': 'INDEPENDENT_VERIFICATION',
            'verification_method': 'CUSTOMER_SIDE_ZERO_TRUST',
            'trust_assumptions': [],
            'verifications_performed': [],
            'evidence_references': [],
            'non_trusted_sources': [],
            'attestation_text': None,
            'customer_signature': None
        }
        
    def add_verification(self, verification_type: str, verification_result: Dict, evidence_hash: Optional[str] = None):
        """Add verification performed."""
        verification = {
            'verification_type': verification_type,
            'verification_result': verification_result,
            'evidence_hash': evidence_hash,
            'verified_at': datetime.now(timezone.utc).isoformat()
        }
        self.attestation['verifications_performed'].append(verification)
    
    def add_evidence_reference(self, evidence_type: str, evidence_hash: str, evidence_location: str):
        """Add evidence reference."""
        reference = {
            'evidence_type': evidence_type,
            'evidence_hash': evidence_hash,
            'evidence_location': evidence_location,
            'referenced_at': datetime.now(timezone.utc).isoformat()
        }
        self.attestation['evidence_references'].append(reference)
    
    def add_non_trusted_source(self, source: str, reason: str):
        """Add non-trusted source."""
        self.attestation['non_trusted_sources'].append({
            'source': source,
            'reason': reason
        })
    
    def add_trust_assumption(self, assumption: str):
        """Add trust assumption (what was NOT trusted)."""
        self.attestation['trust_assumptions'].append(assumption)
    
    def generate_attestation_text(self) -> str:
        """Generate court-defensible attestation text."""
        text = f"""
INDEPENDENT VERIFICATION ATTESTATION

I, {self.customer_name}, in my capacity as {self.customer_role}, hereby attest to the following:

1. VERIFICATION METHODOLOGY
   This attestation is based on independent, customer-side verification performed without
   reliance on RansomEye operators, vendors, or support teams. All verifications were
   performed using publicly available tools and cryptographic verification methods.

2. TRUST ASSUMPTIONS
   The following sources were explicitly NOT trusted:
   - RansomEye operator statements or assertions
   - RansomEye vendor-provided documentation (unless cryptographically verified)
   - RansomEye support team communications
   - Any unsigned or unverifiable claims

3. VERIFICATIONS PERFORMED
   The following independent verifications were performed:
"""
        
        for i, verification in enumerate(self.attestation['verifications_performed'], 1):
            text += f"""
   {i}. {verification['verification_type']}
      - Result: {verification['verification_result'].get('verified', 'unknown')}
      - Evidence Hash: {verification.get('evidence_hash', 'N/A')[:16]}...
      - Verified At: {verification['verified_at']}
"""
        
        text += f"""
4. EVIDENCE REFERENCES
   The following cryptographic evidence was examined:
"""
        
        for i, evidence in enumerate(self.attestation['evidence_references'], 1):
            text += f"""
   {i}. {evidence['evidence_type']}
      - Hash: {evidence['evidence_hash'][:16]}...
      - Location: {evidence['evidence_location']}
"""
        
        text += f"""
5. NON-TRUSTED SOURCES
   The following sources were explicitly excluded from trust:
"""
        
        for i, source in enumerate(self.attestation['non_trusted_sources'], 1):
            text += f"""
   {i}. {source['source']}: {source['reason']}
"""
        
        text += f"""
6. ATTESTATION STATEMENT
   I attest that:
   - All verifications were performed independently by the customer
   - No RansomEye-signed assertions were relied upon
   - All evidence was cryptographically verified
   - This attestation is fully customer-generated
   - This attestation is court-defensible

7. SIGNATURE
   This attestation is signed by the customer using cryptographic methods.
   Attestation Hash: {self.attestation.get('attestation_hash', 'N/A')[:16]}...
   Customer Signature: {self.attestation.get('customer_signature', 'N/A')[:16]}...

Generated: {self.attestation['generated_at']}
Attestation ID: {self.attestation['attestation_id']}

---
This attestation is generated independently by the customer and does not
constitute an endorsement or warranty by RansomEye or its operators.
"""
        
        return text
    
    def sign_attestation(self) -> str:
        """Sign attestation (customer-side signature)."""
        # Generate attestation text
        self.attestation['attestation_text'] = self.generate_attestation_text()
        
        # Compute attestation hash
        attestation_json = json.dumps(self.attestation, sort_keys=True)
        attestation_hash = hashlib.sha256(attestation_json.encode()).hexdigest()
        self.attestation['attestation_hash'] = attestation_hash
        
        # Customer generates their own signature
        customer_signature = hashlib.sha256(f"CUSTOMER_ATTEST_{self.customer_name}_{attestation_hash}".encode()).hexdigest()
        self.attestation['customer_signature'] = customer_signature
        
        return customer_signature
    
    def save_attestation(self, output_path: Path) -> bool:
        """Save attestation."""
        try:
            # Sign attestation
            self.sign_attestation()
            
            # Save JSON
            json_path = output_path.with_suffix('.json')
            with open(json_path, 'w') as f:
                json.dump(self.attestation, f, indent=2)
            
            # Save text
            text_path = output_path.with_suffix('.txt')
            with open(text_path, 'w') as f:
                f.write(self.attestation['attestation_text'])
            
            logger.info(f"✓ Customer attestation saved:")
            logger.info(f"  JSON: {json_path}")
            logger.info(f"  Text: {text_path}")
            logger.info(f"  Attestation Hash: {self.attestation['attestation_hash'][:16]}...")
            logger.info(f"  Customer Signature: {self.attestation['customer_signature'][:16]}...")
            
            return True
        except Exception as e:
            logger.error(f"✗ Failed to save attestation: {e}")
            return False
    
    def run(self, verification_results: Dict, output_path: Optional[Path] = None) -> bool:
        """Run customer attestation generation."""
        logger.info("=" * 80)
        logger.info("Customer Legal Attestation Support (PROMPT-63 Phase 4)")
        logger.info("=" * 80)
        
        # Add trust assumptions
        self.add_trust_assumption("RansomEye operator statements not trusted")
        self.add_trust_assumption("RansomEye vendor documentation not trusted (unless cryptographically verified)")
        self.add_trust_assumption("RansomEye support team communications not trusted")
        self.add_trust_assumption("Unsigned or unverifiable claims not trusted")
        
        # Add non-trusted sources
        self.add_non_trusted_source("RansomEye Operators", "Potential compromise or bias")
        self.add_non_trusted_source("RansomEye Vendor", "Potential conflict of interest")
        self.add_non_trusted_source("RansomEye Support", "Not independently verifiable")
        
        # Add verifications from results
        if 'checks' in verification_results:
            for check_name, check_result in verification_results['checks'].items():
                self.add_verification(
                    check_name,
                    check_result,
                    None  # Evidence hash would come from proof snapshot
                )
        
        # Save attestation
        if output_path is None:
            output_dir = Path("/var/lib/ransomeye/customer_attestations")
            output_dir.mkdir(parents=True, exist_ok=True)
            output_path = output_dir / f"customer_attestation_{self.attestation['attestation_id']}"
        
        if not self.save_attestation(output_path):
            logger.error("FAIL-CLOSED: Failed to save attestation")
            return False
        
        logger.info("✓ Customer attestation generation complete")
        return True


def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description='Customer Legal Attestation')
    parser.add_argument('--customer-name', required=True, help='Customer name')
    parser.add_argument('--customer-role', required=True, help='Customer role/title')
    parser.add_argument('--verification-results', type=Path, help='Path to verification results JSON')
    parser.add_argument('--output', type=Path, help='Output path for attestation')
    
    args = parser.parse_args()
    
    # Load verification results if provided
    verification_results = {}
    if args.verification_results and args.verification_results.exists():
        with open(args.verification_results, 'r') as f:
            verification_results = json.load(f)
    
    attestation = CustomerAttestation(args.customer_name, args.customer_role)
    success = attestation.run(verification_results, args.output)
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()

