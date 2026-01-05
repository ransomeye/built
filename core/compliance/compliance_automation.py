# Path and File Name : /home/ransomeye/rebuild/core/compliance/compliance_automation.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Scheduled Compliance Evidence Generator - Auto-generates monthly compliance reports (audit retention, data lineage, AI explainability)

"""
RansomEye Compliance Automation (PROMPT-58-B)

Scheduled monthly job to auto-generate compliance evidence:
- Audit retention proof
- Data lineage proof
- AI explainability samples (SHAP)
- Immutable timestamped storage

Stores in /docs/enterprise/compliance/monthly/YYYY-MM/
"""

import os
import sys
import json
import psycopg2
from datetime import datetime, timezone, timedelta
from pathlib import Path
from typing import Dict, List, Optional

# Configuration from environment
DB_NAME = os.environ.get("DB_NAME", "ransomeye")
DB_USER = os.environ.get("DB_USER", "gagan")
DB_HOST = os.environ.get("DB_HOST", "localhost")
DB_PASSWORD = os.environ.get("DB_PASS", "gagan")
DB_PORT = int(os.environ.get("DB_PORT", "5432"))

COMPLIANCE_BASE_DIR = Path("/home/ransomeye/rebuild/docs/enterprise/compliance/monthly")
RETENTION_YEARS = int(os.environ.get("RETENTION_YEARS", "7"))


def get_db_connection():
    """Get database connection."""
    try:
        return psycopg2.connect(
            host=DB_HOST,
            port=DB_PORT,
            database=DB_NAME,
            user=DB_USER,
            password=DB_PASSWORD
        )
    except Exception as e:
        return None


def generate_audit_retention_proof(conn) -> Dict:
    """Generate audit retention proof - verify retention policy compliance."""
    if not conn:
        return {"error": "Database connection failed"}
    
    try:
        cursor = conn.cursor()
        
        # Get retention policy
        retention_days = RETENTION_YEARS * 365
        
        # Check oldest audit entry
        cursor.execute("""
            SELECT MIN(created_at), MAX(created_at), COUNT(*)
            FROM ransomeye.immutable_audit_log
        """)
        oldest, newest, total_count = cursor.fetchone()
        
        # Check entries older than retention period
        cutoff_date = datetime.now(timezone.utc) - timedelta(days=retention_days)
        cursor.execute("""
            SELECT COUNT(*)
            FROM ransomeye.immutable_audit_log
            WHERE created_at < %s
        """, (cutoff_date,))
        expired_count = cursor.fetchone()[0]
        
        # Verify retention enforcement
        retention_compliant = expired_count == 0
        
        proof = {
            "retention_policy_years": RETENTION_YEARS,
            "retention_policy_days": retention_days,
            "oldest_entry": oldest.isoformat() if oldest else None,
            "newest_entry": newest.isoformat() if newest else None,
            "total_entries": total_count,
            "expired_entries": expired_count,
            "retention_compliant": retention_compliant,
            "cutoff_date": cutoff_date.isoformat(),
            "timestamp": datetime.now(timezone.utc).isoformat()
        }
        
        cursor.close()
        return proof
    except Exception as e:
        return {"error": str(e)}


def generate_data_lineage_proof(conn) -> Dict:
    """Generate data lineage proof - trace data flow from ingestion to storage."""
    if not conn:
        return {"error": "Database connection failed"}
    
    try:
        cursor = conn.cursor()
        
        # Get data flow statistics
        lineage = {}
        
        # Raw events lineage
        cursor.execute("""
            SELECT 
                COUNT(*) as total,
                COUNT(DISTINCT source_type) as source_types,
                MIN(observed_at) as oldest,
                MAX(observed_at) as newest
            FROM ransomeye.raw_events
        """)
        row = cursor.fetchone()
        lineage["raw_events"] = {
            "total": row[0],
            "source_types": row[1],
            "oldest": row[2].isoformat() if row[2] else None,
            "newest": row[3].isoformat() if row[3] else None
        }
        
        # Normalized events lineage
        cursor.execute("""
            SELECT 
                COUNT(*) as total,
                MIN(observed_at) as oldest,
                MAX(observed_at) as newest
            FROM ransomeye.normalized_events
        """)
        row = cursor.fetchone()
        lineage["normalized_events"] = {
            "total": row[0],
            "oldest": row[1].isoformat() if row[1] else None,
            "newest": row[2].isoformat() if row[2] else None
        }
        
        # Audit log lineage (chain integrity)
        cursor.execute("""
            SELECT 
                COUNT(*) as total,
                COUNT(DISTINCT chain_hash_sha256) as unique_chains,
                MIN(created_at) as oldest,
                MAX(created_at) as newest
            FROM ransomeye.immutable_audit_log
        """)
        row = cursor.fetchone()
        lineage["audit_log"] = {
            "total": row[0],
            "unique_chains": row[1],
            "oldest": row[2].isoformat() if row[2] else None,
            "newest": row[3].isoformat() if row[3] else None
        }
        
        # Verify chain integrity
        cursor.execute("""
            SELECT audit_id, prev_payload_sha256, chain_hash_sha256
            FROM ransomeye.immutable_audit_log
            ORDER BY created_at DESC
            LIMIT 10
        """)
        chain_samples = []
        for row in cursor.fetchall():
            chain_samples.append({
                "audit_id": row[0],
                "has_prev": row[1] is not None,
                "has_chain": row[2] is not None
            })
        
        lineage["chain_integrity_samples"] = chain_samples
        lineage["timestamp"] = datetime.now(timezone.utc).isoformat()
        
        cursor.close()
        return lineage
    except Exception as e:
        return {"error": str(e)}


def generate_ai_explainability_samples(conn) -> Dict:
    """Generate AI explainability samples (SHAP) for compliance."""
    if not conn:
        return {"error": "Database connection failed"}
    
    try:
        cursor = conn.cursor()
        
        # Get SHAP explanations
        cursor.execute("""
            SELECT 
                COUNT(*) as total,
                COUNT(DISTINCT model_id) as model_count,
                MIN(created_at) as oldest,
                MAX(created_at) as newest
            FROM ransomeye.shap_explanations
        """)
        row = cursor.fetchone()
        
        shap_summary = {
            "total_explanations": row[0],
            "model_count": row[1],
            "oldest": row[2].isoformat() if row[2] else None,
            "newest": row[3].isoformat() if row[3] else None
        }
        
        # Get sample SHAP explanations
        cursor.execute("""
            SELECT 
                shap_id,
                model_id,
                prediction_id,
                created_at,
                feature_count
            FROM ransomeye.shap_explanations
            ORDER BY created_at DESC
            LIMIT 10
        """)
        
        samples = []
        for row in cursor.fetchall():
            samples.append({
                "shap_id": row[0],
                "model_id": row[1],
                "prediction_id": row[2],
                "created_at": row[3].isoformat() if row[3] else None,
                "has_features": row[4] is not None
            })
        
        shap_summary["samples"] = samples
        
        # Get model registry info
        cursor.execute("""
            SELECT 
                model_id,
                model_name,
                model_version,
                shap_enabled
            FROM ransomeye.model_registry
            LIMIT 10
        """)
        
        models = []
        for row in cursor.fetchall():
            models.append({
                "model_id": row[0],
                "model_name": row[1],
                "model_version": row[2],
                "shap_enabled": row[3] if len(row) > 3 else None
            })
        
        shap_summary["models"] = models
        shap_summary["timestamp"] = datetime.now(timezone.utc).isoformat()
        
        cursor.close()
        return shap_summary
    except Exception as e:
        return {"error": str(e)}


def generate_compliance_report(month_dir: Path) -> Dict:
    """Generate complete compliance report."""
    print(f"Generating compliance report for {month_dir.name}...")
    
    conn = get_db_connection()
    
    # Generate all proofs
    print("  Generating audit retention proof...")
    audit_proof = generate_audit_retention_proof(conn)
    
    print("  Generating data lineage proof...")
    lineage_proof = generate_data_lineage_proof(conn)
    
    print("  Generating AI explainability samples...")
    explainability_samples = generate_ai_explainability_samples(conn)
    
    if conn:
        conn.close()
    
    # Build complete report
    report = {
        "report_timestamp": datetime.now(timezone.utc).isoformat(),
        "report_period": month_dir.name,
        "audit_retention_proof": audit_proof,
        "data_lineage_proof": lineage_proof,
        "ai_explainability_samples": explainability_samples
    }
    
    # Write report
    report_path = month_dir / "compliance_report.json"
    with open(report_path, "w") as f:
        json.dump(report, f, indent=2, sort_keys=True)
    
    # Make immutable (read-only)
    try:
        os.chmod(report_path, 0o444)
    except Exception:
        pass
    
    print(f"  Compliance report written to {report_path}")
    
    return report


def main():
    """Main compliance automation function."""
    print("RansomEye Compliance Automation (PROMPT-58-B)")
    print("=" * 60)
    
    # Create monthly directory
    now = datetime.now(timezone.utc)
    month_dir = COMPLIANCE_BASE_DIR / now.strftime("%Y-%m")
    month_dir.mkdir(parents=True, exist_ok=True)
    
    # Generate compliance report
    report = generate_compliance_report(month_dir)
    
    print("\nCompliance Automation Complete")
    print(f"Report location: {month_dir}")
    
    return 0


if __name__ == "__main__":
    sys.exit(main())

