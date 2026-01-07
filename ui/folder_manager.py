# Path and File Name : /home/ransomeye/rebuild/ui/folder_manager.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: Dashboard folder manager for system-scoped folder organization

"""
RansomEye Dashboard Folder Manager
- System-scoped folder management (no per-user logic)
- JSON-based persistence with atomic writes
- Backup on overwrite
- Strict validation and fail-closed on corruption
- Audit logging for all changes
"""

import json
import logging
import shutil
import re
from pathlib import Path
from typing import Dict, List, Any, Optional, Tuple
from datetime import datetime, timezone

logger = logging.getLogger(__name__)


class FolderManager:
    """Manager for dashboard folder metadata (system-scoped)."""
    
    DEFAULT_FOLDER_ID = "general"
    DEFAULT_FOLDER_NAME = "General"
    
    def __init__(self, folders_file: Path):
        """
        Initialize folder manager.
        
        Args:
            folders_file: Path to JSON file storing folder metadata
        """
        self.folders_file = Path(folders_file)
        self.folders_file.parent.mkdir(parents=True, exist_ok=True)
        self._cache: Optional[Dict[str, Dict]] = None
        self._cache_timestamp: Optional[float] = None
        
        # Ensure default folder exists
        self._ensure_default_folder()
    
    def _ensure_default_folder(self):
        """Ensure default 'General' folder exists."""
        folders = self._load_folders()
        if self.DEFAULT_FOLDER_ID not in folders:
            default_folder = {
                "id": self.DEFAULT_FOLDER_ID,
                "name": self.DEFAULT_FOLDER_NAME,
                "description": "Default folder for dashboards",
                "order": 0
            }
            folders[self.DEFAULT_FOLDER_ID] = default_folder
            self._save_folders(folders)
            logger.info(f"Created default folder: {self.DEFAULT_FOLDER_ID}")
    
    def _load_folders(self) -> Dict[str, Dict[str, Any]]:
        """
        Load folders from JSON file (with caching).
        
        Returns:
            Dict mapping folder_id to folder metadata
        """
        # Check cache
        if self.folders_file.exists():
            mtime = self.folders_file.stat().st_mtime
            if self._cache is not None and self._cache_timestamp == mtime:
                return self._cache.copy()
        
        # Load from file
        folders = {}
        if self.folders_file.exists():
            try:
                with open(self.folders_file, 'r', encoding='utf-8') as f:
                    data = json.load(f)
                
                # Validate structure
                if not isinstance(data, dict):
                    logger.error("Folders file must contain a JSON object")
                    return self._create_default_folders()
                
                # Validate each folder
                for folder_id, folder_data in data.items():
                    if self._validate_folder(folder_data, folder_id):
                        folders[folder_id] = folder_data
                    else:
                        logger.warning(f"Skipping invalid folder: {folder_id}")
                
                # Ensure default folder exists
                if self.DEFAULT_FOLDER_ID not in folders:
                    folders[self.DEFAULT_FOLDER_ID] = {
                        "id": self.DEFAULT_FOLDER_ID,
                        "name": self.DEFAULT_FOLDER_NAME,
                        "description": "Default folder for dashboards",
                        "order": 0
                    }
                
            except json.JSONDecodeError as e:
                logger.error(f"Invalid JSON in folders file: {e}")
                # Create backup of corrupted file
                backup_path = self.folders_file.with_suffix('.json.corrupted')
                if not backup_path.exists():
                    try:
                        shutil.copy2(self.folders_file, backup_path)
                        logger.warning(f"Backed up corrupted file to: {backup_path}")
                    except Exception as backup_error:
                        logger.error(f"Failed to backup corrupted file: {backup_error}")
                return self._create_default_folders()
            except Exception as e:
                logger.error(f"Error loading folders: {e}", exc_info=True)
                return self._create_default_folders()
        else:
            # File doesn't exist, create default
            folders = self._create_default_folders()
            self._save_folders(folders)
        
        # Update cache
        self._cache = folders.copy()
        self._cache_timestamp = self.folders_file.stat().st_mtime if self.folders_file.exists() else 0
        
        return folders
    
    def _create_default_folders(self) -> Dict[str, Dict[str, Any]]:
        """Create default folders structure."""
        return {
            self.DEFAULT_FOLDER_ID: {
                "id": self.DEFAULT_FOLDER_ID,
                "name": self.DEFAULT_FOLDER_NAME,
                "description": "Default folder for dashboards",
                "order": 0
            }
        }
    
    def _validate_folder(self, folder: Dict, folder_id: str) -> bool:
        """
        Validate folder structure.
        
        Args:
            folder: Folder metadata dict
            folder_id: Expected folder ID
            
        Returns:
            True if valid, False otherwise
        """
        if not isinstance(folder, dict):
            return False
        
        # Required fields
        required_fields = ['id', 'name', 'order']
        if not all(field in folder for field in required_fields):
            logger.error(f"Folder {folder_id} missing required fields: {required_fields}")
            return False
        
        # Validate ID matches
        if folder['id'] != folder_id:
            logger.error(f"Folder ID mismatch: expected {folder_id}, got {folder['id']}")
            return False
        
        # Validate ID format (slug-safe)
        if not self._is_valid_folder_id(folder_id):
            logger.error(f"Invalid folder ID format: {folder_id}")
            return False
        
        # Validate name
        if not isinstance(folder['name'], str) or not folder['name'].strip():
            logger.error(f"Folder {folder_id} name must be a non-empty string")
            return False
        
        # Validate order
        if not isinstance(folder['order'], int):
            logger.error(f"Folder {folder_id} order must be an integer")
            return False
        
        # Validate description (optional)
        if 'description' in folder and not isinstance(folder['description'], str):
            logger.error(f"Folder {folder_id} description must be a string")
            return False
        
        # Check for unknown fields (strict whitelist)
        allowed_fields = {'id', 'name', 'description', 'order'}
        for field in folder.keys():
            if field not in allowed_fields:
                logger.error(f"Folder {folder_id} has unknown field: {field}")
                return False
        
        return True
    
    def _is_valid_folder_id(self, folder_id: str) -> bool:
        """
        Validate folder ID is slug-safe.
        
        Args:
            folder_id: Folder ID to validate
            
        Returns:
            True if valid, False otherwise
        """
        if not isinstance(folder_id, str) or not folder_id:
            return False
        
        # Allow lowercase alphanumeric, hyphens, underscores
        # Must start with letter or number
        pattern = r'^[a-z0-9][a-z0-9_-]*$'
        return bool(re.match(pattern, folder_id))
    
    def _save_folders(self, folders: Dict[str, Dict[str, Any]]) -> bool:
        """
        Save folders to JSON file with atomic write and backup.
        
        Args:
            folders: Dict mapping folder_id to folder metadata
            
        Returns:
            True if saved successfully, False otherwise
        """
        try:
            # Create backup if file exists
            backup_path = None
            if self.folders_file.exists():
                backup_path = self.folders_file.with_suffix('.json.backup')
                shutil.copy2(self.folders_file, backup_path)
                logger.info(f"Created backup: {backup_path}")
            
            # Preserve original file permissions if exists
            original_mode = None
            if self.folders_file.exists():
                original_mode = self.folders_file.stat().st_mode
            
            # Atomic write: write to temp file first, then rename
            temp_path = self.folders_file.with_suffix('.json.tmp')
            
            with open(temp_path, 'w', encoding='utf-8') as f:
                json.dump(folders, f, indent=2, ensure_ascii=False)
            
            # Restore permissions if original existed
            if original_mode:
                temp_path.chmod(original_mode)
            else:
                # Default permissions: rw-r--r--
                temp_path.chmod(0o644)
            
            # Atomic rename
            temp_path.replace(self.folders_file)
            
            # Update cache
            self._cache = folders.copy()
            self._cache_timestamp = self.folders_file.stat().st_mtime
            
            logger.info(f"Saved {len(folders)} folders")
            return True
            
        except Exception as e:
            logger.error(f"Error saving folders: {e}", exc_info=True)
            # Restore from backup if write failed
            if backup_path and backup_path.exists() and not self.folders_file.exists():
                try:
                    backup_path.replace(self.folders_file)
                    logger.info("Restored folders from backup")
                except Exception as restore_error:
                    logger.error(f"Failed to restore from backup: {restore_error}")
            return False
    
    def list_folders(self) -> List[Dict[str, Any]]:
        """
        List all folders sorted by order.
        
        Returns:
            List of folder metadata dicts
        """
        folders = self._load_folders()
        folder_list = list(folders.values())
        # Sort by order, then by name
        folder_list.sort(key=lambda f: (f['order'], f['name'].lower()))
        return folder_list
    
    def get_folder(self, folder_id: str) -> Optional[Dict[str, Any]]:
        """
        Get folder by ID.
        
        Args:
            folder_id: Folder ID
            
        Returns:
            Folder metadata dict or None if not found
        """
        folders = self._load_folders()
        return folders.get(folder_id)
    
    def create_folder(self, folder_data: Dict[str, Any]) -> Tuple[bool, Optional[str]]:
        """
        Create a new folder (strict validation, fail-closed).
        
        Args:
            folder_data: Folder metadata dict with id, name, description (optional), order
            
        Returns:
            Tuple of (success: bool, error_message: Optional[str])
        """
        # Validate required fields
        if 'id' not in folder_data:
            return False, "Folder ID is required"
        
        folder_id = folder_data['id']
        
        # Validate ID format
        if not self._is_valid_folder_id(folder_id):
            return False, f"Invalid folder ID format: {folder_id} (must be lowercase alphanumeric, hyphens, underscores, start with letter/number)"
        
        # Check if folder already exists
        folders = self._load_folders()
        if folder_id in folders:
            return False, f"Folder '{folder_id}' already exists"
        
        # Set defaults
        new_folder = {
            "id": folder_id,
            "name": folder_data.get('name', folder_id.replace('_', ' ').title()),
            "description": folder_data.get('description', ''),
            "order": folder_data.get('order', 999)
        }
        
        # Validate folder structure
        if not self._validate_folder(new_folder, folder_id):
            return False, "Invalid folder structure"
        
        # Add folder
        folders[folder_id] = new_folder
        
        # Save
        if self._save_folders(folders):
            logger.info(f"Created folder: {folder_id}")
            # Audit log
            self._audit_log("create", folder_id, new_folder)
            return True, None
        else:
            return False, "Failed to save folder"
    
    def _audit_log(self, action: str, folder_id: str, data: Dict[str, Any]):
        """
        Log folder changes for audit trail.
        
        Args:
            action: Action performed (create, update, delete)
            folder_id: Folder ID
            data: Folder data or change details
        """
        audit_entry = {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "action": action,
            "folder_id": folder_id,
            "data": data
        }
        logger.info(f"Folder audit: {json.dumps(audit_entry)}")
    
    def clear_cache(self):
        """Clear folder cache."""
        self._cache = None
        self._cache_timestamp = None
        logger.info("Folder cache cleared")

