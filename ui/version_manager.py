# Path and File Name : /home/ransomeye/rebuild/ui/version_manager.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: Dashboard version history manager - immutable version snapshots for personal dashboards

"""
RansomEye Dashboard Version Manager
- Manages immutable version history for personal dashboards
- Captures versions on save, rename, duplicate, import
- Enforces retention policy (keep last N versions)
- Atomic writes with audit logging
- Fail-closed on version capture failure
"""

import json
import logging
import os
import hashlib
import shutil
from pathlib import Path
from typing import Dict, List, Any, Optional
from datetime import datetime, timezone

logger = logging.getLogger(__name__)


class VersionManager:
    """Manager for dashboard version history."""
    
    def __init__(self, base_dir: Path, max_versions: int = 50):
        """
        Initialize version manager.
        
        Args:
            base_dir: Base directory for dashboard storage (parent of system dashboards)
            max_versions: Maximum number of versions to keep per dashboard (default: 50)
        """
        self.base_dir = Path(base_dir)
        self.versions_dir = self.base_dir / 'dashboard_versions'
        self.versions_dir.mkdir(parents=True, exist_ok=True)
        self.max_versions = max_versions
        
        # Audit log file
        self.audit_log_file = self.base_dir / 'version_audit.log'
    
    def get_user_id(self) -> str:
        """
        Get current user ID from environment.
        
        Returns:
            User ID string, defaults to 'system' if not set
        """
        user_id = os.environ.get('RANSOMEYE_UI_USER_ID', 'system')
        # Sanitize user_id (remove path separators and dangerous chars)
        user_id = user_id.replace('/', '_').replace('\\', '_').replace('..', '_')
        if not user_id or user_id.strip() == '':
            user_id = 'system'
        return user_id.strip()
    
    def get_version_dir(self, dashboard_name: str, user_id: Optional[str] = None) -> Path:
        """
        Get version directory for a specific dashboard.
        
        Args:
            dashboard_name: Dashboard name
            user_id: User ID (defaults to current user from env)
            
        Returns:
            Path to dashboard's version directory
        """
        if user_id is None:
            user_id = self.get_user_id()
        
        # Sanitize dashboard name for filesystem
        safe_name = dashboard_name.replace('/', '_').replace('\\', '_').replace('..', '_')
        version_dir = self.versions_dir / user_id / safe_name
        version_dir.mkdir(parents=True, exist_ok=True)
        return version_dir
    
    def _calculate_hash(self, dashboard_json: Dict[str, Any]) -> str:
        """
        Calculate SHA-256 hash of dashboard JSON.
        
        Args:
            dashboard_json: Dashboard definition dict
            
        Returns:
            Hex digest of SHA-256 hash
        """
        json_str = json.dumps(dashboard_json, sort_keys=True, ensure_ascii=False)
        return hashlib.sha256(json_str.encode('utf-8')).hexdigest()
    
    def capture_version(self, dashboard: Dict[str, Any], dashboard_name: str, 
                       action: str, user_id: Optional[str] = None) -> bool:
        """
        Capture an immutable version snapshot of a dashboard.
        
        Args:
            dashboard: Dashboard definition dict (must be validated)
            dashboard_name: Dashboard name
            action: Action that triggered version capture (save, rename, duplicate, import)
            user_id: User ID (defaults to current user from env)
            
        Returns:
            True if version captured successfully, False otherwise
            
        Note:
            This method is fail-closed - if version capture fails, the operation should fail.
        """
        if user_id is None:
            user_id = self.get_user_id()
        
        # Ensure dashboard name matches
        dashboard['name'] = dashboard_name
        
        # Calculate JSON hash
        json_hash = self._calculate_hash(dashboard)
        
        # Generate version ID (timestamp-based for uniqueness and ordering)
        timestamp = datetime.now(timezone.utc)
        version_id = timestamp.strftime('%Y%m%d_%H%M%S_%f')
        
        version_dir = self.get_version_dir(dashboard_name, user_id)
        
        try:
            # Create version file with atomic write
            version_file = version_dir / f"{version_id}.json"
            temp_file = version_dir / f"{version_id}.json.tmp"
            
            # Create version metadata
            version_data = {
                'version_id': version_id,
                'timestamp': timestamp.isoformat(),
                'user_id': user_id,
                'dashboard_name': dashboard_name,
                'action': action,
                'json_hash': json_hash,
                'dashboard': dashboard  # Full dashboard JSON
            }
            
            # Atomic write
            with open(temp_file, 'w', encoding='utf-8') as f:
                json.dump(version_data, f, indent=2, ensure_ascii=False)
            
            # Set permissions
            temp_file.chmod(0o644)
            
            # Atomic rename
            temp_file.replace(version_file)
            
            # Enforce retention policy (prune old versions)
            self._enforce_retention(dashboard_name, user_id)
            
            # Audit log
            self._audit_log('capture_version', user_id, dashboard_name, 
                          success=True, 
                          error=f"version_id:{version_id},action:{action},hash:{json_hash[:16]}")
            
            logger.info(f"Captured version {version_id} for dashboard '{dashboard_name}' (action: {action}, user: {user_id})")
            return True
            
        except Exception as e:
            logger.error(f"Error capturing version for dashboard '{dashboard_name}': {e}", exc_info=True)
            self._audit_log('capture_version', user_id, dashboard_name, 
                          success=False, error=str(e))
            return False
    
    def _enforce_retention(self, dashboard_name: str, user_id: Optional[str] = None):
        """
        Enforce retention policy by pruning old versions.
        
        Args:
            dashboard_name: Dashboard name
            user_id: User ID (defaults to current user from env)
        """
        if user_id is None:
            user_id = self.get_user_id()
        
        version_dir = self.get_version_dir(dashboard_name, user_id)
        
        if not version_dir.exists():
            return
        
        try:
            # Get all version files
            version_files = sorted(version_dir.glob("*.json"), key=lambda p: p.name, reverse=True)
            
            # If we have more than max_versions, delete the oldest
            if len(version_files) > self.max_versions:
                files_to_delete = version_files[self.max_versions:]
                for old_file in files_to_delete:
                    try:
                        old_file.unlink()
                        logger.debug(f"Pruned old version: {old_file.name}")
                    except Exception as e:
                        logger.warning(f"Failed to prune version {old_file.name}: {e}")
                
                if files_to_delete:
                    logger.info(f"Pruned {len(files_to_delete)} old versions for dashboard '{dashboard_name}' (kept {self.max_versions})")
                    
        except Exception as e:
            logger.error(f"Error enforcing retention for dashboard '{dashboard_name}': {e}", exc_info=True)
    
    def list_versions(self, dashboard_name: str, user_id: Optional[str] = None, 
                     include_json: bool = False) -> List[Dict[str, Any]]:
        """
        List all versions for a dashboard (metadata only by default).
        
        Args:
            dashboard_name: Dashboard name
            user_id: User ID (defaults to current user from env)
            include_json: If True, include full dashboard JSON in response
            
        Returns:
            List of version metadata dicts, sorted by timestamp (newest first)
        """
        if user_id is None:
            user_id = self.get_user_id()
        
        version_dir = self.get_version_dir(dashboard_name, user_id)
        
        if not version_dir.exists():
            return []
        
        versions = []
        
        try:
            # Get all version files
            version_files = sorted(version_dir.glob("*.json"), key=lambda p: p.name, reverse=True)
            
            for version_file in version_files:
                try:
                    with open(version_file, 'r', encoding='utf-8') as f:
                        version_data = json.load(f)
                    
                    # Extract metadata (exclude full dashboard JSON unless requested)
                    version_metadata = {
                        'version_id': version_data.get('version_id'),
                        'timestamp': version_data.get('timestamp'),
                        'user_id': version_data.get('user_id'),
                        'dashboard_name': version_data.get('dashboard_name'),
                        'action': version_data.get('action'),
                        'json_hash': version_data.get('json_hash'),
                        'json_hash_short': version_data.get('json_hash', '')[:16] if version_data.get('json_hash') else ''
                    }
                    
                    # Include full dashboard JSON if requested
                    if include_json:
                        version_metadata['dashboard'] = version_data.get('dashboard')
                    
                    versions.append(version_metadata)
                    
                except json.JSONDecodeError as e:
                    logger.warning(f"Invalid JSON in version file {version_file}: {e}")
                    continue
                except Exception as e:
                    logger.warning(f"Error reading version file {version_file}: {e}")
                    continue
            
            # Sort by timestamp (newest first)
            versions.sort(key=lambda v: v.get('timestamp', ''), reverse=True)
            
        except Exception as e:
            logger.error(f"Error listing versions for dashboard '{dashboard_name}': {e}", exc_info=True)
        
        return versions
    
    def get_version(self, dashboard_name: str, version_id: str, 
                   user_id: Optional[str] = None) -> Optional[Dict[str, Any]]:
        """
        Get a specific version by version_id.
        
        Args:
            dashboard_name: Dashboard name
            version_id: Version ID
            user_id: User ID (defaults to current user from env)
            
        Returns:
            Version data dict or None if not found
        """
        if user_id is None:
            user_id = self.get_user_id()
        
        version_dir = self.get_version_dir(dashboard_name, user_id)
        version_file = version_dir / f"{version_id}.json"
        
        if not version_file.exists():
            return None
        
        try:
            with open(version_file, 'r', encoding='utf-8') as f:
                return json.load(f)
        except Exception as e:
            logger.error(f"Error reading version {version_id} for dashboard '{dashboard_name}': {e}", exc_info=True)
            return None
    
    def restore_version(self, dashboard_name: str, version_id: str,
                       user_id: Optional[str] = None,
                       overlay_manager: Optional[Any] = None) -> bool:
        """
        Restore a dashboard to a specific version.
        
        This method:
        1. Retrieves the version dashboard JSON
        2. Backs up the current overlay (if exists)
        3. Replaces current overlay with version JSON
        4. Creates a new version snapshot with action='restore'
        
        Args:
            dashboard_name: Dashboard name
            version_id: Version ID to restore to
            user_id: User ID (defaults to current user from env)
            overlay_manager: OverlayManager instance (for saving overlay)
            
        Returns:
            True if restore succeeded, False otherwise
            
        Note:
            This is fail-closed - if restore fails, the current overlay is not modified.
        """
        if user_id is None:
            user_id = self.get_user_id()
        
        # Get version data
        version_data = self.get_version(dashboard_name, version_id, user_id)
        if not version_data:
            logger.error(f"Version '{version_id}' not found for dashboard '{dashboard_name}'")
            self._audit_log('restore_version', user_id, dashboard_name,
                          success=False, error=f"version_not_found:{version_id}")
            return False
        
        # Validate version data structure
        if 'dashboard' not in version_data:
            logger.error(f"Version '{version_id}' for dashboard '{dashboard_name}' is corrupted (missing dashboard data)")
            self._audit_log('restore_version', user_id, dashboard_name,
                          success=False, error="version_corrupted")
            return False
        
        # Get dashboard JSON from version
        dashboard_json = version_data.get('dashboard')
        if not dashboard_json:
            logger.error(f"Version '{version_id}' dashboard JSON is empty")
            self._audit_log('restore_version', user_id, dashboard_name,
                          success=False, error="empty_dashboard_json")
            return False
        
        # Ensure dashboard name matches
        dashboard_json['name'] = dashboard_name
        
        # Restore via overlay manager (if provided)
        if overlay_manager:
            try:
                # Save overlay with 'restore' action - this will create backup and capture new version
                success = overlay_manager.save_overlay(dashboard_json, dashboard_name, user_id, version_action='restore')
                
                if success:
                    self._audit_log('restore_version', user_id, dashboard_name,
                                  success=True,
                                  error=f"restored_to_version:{version_id},hash:{version_data.get('json_hash', '')[:16]}")
                    logger.info(f"Restored dashboard '{dashboard_name}' to version {version_id} (user: {user_id})")
                else:
                    self._audit_log('restore_version', user_id, dashboard_name,
                                  success=False, error="overlay_save_failed")
                
                return success
                
            except Exception as e:
                logger.error(f"Error restoring dashboard '{dashboard_name}' to version {version_id}: {e}", exc_info=True)
                self._audit_log('restore_version', user_id, dashboard_name,
                              success=False, error=str(e))
                return False
        else:
            logger.error("OverlayManager not provided to restore_version - cannot save overlay")
            self._audit_log('restore_version', user_id, dashboard_name,
                          success=False, error="overlay_manager_missing")
            return False
    
    def _audit_log(self, action: str, user_id: str, dashboard_name: str, 
                   success: bool, error: Optional[str] = None):
        """
        Write audit log entry.
        
        Args:
            action: Action performed (capture_version, etc.)
            user_id: User ID
            dashboard_name: Dashboard name
            success: Whether action succeeded
            error: Optional error message or additional context
        """
        try:
            timestamp = datetime.now(timezone.utc).isoformat()
            log_entry = {
                'timestamp': timestamp,
                'action': action,
                'user_id': user_id,
                'dashboard_name': dashboard_name,
                'success': success
            }
            if error:
                log_entry['error'] = error
            
            with open(self.audit_log_file, 'a', encoding='utf-8') as f:
                f.write(json.dumps(log_entry) + '\n')
        except Exception as e:
            logger.error(f"Failed to write version audit log: {e}", exc_info=True)

