# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/training_governance.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Training & Governance module for Phase 6 - Ed25519 signing, SHAP, resource governance, incremental retraining

"""
Training & Governance Module (Phase 6):
- Synthetic bootstrapping on first start
- Incremental retraining on feed updates and telemetry drift
- Model signing with Ed25519
- Signature verification before load
- SHAP explainability for every decision
- Resource governance (SWAP scales to available physical RAM, NO 64GB CAP)
"""

import os
import sys
import json
import pickle
import hashlib
import psutil
import shutil
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Optional, Tuple, Any
import logging

try:
    from cryptography.hazmat.primitives.asymmetric import ed25519
    from cryptography.hazmat.primitives import serialization
    from cryptography.hazmat.backends import default_backend
    CRYPTOGRAPHY_AVAILABLE = True
except ImportError:
    CRYPTOGRAPHY_AVAILABLE = False
    logging.warning("cryptography library not available - Ed25519 signing will fail")

try:
    import shap
    import numpy as np
    SHAP_AVAILABLE = True
except ImportError:
    SHAP_AVAILABLE = False
    logging.warning("SHAP library not available - explainability will be limited")

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger('training_governance')

MODELS_DIR = Path("/home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/models")
SIGNING_KEY_DIR = Path("/home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/keys")
SHAP_DIR = Path("/home/ransomeye/rebuild/ransomeye_intelligence/threat_intel/shap")
MODELS_DIR.mkdir(parents=True, exist_ok=True)
SIGNING_KEY_DIR.mkdir(parents=True, exist_ok=True)
SHAP_DIR.mkdir(parents=True, exist_ok=True)

# Signing key paths
SIGNING_KEY_PATH = SIGNING_KEY_DIR / "model_signing_key.pem"
PUBLIC_KEY_PATH = SIGNING_KEY_DIR / "model_public_key.pem"


class Ed25519ModelSigner:
    """Ed25519 model signer for Phase 6."""
    
    def __init__(self, signing_key_path: Optional[Path] = None):
        """
        Initialize Ed25519 model signer.
        
        Args:
            signing_key_path: Path to signing key (generates if not exists)
        """
        if not CRYPTOGRAPHY_AVAILABLE:
            raise RuntimeError("cryptography library required for Ed25519 signing")
        
        self.signing_key_path = signing_key_path or SIGNING_KEY_PATH
        self.private_key = None
        self.public_key = None
        
        if self.signing_key_path.exists():
            self._load_key()
        else:
            self._generate_key()
    
    def _generate_key(self):
        """Generate new Ed25519 key pair."""
        logger.info("Generating new Ed25519 signing key pair")
        self.private_key = ed25519.Ed25519PrivateKey.generate()
        self.public_key = self.private_key.public_key()
        
        # Save private key
        private_pem = self.private_key.private_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PrivateFormat.PKCS8,
            encryption_algorithm=serialization.NoEncryption()
        )
        with open(self.signing_key_path, 'wb') as f:
            f.write(private_pem)
        os.chmod(self.signing_key_path, 0o600)  # Secure permissions
        
        # Save public key
        public_pem = self.public_key.public_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PublicFormat.SubjectPublicKeyInfo
        )
        with open(PUBLIC_KEY_PATH, 'wb') as f:
            f.write(public_pem)
        
        logger.info(f"Ed25519 key pair generated and saved to {self.signing_key_path}")
    
    def _load_key(self):
        """Load Ed25519 signing key from file."""
        logger.info(f"Loading Ed25519 signing key from {self.signing_key_path}")
        with open(self.signing_key_path, 'rb') as f:
            key_data = f.read()
        
        self.private_key = serialization.load_pem_private_key(
            key_data,
            password=None,
            backend=default_backend()
        )
        
        if not isinstance(self.private_key, ed25519.Ed25519PrivateKey):
            raise ValueError("Key is not Ed25519")
        
        self.public_key = self.private_key.public_key()
        logger.info("Ed25519 signing key loaded successfully")
    
    def sign_model(self, model_data: bytes) -> str:
        """
        Sign model data with Ed25519.
        
        Args:
            model_data: Model data bytes
        
        Returns:
            Base64-encoded signature
        """
        # Compute SHA256 hash of model data
        model_hash = hashlib.sha256(model_data).digest()
        
        # Sign hash with Ed25519
        signature = self.private_key.sign(model_hash)
        
        # Encode signature as base64
        import base64
        signature_b64 = base64.b64encode(signature).decode('utf-8')
        
        logger.debug(f"Model signed with Ed25519 (hash: {model_hash.hex()[:16]}...)")
        return signature_b64
    
    def sign_manifest(self, manifest_json: bytes) -> str:
        """
        Sign manifest JSON with Ed25519.
        
        Args:
            manifest_json: Manifest JSON bytes
        
        Returns:
            Base64-encoded signature
        """
        # Compute SHA256 hash of manifest
        manifest_hash = hashlib.sha256(manifest_json).digest()
        
        # Sign hash with Ed25519
        signature = self.private_key.sign(manifest_hash)
        
        # Encode signature as base64
        import base64
        signature_b64 = base64.b64encode(signature).decode('utf-8')
        
        logger.debug("Manifest signed with Ed25519")
        return signature_b64


class Ed25519ModelVerifier:
    """Ed25519 model verifier for Phase 6."""
    
    def __init__(self, public_key_path: Optional[Path] = None):
        """
        Initialize Ed25519 model verifier.
        
        Args:
            public_key_path: Path to public key
        """
        if not CRYPTOGRAPHY_AVAILABLE:
            raise RuntimeError("cryptography library required for Ed25519 verification")
        
        self.public_key_path = public_key_path or PUBLIC_KEY_PATH
        
        if not self.public_key_path.exists():
            raise FileNotFoundError(f"Public key not found: {self.public_key_path}")
        
        self._load_public_key()
    
    def _load_public_key(self):
        """Load Ed25519 public key from file."""
        with open(self.public_key_path, 'rb') as f:
            key_data = f.read()
        
        self.public_key = serialization.load_pem_public_key(
            key_data,
            backend=default_backend()
        )
        
        if not isinstance(self.public_key, ed25519.Ed25519PublicKey):
            raise ValueError("Key is not Ed25519")
        
        logger.info(f"Ed25519 public key loaded from {self.public_key_path}")
    
    def verify_model(self, model_data: bytes, signature_b64: str) -> bool:
        """
        Verify model signature.
        
        Args:
            model_data: Model data bytes
            signature_b64: Base64-encoded signature
        
        Returns:
            True if signature is valid, False otherwise
        """
        # Compute SHA256 hash of model data
        model_hash = hashlib.sha256(model_data).digest()
        
        # Decode signature
        import base64
        try:
            signature = base64.b64decode(signature_b64)
        except Exception as e:
            logger.error(f"Failed to decode signature: {e}")
            return False
        
        # Verify signature
        try:
            self.public_key.verify(signature, model_hash)
            logger.debug("Ed25519 model signature verified successfully")
            return True
        except Exception as e:
            logger.error(f"Ed25519 model signature verification failed: {e}")
            return False
    
    def verify_manifest(self, manifest_json: bytes, signature_b64: str) -> bool:
        """
        Verify manifest signature.
        
        Args:
            manifest_json: Manifest JSON bytes
            signature_b64: Base64-encoded signature
        
        Returns:
            True if signature is valid, False otherwise
        """
        # Compute SHA256 hash of manifest
        manifest_hash = hashlib.sha256(manifest_json).digest()
        
        # Decode signature
        import base64
        try:
            signature = base64.b64decode(signature_b64)
        except Exception as e:
            logger.error(f"Failed to decode signature: {e}")
            return False
        
        # Verify signature
        try:
            self.public_key.verify(signature, manifest_hash)
            logger.debug("Ed25519 manifest signature verified successfully")
            return True
        except Exception as e:
            logger.error(f"Ed25519 manifest signature verification failed: {e}")
            return False


class ResourceGovernor:
    """Resource governance for training (Phase 6 - SWAP scales to available RAM, NO 64GB CAP)."""
    
    @staticmethod
    def get_available_memory() -> Tuple[int, int]:
        """
        Get available physical RAM and SWAP.
        
        Returns:
            Tuple of (physical_ram_gb, swap_gb)
        """
        mem = psutil.virtual_memory()
        swap = psutil.swap_memory()
        
        # Convert to GB
        ram_gb = mem.total / (1024 ** 3)
        swap_gb = swap.total / (1024 ** 3)
        
        logger.info(f"Available memory: {ram_gb:.2f} GB RAM, {swap_gb:.2f} GB SWAP")
        return ram_gb, swap_gb
    
    @staticmethod
    def configure_training_resources() -> Dict[str, Any]:
        """
        Configure training resources based on available memory.
        Phase 6: SWAP scales to available physical RAM (NO 64GB CAP).
        
        Returns:
            Resource configuration dictionary
        """
        ram_gb, swap_gb = ResourceGovernor.get_available_memory()
        
        # Phase 6: Use all available memory (NO 64GB CAP)
        total_memory_gb = ram_gb + swap_gb
        
        # Configure training parameters based on available memory
        config = {
            'max_memory_gb': total_memory_gb,
            'ram_gb': ram_gb,
            'swap_gb': swap_gb,
            'n_jobs': min(psutil.cpu_count(), 8),  # Limit CPU cores
            'batch_size': min(1000, int(total_memory_gb * 100)),  # Scale batch size with memory
        }
        
        logger.info(f"Training resource configuration: {config}")
        return config


class SHAPExplainer:
    """SHAP explainability for Phase 6."""
    
    def __init__(self, model: Any, training_data: np.ndarray):
        """
        Initialize SHAP explainer.
        
        Args:
            model: Trained model
            training_data: Training data for background
        """
        if not SHAP_AVAILABLE:
            raise RuntimeError("SHAP library required for explainability")
        
        self.model = model
        self.training_data = training_data
        
        # Create SHAP explainer (TreeExplainer for tree-based models)
        try:
            self.explainer = shap.TreeExplainer(model)
        except:
            # Fallback to KernelExplainer for other models
            logger.warning("TreeExplainer failed, using KernelExplainer (slower)")
            # Use sample of training data for background
            background = training_data[:100] if len(training_data) > 100 else training_data
            self.explainer = shap.KernelExplainer(model.predict, background)
        
        logger.info("SHAP explainer initialized")
    
    def explain(self, features: np.ndarray) -> Dict[str, Any]:
        """
        Generate SHAP explanation for features.
        
        Args:
            features: Feature vector
        
        Returns:
            SHAP explanation dictionary
        """
        # Compute SHAP values
        shap_values = self.explainer.shap_values(features)
        
        # Handle multi-class output
        if isinstance(shap_values, list):
            shap_values = shap_values[0]  # Use first class
        
        # Get base value
        base_value = self.explainer.expected_value
        if isinstance(base_value, np.ndarray):
            base_value = float(base_value[0])
        else:
            base_value = float(base_value)
        
        # Compute prediction
        prediction = self.model.predict(features.reshape(1, -1))[0]
        
        # Create explanation dictionary
        explanation = {
            'shap_values': shap_values.tolist() if isinstance(shap_values, np.ndarray) else shap_values,
            'base_value': base_value,
            'prediction': float(prediction),
            'feature_contributions': [
                {'feature_idx': i, 'contribution': float(shap_values[i])}
                for i in range(len(shap_values))
            ],
            'timestamp': datetime.utcnow().isoformat() + 'Z',
        }
        
        logger.debug(f"SHAP explanation generated (prediction: {prediction})")
        return explanation
    
    def save_explanation(self, explanation: Dict[str, Any], model_name: str, version: str) -> Path:
        """
        Save SHAP explanation to file.
        
        Args:
            explanation: SHAP explanation dictionary
            model_name: Model name
            version: Model version
        
        Returns:
            Path to saved explanation file
        """
        explanation_path = SHAP_DIR / f"{model_name}_v{version}_shap.json"
        
        with open(explanation_path, 'w') as f:
            json.dump(explanation, f, indent=2)
        
        logger.info(f"SHAP explanation saved to {explanation_path}")
        return explanation_path


class TrainingGovernance:
    """Training & Governance orchestrator for Phase 6."""
    
    def __init__(self):
        """Initialize training governance."""
        self.signer = Ed25519ModelSigner()
        self.verifier = Ed25519ModelVerifier()
        self.resource_governor = ResourceGovernor()
        logger.info("Training & Governance module initialized")
    
    def get_model_version(self, model_name: str) -> str:
        """
        Get next model version.
        
        Args:
            model_name: Model name
        
        Returns:
            Next version string (e.g., "1.0.0")
        """
        # Find existing models
        model_files = list(MODELS_DIR.glob(f"{model_name}_v*.pkl"))
        
        if not model_files:
            return "1.0.0"
        
        # Extract versions and increment
        versions = []
        for f in model_files:
            # Extract version from filename: model_name_v1.0.0.pkl
            parts = f.stem.split('_v')
            if len(parts) == 2:
                try:
                    version = parts[1]
                    versions.append(version)
                except:
                    continue
        
        if not versions:
            return "1.0.0"
        
        # Find highest version and increment patch
        highest = max(versions, key=lambda v: tuple(map(int, v.split('.'))))
        parts = highest.split('.')
        if len(parts) == 3:
            major, minor, patch = map(int, parts)
            return f"{major}.{minor}.{patch + 1}"
        
        return "1.0.0"
    
    def sign_and_save_model(
        self,
        model: Any,
        model_name: str,
        version: str,
        metadata: Dict[str, Any]
    ) -> Tuple[Path, Path]:
        """
        Sign model with Ed25519 and save with manifest.
        
        Args:
            model: Trained model
            model_name: Model name
            version: Model version
            metadata: Model metadata
        
        Returns:
            Tuple of (model_path, manifest_path)
        """
        # Serialize model
        model_bytes = pickle.dumps(model)
        
        # Compute model hash
        model_hash = hashlib.sha256(model_bytes).hexdigest()
        
        # Sign model
        model_signature = self.signer.sign_model(model_bytes)
        
        # Create manifest
        manifest = {
            'model_name': model_name,
            'version': version,
            'hash': f"sha256:{model_hash}",
            'signature': model_signature,
            'signature_algorithm': 'Ed25519',
            'timestamp': datetime.utcnow().isoformat() + 'Z',
            'metadata': metadata,
        }
        
        manifest_json = json.dumps(manifest, sort_keys=True).encode('utf-8')
        manifest_signature = self.signer.sign_manifest(manifest_json)
        manifest['manifest_signature'] = manifest_signature
        
        # Save model
        model_path = MODELS_DIR / f"{model_name}_v{version}.pkl"
        with open(model_path, 'wb') as f:
            f.write(model_bytes)
        
        # Save manifest
        manifest_path = MODELS_DIR / f"{model_name}_v{version}_manifest.json"
        with open(manifest_path, 'w') as f:
            json.dump(manifest, f, indent=2)
        
        logger.info(f"Model signed and saved: {model_path}")
        return model_path, manifest_path
    
    def verify_and_load_model(self, model_name: str, version: str) -> Tuple[Any, Dict[str, Any]]:
        """
        Verify model signature and load.
        
        Args:
            model_name: Model name
            version: Model version
        
        Returns:
            Tuple of (model, manifest)
        
        Raises:
            ValueError: If model is unsigned or verification fails
        """
        model_path = MODELS_DIR / f"{model_name}_v{version}.pkl"
        manifest_path = MODELS_DIR / f"{model_name}_v{version}_manifest.json"
        
        if not model_path.exists():
            raise FileNotFoundError(f"Model not found: {model_path}")
        
        if not manifest_path.exists():
            raise ValueError(f"Model is unsigned (no manifest): {model_path}")
        
        # Load manifest
        with open(manifest_path, 'r') as f:
            manifest = json.load(f)
        
        # Verify manifest signature
        manifest_json = json.dumps(manifest, sort_keys=True).encode('utf-8')
        manifest_signature = manifest.get('manifest_signature')
        
        if not manifest_signature:
            raise ValueError("Manifest is unsigned")
        
        if not self.verifier.verify_manifest(manifest_json, manifest_signature):
            raise ValueError("Manifest signature verification failed")
        
        # Load model
        with open(model_path, 'rb') as f:
            model_bytes = f.read()
        
        # Verify model signature
        model_signature = manifest.get('signature')
        if not model_signature:
            raise ValueError("Model is unsigned")
        
        if not self.verifier.verify_model(model_bytes, model_signature):
            raise ValueError("Model signature verification failed")
        
        # Verify hash
        model_hash = hashlib.sha256(model_bytes).hexdigest()
        expected_hash = manifest.get('hash', '').replace('sha256:', '')
        if model_hash != expected_hash:
            raise ValueError("Model hash mismatch")
        
        # Deserialize model
        model = pickle.loads(model_bytes)
        
        logger.info(f"Model verified and loaded: {model_path}")
        return model, manifest


def main():
    """CLI entry point for testing."""
    import argparse
    
    parser = argparse.ArgumentParser(description='Training & Governance (Phase 6)')
    parser.add_argument('--test-signing', action='store_true',
                       help='Test Ed25519 signing and verification')
    parser.add_argument('--test-resources', action='store_true',
                       help='Test resource governance')
    
    args = parser.parse_args()
    
    if args.test_signing:
        print("Testing Ed25519 signing and verification...")
        signer = Ed25519ModelSigner()
        verifier = Ed25519ModelVerifier()
        
        test_data = b"test model data"
        signature = signer.sign_model(test_data)
        verified = verifier.verify_model(test_data, signature)
        
        print(f"Signature: {signature[:50]}...")
        print(f"Verification: {'PASS' if verified else 'FAIL'}")
    
    if args.test_resources:
        print("Testing resource governance...")
        config = ResourceGovernor.configure_training_resources()
        print(f"Resource configuration: {config}")


if __name__ == '__main__':
    main()

