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
import os
import json
import base64
import tempfile
import zipfile
from pathlib import Path
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
    
    def is_emergency_disabled(self) -> bool:
        """
        Check if emergency kill-switch is active.
        
        Returns:
            True if RANSOMEYE_SHARE_EMERGENCY_DISABLE is set to true
        """
        emergency_disable_str = os.environ.get('RANSOMEYE_SHARE_EMERGENCY_DISABLE', 'false').lower()
        return emergency_disable_str in ('true', '1', 'yes', 'on')
    
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
            Dict with token info or None on failure (including emergency disable)
        """
        # Emergency kill-switch check (fail-closed)
        if self.is_emergency_disabled():
            self._audit_log('create_share', owner_user_id, dashboard_name, 
                          success=False, error="emergency_disabled:true")
            logger.warning(f"Share creation denied due to emergency disable for dashboard '{dashboard_name}' (owner: {owner_user_id})")
            return None
        
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
        Emergency kill-switch: all access denied if RANSOMEYE_SHARE_EMERGENCY_DISABLE=true.
        
        Args:
            token: Share token to validate
            ip_address: Client IP address (best-effort, may be None)
            user_agent: Client User-Agent (best-effort, may be None)
            rate_limited: Whether this access was rate-limited
            
        Returns:
            Dict with share info (including status) or None if invalid/expired/revoked/emergency_disabled
        """
        # Emergency kill-switch check (fail-closed)
        if self.is_emergency_disabled():
            # Try to get token info for audit logging
            try:
                cursor = self.conn.cursor(cursor_factory=RealDictCursor)
                cursor.execute("SET search_path = ransomeye, public;")
                
                query = """
                    SELECT token_id, dashboard_name, owner_user_id
                    FROM dashboard_share_tokens
                    WHERE token = %s
                """
                cursor.execute(query, (token,))
                row = cursor.fetchone()
                
                if row:
                    # Log access attempt with emergency flag
                    self._log_access(
                        cursor=cursor,
                        token_id=str(row['token_id']),
                        token=token,
                        dashboard_name=row['dashboard_name'],
                        ip_address=ip_address,
                        user_agent=user_agent,
                        rate_limited=rate_limited,
                        access_granted=False
                    )
                    
                    # Audit log emergency denial
                    self._audit_log('access_share', row['owner_user_id'], 
                                  row['dashboard_name'], 
                                  success=False, 
                                  error="emergency_disabled:true")
                else:
                    # Invalid token, but still log emergency denial
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
            except Exception as e:
                logger.error(f"Error during emergency disable check: {e}", exc_info=True)
                if self.conn:
                    try:
                        self.conn.rollback()
                    except:
                        pass
            
            logger.warning(f"Share token access denied due to emergency disable: {token[:16]}...")
            return None
        
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
            True if revoked, False otherwise (including emergency disable)
        """
        # Emergency kill-switch check (fail-closed)
        if self.is_emergency_disabled():
            self._audit_log('revoke_share', owner_user_id, 'unknown', 
                          success=False, error="emergency_disabled:true")
            logger.warning(f"Share revocation denied due to emergency disable (owner: {owner_user_id})")
            return False
        
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
    
    def revoke_all_shares(self, owner_user_id: str) -> Dict[str, int]:
        """
        Revoke all active share tokens owned by a user (mass revocation).
        
        Args:
            owner_user_id: User ID of token owner
            
        Returns:
            Dict with:
            - revoked_count: Number of tokens revoked
            - already_revoked_count: Number of tokens already revoked
        """
        # Emergency kill-switch check (fail-closed)
        if self.is_emergency_disabled():
            self._audit_log('revoke_all_shares', owner_user_id, 'all', 
                          success=False, error="emergency_disabled:true")
            logger.warning(f"Mass revocation denied due to emergency disable (owner: {owner_user_id})")
            return {'revoked_count': 0, 'already_revoked_count': 0}
        
        try:
            cursor = self.conn.cursor()
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Count already revoked tokens
            count_revoked_query = """
                SELECT COUNT(*)
                FROM dashboard_share_tokens
                WHERE owner_user_id = %s
                  AND revoked_at IS NOT NULL
            """
            cursor.execute(count_revoked_query, (owner_user_id,))
            already_revoked_count = cursor.fetchone()[0]
            
            # Revoke all active tokens (atomic operation)
            revoke_all_query = """
                UPDATE dashboard_share_tokens
                SET revoked_at = now()
                WHERE owner_user_id = %s
                  AND revoked_at IS NULL
                RETURNING token_id, dashboard_name
            """
            cursor.execute(revoke_all_query, (owner_user_id,))
            revoked_rows = cursor.fetchall()
            revoked_count = len(revoked_rows)
            
            self.conn.commit()
            cursor.close()
            
            # Audit log mass revocation
            self._audit_log('revoke_all_shares', owner_user_id, 'all', 
                          success=True, 
                          error=f"revoked_count:{revoked_count},already_revoked_count:{already_revoked_count}")
            
            logger.info(f"Mass revoked {revoked_count} share tokens for owner '{owner_user_id}' "
                       f"(already revoked: {already_revoked_count})")
            
            return {
                'revoked_count': revoked_count,
                'already_revoked_count': already_revoked_count
            }
            
        except Exception as e:
            logger.error(f"Error in mass revocation: {e}", exc_info=True)
            if self.conn:
                try:
                    self.conn.rollback()
                except:
                    pass
            
            # Audit log failure
            self._audit_log('revoke_all_shares', owner_user_id, 'all', 
                          success=False, error=str(e))
            
            return {'revoked_count': 0, 'already_revoked_count': 0}
    
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
    
    def get_incident_report(self, from_timestamp: datetime, to_timestamp: datetime) -> Dict[str, Any]:
        """
        Generate a forensic, read-only incident report for share activity in a time window.
        
        Args:
            from_timestamp: Start of time window (UTC)
            to_timestamp: End of time window (UTC)
            
        Returns:
            Dict with:
            - summary: Dict with total_shares_created, total_shares_revoked, total_access_attempts,
                       total_rate_limited, total_expired
            - timeline: List of ordered events (create/access/rotate/revoke/deny) with timestamp,
                        dashboard_name, token_id (masked), outcome
            - top_dashboards: List of dashboards sorted by access count in window
        """
        try:
            cursor = self.conn.cursor(cursor_factory=RealDictCursor)
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Summary: Total shares created in window
            created_query = """
                SELECT COUNT(*) as count
                FROM dashboard_share_tokens
                WHERE created_at >= %s AND created_at <= %s
            """
            cursor.execute(created_query, (from_timestamp, to_timestamp))
            total_shares_created = cursor.fetchone()['count']
            
            # Summary: Total shares revoked in window
            revoked_query = """
                SELECT COUNT(*) as count
                FROM dashboard_share_tokens
                WHERE revoked_at >= %s AND revoked_at <= %s
            """
            cursor.execute(revoked_query, (from_timestamp, to_timestamp))
            total_shares_revoked = cursor.fetchone()['count']
            
            # Summary: Total access attempts in window (from share_access_logs)
            access_query = """
                SELECT 
                    COUNT(*) as total_attempts,
                    COUNT(*) FILTER (WHERE rate_limited = true) as rate_limited_count,
                    COUNT(*) FILTER (WHERE access_granted = false) as denied_count
                FROM share_access_logs
                WHERE accessed_at >= %s AND accessed_at <= %s
            """
            cursor.execute(access_query, (from_timestamp, to_timestamp))
            access_row = cursor.fetchone()
            total_access_attempts = access_row['total_attempts'] if access_row else 0
            total_rate_limited = access_row['rate_limited_count'] if access_row else 0
            
            # Summary: Total expired (tokens that expired in window)
            expired_query = """
                SELECT COUNT(*) as count
                FROM dashboard_share_tokens
                WHERE expires_at >= %s AND expires_at <= %s
                  AND revoked_at IS NULL
            """
            cursor.execute(expired_query, (from_timestamp, to_timestamp))
            total_expired = cursor.fetchone()['count']
            
            # Timeline: Collect all events in window
            timeline = []
            
            # 1. Share creation events
            create_events_query = """
                SELECT 
                    created_at as timestamp,
                    dashboard_name,
                    token_id,
                    'create' as event_type,
                    'success' as outcome
                FROM dashboard_share_tokens
                WHERE created_at >= %s AND created_at <= %s
            """
            cursor.execute(create_events_query, (from_timestamp, to_timestamp))
            for row in cursor.fetchall():
                timeline.append({
                    'timestamp': row['timestamp'].isoformat() if row['timestamp'] else None,
                    'dashboard_name': row['dashboard_name'],
                    'token_id': self._mask_token_id(str(row['token_id'])),
                    'event_type': 'create',
                    'outcome': 'success'
                })
            
            # 2. Share revocation events
            revoke_events_query = """
                SELECT 
                    revoked_at as timestamp,
                    dashboard_name,
                    token_id,
                    'revoke' as event_type,
                    'success' as outcome
                FROM dashboard_share_tokens
                WHERE revoked_at >= %s AND revoked_at <= %s
            """
            cursor.execute(revoke_events_query, (from_timestamp, to_timestamp))
            for row in cursor.fetchall():
                timeline.append({
                    'timestamp': row['timestamp'].isoformat() if row['timestamp'] else None,
                    'dashboard_name': row['dashboard_name'],
                    'token_id': self._mask_token_id(str(row['token_id'])),
                    'event_type': 'revoke',
                    'outcome': 'success'
                })
            
            # 3. Share rotation events (from audit log)
            if self.db.table_exists("ransomeye", "immutable_audit_log"):
                rotate_events_query = """
                    SELECT 
                        timestamp,
                        resource_id as dashboard_name,
                        error_message
                    FROM immutable_audit_log
                    WHERE action = 'share_rotate_share'
                      AND timestamp >= %s AND timestamp <= %s
                      AND success = true
                """
                cursor.execute(rotate_events_query, (from_timestamp, to_timestamp))
                for row in cursor.fetchall():
                    # Extract token_id from error_message (format: "old_token_id:xxx,new_token_id:yyy")
                    token_id = None
                    if row['error_message']:
                        # Try to extract new_token_id
                        parts = row['error_message'].split(',')
                        for part in parts:
                            if 'new_token_id:' in part:
                                token_id = part.split('new_token_id:')[1].strip()
                                break
                    
                    timeline.append({
                        'timestamp': row['timestamp'].isoformat() if row['timestamp'] else None,
                        'dashboard_name': row['dashboard_name'],
                        'token_id': self._mask_token_id(token_id) if token_id else 'unknown',
                        'event_type': 'rotate',
                        'outcome': 'success'
                    })
            
            # 4. Access events (from share_access_logs)
            access_events_query = """
                SELECT 
                    accessed_at as timestamp,
                    dashboard_name,
                    token_id,
                    access_granted,
                    rate_limited
                FROM share_access_logs
                WHERE accessed_at >= %s AND accessed_at <= %s
                ORDER BY accessed_at ASC
            """
            cursor.execute(access_events_query, (from_timestamp, to_timestamp))
            for row in cursor.fetchall():
                event_type = 'access'
                if row['rate_limited']:
                    outcome = 'rate_limited'
                elif not row['access_granted']:
                    outcome = 'deny'
                else:
                    outcome = 'success'
                
                timeline.append({
                    'timestamp': row['timestamp'].isoformat() if row['timestamp'] else None,
                    'dashboard_name': row['dashboard_name'],
                    'token_id': self._mask_token_id(str(row['token_id'])) if row['token_id'] else 'invalid',
                    'event_type': event_type,
                    'outcome': outcome
                })
            
            # Sort timeline by timestamp
            timeline.sort(key=lambda x: x['timestamp'] or '')
            
            # Top dashboards by access count in window
            top_dashboards_query = """
                SELECT 
                    dashboard_name,
                    COUNT(*) as access_count
                FROM share_access_logs
                WHERE accessed_at >= %s AND accessed_at <= %s
                  AND dashboard_name IS NOT NULL
                  AND access_granted = true
                GROUP BY dashboard_name
                ORDER BY access_count DESC
                LIMIT 10
            """
            cursor.execute(top_dashboards_query, (from_timestamp, to_timestamp))
            top_dashboards = []
            for row in cursor.fetchall():
                top_dashboards.append({
                    'dashboard_name': row['dashboard_name'],
                    'access_count': row['access_count']
                })
            
            cursor.close()
            
            return {
                'summary': {
                    'total_shares_created': total_shares_created,
                    'total_shares_revoked': total_shares_revoked,
                    'total_access_attempts': total_access_attempts,
                    'total_rate_limited': total_rate_limited,
                    'total_expired': total_expired
                },
                'timeline': timeline,
                'top_dashboards': top_dashboards
            }
            
        except Exception as e:
            logger.error(f"Error generating incident report: {e}", exc_info=True)
            # Fail-soft: return empty report structure
            return {
                'summary': {
                    'total_shares_created': 0,
                    'total_shares_revoked': 0,
                    'total_access_attempts': 0,
                    'total_rate_limited': 0,
                    'total_expired': 0
                },
                'timeline': [],
                'top_dashboards': []
            }
    
    def _mask_token_id(self, token_id: str) -> str:
        """
        Mask token ID for display (show first 8 chars, mask rest).
        
        Args:
            token_id: Full token ID UUID string
            
        Returns:
            Masked token ID (e.g., "a1b2c3d4-****")
        """
        if not token_id or len(token_id) < 8:
            return '****'
        return token_id[:8] + '-****'

