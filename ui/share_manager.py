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
from typing import Dict, Optional, Tuple, Any
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
    ) -> Optional[Dict[str, Any]]:
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
                'access_count': row[7] if len(row) > 7 else 0
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
    
    def _get_token_status(self, expires_at, revoked_at) -> str:
        """
        Determine token status based on expiry and revocation.
        
        Args:
            expires_at: Expiration datetime or None
            revoked_at: Revocation datetime or None
            
        Returns:
            'active', 'expired', or 'revoked'
        """
        if revoked_at:
            return 'revoked'
        if expires_at:
            if expires_at < datetime.now(timezone.utc):
                return 'expired'
        return 'active'
    
    def validate_token(self, token: str, ip_address: Optional[str] = None, 
                      user_agent: Optional[str] = None, 
                      rate_limited: bool = False) -> Optional[Dict[str, Any]]:
        """
        Validate a share token and return share info if valid.
        
        Centralized expiry check: expired tokens are rejected (returns None).
        Expired token access attempts are audit-logged.
        
        Args:
            token: Share token to validate
            ip_address: Client IP address (best-effort, may be None)
            user_agent: Client User-Agent (best-effort, may be None)
            rate_limited: Whether this access was rate-limited
            
        Returns:
            Dict with share info (including status) or None if invalid/expired/revoked
        """
        try:
            cursor = self.conn.cursor(cursor_factory=RealDictCursor)
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Query for token (including expired/revoked to determine status)
            query = """
                SELECT token_id, token, dashboard_name, owner_user_id, permissions,
                       expires_at, created_at, revoked_at, access_count, last_accessed_at
                FROM dashboard_share_tokens
                WHERE token = %s
            """
            
            cursor.execute(query, (token,))
            row = cursor.fetchone()
            
            if not row:
                # Token doesn't exist - log and return None
                self._log_access(
                    cursor=cursor,
                    token_id=None,
                    token=token,
                    dashboard_name=None,
                    ip_address=ip_address,
                    user_agent=user_agent,
                    rate_limited=rate_limited,
                    access_granted=False
                )
                self.conn.commit()
                cursor.close()
                logger.warning(f"Invalid share token: {token[:16]}...")
                return None
            
            # Determine status
            expires_at_dt = row['expires_at']
            revoked_at_dt = row['revoked_at']
            status = self._get_token_status(expires_at_dt, revoked_at_dt)
            
            # Check if token is valid for access (not revoked and not expired)
            is_valid = (revoked_at_dt is None and 
                       (expires_at_dt is None or expires_at_dt > datetime.now(timezone.utc)))
            
            access_granted = is_valid
            
            # Log access attempt (even if expired/invalid) to share_access_logs
            token_id = row['token_id']
            dashboard_name = row['dashboard_name']
            
            # Log access to share_access_logs (append-only)
            self._log_access(
                cursor=cursor,
                token_id=token_id,
                token=token,
                dashboard_name=dashboard_name,
                ip_address=ip_address,
                user_agent=user_agent,
                rate_limited=rate_limited,
                access_granted=access_granted
            )
            
            # If expired or revoked, audit-log and return None
            if not is_valid:
                self.conn.commit()
                cursor.close()
                
                # Audit log expiry/revocation access attempt
                error_msg = f"token_id:{token_id},status:{status}"
                if rate_limited:
                    error_msg += ",rate_limited:true"
                
                self._audit_log('access_share', row['owner_user_id'], 
                              dashboard_name, 
                              success=False, error=error_msg)
                
                logger.warning(f"Share token access denied: {token[:16]}... (status: {status})")
                return None
            
            # Convert to dict (include status)
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
                'last_accessed_at': row['last_accessed_at'].isoformat() if row['last_accessed_at'] else None,
                'status': status
            }
            
            # Update access count and last accessed timestamp (only if not rate-limited)
            if not rate_limited:
                update_query = """
                    UPDATE dashboard_share_tokens
                    SET access_count = access_count + 1,
                        last_accessed_at = now()
                    WHERE token_id = %s
                """
                cursor.execute(update_query, (row['token_id'],))
            
            self.conn.commit()
            cursor.close()
            
            # Audit log access (with rate-limited flag and metadata presence)
            metadata_present = bool(ip_address or user_agent)
            error_msg = f"token_id:{share_info['token_id']}"
            if rate_limited:
                error_msg += ",rate_limited:true"
            if metadata_present:
                error_msg += ",metadata:present"
            
            self._audit_log('access_share', share_info['owner_user_id'], 
                          share_info['dashboard_name'], 
                          success=True, error=error_msg)
            
            logger.info(f"Validated share token for dashboard '{share_info['dashboard_name']}' "
                       f"(access count: {share_info['access_count'] + 1}, "
                       f"rate_limited: {rate_limited}, metadata: {metadata_present})")
            
            return share_info
            
        except Exception as e:
            logger.error(f"Error validating share token: {e}", exc_info=True)
            if self.conn:
                try:
                    self.conn.rollback()
                except:
                    pass
            return None
    
    def _log_access(self, cursor, token_id: Optional[str], token: str, 
                   dashboard_name: Optional[str], ip_address: Optional[str],
                   user_agent: Optional[str], rate_limited: bool, 
                   access_granted: bool):
        """
        Log access attempt to share_access_logs table (append-only).
        
        Args:
            cursor: Database cursor
            token_id: Token ID (may be None for invalid tokens)
            token: Share token
            dashboard_name: Dashboard name (may be None for invalid tokens)
            ip_address: Client IP address (best-effort)
            user_agent: Client User-Agent (best-effort)
            rate_limited: Whether access was rate-limited
            access_granted: Whether access was granted
        """
        try:
            # Check if share_access_logs table exists
            if not self.db.table_exists("ransomeye", "share_access_logs"):
                logger.debug("share_access_logs table does not exist, skipping access log")
                return
            
            # Insert access log (token_id and dashboard_name can be NULL for invalid tokens)
            insert_query = """
                INSERT INTO share_access_logs 
                (token_id, token, dashboard_name, ip_address, user_agent, rate_limited, access_granted)
                VALUES (%s, %s, %s, %s, %s, %s, %s)
            """
            cursor.execute(insert_query, (
                token_id, token, dashboard_name,
                ip_address, user_agent, rate_limited, access_granted
            ))
        except Exception as e:
            logger.debug(f"Failed to log share access: {e}", exc_info=True)
            # Fail-soft: don't break token validation if logging fails
    
    def rotate_token(self, token: str, owner_user_id: str, 
                     expires_in_days: Optional[int] = None) -> Optional[Dict[str, Any]]:
        """
        Rotate a share token: revoke old token and create new one.
        
        Args:
            token: Share token to rotate
            owner_user_id: User ID of token owner (for authorization)
            expires_in_days: Optional new expiration in days (None = preserve old expiry)
            
        Returns:
            Dict with new token info or None on failure
        """
        try:
            cursor = self.conn.cursor(cursor_factory=RealDictCursor)
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Get current token info (validate ownership and get expiry)
            query = """
                SELECT token_id, dashboard_name, owner_user_id, expires_at, permissions
                FROM dashboard_share_tokens
                WHERE token = %s
                  AND owner_user_id = %s
                  AND revoked_at IS NULL
            """
            
            cursor.execute(query, (token, owner_user_id))
            row = cursor.fetchone()
            
            if not row:
                cursor.close()
                logger.warning(f"Token not found or not owned by user: {token[:16]}... (owner: {owner_user_id})")
                return None
            
            # Check if token is expired (cannot rotate expired tokens)
            old_expires_at = row['expires_at']
            if old_expires_at and old_expires_at < datetime.now(timezone.utc):
                cursor.close()
                logger.warning(f"Cannot rotate expired token: {token[:16]}... (owner: {owner_user_id})")
                return None
            
            old_token_id = str(row['token_id'])
            dashboard_name = row['dashboard_name']
            old_expires_at = row['expires_at']
            permissions = row['permissions']
            
            # Calculate new expiration
            new_expires_at = old_expires_at
            if expires_in_days is not None:
                # Override with new expiry
                if expires_in_days > 0:
                    new_expires_at = datetime.now(timezone.utc) + timedelta(days=expires_in_days)
                else:
                    new_expires_at = None
            # Otherwise preserve old expiry
            
            # Generate new token
            new_token = self.generate_token()
            
            # Revoke old token (set revoked_at)
            revoke_query = """
                UPDATE dashboard_share_tokens
                SET revoked_at = now()
                WHERE token_id = %s
            """
            cursor.execute(revoke_query, (row['token_id'],))
            
            # Create new token
            insert_query = """
                INSERT INTO dashboard_share_tokens 
                (token, dashboard_name, owner_user_id, permissions, expires_at)
                VALUES (%s, %s, %s, %s, %s)
                RETURNING token_id, token, dashboard_name, owner_user_id, permissions, 
                          expires_at, created_at, access_count
            """
            
            cursor.execute(insert_query, (new_token, dashboard_name, owner_user_id, permissions, new_expires_at))
            new_row = cursor.fetchone()
            
            if not new_row:
                cursor.close()
                self.conn.rollback()
                logger.error(f"Failed to create new token during rotation")
                return None
            
            # Convert to dict
            new_share_info = {
                'token_id': str(new_row['token_id']),
                'token': new_row['token'],
                'dashboard_name': new_row['dashboard_name'],
                'owner_user_id': new_row['owner_user_id'],
                'permissions': new_row['permissions'],
                'expires_at': new_row['expires_at'].isoformat() if new_row['expires_at'] else None,
                'created_at': new_row['created_at'].isoformat() if new_row['created_at'] else None,
                'access_count': new_row['access_count']
            }
            
            self.conn.commit()
            cursor.close()
            
            # Audit log rotation (with old and new token IDs)
            self._audit_log('rotate_share', owner_user_id, dashboard_name, 
                          success=True, 
                          error=f"old_token_id:{old_token_id},new_token_id:{new_share_info['token_id']}")
            
            logger.info(f"Rotated share token for dashboard '{dashboard_name}' "
                       f"(old_token_id: {old_token_id}, new_token_id: {new_share_info['token_id']}, owner: {owner_user_id})")
            
            return new_share_info
            
        except Exception as e:
            logger.error(f"Error rotating share token: {e}", exc_info=True)
            if self.conn:
                try:
                    self.conn.rollback()
                except:
                    pass
            
            # Audit log failure
            self._audit_log('rotate_share', owner_user_id, 'unknown', 
                          success=False, error=str(e))
            
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
        List all shares for a dashboard (including expired, but excluding revoked).
        
        Args:
            dashboard_name: Dashboard name
            owner_user_id: User ID of dashboard owner
            
        Returns:
            List of share info dicts with status field ('active', 'expired', 'revoked')
        """
        try:
            cursor = self.conn.cursor(cursor_factory=RealDictCursor)
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Include expired tokens (but exclude revoked for owner visibility)
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
                status = self._get_token_status(row['expires_at'], row['revoked_at'])
                share_info = {
                    'token_id': str(row['token_id']),
                    'token': row['token'],
                    'dashboard_name': row['dashboard_name'],
                    'owner_user_id': row['owner_user_id'],
                    'permissions': row['permissions'],
                    'expires_at': row['expires_at'].isoformat() if row['expires_at'] else None,
                    'created_at': row['created_at'].isoformat() if row['created_at'] else None,
                    'access_count': row['access_count'],
                    'last_accessed_at': row['last_accessed_at'].isoformat() if row['last_accessed_at'] else None,
                    'status': status
                }
                shares.append(share_info)
            
            return shares
            
        except Exception as e:
            logger.error(f"Error listing shares: {e}", exc_info=True)
            return []
    
    def get_all_share_activity(self) -> list:
        """
        Get all share tokens with activity data (read-only audit view).
        
        Returns:
            List of share info dicts with status field, sorted by last_accessed_at DESC
        """
        try:
            cursor = self.conn.cursor(cursor_factory=RealDictCursor)
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Get all tokens (including revoked) sorted by last_accessed_at DESC
            query = """
                SELECT token_id, token, dashboard_name, owner_user_id, permissions,
                       expires_at, created_at, revoked_at, access_count, last_accessed_at
                FROM dashboard_share_tokens
                ORDER BY 
                    CASE WHEN last_accessed_at IS NULL THEN created_at ELSE last_accessed_at END DESC,
                    created_at DESC
            """
            
            cursor.execute(query)
            rows = cursor.fetchall()
            cursor.close()
            
            shares = []
            for row in rows:
                status = self._get_token_status(row['expires_at'], row['revoked_at'])
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
                    'last_accessed_at': row['last_accessed_at'].isoformat() if row['last_accessed_at'] else None,
                    'status': status
                }
                shares.append(share_info)
            
            return shares
            
        except Exception as e:
            logger.error(f"Error getting share activity: {e}", exc_info=True)
            return []
    
    def cleanup_expired_tokens(self) -> Dict[str, int]:
        """
        Background-safe cleanup helper: mark expired tokens (no deletion).
        This is a read-only operation that can be run periodically.
        
        Returns:
            Dict with counts: {'checked': int, 'expired': int, 'already_marked': int}
        """
        try:
            cursor = self.conn.cursor()
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Count expired tokens (expires_at < now() and revoked_at IS NULL)
            count_query = """
                SELECT COUNT(*) 
                FROM dashboard_share_tokens
                WHERE expires_at IS NOT NULL
                  AND expires_at < now()
                  AND revoked_at IS NULL
            """
            cursor.execute(count_query)
            expired_count = cursor.fetchone()[0]
            
            # Note: We don't actually mark tokens as expired in the DB
            # because status is computed on-the-fly. This method is for
            # monitoring/auditing purposes only.
            
            cursor.close()
            
            return {
                'checked': expired_count,
                'expired': expired_count,
                'already_marked': 0  # Status computed dynamically
            }
            
        except Exception as e:
            logger.error(f"Error in cleanup_expired_tokens: {e}", exc_info=True)
            return {'checked': 0, 'expired': 0, 'already_marked': 0, 'error': str(e)}
    
    def _audit_log(self, action: str, user_id: str, dashboard_name: str, 
                   success: bool, error: Optional[str] = None):
        """
        Log share action to immutable audit log.
        
        Enhanced with rate-limited flag and access metadata presence indicators.
        
        Args:
            action: Action name (create_share, access_share, revoke_share)
            user_id: User ID
            dashboard_name: Dashboard name
            success: Whether action succeeded
            error: Optional error message or metadata (may include rate_limited and metadata flags)
        """
        try:
            if not self.db.table_exists("ransomeye", "immutable_audit_log"):
                return
            
            cursor = self.conn.cursor()
            cursor.execute("SET search_path = ransomeye, public;")
            
            action_name = f"share_{action}"
            error_msg = error if error else ("success" if success else "unknown_error")
            
            # Enhanced error message includes:
            # - rate_limited flag (if present in error string)
            # - metadata presence indicator (if present in error string)
            # These are parsed from error string format: "token_id:xxx,rate_limited:true,metadata:present"
            
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

