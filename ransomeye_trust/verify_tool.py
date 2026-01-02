# Path and File Name : /home/ransomeye/rebuild/ransomeye_trust/verify_tool.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Generic verifier for signed artifacts and manifests

"""
Verify Tool: Generic verifier for RansomEye signed artifacts.
Verifies manifest.sig signatures and validates trust chain.
"""

import json
from pathlib import Path
from typing import Optional, Dict
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import padding
from cryptography.hazmat.backends import default_backend
from cryptography import x509
from cryptography.exceptions import InvalidSignature


class VerifyTool:
    """Generic verifier for RansomEye artifacts."""
    
    def __init__(self, trust_dir: Optional[str] = None):
        # Runtime-safe default: use the installed module directory (contains certs/ by design).
        if trust_dir is None:
            trust_dir = str(Path(__file__).resolve().parent)
        self.trust_dir = Path(trust_dir)
        self.certs_dir = self.trust_dir / "certs"
        self.root_ca_path = self.certs_dir / "root_ca.crt"
    
    def load_certificate(self, domain: str = "artifacts") -> x509.Certificate:
        """Load certificate for a trust domain."""
        cert_path = self.certs_dir / f"{domain}_signing.crt"
        
        if not cert_path.exists():
            raise FileNotFoundError(f"Certificate not found: {cert_path}")
        
        with open(cert_path, 'rb') as f:
            certificate = x509.load_pem_x509_certificate(f.read(), default_backend())
        
        return certificate
    
    def verify_certificate_chain(self, certificate: x509.Certificate) -> bool:
        """Verify certificate chain against Root CA."""
        if not self.root_ca_path.exists():
            # If no Root CA, skip chain verification (development mode)
            return True
        
        with open(self.root_ca_path, 'rb') as f:
            root_ca = x509.load_pem_x509_certificate(f.read(), default_backend())
        
        # Verify certificate is signed by Root CA
        try:
            # cryptography >= 42 uses `signature_hash_algorithm`; older code used `signature_algorithm_hash`
            hash_alg = getattr(certificate, "signature_hash_algorithm", None)
            if hash_alg is None:
                hash_alg = getattr(certificate, "signature_algorithm_hash", None)
            if hash_alg is None:
                raise RuntimeError("Certificate object missing signature hash algorithm attribute (FAIL-CLOSED)")

            root_ca.public_key().verify(
                certificate.signature,
                certificate.tbs_certificate_bytes,
                padding.PKCS1v15(),
                hash_alg,
            )
            return True
        except InvalidSignature:
            return False
    
    def verify_signature(self, data: bytes, signature: bytes, certificate: x509.Certificate) -> bool:
        """
        Verify signature against certificate public key.
        
        Args:
            data: Original data
            signature: Signature bytes
            certificate: Certificate containing public key
        
        Returns:
            True if signature is valid
        """
        try:
            certificate.public_key().verify(
                signature,
                data,
                padding.PSS(
                    mgf=padding.MGF1(hashes.SHA256()),
                    salt_length=padding.PSS.MAX_LENGTH
                ),
                hashes.SHA256()
            )
            return True
        except InvalidSignature:
            return False
    
    def verify_file(self, file_path: Path, sig_path: Optional[Path] = None, domain: str = "artifacts") -> bool:
        """
        Verify file signature.
        
        Args:
            file_path: Path to file
            sig_path: Path to signature file (default: file_path.sig)
            domain: Trust domain
        
        Returns:
            True if signature is valid
        """
        if sig_path is None:
            sig_path = file_path.parent / f"{file_path.stem}.sig"
        
        if not sig_path.exists():
            return False
        
        # Load certificate
        certificate = self.load_certificate(domain)
        
        # Verify certificate chain
        if not self.verify_certificate_chain(certificate):
            return False
        
        # Read file and signature
        with open(file_path, 'rb') as f:
            file_data = f.read()
        
        with open(sig_path, 'rb') as f:
            signature = f.read()
        
        # Verify signature
        return self.verify_signature(file_data, signature, certificate)
    
    def verify_manifest(self, manifest_path: Path, domain: Optional[str] = None) -> Dict:
        """
        Verify manifest.json and its signature.
        
        Args:
            manifest_path: Path to manifest.json
            domain: Trust domain (if None, read from manifest)
        
        Returns:
            Dictionary with verification results
        """
        sig_path = manifest_path.parent / f"{manifest_path.stem}.sig"
        
        if not sig_path.exists():
            return {
                'valid': False,
                'error': 'Signature file not found'
            }
        
        # Load manifest
        try:
            with open(manifest_path, 'r') as f:
                manifest = json.load(f)
        except Exception as e:
            return {
                'valid': False,
                'error': f'Error reading manifest: {str(e)}'
            }
        
        # Get domain from manifest if not provided
        if domain is None:
            domain = manifest.get('signer', 'artifacts')
        
        # Verify manifest signature
        if not self.verify_file(manifest_path, sig_path, domain):
            return {
                'valid': False,
                'error': 'Invalid signature'
            }
        
        # Verify certificate chain
        try:
            certificate = self.load_certificate(domain)
            if not self.verify_certificate_chain(certificate):
                return {
                    'valid': False,
                    'error': 'Certificate chain verification failed'
                }
        except Exception as e:
            return {
                'valid': False,
                'error': f'Certificate error: {str(e)}'
            }
        
        return {
            'valid': True,
            'manifest': manifest,
            'domain': domain
        }
    
    def verify_artifact(self, artifact_path: Path) -> Dict:
        """
        Verify artifact with its manifest and signature.
        
        Args:
            artifact_path: Path to artifact file
        
        Returns:
            Dictionary with verification results
        """
        # Find manifest
        manifest_path = artifact_path.parent / f"{artifact_path.stem}_manifest.json"
        if not manifest_path.exists():
            # Try generic manifest
            manifest_path = artifact_path.parent / "manifest.json"
            if not manifest_path.exists():
                return {
                    'valid': False,
                    'error': 'Manifest not found'
                }
        
        # Verify manifest
        manifest_result = self.verify_manifest(manifest_path)
        if not manifest_result.get('valid'):
            return manifest_result
        
        manifest = manifest_result['manifest']
        
        # Verify artifact hash matches manifest
        import hashlib
        sha256 = hashlib.sha256()
        with open(artifact_path, 'rb') as f:
            for chunk in iter(lambda: f.read(4096), b''):
                sha256.update(chunk)
        computed_hash = sha256.hexdigest()
        
        if computed_hash != manifest.get('hash'):
            return {
                'valid': False,
                'error': 'Artifact hash mismatch'
            }
        
        return {
            'valid': True,
            'manifest': manifest,
            'domain': manifest_result['domain'],
            'hash': computed_hash
        }


def main():
    """CLI entry point for verify tool."""
    import argparse
    
    parser = argparse.ArgumentParser(description='RansomEye Verify Tool')
    parser.add_argument('file', type=Path, help='File or manifest to verify')
    parser.add_argument('--domain', default=None, help='Trust domain (auto-detect from manifest if not provided)')
    parser.add_argument('--manifest-only', action='store_true', help='Verify manifest only, not artifact')
    
    args = parser.parse_args()
    
    verifier = VerifyTool()
    
    if args.file.suffix == '.json' or 'manifest' in args.file.name:
        # Verify manifest
        result = verifier.verify_manifest(args.file, args.domain)
    else:
        # Verify artifact
        result = verifier.verify_artifact(args.file)
    
    if result.get('valid'):
        print("✓ Verification successful")
        if 'manifest' in result:
            print(f"  Version: {result['manifest'].get('version')}")
            print(f"  Domain: {result.get('domain')}")
            if 'hash' in result:
                print(f"  Hash: {result['hash'][:16]}...")
    else:
        print(f"✗ Verification failed: {result.get('error')}")
        exit(1)


if __name__ == '__main__':
    main()

