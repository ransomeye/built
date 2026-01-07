# Path and File Name : /home/ransomeye/rebuild/ui/overlay_manager.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: User dashboard overlay manager - handles per-user dashboard customizations as overlays on system dashboards

"""
RansomEye Dashboard Overlay Manager
- Manages per-user dashboard overlays (customizations)
- System dashboards remain immutable
- User overlays are stored separately and merged at load time
- Atomic writes with backups
- Strict validation and audit logging
"""

import json
import logging
import os
import shutil
from pathlib import Path
from typing import Dict, List, Any, Optional
from datetime import datetime, timezone

logger = logging.getLogger(__name__)

# Import version manager (circular import handled lazily)
_version_manager = None

def get_version_manager(base_dir: Path):
    """Get or create version manager instance (lazy import to avoid circular dependency)."""
    global _version_manager
    if _version_manager is None:
        from version_manager import VersionManager
        _version_manager = VersionManager(base_dir)
    return _version_manager


class OverlayManager:
    """Manager for user dashboard overlays."""
    
    def __init__(self, base_dir: Path):
        """
        Initialize overlay manager.
        
        Args:
            base_dir: Base directory for dashboard storage (parent of system dashboards)
        """
        self.base_dir = Path(base_dir)
        self.overlays_dir = self.base_dir / 'user_overlays'
        self.overlays_dir.mkdir(parents=True, exist_ok=True)
        
        # Audit log file
        self.audit_log_file = self.base_dir / 'overlay_audit.log'
    
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
    
    def get_user_overlay_dir(self, user_id: Optional[str] = None) -> Path:
        """
        Get overlay directory for a specific user.
        
        Args:
            user_id: User ID (defaults to current user from env)
            
        Returns:
            Path to user's overlay directory
        """
        if user_id is None:
            user_id = self.get_user_id()
        user_dir = self.overlays_dir / user_id
        user_dir.mkdir(parents=True, exist_ok=True)
        return user_dir
    
    def get_overlay_path(self, dashboard_name: str, user_id: Optional[str] = None) -> Path:
        """
        Get path to user overlay file for a dashboard.
        
        Args:
            dashboard_name: Name of dashboard (without .json extension)
            user_id: User ID (defaults to current user from env)
            
        Returns:
            Path to overlay JSON file
        """
        user_dir = self.get_user_overlay_dir(user_id)
        return user_dir / f"{dashboard_name}.json"
    
    def load_overlay(self, dashboard_name: str, user_id: Optional[str] = None) -> Optional[Dict[str, Any]]:
        """
        Load user overlay for a dashboard.
        
        Args:
            dashboard_name: Name of dashboard
            user_id: User ID (defaults to current user from env)
            
        Returns:
            Overlay dict or None if not found
        """
        overlay_path = self.get_overlay_path(dashboard_name, user_id)
        
        if not overlay_path.exists():
            return None
        
        try:
            with open(overlay_path, 'r', encoding='utf-8') as f:
                overlay = json.load(f)
            
            # Validate overlay structure (must have 'name' matching dashboard_name)
            if not isinstance(overlay, dict):
                logger.error(f"Invalid overlay structure (not a dict): {overlay_path}")
                return None
            
            if overlay.get('name') != dashboard_name:
                logger.warning(f"Overlay name mismatch: expected '{dashboard_name}', got '{overlay.get('name')}'")
                overlay['name'] = dashboard_name
            
            logger.debug(f"Loaded overlay for dashboard '{dashboard_name}' (user: {user_id or self.get_user_id()})")
            return overlay
            
        except json.JSONDecodeError as e:
            logger.error(f"Invalid JSON in overlay {overlay_path}: {e}")
            return None
        except Exception as e:
            logger.error(f"Error loading overlay {overlay_path}: {e}", exc_info=True)
            return None
    
    def save_overlay(self, overlay: Dict[str, Any], dashboard_name: str, user_id: Optional[str] = None, 
                    version_action: str = 'save') -> bool:
        """
        Save user overlay with atomic write and backup.
        
        Args:
            overlay: Overlay dict (must be validated)
            dashboard_name: Dashboard name
            user_id: User ID (defaults to current user from env)
            version_action: Action for version capture (save, create, duplicate, import, rename)
            
        Returns:
            True if saved successfully, False otherwise
        """
        if user_id is None:
            user_id = self.get_user_id()
        
        # Ensure overlay name matches
        overlay['name'] = dashboard_name
        
        overlay_path = self.get_overlay_path(dashboard_name, user_id)
        
        try:
            # Create backup if file exists
            backup_path = None
            if overlay_path.exists():
                backup_path = overlay_path.parent / f"{dashboard_name}.json.backup"
                shutil.copy2(overlay_path, backup_path)
                logger.debug(f"Created overlay backup: {backup_path}")
            
            # Preserve original file permissions if exists
            original_mode = None
            if overlay_path.exists():
                original_mode = overlay_path.stat().st_mode
            
            # Atomic write: write to temp file first, then rename
            temp_path = overlay_path.parent / f"{dashboard_name}.json.tmp"
            
            with open(temp_path, 'w', encoding='utf-8') as f:
                json.dump(overlay, f, indent=2, ensure_ascii=False)
            
            # Restore permissions if original existed
            if original_mode:
                temp_path.chmod(original_mode)
            else:
                # Default permissions: rw-r--r--
                temp_path.chmod(0o644)
            
            # Atomic rename
            temp_path.replace(overlay_path)
            
            # Capture version snapshot (fail-closed: if version capture fails, operation fails)
            version_manager = get_version_manager(self.base_dir)
            version_captured = version_manager.capture_version(overlay, dashboard_name, version_action, user_id)
            if not version_captured:
                logger.error(f"Version capture failed for dashboard '{dashboard_name}' - operation aborted")
                self._audit_log('save_overlay', user_id, dashboard_name, success=False, error="version_capture_failed")
                # Restore from backup if exists
                if backup_path and backup_path.exists():
                    try:
                        backup_path.replace(overlay_path)
                        logger.info(f"Restored overlay from backup due to version capture failure: {dashboard_name}")
                    except Exception as restore_error:
                        logger.error(f"Failed to restore overlay from backup: {restore_error}")
                return False
            
            # Audit log
            self._audit_log('save_overlay', user_id, dashboard_name, success=True)
            
            logger.info(f"Saved overlay for dashboard '{dashboard_name}' (user: {user_id})")
            return True
            
        except Exception as e:
            logger.error(f"Error saving overlay {overlay_path}: {e}", exc_info=True)
            
            # Restore from backup if write failed
            if backup_path and backup_path.exists() and not overlay_path.exists():
                try:
                    backup_path.replace(overlay_path)
                    logger.info(f"Restored overlay from backup: {dashboard_name}")
                except Exception as restore_error:
                    logger.error(f"Failed to restore overlay from backup: {restore_error}")
            
            self._audit_log('save_overlay', user_id, dashboard_name, success=False, error=str(e))
            return False
    
    def delete_overlay(self, dashboard_name: str, user_id: Optional[str] = None) -> bool:
        """
        Delete user overlay (restores to system dashboard).
        
        Args:
            dashboard_name: Dashboard name
            user_id: User ID (defaults to current user from env)
            
        Returns:
            True if deleted successfully, False otherwise
        """
        if user_id is None:
            user_id = self.get_user_id()
        
        overlay_path = self.get_overlay_path(dashboard_name, user_id)
        
        if not overlay_path.exists():
            logger.debug(f"Overlay does not exist: {overlay_path}")
            return True  # Already deleted
        
        try:
            # Create backup before deletion
            backup_path = overlay_path.parent / f"{dashboard_name}.json.backup"
            if overlay_path.exists():
                shutil.copy2(overlay_path, backup_path)
            
            overlay_path.unlink()
            
            # Audit log
            self._audit_log('delete_overlay', user_id, dashboard_name, success=True)
            
            logger.info(f"Deleted overlay for dashboard '{dashboard_name}' (user: {user_id})")
            return True
            
        except Exception as e:
            logger.error(f"Error deleting overlay {overlay_path}: {e}", exc_info=True)
            self._audit_log('delete_overlay', user_id, dashboard_name, success=False, error=str(e))
            return False
    
    def merge_overlay(self, system_dashboard: Dict[str, Any], overlay: Optional[Dict[str, Any]]) -> Dict[str, Any]:
        """
        Merge system dashboard with user overlay.
        
        Merge strategy:
        - Top-level fields: overlay overrides system (except 'name' which must match)
        - Panels: overlay panels replace system panels by ID, then append new panels
        - Deep merge for nested structures (e.g., data_source)
        
        Args:
            system_dashboard: System dashboard dict
            overlay: User overlay dict (can be None)
            
        Returns:
            Merged dashboard dict
        """
        if overlay is None:
            return system_dashboard.copy()
        
        # Start with system dashboard copy
        merged = system_dashboard.copy()
        
        # Merge top-level fields (except 'name' and 'panels' which need special handling)
        for key, value in overlay.items():
            if key == 'name':
                # Name must match, already validated
                continue
            elif key == 'panels':
                # Panels need special merge logic
                continue
            elif key in ['title', 'description', 'category', 'type', 'folder_id']:
                # Simple override for these fields
                merged[key] = value
            else:
                # For other fields, do deep merge if dict, else override
                if isinstance(value, dict) and isinstance(merged.get(key), dict):
                    merged[key] = {**merged.get(key, {}), **value}
                else:
                    merged[key] = value
        
        # Merge panels: overlay panels override system panels by ID
        if 'panels' in overlay:
            system_panels = {panel['id']: panel for panel in merged.get('panels', [])}
            overlay_panels = {panel['id']: panel for panel in overlay['panels']}
            
            # Start with system panels
            merged_panels = list(system_panels.values())
            
            # Override with overlay panels (by ID)
            for panel_id, overlay_panel in overlay_panels.items():
                # Find existing panel index
                found = False
                for i, panel in enumerate(merged_panels):
                    if panel['id'] == panel_id:
                        # Deep merge panel (overlay overrides system)
                        merged_panels[i] = {**panel, **overlay_panel}
                        found = True
                        break
                
                # If panel not found, append it
                if not found:
                    merged_panels.append(overlay_panel)
            
            merged['panels'] = merged_panels
        
        return merged
    
    def list_user_overlays(self, user_id: Optional[str] = None) -> List[str]:
        """
        List all dashboard names that have overlays for a user.
        
        Args:
            user_id: User ID (defaults to current user from env)
            
        Returns:
            List of dashboard names (without .json extension)
        """
        if user_id is None:
            user_id = self.get_user_id()
        
        user_dir = self.get_user_overlay_dir(user_id)
        
        if not user_dir.exists():
            return []
        
        overlays = []
        for json_file in user_dir.glob("*.json"):
            # Skip backup and temp files
            if json_file.suffix == '.json' and not json_file.name.endswith('.backup') and not json_file.name.endswith('.tmp'):
                overlays.append(json_file.stem)
        
        return sorted(overlays)
    
    def has_overlay(self, dashboard_name: str, user_id: Optional[str] = None) -> bool:
        """
        Check if user has an overlay for a dashboard.
        
        Args:
            dashboard_name: Dashboard name
            user_id: User ID (defaults to current user from env)
            
        Returns:
            True if overlay exists, False otherwise
        """
        overlay_path = self.get_overlay_path(dashboard_name, user_id)
        return overlay_path.exists()
    
    def create_personal_dashboard(self, dashboard_name: str, title: str, 
                                   source_dashboard: Optional[str] = None,
                                   user_id: Optional[str] = None) -> bool:
        """
        Create a new personal dashboard.
        
        Args:
            dashboard_name: Dashboard name (slug-safe, validated)
            title: Dashboard title
            source_dashboard: Optional system dashboard name to use as template
            user_id: User ID (defaults to current user from env)
            
        Returns:
            True if created successfully, False otherwise
        """
        if user_id is None:
            user_id = self.get_user_id()
        
        # Check if overlay already exists (fail-closed on collision)
        if self.has_overlay(dashboard_name, user_id):
            logger.error(f"Dashboard '{dashboard_name}' already exists for user '{user_id}'")
            return False
        
        try:
            # Create dashboard structure
            if source_dashboard:
                # Load system dashboard as template
                system_path = self.base_dir / 'dashboards' / f"{source_dashboard}.json"
                if not system_path.exists():
                    logger.error(f"Source dashboard '{source_dashboard}' not found")
                    return False
                
                with open(system_path, 'r', encoding='utf-8') as f:
                    template = json.load(f)
                
                # Create personal dashboard from template
                dashboard = template.copy()
                dashboard['name'] = dashboard_name
                dashboard['title'] = title
                # Keep panels and other structure from template
            else:
                # Create blank minimal dashboard
                dashboard = {
                    'name': dashboard_name,
                    'title': title,
                    'description': '',
                    'folder_id': 'general',
                    'panels': [
                        {
                            'id': 'welcome_panel',
                            'title': 'Welcome',
                            'x': 0,
                            'y': 0,
                            'w': 12,
                            'h': 2,
                            'status': 'info',
                            'content': f'Welcome to {title}. Add panels to customize this dashboard.'
                        }
                    ]
                }
            
            # Save as overlay with 'create' action for version capture
            success = self.save_overlay(dashboard, dashboard_name, user_id, version_action='create')
            
            if success:
                self._audit_log('create_personal_dashboard', user_id, dashboard_name, 
                              success=True, 
                              error=None if not source_dashboard else f"from_template:{source_dashboard}")
            
            return success
            
        except Exception as e:
            logger.error(f"Error creating personal dashboard '{dashboard_name}': {e}", exc_info=True)
            self._audit_log('create_personal_dashboard', user_id, dashboard_name, 
                          success=False, error=str(e))
            return False
    
    def duplicate_dashboard(self, source_dashboard_name: str, new_dashboard_name: str, 
                            new_title: str, user_id: Optional[str] = None) -> bool:
        """
        Duplicate a personal dashboard into a new personal dashboard.
        
        Args:
            source_dashboard_name: Name of source dashboard (must be personal)
            new_dashboard_name: Name for new dashboard (slug-safe, validated)
            new_title: Title for new dashboard
            user_id: User ID (defaults to current user from env)
            
        Returns:
            True if duplicated successfully, False otherwise
        """
        if user_id is None:
            user_id = self.get_user_id()
        
        # Validate source dashboard exists and is personal (has overlay)
        if not self.has_overlay(source_dashboard_name, user_id):
            logger.error(f"Source dashboard '{source_dashboard_name}' is not a personal dashboard (no overlay found)")
            return False
        
        # Check for collision with system dashboard
        system_dashboard_path = self.base_dir / 'dashboards' / f"{new_dashboard_name}.json"
        if system_dashboard_path.exists():
            logger.error(f"Dashboard name '{new_dashboard_name}' conflicts with a system dashboard")
            return False
        
        # Check for collision with existing user overlay
        if self.has_overlay(new_dashboard_name, user_id):
            logger.error(f"Dashboard '{new_dashboard_name}' already exists for user '{user_id}'")
            return False
        
        try:
            # Load source overlay
            source_overlay = self.load_overlay(source_dashboard_name, user_id)
            if not source_overlay:
                logger.error(f"Failed to load source overlay for '{source_dashboard_name}'")
                return False
            
            # Clone overlay
            new_dashboard = source_overlay.copy()
            
            # Update name and title
            new_dashboard['name'] = new_dashboard_name
            new_dashboard['title'] = new_title
            
            # Preserve all other fields (panels, layout, refresh intervals, etc.)
            # The copy() already preserves everything, we just update name and title
            
            # Save as new overlay with 'duplicate' action for version capture
            success = self.save_overlay(new_dashboard, new_dashboard_name, user_id, version_action='duplicate')
            
            if success:
                self._audit_log('duplicate_dashboard', user_id, new_dashboard_name, 
                              success=True, 
                              error=f"from:{source_dashboard_name}")
            
            return success
            
        except Exception as e:
            logger.error(f"Error duplicating dashboard '{source_dashboard_name}' to '{new_dashboard_name}': {e}", exc_info=True)
            self._audit_log('duplicate_dashboard', user_id, new_dashboard_name, 
                          success=False, error=str(e))
            return False
    
    def _audit_log(self, action: str, user_id: str, dashboard_name: str, success: bool, error: Optional[str] = None):
        """
        Write audit log entry.
        
        Args:
            action: Action performed (save_overlay, delete_overlay, create_personal_dashboard, etc.)
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
            logger.error(f"Failed to write audit log: {e}", exc_info=True)

