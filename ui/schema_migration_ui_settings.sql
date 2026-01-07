-- Path and File Name : /home/ransomeye/rebuild/ui/schema_migration_ui_settings.sql
-- Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
-- Details: Database schema migration for UI Settings framework - per-user appearance preferences

-- ============================================================================
-- UI SETTINGS TABLE
-- ============================================================================
-- Purpose: Store per-user UI appearance preferences (theme, density, font_size)
-- Forward-compatible with RBAC and multi-user support
-- Fail-safe defaults enforced via constraints and application logic

BEGIN;

SET search_path = ransomeye, public;

-- UI Settings enum types
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'ui_theme_type') THEN
        CREATE TYPE ui_theme_type AS ENUM ('soc_dark', 'high_contrast', 'executive');
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'ui_density_type') THEN
        CREATE TYPE ui_density_type AS ENUM ('compact', 'comfortable');
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'ui_font_size_type') THEN
        CREATE TYPE ui_font_size_type AS ENUM ('small', 'medium', 'large');
    END IF;
END $$;

-- UI Settings table
CREATE TABLE IF NOT EXISTS ui_settings (
    settings_id            uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_identity          text NOT NULL,
    theme                  ui_theme_type NOT NULL DEFAULT 'soc_dark',
    density                ui_density_type NOT NULL DEFAULT 'comfortable',
    font_size              ui_font_size_type NOT NULL DEFAULT 'medium',
    created_at             timestamptz NOT NULL DEFAULT now(),
    updated_at             timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT ui_settings_user_unique UNIQUE (user_identity)
);

COMMENT ON TABLE ui_settings IS
'Purpose: Per-user UI appearance preferences (theme, density, font_size).\n'
'Writing module(s): UI Server (settings API endpoints).\n'
'Reading module(s): UI Server (settings retrieval).\n'
'Forward-compatible: user_identity column supports RBAC when authentication is added.\n'
'Fail-safe: Defaults enforced at database level and application level.';

COMMENT ON COLUMN ui_settings.settings_id IS 'Primary key. UUID for settings record.';
COMMENT ON COLUMN ui_settings.user_identity IS 'User identifier (currently system-default, future: user ID from RBAC).';
COMMENT ON COLUMN ui_settings.theme IS 'UI theme preference (soc_dark, high_contrast, executive).';
COMMENT ON COLUMN ui_settings.density IS 'UI density preference (compact, comfortable).';
COMMENT ON COLUMN ui_settings.font_size IS 'Font size preference (small, medium, large).';
COMMENT ON COLUMN ui_settings.created_at IS 'Settings record creation timestamp.';
COMMENT ON COLUMN ui_settings.updated_at IS 'Settings record last update timestamp.';

-- Index for user lookup (most common query pattern)
CREATE INDEX IF NOT EXISTS idx_ui_settings_user_identity ON ui_settings (user_identity);

-- Index for updated_at (for cleanup/maintenance queries)
CREATE INDEX IF NOT EXISTS idx_ui_settings_updated_at ON ui_settings (updated_at);

-- Grant permissions (following ransomeye schema pattern)
ALTER TABLE ui_settings OWNER TO ransomeye_owner;
GRANT SELECT, INSERT, UPDATE ON ui_settings TO ransomeye_rw;
GRANT SELECT ON ui_settings TO ransomeye_ro;

COMMIT;

