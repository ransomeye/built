# Path and File Name : /home/ransomeye/rebuild/ui/settings_manager.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: UI Settings Manager - Database operations and audit logging

"""
UI Settings Manager:
- Database operations for settings persistence
- Audit logging integration with immutable_audit_log
- Fail-safe defaults when settings missing
"""

import logging
import json
import hashlib
import uuid
from typing import Dict, Optional, Tuple
from datetime import datetime, timezone
import psycopg2
from psycopg2.extras import RealDictCursor

from settings import (
    validate_settings,
    get_default_settings,
    merge_with_defaults,
    get_user_identity,
    SettingsValidationError
)
from schema_helper import SchemaAwareDB

logger = logging.getLogger(__name__)


class SettingsManager:
    """Manages UI settings persistence and retrieval."""
    
    def __init__(self, db_conn: psycopg2.extensions.connection):
        """
        Initialize settings manager.
        
        Args:
            db_conn: PostgreSQL connection (must be in ransomeye schema context)
        """
        self.conn = db_conn
        self.db = SchemaAwareDB(db_conn)
        self._ensure_table_exists()
    
    def _ensure_table_exists(self):
        """Ensure UI settings table exists (fail-soft if missing)."""
        if not self.db.table_exists("ransomeye", "ui_settings"):
            logger.warning("ui_settings table does not exist. Run schema migration first.")
    
    def get_settings(self, user_id: Optional[str] = None) -> Dict[str, str]:
        """
        Get UI settings for a user (with fail-safe defaults).
        
        Args:
            user_id: User identity (defaults to system-default if None)
            
        Returns:
            Settings dictionary with theme, density, font_size
        """
        if user_id is None:
            user_id = get_user_identity()
        
        # Try to fetch from database
        if self.db.table_exists("ransomeye", "ui_settings"):
            try:
                cursor = self.conn.cursor(cursor_factory=RealDictCursor)
                cursor.execute("""
                    SELECT theme, density, font_size
                    FROM ransomeye.ui_settings
                    WHERE user_identity = %s
                """, (user_id,))
                row = cursor.fetchone()
                cursor.close()
                
                if row:
                    return {
                        "theme": row["theme"],
                        "density": row["density"],
                        "font_size": row["font_size"]
                    }
            except Exception as e:
                logger.error(f"Error fetching settings for user {user_id}: {e}", exc_info=True)
        
        # Fallback to defaults (fail-safe)
        return get_default_settings()
    
    def save_settings(
        self, 
        settings: Dict[str, str], 
        user_id: Optional[str] = None,
        audit_actor: Optional[str] = None
    ) -> Tuple[bool, Optional[str]]:
        """
        Save UI settings for a user.
        
        Args:
            settings: Validated settings dictionary
            user_id: User identity (defaults to system-default if None)
            audit_actor: Actor identifier for audit logging (defaults to 'ui_server')
            
        Returns:
            Tuple of (success: bool, error_message: Optional[str])
        """
        if user_id is None:
            user_id = get_user_identity()
        
        if audit_actor is None:
            audit_actor = "ui_server"
        
        # Validate settings
        try:
            validated = validate_settings(settings)
            # Merge with defaults to ensure all keys present
            final_settings = merge_with_defaults(validated)
        except SettingsValidationError as e:
            logger.error(f"Settings validation failed: {e}")
            return False, str(e)
        
        # Ensure table exists
        if not self.db.table_exists("ransomeye", "ui_settings"):
            logger.error("Cannot save settings: ui_settings table does not exist")
            return False, "Settings table not found. Run schema migration first."
        
        try:
            cursor = self.conn.cursor()
            
            # Upsert settings (INSERT ... ON CONFLICT UPDATE)
            cursor.execute("""
                INSERT INTO ransomeye.ui_settings (user_identity, theme, density, font_size, created_at, updated_at)
                VALUES (%s, %s, %s, %s, now(), now())
                ON CONFLICT (user_identity)
                DO UPDATE SET
                    theme = EXCLUDED.theme,
                    density = EXCLUDED.density,
                    font_size = EXCLUDED.font_size,
                    updated_at = now()
                RETURNING settings_id
            """, (
                user_id,
                final_settings["theme"],
                final_settings["density"],
                final_settings["font_size"]
            ))
            
            settings_id = cursor.fetchone()[0]
            self.conn.commit()
            cursor.close()
            
            # Audit log the settings change
            try:
                self._audit_settings_change(
                    settings_id=settings_id,
                    user_id=user_id,
                    settings=final_settings,
                    actor=audit_actor
                )
            except Exception as e:
                logger.error(f"Failed to audit settings change (non-fatal): {e}", exc_info=True)
                # Non-fatal: settings saved successfully even if audit fails
            
            logger.info(f"Settings saved successfully for user {user_id}")
            return True, None
            
        except Exception as e:
            logger.error(f"Error saving settings for user {user_id}: {e}", exc_info=True)
            try:
                self.conn.rollback()
            except:
                pass
            return False, f"Database error: {str(e)}"
    
    def _audit_settings_change(
        self,
        settings_id: uuid.UUID,
        user_id: str,
        settings: Dict[str, str],
        actor: str
    ):
        """
        Log settings change to immutable_audit_log.
        
        Args:
            settings_id: Settings record UUID
            user_id: User identity
            settings: Settings dictionary
            actor: Actor identifier
        """
        if not self.db.table_exists("ransomeye", "immutable_audit_log"):
            logger.warning("immutable_audit_log table does not exist, skipping audit")
            return
        
        # Build audit payload
        payload = {
            "settings_id": str(settings_id),
            "user_identity": user_id,
            "theme": settings["theme"],
            "density": settings["density"],
            "font_size": settings["font_size"],
            "timestamp": datetime.now(timezone.utc).isoformat()
        }
        
        payload_json = json.dumps(payload, sort_keys=True)
        payload_sha256 = hashlib.sha256(payload_json.encode('utf-8')).digest()
        
        # Get previous audit hash (for chain)
        cursor = self.conn.cursor()
        
        # Get last audit entry for chain linkage
        cursor.execute("""
            SELECT chain_hash_sha256, audit_id, payload_sha256
            FROM ransomeye.immutable_audit_log
            ORDER BY created_at DESC
            LIMIT 1
        """)
        
        prev_chain_hash = None
        prev_audit_id = None
        prev_payload_sha256 = None
        row = cursor.fetchone()
        if row:
            prev_chain_hash = row[0]
            prev_audit_id = row[1]
            prev_payload_sha256 = row[2]
        
        # Compute chain hash: SHA256(prev_chain_hash || payload_sha256 || metadata)
        # Metadata: action + object_type + object_id as bytes
        if prev_chain_hash:
            metadata = f"update_ui_settings|other|{settings_id}".encode('utf-8')
            chain_input = prev_chain_hash + payload_sha256 + metadata
        else:
            # Genesis entry: SHA256(payload_sha256 || metadata)
            metadata = f"update_ui_settings|other|{settings_id}".encode('utf-8')
            chain_input = payload_sha256 + metadata
        chain_hash_sha256 = hashlib.sha256(chain_input).digest()
        
        # Insert audit entry
        audit_id = uuid.uuid4()
        cursor.execute("""
            INSERT INTO ransomeye.immutable_audit_log (
                audit_id, created_at, action, object_type, object_id,
                payload_json, payload_sha256, prev_audit_id, prev_payload_sha256,
                chain_hash_sha256, signature_status
            )
            VALUES (
                %s, now(), %s, %s, %s, %s, %s, %s, %s, %s, 'unknown'
            )
        """, (
            audit_id,
            "update_ui_settings",
            "other",  # trust_object_type enum value
            settings_id,
            payload_json,
            payload_sha256,
            prev_audit_id,
            prev_payload_sha256,  # Previous entry's payload_sha256 for chain linkage
            chain_hash_sha256
        ))
        
        self.conn.commit()
        cursor.close()
        
        logger.info(f"Settings change audited: audit_id={audit_id}, user={user_id}")

