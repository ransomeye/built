# Path and File Name : /home/ransomeye/rebuild/ui/share_manager.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: Dashboard Share Manager - token generation, validation, and revocation for read-only dashboard sharing

"""
Dashboard Share Manager:
- Generates cryptographically strong share tokens
- Validates token access (expiry, revocation)
- Tracks access counts and timestamps
- Enforces read-only permissions
- Audit logging integration
"""

import secrets
import logging
import hashlib
from typing import Dict, Optional, Tuple
from datetime import datetime, timezone, timedelta
import psycopg2
from psycopg2.extras import RealDictCursor
from schema_helper import SchemaAwareDB

logger = logging.getLogger(__name__)


class ShareManager:
    """Manages dashboard share tokens for read-only link-based sharing."""
    
    def __init__(self, db_conn: psycopg2.extensions.connection):
        """
        Initialize share manager.
        
        Args:
            db_conn: PostgreSQL connection (must be in ransomeye schema context)
        """
        self.conn = db_conn
        self.db = SchemaAwareDB(db_conn)
        self._ensure_table_exists()
    
    def _ensure_table_exists(self):
        """Ensure dashboard_share_tokens table exists (fail-soft if missing)."""
        if not self.db.table_exists("ransomeye", "dashboard_share_tokens"):
            logger.warning("dashboard_share_tokens table does not exist. Run schema migration first.")
    
    def generate_token(self) -> str:
        """
        Generate a cryptographically strong share token.
        
        Returns:
            64-character URL-safe token
        """
        # Generate 32 bytes of random data, encode as URL-safe base64 (48 chars)
        # Then add hex digest for additional entropy (16 chars) = 64 chars total
        random_bytes = secrets.token_bytes(32)
        token_part1 = secrets.token_urlsafe(32)[:48]
        token_part2 = hashlib.sha256(random_bytes).hexdigest()[:16]
        return f"{token_part1}{token_part2}"
    
    def create_share(
        self,
        dashboard_name: str,
        owner_user_id: str,
        expires_in_days: Optional[int] = None
    ) -> Optional[Dict[str, any]]:
        """
        Create a new share token for a dashboard.
        
        Args:
            dashboard_name: Dashboard name (slug)
            owner_user_id: User ID of dashboard owner
            expires_in_days: Optional expiration in days (None = no expiration)
            
        Returns:
            Dict with token info or None on failure
        """
        try:
            # Generate token
            token = self.generate_token()
            
            # Calculate expiration
            expires_at = None
            if expires_in_days is not None and expires_in_days > 0:
                expires_at = datetime.now(timezone.utc) + timedelta(days=expires_in_days)
            
            # Insert token
            cursor = self.conn.cursor()
            cursor.execute("SET search_path = ransomeye, public;")
            
            insert_query = """
                INSERT INTO dashboard_share_tokens 
                (token, dashboard_name, owner_user_id, permissions, expires_at)
                VALUES (%s, %s, %s, 'read_only', %s)
                RETURNING token_id, token, dashboard_name, owner_user_id, permissions, 
                          expires_at, created_at, access_count
            """
            
            cursor.execute(insert_query, (token, dashboard_name, owner_user_id, expires_at))
            row = cursor.fetchone()
            
            if not row:
                cursor.close()
                return None
            
            # Convert to dict
            share_info = {
                'token_id': str(row[0]),
                'token': row[1],
                'dashboard_name': row[2],
                'owner_user_id': row[3],
                'permissions': row[4],
                'expires_at': row[5].isoformat() if row[5] else None,
                'created_at': row[6].isoformat() if row[6] else None,
                'access_count': row[5] if len(row) > 7 else 0
            }
            
            self.conn.commit()
            cursor.close()
            
            # Audit log
            self._audit_log('create_share', owner_user_id, dashboard_name, 
                          success=True, error=f"token_id:{share_info['token_id']}")
            
            logger.info(f"Created share token for dashboard '{dashboard_name}' (owner: {owner_user_id})")
            
            return share_info
            
        except Exception as e:
            logger.error(f"Error creating share token: {e}", exc_info=True)
            if self.conn:
                try:
                    self.conn.rollback()
                except:
                    pass
            
            # Audit log failure
            self._audit_log('create_share', owner_user_id, dashboard_name, 
                          success=False, error=str(e))
            
            return None
    
    def validate_token(self, token: str) -> Optional[Dict[str, any]]:
        """
        Validate a share token and return share info if valid.
        
        Args:
            token: Share token to validate
            
        Returns:
            Dict with share info or None if invalid/expired/revoked
        """
        try:
            cursor = self.conn.cursor(cursor_factory=RealDictCursor)
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Query for active, non-expired token
            query = """
                SELECT token_id, token, dashboard_name, owner_user_id, permissions,
                       expires_at, created_at, revoked_at, access_count, last_accessed_at
                FROM dashboard_share_tokens
                WHERE token = %s
                  AND revoked_at IS NULL
                  AND (expires_at IS NULL OR expires_at > now())
            """
            
            cursor.execute(query, (token,))
            row = cursor.fetchone()
            
            if not row:
                cursor.close()
                logger.warning(f"Invalid or expired share token: {token[:16]}...")
                return None
            
            # Convert to dict
            share_info = {
                'token_id': str(row['token_id']),
                'token': row['token'],
                'dashboard_name': row['dashboard_name'],
                'owner_user_id': row['owner_user_id'],
                'permissions': row['permissions'],
                'expires_at': row['expires_at'].isoformat() if row['expires_at'] else None,
                'created_at': row['created_at'].isoformat() if row['created_at'] else None,
                'revoked_at': row['revoked_at'].isoformat() if row['revoked_at'] else None,
                'access_count': row['access_count'],
                'last_accessed_at': row['last_accessed_at'].isoformat() if row['last_accessed_at'] else None
            }
            
            # Update access count and last accessed timestamp
            update_query = """
                UPDATE dashboard_share_tokens
                SET access_count = access_count + 1,
                    last_accessed_at = now()
                WHERE token_id = %s
            """
            cursor.execute(update_query, (row['token_id'],))
            self.conn.commit()
            cursor.close()
            
            # Audit log access
            self._audit_log('access_share', share_info['owner_user_id'], 
                          share_info['dashboard_name'], 
                          success=True, error=f"token_id:{share_info['token_id']}")
            
            logger.info(f"Validated share token for dashboard '{share_info['dashboard_name']}' (access count: {share_info['access_count'] + 1})")
            
            return share_info
            
        except Exception as e:
            logger.error(f"Error validating share token: {e}", exc_info=True)
            if self.conn:
                try:
                    self.conn.rollback()
                except:
                    pass
            return None
    
    def revoke_token(self, token: str, owner_user_id: str) -> bool:
        """
        Revoke a share token (soft delete).
        
        Args:
            token: Share token to revoke
            owner_user_id: User ID of token owner (for authorization)
            
        Returns:
            True if revoked, False otherwise
        """
        try:
            cursor = self.conn.cursor()
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Update revoked_at timestamp (only if owned by user)
            update_query = """
                UPDATE dashboard_share_tokens
                SET revoked_at = now()
                WHERE token = %s
                  AND owner_user_id = %s
                  AND revoked_at IS NULL
                RETURNING dashboard_name
            """
            
            cursor.execute(update_query, (token, owner_user_id))
            row = cursor.fetchone()
            
            if not row:
                cursor.close()
                logger.warning(f"Token not found or not owned by user: {token[:16]}... (owner: {owner_user_id})")
                return False
            
            dashboard_name = row[0]
            self.conn.commit()
            cursor.close()
            
            # Audit log revocation
            self._audit_log('revoke_share', owner_user_id, dashboard_name, 
                          success=True, error=f"token:{token[:16]}...")
            
            logger.info(f"Revoked share token for dashboard '{dashboard_name}' (owner: {owner_user_id})")
            
            return True
            
        except Exception as e:
            logger.error(f"Error revoking share token: {e}", exc_info=True)
            if self.conn:
                try:
                    self.conn.rollback()
                except:
                    pass
            
            # Audit log failure
            self._audit_log('revoke_share', owner_user_id, 'unknown', 
                          success=False, error=str(e))
            
            return False
    
    def list_shares(self, dashboard_name: str, owner_user_id: str) -> list:
        """
        List all active shares for a dashboard.
        
        Args:
            dashboard_name: Dashboard name
            owner_user_id: User ID of dashboard owner
            
        Returns:
            List of share info dicts
        """
        try:
            cursor = self.conn.cursor(cursor_factory=RealDictCursor)
            cursor.execute("SET search_path = ransomeye, public;")
            
            query = """
                SELECT token_id, token, dashboard_name, owner_user_id, permissions,
                       expires_at, created_at, revoked_at, access_count, last_accessed_at
                FROM dashboard_share_tokens
                WHERE dashboard_name = %s
                  AND owner_user_id = %s
                  AND revoked_at IS NULL
                ORDER BY created_at DESC
            """
            
            cursor.execute(query, (dashboard_name, owner_user_id))
            rows = cursor.fetchall()
            cursor.close()
            
            shares = []
            for row in rows:
                share_info = {
                    'token_id': str(row['token_id']),
                    'token': row['token'],
                    'dashboard_name': row['dashboard_name'],
                    'owner_user_id': row['owner_user_id'],
                    'permissions': row['permissions'],
                    'expires_at': row['expires_at'].isoformat() if row['expires_at'] else None,
                    'created_at': row['created_at'].isoformat() if row['created_at'] else None,
                    'access_count': row['access_count'],
                    'last_accessed_at': row['last_accessed_at'].isoformat() if row['last_accessed_at'] else None
                }
                shares.append(share_info)
            
            return shares
            
        except Exception as e:
            logger.error(f"Error listing shares: {e}", exc_info=True)
            return []
    
    def _audit_log(self, action: str, user_id: str, dashboard_name: str, 
                   success: bool, error: Optional[str] = None):
        """
        Log share action to immutable audit log.
        
        Args:
            action: Action name (create_share, access_share, revoke_share)
            user_id: User ID
            dashboard_name: Dashboard name
            success: Whether action succeeded
            error: Optional error message or metadata
        """
        try:
            if not self.db.table_exists("ransomeye", "immutable_audit_log"):
                return
            
            cursor = self.conn.cursor()
            cursor.execute("SET search_path = ransomeye, public;")
            
            action_name = f"share_{action}"
            error_msg = error if error else ("success" if success else "unknown_error")
            
            insert_query = """
                INSERT INTO immutable_audit_log (action, user_id, resource_type, resource_id, success, error_message)
                VALUES (%s, %s, 'dashboard_share', %s, %s, %s)
            """
            
            cursor.execute(insert_query, (action_name, user_id, dashboard_name, success, error_msg))
            self.conn.commit()
            cursor.close()
            
        except Exception as e:
            logger.debug(f"Failed to audit log share action: {e}", exc_info=True)
            # Fail-soft: don't break share operations if audit logging fails

