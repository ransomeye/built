# Path and File Name : /home/ransomeye/rebuild/ransomeye_trust/root_ca_generator.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Generates offline Root CA and certificate hierarchy for RansomEye trust infrastructure

"""
Root CA Generator: Creates offline Root CA and certificate hierarchy.
All certificates are RSA-4096 for maximum security.
"""

import os
from pathlib import Path
from datetime import datetime, timedelta
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography import x509
from cryptography.x509.oid import NameOID
import json
from typing import Dict


class RootCAGenerator:
    """Generates Root CA and certificate hierarchy."""
    
    def __init__(self, trust_dir: str = "/home/ransomeye/rebuild/ransomeye_trust"):
        self.trust_dir = Path(trust_dir)
        self.trust_dir.mkdir(parents=True, exist_ok=True)
        self.keys_dir = self.trust_dir / "keys"
        self.certs_dir = self.trust_dir / "certs"
        self.keys_dir.mkdir(exist_ok=True)
        self.certs_dir.mkdir(exist_ok=True)
    
    def generate_root_ca(self, validity_years: int = 10) -> tuple:
        """
        Generate Root CA certificate and private key.
        
        Args:
            validity_years: Certificate validity period in years
        
        Returns:
            Tuple of (private_key, certificate)
        """
        # Generate private key (RSA-4096)
        private_key = rsa.generate_private_key(
            public_exponent=65537,
            key_size=4096,
        )
        
        # Create self-signed certificate
        subject = issuer = x509.Name([
            x509.NameAttribute(NameOID.COUNTRY_NAME, "US"),
            x509.NameAttribute(NameOID.STATE_OR_PROVINCE_NAME, "CA"),
            x509.NameAttribute(NameOID.LOCALITY_NAME, "San Francisco"),
            x509.NameAttribute(NameOID.ORGANIZATION_NAME, "RansomEye"),
            x509.NameAttribute(NameOID.COMMON_NAME, "RansomEye Root CA"),
        ])
        
        cert = x509.CertificateBuilder().subject_name(
            subject
        ).issuer_name(
            issuer
        ).public_key(
            private_key.public_key()
        ).serial_number(
            x509.random_serial_number()
        ).not_valid_before(
            datetime.utcnow()
        ).not_valid_after(
            datetime.utcnow() + timedelta(days=validity_years * 365)
        ).add_extension(
            x509.BasicConstraints(ca=True, path_length=None),
            critical=True,
        ).add_extension(
            x509.KeyUsage(
                key_cert_sign=True,
                crl_sign=True,
                digital_signature=True,
                key_encipherment=False,
                content_commitment=False,
                data_encipherment=False,
                key_agreement=False,
                encipher_only=False,
                decipher_only=False,
            ),
            critical=True,
        ).sign(private_key, hashes.SHA256())
        
        return private_key, cert
    
    def save_root_ca(self, private_key, certificate) -> Dict[str, Path]:
        """
        Save Root CA key and certificate to disk.
        
        Returns:
            Dictionary with paths to saved files
        """
        # Save private key (encrypted)
        key_path = self.keys_dir / "root_ca.key"
        with open(key_path, 'wb') as f:
            f.write(private_key.private_bytes(
                encoding=serialization.Encoding.PEM,
                format=serialization.PrivateFormat.PKCS8,
                encryption_algorithm=serialization.NoEncryption(),  # In production, use encryption
            ))
        os.chmod(key_path, 0o600)
        
        # Save certificate
        cert_path = self.certs_dir / "root_ca.crt"
        with open(cert_path, 'wb') as f:
            f.write(certificate.public_bytes(serialization.Encoding.PEM))
        os.chmod(cert_path, 0o644)
        
        return {
            'private_key': key_path,
            'certificate': cert_path
        }
    
    def generate_signing_key(self, domain: str, validity_years: int = 5) -> tuple:
        """
        Generate signing key and certificate for a trust domain.
        
        Args:
            domain: Trust domain name (e.g., 'artifacts', 'agents', 'updates')
            validity_years: Certificate validity period
        
        Returns:
            Tuple of (private_key, certificate, root_ca_key, root_ca_cert)
        """
        # Load or generate Root CA
        root_ca_key_path = self.keys_dir / "root_ca.key"
        root_ca_cert_path = self.certs_dir / "root_ca.crt"
        
        if root_ca_key_path.exists() and root_ca_cert_path.exists():
            # Load existing Root CA
            with open(root_ca_key_path, 'rb') as f:
                root_ca_key = serialization.load_pem_private_key(f.read(), password=None)
            with open(root_ca_cert_path, 'rb') as f:
                root_ca_cert = x509.load_pem_x509_certificate(f.read())
        else:
            # Generate new Root CA
            root_ca_key, root_ca_cert = self.generate_root_ca()
            self.save_root_ca(root_ca_key, root_ca_cert)
        
        # Generate signing key
        private_key = rsa.generate_private_key(
            public_exponent=65537,
            key_size=4096,
        )
        
        # Create certificate signed by Root CA
        subject = x509.Name([
            x509.NameAttribute(NameOID.COUNTRY_NAME, "US"),
            x509.NameAttribute(NameOID.STATE_OR_PROVINCE_NAME, "CA"),
            x509.NameAttribute(NameOID.LOCALITY_NAME, "San Francisco"),
            x509.NameAttribute(NameOID.ORGANIZATION_NAME, "RansomEye"),
            x509.NameAttribute(NameOID.ORGANIZATIONAL_UNIT_NAME, domain),
            x509.NameAttribute(NameOID.COMMON_NAME, f"RansomEye {domain.title()} Signing Key"),
        ])
        
        cert = x509.CertificateBuilder().subject_name(
            subject
        ).issuer_name(
            root_ca_cert.subject
        ).public_key(
            private_key.public_key()
        ).serial_number(
            x509.random_serial_number()
        ).not_valid_before(
            datetime.utcnow()
        ).not_valid_after(
            datetime.utcnow() + timedelta(days=validity_years * 365)
        ).add_extension(
            x509.BasicConstraints(ca=False, path_length=None),
            critical=True,
        ).add_extension(
            x509.KeyUsage(
                digital_signature=True,
                key_encipherment=False,
                content_commitment=True,
                data_encipherment=False,
                key_agreement=False,
                key_cert_sign=False,
                crl_sign=False,
                encipher_only=False,
                decipher_only=False,
            ),
            critical=True,
        ).sign(root_ca_key, hashes.SHA256())
        
        return private_key, cert, root_ca_key, root_ca_cert
    
    def save_signing_key(self, domain: str, private_key, certificate) -> Dict[str, Path]:
        """Save signing key and certificate for a domain."""
        key_path = self.keys_dir / f"{domain}_signing.key"
        cert_path = self.certs_dir / f"{domain}_signing.crt"
        
        with open(key_path, 'wb') as f:
            f.write(private_key.private_bytes(
                encoding=serialization.Encoding.PEM,
                format=serialization.PrivateFormat.PKCS8,
                encryption_algorithm=serialization.NoEncryption(),
            ))
        os.chmod(key_path, 0o600)
        
        with open(cert_path, 'wb') as f:
            f.write(certificate.public_bytes(serialization.Encoding.PEM))
        os.chmod(cert_path, 0o644)
        
        return {
            'private_key': key_path,
            'certificate': cert_path
        }
    
    def initialize_trust_hierarchy(self) -> None:
        """Initialize complete trust hierarchy for RansomEye."""
        # Generate Root CA
        print("Generating Root CA...")
        root_ca_key, root_ca_cert = self.generate_root_ca()
        self.save_root_ca(root_ca_key, root_ca_cert)
        
        # Generate signing keys for each trust domain
        domains = ['artifacts', 'agents', 'updates', 'config', 'reports']
        
        key_hierarchy = {
            'root_ca': {
                'key': str(self.keys_dir / "root_ca.key"),
                'cert': str(self.certs_dir / "root_ca.crt"),
            },
            'domains': {}
        }
        
        for domain in domains:
            print(f"Generating signing key for domain: {domain}...")
            private_key, cert, _, _ = self.generate_signing_key(domain)
            paths = self.save_signing_key(domain, private_key, cert)
            
            key_hierarchy['domains'][domain] = {
                'key': str(paths['private_key']),
                'cert': str(paths['certificate']),
            }
        
        # Save key hierarchy JSON
        hierarchy_path = self.trust_dir / "key_hierarchy.json"
        with open(hierarchy_path, 'w') as f:
            json.dump(key_hierarchy, f, indent=2)
        
        print(f"Trust hierarchy initialized. Key hierarchy saved to: {hierarchy_path}")


def main():
    """CLI entry point for Root CA generator."""
    import argparse
    
    parser = argparse.ArgumentParser(description='RansomEye Root CA Generator')
    parser.add_argument('--trust-dir', default='/home/ransomeye/rebuild/ransomeye_trust',
                       help='Trust directory path')
    parser.add_argument('--init', action='store_true',
                       help='Initialize complete trust hierarchy')
    
    args = parser.parse_args()
    
    generator = RootCAGenerator(args.trust_dir)
    
    if args.init:
        generator.initialize_trust_hierarchy()
    else:
        # Just generate Root CA
        root_ca_key, root_ca_cert = generator.generate_root_ca()
        paths = generator.save_root_ca(root_ca_key, root_ca_cert)
        print(f"Root CA generated:")
        print(f"  Key: {paths['private_key']}")
        print(f"  Cert: {paths['certificate']}")


if __name__ == '__main__':
    main()

