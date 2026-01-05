-- Path and File Name : /home/ransomeye/rebuild/ransomeye_db_core/schema/migrations/add_threat_intel_delta.sql
-- Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
-- Details of functionality of this file: Database migration to add threat_intel_delta table for PROMPT-61

-- Migration: Add threat_intel_delta table (PROMPT-61 Phase 1)
-- Purpose: Append-only delta storage for threat intelligence evolution tracking

BEGIN;

SET search_path = ransomeye, public;

CREATE TABLE IF NOT EXISTS threat_intel_delta (
    delta_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    baseline_snapshot_hash bytea NOT NULL,
    delta_type text NOT NULL,
    ioc_type text,
    ioc_value text,
    source text,
    old_value jsonb,
    new_value jsonb,
    delta_hash bytea NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT threat_intel_delta_hash_len_chk CHECK (octet_length(delta_hash) = 32),
    CONSTRAINT threat_intel_delta_baseline_hash_len_chk CHECK (octet_length(baseline_snapshot_hash) = 32)
);

COMMENT ON TABLE threat_intel_delta IS
'Purpose: Append-only delta storage for threat intelligence evolution tracking.\n'
'Stores deltas between baseline snapshots and current state.\n'
'Writing module(s): Threat Delta Capture (core/intel/threat_delta_capture.py).\n'
'Reading module(s): Shadow Retraining, Analytics.\n'
'Retention expectation: long (append-only, no deletion).';

COMMENT ON COLUMN threat_intel_delta.delta_id IS 'Primary key. UUID for delta record.';
COMMENT ON COLUMN threat_intel_delta.baseline_snapshot_hash IS 'SHA-256 hash of baseline snapshot used for comparison.';
COMMENT ON COLUMN threat_intel_delta.delta_type IS 'Delta classification: new_ioc, ioc_mutation, confidence_shift, ttp_pattern.';
COMMENT ON COLUMN threat_intel_delta.ioc_type IS 'IOC type (ip, domain, hash, url, email, etc.).';
COMMENT ON COLUMN threat_intel_delta.ioc_value IS 'IOC value.';
COMMENT ON COLUMN threat_intel_delta.source IS 'Threat intelligence source.';
COMMENT ON COLUMN threat_intel_delta.old_value IS 'Previous IOC state (JSONB).';
COMMENT ON COLUMN threat_intel_delta.new_value IS 'New IOC state (JSONB).';
COMMENT ON COLUMN threat_intel_delta.delta_hash IS 'SHA-256 hash of delta record for integrity.';
COMMENT ON COLUMN threat_intel_delta.created_at IS 'Delta creation timestamp (append-only).';

CREATE INDEX IF NOT EXISTS idx_threat_intel_delta_baseline_hash 
ON threat_intel_delta (baseline_snapshot_hash);

CREATE INDEX IF NOT EXISTS idx_threat_intel_delta_created_at 
ON threat_intel_delta (created_at);

CREATE INDEX IF NOT EXISTS idx_threat_intel_delta_type 
ON threat_intel_delta (delta_type);

CREATE INDEX IF NOT EXISTS idx_threat_intel_delta_ioc 
ON threat_intel_delta (ioc_type, ioc_value, source);

COMMIT;

