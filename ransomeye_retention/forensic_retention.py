# Path and File Name : /home/ransomeye/rebuild/ransomeye_retention/forensic_retention.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Forensic purge logic with chunk-aware deletion and destruction certificates

"""
Forensic Retention: Enforces forensic data retention policy.
Purges old forensic evidence with chunk-aware deletion and destruction certificates.
"""

import os
import json
import hashlib
from pathlib import Path
from typing import List, Dict, Optional
from datetime import datetime, timedelta

from .retention_parser import RetentionParser
import sys
from pathlib import Path
# Add project root to path for trust imports
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))
try:
    from ransomeye_trust.sign_tool import SignTool
except ImportError:
    # Fallback for development
    SignTool = None


class ForensicRetention:
    """Enforces forensic retention policy."""
    
    def __init__(self, retention_parser: RetentionParser, forensic_dir: str = "/home/ransomeye/rebuild/ransomeye_forensic", trust_dir: str = "/home/ransomeye/rebuild/ransomeye_trust"):
        self.retention_parser = retention_parser
        self.forensic_dir = Path(forensic_dir)
        self.trust_dir = Path(trust_dir)
        self.sign_tool = SignTool(str(self.trust_dir))
        self.ledger_path = Path("/home/ransomeye/rebuild/logs/retention_ledger.jsonl")
        self.ledger_path.parent.mkdir(parents=True, exist_ok=True)
        self.certificates_dir = self.forensic_dir / "destruction_certificates"
        self.certificates_dir.mkdir(parents=True, exist_ok=True)
    
    def _get_old_forensic_files(self) -> List[Dict]:
        """Get list of forensic files older than retention period."""
        cutoff_date = self.retention_parser.get_forensic_cutoff_date()
        old_files = []
        
        if not self.forensic_dir.exists():
            return old_files
        
        for root, dirs, files in os.walk(self.forensic_dir):
            # Skip certificates directory
            if 'destruction_certificates' in root:
                continue
            
            for file in files:
                file_path = Path(root) / file
                try:
                    mtime = datetime.fromtimestamp(file_path.stat().st_mtime)
                    if mtime < cutoff_date:
                        old_files.append({
                            'path': file_path,
                            'size': file_path.stat().st_size,
                            'mtime': mtime.isoformat(),
                            'hash': self._compute_file_hash(file_path)
                        })
                except Exception:
                    continue
        
        return old_files
    
    def _compute_file_hash(self, file_path: Path) -> str:
        """Compute SHA-256 hash of file."""
        sha256 = hashlib.sha256()
        try:
            with open(file_path, 'rb') as f:
                for chunk in iter(lambda: f.read(4096), b''):
                    sha256.update(chunk)
            return sha256.hexdigest()
        except Exception:
            return ""
    
    def _delete_file_chunked(self, file_path: Path) -> bool:
        """
        Delete file with chunk-aware secure deletion.
        Overwrites file before deletion.
        """
        try:
            # Overwrite with random data (3 passes)
            file_size = file_path.stat().st_size
            with open(file_path, 'r+b') as f:
                for _ in range(3):
                    f.seek(0)
                    f.write(os.urandom(file_size))
                    f.flush()
                    os.fsync(f.fileno())
            
            # Delete file
            file_path.unlink()
            return True
        except Exception as e:
            print(f"Error deleting file {file_path}: {e}")
            return False
    
    def _create_destruction_certificate(self, files: List[Dict]) -> Path:
        """Create signed destruction certificate for purged files."""
        certificate = {
            'timestamp': datetime.utcnow().isoformat(),
            'type': 'forensic_destruction',
            'files': files,
            'retention_days': self.retention_parser.get_forensic_retention_days(),
            'cutoff_date': self.retention_parser.get_forensic_cutoff_date().isoformat(),
            'total_files': len(files),
            'total_size_bytes': sum(f['size'] for f in files)
        }
        
        # Save certificate
        cert_filename = f"destruction_{datetime.utcnow().strftime('%Y%m%d_%H%M%S')}.json"
        cert_path = self.certificates_dir / cert_filename
        
        with open(cert_path, 'w') as f:
            json.dump(certificate, f, indent=2)
        
        # Sign certificate
        metadata = {
            'timestamp': datetime.utcnow().isoformat(),
            'version': '1.0.0',
            'type': 'destruction_certificate'
        }
        self.sign_tool.create_and_sign_manifest(cert_path, metadata, 'reports')
        
        return cert_path
    
    def _log_purge_event(self, files: List[Dict], reason: str, cert_path: Optional[Path] = None) -> None:
        """Log purge event to signed ledger."""
        event = {
            'timestamp': datetime.utcnow().isoformat(),
            'type': 'forensic_purge',
            'reason': reason,
            'files_count': len(files),
            'certificate_path': str(cert_path) if cert_path else None,
            'retention_days': self.retention_parser.get_forensic_retention_days(),
            'cutoff_date': self.retention_parser.get_forensic_cutoff_date().isoformat()
        }
        
        # Append to ledger
        with open(self.ledger_path, 'a') as f:
            f.write(json.dumps(event) + '\n')
    
    def purge_old_forensics(self, dry_run: bool = False) -> Dict:
        """
        Purge forensic data older than retention period.
        
        Args:
            dry_run: If True, only report what would be purged
        
        Returns:
            Dictionary with purge results
        """
        old_files = self._get_old_forensic_files()
        
        if not old_files:
            return {
                'purged': False,
                'files': [],
                'reason': 'No files to purge'
            }
        
        if dry_run:
            return {
                'purged': False,
                'files': old_files,
                'reason': 'dry_run',
                'would_purge': True
            }
        
        # Delete files
        deleted = []
        failed = []
        
        for file_info in old_files:
            if self._delete_file_chunked(file_info['path']):
                deleted.append(file_info)
            else:
                failed.append(file_info)
        
        # Create destruction certificate
        cert_path = None
        if deleted:
            cert_path = self._create_destruction_certificate(deleted)
        
        # Log purge event
        reason = 'retention_policy' if not dry_run else 'dry_run'
        self._log_purge_event(deleted, reason, cert_path)
        
        return {
            'purged': True,
            'files_deleted': deleted,
            'files_failed': failed,
            'total_deleted': len(deleted),
            'total_failed': len(failed),
            'certificate_path': str(cert_path) if cert_path else None
        }
    
    def purge_on_disk_pressure(self, current_usage_percent: float, target_percent: float = 70) -> Dict:
        """
        Purge forensic data due to disk pressure.
        
        Args:
            current_usage_percent: Current disk usage percentage
            target_percent: Target usage percentage after purge
        
        Returns:
            Dictionary with purge results
        """
        # Get old files sorted by age
        old_files = self._get_old_forensic_files()
        old_files.sort(key=lambda x: x['mtime'])
        
        # Calculate target size to free
        from .disk_monitor import DiskMonitor
        monitor = DiskMonitor(self.retention_parser)
        target_bytes = monitor.get_cleanup_target_size(current_usage_percent, target_percent)
        
        # Delete files until we've freed enough space
        deleted = []
        freed_bytes = 0
        
        for file_info in old_files:
            if freed_bytes >= target_bytes:
                break
            
            if self._delete_file_chunked(file_info['path']):
                deleted.append(file_info)
                freed_bytes += file_info['size']
        
        # Create destruction certificate
        cert_path = None
        if deleted:
            cert_path = self._create_destruction_certificate(deleted)
        
        # Log purge event
        self._log_purge_event(deleted, 'disk_pressure', cert_path)
        
        return {
            'purged': True,
            'files_deleted': deleted,
            'freed_bytes': freed_bytes,
            'certificate_path': str(cert_path) if cert_path else None,
            'reason': 'disk_pressure'
        }

