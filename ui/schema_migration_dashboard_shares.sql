# Path and File Name : /home/ransomeye/rebuild/ui/schema_migration_dashboard_shares.sql
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: Database schema migration for dashboard share tokens - read-only link-based sharing

-- ============================================================================
-- DASHBOARD SHARE TOKENS TABLE
-- ============================================================================
-- Purpose: Store cryptographically strong tokens for read-only dashboard sharing
-- Only personal dashboards can be shared (enforced at application level)
-- Tokens are revocable and optionally expire
-- Fail-closed validation on token access

BEGIN;

SET search_path = ransomeye, public;

-- Dashboard Share Tokens table
CREATE TABLE IF NOT EXISTS dashboard_share_tokens (
    token_id                uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    token                   text NOT NULL UNIQUE,
    dashboard_name          text NOT NULL,
    owner_user_id           text NOT NULL,
    permissions             text NOT NULL DEFAULT 'read_only',
    expires_at              timestamptz,
    created_at              timestamptz NOT NULL DEFAULT now(),
    revoked_at              timestamptz,
    access_count            integer NOT NULL DEFAULT 0,
    last_accessed_at        timestamptz,
    CONSTRAINT dashboard_share_tokens_permissions_check CHECK (permissions = 'read_only'),
    CONSTRAINT dashboard_share_tokens_token_length CHECK (char_length(token) >= 32)
);

COMMENT ON TABLE dashboard_share_tokens IS
'Purpose: Store share tokens for read-only dashboard access via link.\n'
'Writing module(s): UI Server (share API endpoints).\n'
'Reading module(s): UI Server (share token validation and access).\n'
'Security: Tokens are cryptographically strong (32+ chars), revocable, optionally expiring.\n'
'Access control: Only personal dashboards can be shared (enforced at application level).';

COMMENT ON COLUMN dashboard_share_tokens.token_id IS 'Primary key. UUID for share token record.';
COMMENT ON COLUMN dashboard_share_tokens.token IS 'Cryptographically strong share token (32+ characters, unique).';
COMMENT ON COLUMN dashboard_share_tokens.dashboard_name IS 'Dashboard name (slug) being shared.';
COMMENT ON COLUMN dashboard_share_tokens.owner_user_id IS 'User ID of dashboard owner (from RANSOMEYE_UI_USER_ID).';
COMMENT ON COLUMN dashboard_share_tokens.permissions IS 'Access permissions (currently only read_only supported).';
COMMENT ON COLUMN dashboard_share_tokens.expires_at IS 'Optional expiration timestamp (NULL = no expiration).';
COMMENT ON COLUMN dashboard_share_tokens.created_at IS 'Token creation timestamp.';
COMMENT ON COLUMN dashboard_share_tokens.revoked_at IS 'Token revocation timestamp (NULL = active).';
COMMENT ON COLUMN dashboard_share_tokens.access_count IS 'Number of times token has been accessed.';
COMMENT ON COLUMN dashboard_share_tokens.last_accessed_at IS 'Last access timestamp.';

-- Index for token lookup (most common query pattern)
CREATE INDEX IF NOT EXISTS idx_dashboard_share_tokens_token ON dashboard_share_tokens (token) WHERE revoked_at IS NULL;

-- Index for owner lookup (for listing/revoking shares)
CREATE INDEX IF NOT EXISTS idx_dashboard_share_tokens_owner ON dashboard_share_tokens (owner_user_id, dashboard_name);

-- Index for expiration cleanup
CREATE INDEX IF NOT EXISTS idx_dashboard_share_tokens_expires ON dashboard_share_tokens (expires_at) WHERE expires_at IS NOT NULL AND revoked_at IS NULL;

-- Grant permissions (following ransomeye schema pattern)
ALTER TABLE dashboard_share_tokens OWNER TO ransomeye_owner;
GRANT SELECT, INSERT, UPDATE ON dashboard_share_tokens TO ransomeye_rw;
GRANT SELECT ON dashboard_share_tokens TO ransomeye_ro;

COMMIT;

