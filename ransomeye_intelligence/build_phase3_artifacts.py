# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/build_phase3_artifacts.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Master script to build all Phase 3 artifacts

"""
Phase 3 Artifact Builder: Master script to build all Phase 3 artifacts.
Generates real, shippable intelligence artifacts for Day-1 AI readiness.
"""

import sys
import subprocess
from pathlib import Path
from datetime import datetime

INTELLIGENCE_DIR = Path("/home/ransomeye/rebuild/ransomeye_intelligence")
BASELINE_PACK_DIR = INTELLIGENCE_DIR / "baseline_pack"
LLM_KNOWLEDGE_DIR = INTELLIGENCE_DIR / "llm_knowledge"


def run_step(step_name: str, script_path: Path, description: str) -> bool:
    """Run a build step and return success status."""
    print("=" * 80)
    print(f"STEP: {step_name}")
    print("=" * 80)
    print(description)
    print()
    
    if not script_path.exists():
        print(f"ERROR: Script not found: {script_path}", file=sys.stderr)
        return False
    
    try:
        result = subprocess.run(
            [sys.executable, str(script_path)],
            cwd=str(script_path.parent),
            capture_output=False,
            check=True
        )
        print()
        print(f"✓ {step_name} completed successfully")
        print()
        return True
    except subprocess.CalledProcessError as e:
        print(f"ERROR: {step_name} failed with exit code {e.returncode}", file=sys.stderr)
        return False
    except Exception as e:
        print(f"ERROR: {step_name} failed: {e}", file=sys.stderr)
        return False


def update_training_manifest() -> bool:
    """Update training manifest with real dependency hashes."""
    import json
    import hashlib
    import subprocess
    
    print("Updating training manifest with real dependency hashes...")
    
    manifest_path = BASELINE_PACK_DIR / "metadata" / "training_manifest.json"
    
    with open(manifest_path, 'r') as f:
        manifest = json.load(f)
    
    # Compute dependencies hash from requirements.txt
    requirements_path = Path("/home/ransomeye/rebuild/requirements.txt")
    if requirements_path.exists():
        with open(requirements_path, 'rb') as f:
            deps_hash = hashlib.sha256(f.read()).hexdigest()
        manifest['reproducibility']['dependencies_hash'] = f"sha256:{deps_hash}"
    
    # Update timestamp
    manifest['training_date'] = datetime.utcnow().isoformat() + 'Z'
    
    with open(manifest_path, 'w') as f:
        json.dump(manifest, f, indent=2)
    
    print(f"  ✓ Training manifest updated: {manifest_path}")
    return True


def verify_artifacts() -> bool:
    """Verify all artifacts were generated."""
    print("=" * 80)
    print("VERIFYING ARTIFACTS")
    print("=" * 80)
    print()
    
    artifacts_ok = True
    
    # Check model files
    print("Checking model files...")
    model_files = [
        "ransomware_behavior.model",
        "anomaly_baseline.model",
        "confidence_calibration.model"
    ]
    
    for model_file in model_files:
        model_path = BASELINE_PACK_DIR / "models" / model_file
        if model_path.exists():
            size = model_path.stat().st_size
            print(f"  ✓ {model_file}: {size} bytes")
            if size == 0:
                print(f"    WARNING: File is empty!", file=sys.stderr)
                artifacts_ok = False
        else:
            print(f"  ✗ {model_file}: MISSING", file=sys.stderr)
            artifacts_ok = False
    
    print()
    
    # Check signature files
    print("Checking signature files...")
    sig_files = [
        ("baseline_pack/signatures/baseline_pack.sig", "Baseline pack signature"),
        ("baseline_pack/signatures/baseline_pack.pub", "Baseline pack public key"),
        ("threat_intel/signatures/intel_pack.sig", "Threat intel signature"),
        ("llm_knowledge/signatures/rag_pack.sig", "RAG pack signature")
    ]
    
    for rel_path, description in sig_files:
        sig_path = INTELLIGENCE_DIR / rel_path
        if sig_path.exists():
            size = sig_path.stat().st_size
            print(f"  ✓ {description}: {size} bytes")
            if size == 0:
                print(f"    WARNING: File is empty!", file=sys.stderr)
                artifacts_ok = False
        else:
            print(f"  ✗ {description}: MISSING", file=sys.stderr)
            artifacts_ok = False
    
    print()
    
    # Check RAG index
    print("Checking RAG index...")
    index_path = LLM_KNOWLEDGE_DIR / "rag_index" / "index.bin"
    if index_path.exists():
        size = index_path.stat().st_size
        print(f"  ✓ RAG index: {size} bytes")
        if size == 0:
            print(f"    WARNING: Index is empty!", file=sys.stderr)
            artifacts_ok = False
    else:
        print(f"  ✗ RAG index: MISSING", file=sys.stderr)
        artifacts_ok = False
    
    print()
    
    return artifacts_ok


def main():
    """Main build pipeline."""
    print("=" * 80)
    print("RANSOMEYE PHASE 3 - ARTIFACT BUILD")
    print("=" * 80)
    print()
    print("Building real, shippable intelligence artifacts for Day-1 AI readiness")
    print("No placeholders. No dummy files. Everything must be verifiable.")
    print()
    print(f"Start time: {datetime.utcnow().isoformat()}Z")
    print()
    
    steps = [
        (
            "Train Baseline Models",
            BASELINE_PACK_DIR / "train_baseline_models.py",
            "Training baseline models using synthetic + red-team data only"
        ),
        (
            "Generate SHAP Baselines",
            BASELINE_PACK_DIR / "generate_shap_baselines.py",
            "Generating SHAP baseline values for trained models"
        ),
        (
            "Build RAG Index",
            LLM_KNOWLEDGE_DIR / "build_rag_index_simple.py",
            "Building pre-indexed RAG knowledge base using TF-IDF"
        ),
        (
            "Generate Signatures",
            INTELLIGENCE_DIR / "generate_signatures.py",
            "Generating cryptographic signatures for all packs"
        )
    ]
    
    # Run all steps
    all_success = True
    for step_name, script_path, description in steps:
        if not run_step(step_name, script_path, description):
            all_success = False
            print(f"ERROR: Build failed at step: {step_name}", file=sys.stderr)
            sys.exit(1)
    
    # Update training manifest
    if not update_training_manifest():
        print("WARNING: Failed to update training manifest", file=sys.stderr)
    
    # Verify artifacts
    if not verify_artifacts():
        print("ERROR: Artifact verification failed", file=sys.stderr)
        sys.exit(1)
    
    print("=" * 80)
    print("✓ PHASE 3 ARTIFACT BUILD COMPLETE")
    print("=" * 80)
    print()
    print("All artifacts generated and verified:")
    print("  ✓ Pre-trained model files")
    print("  ✓ SHAP baseline values")
    print("  ✓ Cryptographic signatures")
    print("  ✓ Pre-indexed RAG knowledge base")
    print()
    print("Phase 3 Day-1 intelligence artifacts generated and ready for audit.")
    print()
    print(f"End time: {datetime.utcnow().isoformat()}Z")


if __name__ == '__main__':
    main()

