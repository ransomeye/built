# Path and File Name : /home/ransomeye/rebuild/ui/schema_migration_share_access_logs.sql
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: Database schema migration for share access logs - append-only access metadata tracking

-- ============================================================================
-- SHARE ACCESS LOGS TABLE
-- ============================================================================
-- Purpose: Append-only log of all share token access attempts with metadata
-- Stores IP address, User-Agent, rate-limited flag, and access timestamp
-- Immutable (INSERT only, no UPDATE/DELETE)
-- Used for audit visibility and abuse detection

BEGIN;

SET search_path = ransomeye, public;

-- Share Access Logs table (append-only)
CREATE TABLE IF NOT EXISTS share_access_logs (
    log_id                  uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    token_id                uuid,
    token                   text NOT NULL,
    dashboard_name          text,
    accessed_at             timestamptz NOT NULL DEFAULT now(),
    ip_address              text,
    user_agent              text,
    rate_limited            boolean NOT NULL DEFAULT false,
    access_granted          boolean NOT NULL DEFAULT true,
    CONSTRAINT share_access_logs_token_id_fk FOREIGN KEY (token_id) 
        REFERENCES dashboard_share_tokens(token_id) ON DELETE CASCADE
);

COMMENT ON TABLE share_access_logs IS
'Purpose: Append-only log of share token access attempts with metadata.\n'
'Writing module(s): UI Server (share access endpoint).\n'
'Reading module(s): UI Server (audit queries), analytics.\n'
'Security: Immutable append-only log for audit trail and abuse detection.\n'
'Metadata: IP address and User-Agent captured best-effort (may be NULL behind proxies).';

COMMENT ON COLUMN share_access_logs.log_id IS 'Primary key. UUID for access log record.';
COMMENT ON COLUMN share_access_logs.token_id IS 'Foreign key to dashboard_share_tokens.token_id (NULL for invalid/expired tokens).';
COMMENT ON COLUMN share_access_logs.token IS 'Share token (denormalized for query performance, always present).';
COMMENT ON COLUMN share_access_logs.dashboard_name IS 'Dashboard name (denormalized for query performance, NULL for invalid tokens).';
COMMENT ON COLUMN share_access_logs.accessed_at IS 'Access timestamp (UTC).';
COMMENT ON COLUMN share_access_logs.ip_address IS 'Client IP address (best-effort, may be NULL behind proxies).';
COMMENT ON COLUMN share_access_logs.user_agent IS 'Client User-Agent string (best-effort, may be NULL).';
COMMENT ON COLUMN share_access_logs.rate_limited IS 'Whether this access was rate-limited (true = blocked, false = allowed).';
COMMENT ON COLUMN share_access_logs.access_granted IS 'Whether access was granted (false = invalid/expired/revoked token).';

-- Index for token-based queries (most common pattern)
CREATE INDEX IF NOT EXISTS idx_share_access_logs_token ON share_access_logs (token_id, accessed_at DESC);

-- Index for dashboard-based queries
CREATE INDEX IF NOT EXISTS idx_share_access_logs_dashboard ON share_access_logs (dashboard_name, accessed_at DESC);

-- Index for rate-limited access queries (abuse detection)
CREATE INDEX IF NOT EXISTS idx_share_access_logs_rate_limited ON share_access_logs (rate_limited, accessed_at DESC) WHERE rate_limited = true;

-- Index for IP-based queries (abuse detection)
CREATE INDEX IF NOT EXISTS idx_share_access_logs_ip ON share_access_logs (ip_address, accessed_at DESC) WHERE ip_address IS NOT NULL;

-- Grant permissions (following ransomeye schema pattern)
ALTER TABLE share_access_logs OWNER TO ransomeye_owner;
GRANT SELECT, INSERT ON share_access_logs TO ransomeye_rw;
GRANT SELECT ON share_access_logs TO ransomeye_ro;

-- Prevent UPDATE/DELETE (immutable append-only)
REVOKE UPDATE, DELETE ON share_access_logs FROM ransomeye_rw;
REVOKE UPDATE, DELETE ON share_access_logs FROM ransomeye_ro;

COMMIT;

