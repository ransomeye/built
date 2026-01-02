-- Path and File Name : /home/ransomeye/rebuild/ransomeye_db_core/schema/schema.sql
-- Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
-- Details of functionality of this file: Authoritative PostgreSQL schema (single source of truth) for all RansomEye modules.

BEGIN;

-- ============================================================================
-- Core principles (enforced by schema where feasible):
-- - Deterministic identity and correlation via canonical entities and hashes
-- - Indexed timestamps and indexed foreign keys across all write-heavy tables
-- - Append-only enforcement for audit-grade tables (immutable log + trust events)
-- - Minimal JSONB usage: only for variable event payloads, structured metadata,
--   and model/LLM artifacts where schema rigidity is counterproductive.
-- - Ownership and access separation via roles (created if absent) and GRANTs.
-- ============================================================================

-- Extensions: pgcrypto provides gen_random_uuid() and digest().
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Roles: created without passwords; credentialing is an operational concern.
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'ransomeye_owner') THEN
    CREATE ROLE ransomeye_owner NOLOGIN;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'ransomeye_rw') THEN
    CREATE ROLE ransomeye_rw NOLOGIN;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'ransomeye_ro') THEN
    CREATE ROLE ransomeye_ro NOLOGIN;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'ransomeye_audit') THEN
    CREATE ROLE ransomeye_audit NOLOGIN;
  END IF;
END $$;

-- Namespace: a dedicated schema for all RansomEye tables.
CREATE SCHEMA IF NOT EXISTS ransomeye AUTHORIZATION ransomeye_owner;
SET search_path = ransomeye, public;

-- ============================================================================
-- Types
-- ============================================================================
CREATE TYPE event_source_type AS ENUM (
  'linux_agent',
  'windows_agent',
  'dpi_probe',
  'core_engine',
  'ai_core',
  'alert_engine',
  'policy_engine',
  'correlation_engine',
  'llm',
  'response_engine',
  'forensic_engine',
  'unknown'
);

CREATE TYPE severity_level AS ENUM ('debug', 'info', 'notice', 'warning', 'error', 'critical');

CREATE TYPE signature_status AS ENUM ('valid', 'invalid', 'unknown', 'expired', 'revoked', 'untrusted_chain');

CREATE TYPE enforcement_decision_type AS ENUM ('allow', 'block', 'quarantine', 'isolate', 'rate_limit', 'log_only', 'escalate', 'unknown');

CREATE TYPE action_status_type AS ENUM ('requested', 'started', 'succeeded', 'failed', 'rolled_back', 'skipped');

CREATE TYPE model_artifact_type AS ENUM ('pkl', 'gguf', 'onnx', 'other');

CREATE TYPE model_task_type AS ENUM ('classification', 'regression', 'ranking', 'anomaly_detection', 'nlp', 'embedding', 'other');

CREATE TYPE llm_role_type AS ENUM ('system', 'user', 'assistant', 'tool');

CREATE TYPE trust_object_type AS ENUM (
  'raw_event',
  'normalized_event',
  'policy',
  'policy_version',
  'model',
  'model_version',
  'inference',
  'llm_request',
  'llm_response',
  'enforcement_decision',
  'action',
  'artifact',
  'other'
);

CREATE TYPE component_type AS ENUM (
  'core_engine',
  'linux_agent',
  'windows_agent',
  'dpi_probe',
  'ai_core',
  'llm',
  'alert_engine',
  'policy_engine',
  'correlation_engine',
  'forensic_engine',
  'response_engine',
  'ui',
  'db_core',
  'master_core',
  'other'
);

-- ============================================================================
-- Utilities: append-only enforcement for audit-grade tables
-- ============================================================================
CREATE OR REPLACE FUNCTION prevent_update_delete()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  RAISE EXCEPTION 'Append-only table: UPDATE/DELETE is not permitted on %.%', TG_TABLE_SCHEMA, TG_TABLE_NAME;
END;
$$;

-- ============================================================================
-- Supporting contract tables (required to make relationships first-class)
-- ============================================================================

-- agents: canonical identity for Linux/Windows agents and DPI probes
CREATE TABLE IF NOT EXISTS agents (
  agent_id               uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  agent_type             event_source_type NOT NULL,
  agent_name             text NULL,
  org_id                 text NULL,
  site_id                text NULL,
  tenant_id              text NULL,
  host_fqdn              text NULL,
  host_hostname          text NULL,
  host_os                text NULL,
  host_arch              text NULL,
  host_kernel            text NULL,
  host_boot_id           uuid NULL,
  hardware_fingerprint   bytea NULL,
  first_seen_at          timestamptz NOT NULL DEFAULT now(),
  last_seen_at           timestamptz NOT NULL DEFAULT now(),
  decommissioned_at      timestamptz NULL,
  is_active              boolean NOT NULL DEFAULT true,
  tags                   jsonb NULL,
  created_at             timestamptz NOT NULL DEFAULT now(),
  updated_at             timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT agents_type_chk CHECK (agent_type IN ('linux_agent','windows_agent','dpi_probe','unknown'))
);

COMMENT ON TABLE agents IS
'Purpose: Canonical identity record for endpoints and probes producing telemetry.\n'
'Writing module(s): Linux Agent, Windows Agent, DPI Probe, Core Engine (enrollment/updates).\n'
'Reading module(s): All modules (telemetry join, correlation, enforcement, UI).\n'
'Retention expectation: long.';

COMMENT ON COLUMN agents.agent_id IS 'Primary key. Stable UUID identity for the agent/probe.';
COMMENT ON COLUMN agents.agent_type IS 'Agent/probe kind. Limited to linux_agent/windows_agent/dpi_probe/unknown.';
COMMENT ON COLUMN agents.agent_name IS 'Human-friendly agent display name (optional).';
COMMENT ON COLUMN agents.org_id IS 'Organization identifier used for multi-org segregation (optional).';
COMMENT ON COLUMN agents.site_id IS 'Site/region identifier (optional).';
COMMENT ON COLUMN agents.tenant_id IS 'Tenant identifier for multi-tenant deployments (optional).';
COMMENT ON COLUMN agents.host_fqdn IS 'FQDN of the host as observed at enrollment/telemetry time (optional).';
COMMENT ON COLUMN agents.host_hostname IS 'Hostname of the host (optional).';
COMMENT ON COLUMN agents.host_os IS 'Operating system name/version string (optional).';
COMMENT ON COLUMN agents.host_arch IS 'CPU architecture string (optional).';
COMMENT ON COLUMN agents.host_kernel IS 'Kernel version string (optional).';
COMMENT ON COLUMN agents.host_boot_id IS 'Host boot session identifier (optional).';
COMMENT ON COLUMN agents.hardware_fingerprint IS 'Binary fingerprint for hardware identity correlation (optional).';
COMMENT ON COLUMN agents.first_seen_at IS 'First observed timestamp for this agent identity.';
COMMENT ON COLUMN agents.last_seen_at IS 'Most recent heartbeat/telemetry observed timestamp.';
COMMENT ON COLUMN agents.decommissioned_at IS 'If set, the time the agent was retired/decommissioned.';
COMMENT ON COLUMN agents.is_active IS 'Operational active flag for scheduling/enforcement decisions.';
COMMENT ON COLUMN agents.tags IS 'Optional structured tags for grouping/filtering agents (JSONB justified for flexible metadata).';
COMMENT ON COLUMN agents.created_at IS 'Row creation timestamp.';
COMMENT ON COLUMN agents.updated_at IS 'Row last update timestamp (mutable table).';

CREATE INDEX IF NOT EXISTS idx_agents_last_seen_at ON agents (last_seen_at);
CREATE INDEX IF NOT EXISTS idx_agents_type ON agents (agent_type);

-- components: canonical identity for services/modules emitting health and audit events
CREATE TABLE IF NOT EXISTS components (
  component_id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  component_type         component_type NOT NULL,
  component_name         text NOT NULL,
  instance_id            text NULL,
  build_hash             text NULL,
  version                text NULL,
  host_agent_id          uuid NULL REFERENCES agents(agent_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  started_at             timestamptz NULL,
  last_heartbeat_at      timestamptz NULL,
  created_at             timestamptz NOT NULL DEFAULT now(),
  updated_at             timestamptz NOT NULL DEFAULT now()
);

-- Unique constraint replacement: PostgreSQL 17 doesn't allow expressions in UNIQUE constraints
CREATE UNIQUE INDEX components_name_uniq_idx
ON components (component_type, component_name, instance_id)
WHERE instance_id IS NOT NULL;

CREATE UNIQUE INDEX components_name_uniq_null_idx
ON components (component_type, component_name)
WHERE instance_id IS NULL;

COMMENT ON TABLE components IS
'Purpose: Canonical identity record for RansomEye services (Core, AI, LLM, etc.) to support health, ops telemetry, and audit attribution.\n'
'Writing module(s): Each component at startup/heartbeat; Master/Core orchestrator.\n'
'Reading module(s): UI, Global Validator, Ops monitoring.\n'
'Retention expectation: long.';

COMMENT ON COLUMN components.component_id IS 'Primary key. Stable UUID for component instance identity.';
COMMENT ON COLUMN components.component_type IS 'Categorical module type (core_engine, ai_core, etc.).';
COMMENT ON COLUMN components.component_name IS 'Logical component name (e.g., ransomeye_llm).';
COMMENT ON COLUMN components.instance_id IS 'Optional instance identifier (e.g., systemd unit instance, container id).';
COMMENT ON COLUMN components.build_hash IS 'Build hash identifier (e.g., git commit) for provenance.';
COMMENT ON COLUMN components.version IS 'Semantic/packaging version string.';
COMMENT ON COLUMN components.host_agent_id IS 'If the component runs on a host with an agent record, links to that agent.';
COMMENT ON COLUMN components.started_at IS 'Component start time (first observed for current runtime session).';
COMMENT ON COLUMN components.last_heartbeat_at IS 'Most recent heartbeat time observed from this component instance.';
COMMENT ON COLUMN components.created_at IS 'Row creation timestamp.';
COMMENT ON COLUMN components.updated_at IS 'Row last update timestamp (mutable table).';

CREATE INDEX IF NOT EXISTS idx_components_last_heartbeat ON components (last_heartbeat_at);
CREATE INDEX IF NOT EXISTS idx_components_host_agent ON components (host_agent_id);

-- entities: deterministic canonical objects for correlation (IPs, domains, processes, files, users, etc.)
CREATE TABLE IF NOT EXISTS entities (
  entity_id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  entity_type            text NOT NULL,
  entity_key             text NOT NULL,
  entity_key_sha256      bytea NOT NULL,
  first_seen_at          timestamptz NOT NULL DEFAULT now(),
  last_seen_at           timestamptz NOT NULL DEFAULT now(),
  attributes             jsonb NULL,
  created_at             timestamptz NOT NULL DEFAULT now(),
  updated_at             timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT entities_key_hash_uniq UNIQUE (entity_type, entity_key_sha256)
);

COMMENT ON TABLE entities IS
'Purpose: Canonical entity registry enabling deterministic correlation across raw/normalized events and detection graphs.\n'
'Writing module(s): Correlation Engine, Normalizer, AI/Detection pipeline.\n'
'Reading module(s): Correlation Engine, Policy Engine, UI, Forensic replay.\n'
'Retention expectation: long.';

COMMENT ON COLUMN entities.entity_id IS 'Primary key. UUID for canonical entity.';
COMMENT ON COLUMN entities.entity_type IS 'Entity kind (e.g., ip, domain, file_hash, process, user, url, registry_key).';
COMMENT ON COLUMN entities.entity_key IS 'Canonical string key for the entity (normalized representation).';
COMMENT ON COLUMN entities.entity_key_sha256 IS 'SHA-256 digest of entity_key for deterministic lookup/uniqueness.';
COMMENT ON COLUMN entities.first_seen_at IS 'First time this entity was observed in telemetry.';
COMMENT ON COLUMN entities.last_seen_at IS 'Most recent time this entity was observed in telemetry.';
COMMENT ON COLUMN entities.attributes IS 'Optional structured attributes (JSONB justified for extensible entity metadata).';
COMMENT ON COLUMN entities.created_at IS 'Row creation timestamp.';
COMMENT ON COLUMN entities.updated_at IS 'Row last update timestamp (mutable table).';

CREATE INDEX IF NOT EXISTS idx_entities_last_seen_at ON entities (last_seen_at);

-- policies: canonical policy registry (policy content stored in versions table)
CREATE TABLE IF NOT EXISTS policies (
  policy_id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  policy_name            text NOT NULL,
  policy_description     text NULL,
  policy_namespace       text NULL,
  owner_component_id     uuid NULL REFERENCES components(component_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  is_active              boolean NOT NULL DEFAULT true,
  created_at             timestamptz NOT NULL DEFAULT now(),
  updated_at             timestamptz NOT NULL DEFAULT now()
);

-- Unique constraint replacement: PostgreSQL 17 doesn't allow expressions in UNIQUE constraints
CREATE UNIQUE INDEX policies_name_uniq_idx
ON policies (policy_namespace, policy_name)
WHERE policy_namespace IS NOT NULL;

CREATE UNIQUE INDEX policies_name_uniq_null_idx
ON policies (policy_name)
WHERE policy_namespace IS NULL;

COMMENT ON TABLE policies IS
'Purpose: Canonical policy identity record (stable ID), decoupled from versioned content.\n'
'Writing module(s): Policy Engine, Alert Engine (policy registry).\n'
'Reading module(s): Policy Engine, Enforcement, UI, Validator.\n'
'Retention expectation: long.';

COMMENT ON COLUMN policies.policy_id IS 'Primary key. Stable UUID identity for a policy.';
COMMENT ON COLUMN policies.policy_name IS 'Human-friendly policy name.';
COMMENT ON COLUMN policies.policy_description IS 'Optional description of policy intent.';
COMMENT ON COLUMN policies.policy_namespace IS 'Optional namespace/grouping (e.g., org/site/package).';
COMMENT ON COLUMN policies.owner_component_id IS 'Component that authored/owns this policy record (optional).';
COMMENT ON COLUMN policies.is_active IS 'Whether the policy is enabled for evaluation.';
COMMENT ON COLUMN policies.created_at IS 'Row creation timestamp.';
COMMENT ON COLUMN policies.updated_at IS 'Row last update timestamp (mutable table).';

CREATE INDEX IF NOT EXISTS idx_policies_active ON policies (is_active);
CREATE INDEX IF NOT EXISTS idx_policies_owner_component ON policies (owner_component_id);

-- policy_versions: immutable versioned policy content + signatures for trust
CREATE TABLE IF NOT EXISTS policy_versions (
  policy_version_id      uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  policy_id              uuid NOT NULL REFERENCES policies(policy_id) ON UPDATE RESTRICT ON DELETE RESTRICT,
  version                bigint NOT NULL,
  policy_format          text NOT NULL,
  policy_body            text NOT NULL,
  policy_sha256          bytea NOT NULL,
  signed_by              text NULL,
  signature_alg          text NULL,
  signature_b64          text NULL,
  signature_status       signature_status NOT NULL DEFAULT 'unknown',
  valid_from             timestamptz NULL,
  valid_to               timestamptz NULL,
  created_at             timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT policy_versions_uniq UNIQUE (policy_id, version),
  CONSTRAINT policy_versions_hash_uniq UNIQUE (policy_id, policy_sha256)
);

COMMENT ON TABLE policy_versions IS
'Purpose: Immutable, versioned policy content with cryptographic signature metadata.\n'
'Writing module(s): Policy Engine.\n'
'Reading module(s): Policy evaluation, Enforcement, Validator, UI.\n'
'Retention expectation: long.';

COMMENT ON COLUMN policy_versions.policy_version_id IS 'Primary key. UUID for a specific policy version.';
COMMENT ON COLUMN policy_versions.policy_id IS 'Foreign key to policies.policy_id.';
COMMENT ON COLUMN policy_versions.version IS 'Monotonic integer version number for this policy.';
COMMENT ON COLUMN policy_versions.policy_format IS 'Policy serialization format (e.g., yaml, json, rego, custom).';
COMMENT ON COLUMN policy_versions.policy_body IS 'Full policy body content (stored as text for deterministic signing).';
COMMENT ON COLUMN policy_versions.policy_sha256 IS 'SHA-256 digest of policy_body for integrity and deduplication.';
COMMENT ON COLUMN policy_versions.signed_by IS 'Signer identity (key id, cert subject, or principal) if provided.';
COMMENT ON COLUMN policy_versions.signature_alg IS 'Signature algorithm identifier (e.g., ed25519, rsa-pss).';
COMMENT ON COLUMN policy_versions.signature_b64 IS 'Base64-encoded signature over policy_body/policy_sha256.';
COMMENT ON COLUMN policy_versions.signature_status IS 'Validation status of the signature.';
COMMENT ON COLUMN policy_versions.valid_from IS 'Optional validity start time for this policy version.';
COMMENT ON COLUMN policy_versions.valid_to IS 'Optional validity end time for this policy version.';
COMMENT ON COLUMN policy_versions.created_at IS 'Row creation timestamp (append-only by convention).';

CREATE INDEX IF NOT EXISTS idx_policy_versions_policy_id ON policy_versions (policy_id);
CREATE INDEX IF NOT EXISTS idx_policy_versions_created_at ON policy_versions (created_at);

-- ============================================================================
-- A. Agent Telemetry (REQUIRED)
-- ============================================================================

CREATE TABLE IF NOT EXISTS linux_agent_telemetry (
  telemetry_id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  agent_id               uuid NOT NULL REFERENCES agents(agent_id) ON UPDATE RESTRICT ON DELETE RESTRICT,
  -- Signed envelope fields (agent protocol contract)
  source_message_id      uuid NULL,
  source_nonce           text NULL,
  source_component_identity text NULL,
  source_host_id         text NULL,
  source_signature_b64   text NULL,
  source_signature_alg   text NULL,
  source_data_hash_hex   text NULL,
  observed_at            timestamptz NOT NULL,
  received_at            timestamptz NOT NULL DEFAULT now(),
  boot_id                uuid NULL,
  pid                    integer NULL,
  ppid                   integer NULL,
  uid                    integer NULL,
  gid                    integer NULL,
  username               text NULL,
  process_name           text NULL,
  process_path           text NULL,
  cmdline                text NULL,
  event_name             text NOT NULL,
  event_category         text NULL,
  severity               severity_level NOT NULL DEFAULT 'info',
  -- Protocol-aligned typed fields (avoid burying required parameters in JSONB)
  file_operation         text NULL,
  file_process_id        integer NULL,
  auth_user              text NULL,
  auth_source            text NULL,
  auth_success           boolean NULL,
  network_process_id     integer NULL,
  file_path              text NULL,
  file_sha256            bytea NULL,
  network_src_ip         inet NULL,
  network_src_port       integer NULL,
  network_dst_ip         inet NULL,
  network_dst_port       integer NULL,
  protocol               text NULL,
  payload                jsonb NULL,
  payload_sha256         bytea NULL,
  correlation_hint       text NULL,
  created_at             timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT linux_agent_nonce_hex_chk CHECK (source_nonce IS NULL OR source_nonce ~ '^[0-9a-f]{64}$'),
  CONSTRAINT linux_agent_data_hash_hex_chk CHECK (source_data_hash_hex IS NULL OR source_data_hash_hex ~ '^[0-9a-f]{64}$'),
  CONSTRAINT linux_agent_ports_chk CHECK (
    (network_src_port IS NULL OR (network_src_port BETWEEN 1 AND 65535)) AND
    (network_dst_port IS NULL OR (network_dst_port BETWEEN 1 AND 65535))
  )
);

COMMENT ON TABLE linux_agent_telemetry IS
'Purpose: High-fidelity Linux endpoint telemetry events (process/file/network) used for ingestion, correlation, and enforcement.\n'
'Writing module(s): Linux Agent.\n'
'Reading module(s): Core Engine ingestion, Normalizer, Correlation Engine, Policy Engine, Forensics, AI/ML.\n'
'Retention expectation: long.';

COMMENT ON COLUMN linux_agent_telemetry.telemetry_id IS 'Primary key. UUID for telemetry record.';
COMMENT ON COLUMN linux_agent_telemetry.agent_id IS 'Foreign key to agents.agent_id (Linux agent identity).';
COMMENT ON COLUMN linux_agent_telemetry.observed_at IS 'Event time on the host (source timestamp).';
COMMENT ON COLUMN linux_agent_telemetry.received_at IS 'Ingestion receive time at server.';
COMMENT ON COLUMN linux_agent_telemetry.boot_id IS 'Host boot id associated with the event (optional).';
COMMENT ON COLUMN linux_agent_telemetry.pid IS 'Process ID that generated the event (optional).';
COMMENT ON COLUMN linux_agent_telemetry.ppid IS 'Parent process ID (optional).';
COMMENT ON COLUMN linux_agent_telemetry.uid IS 'User ID associated with the event (optional).';
COMMENT ON COLUMN linux_agent_telemetry.gid IS 'Group ID associated with the event (optional).';
COMMENT ON COLUMN linux_agent_telemetry.username IS 'Username associated with uid at observation time (optional).';
COMMENT ON COLUMN linux_agent_telemetry.process_name IS 'Process name (optional).';
COMMENT ON COLUMN linux_agent_telemetry.process_path IS 'Full process path if available (optional).';
COMMENT ON COLUMN linux_agent_telemetry.cmdline IS 'Full command line if available (optional).';
COMMENT ON COLUMN linux_agent_telemetry.event_name IS 'Event name/type emitted by agent (e.g., exec, file_write).';
COMMENT ON COLUMN linux_agent_telemetry.event_category IS 'Optional category grouping (process/file/network/auth).';
COMMENT ON COLUMN linux_agent_telemetry.severity IS 'Severity classification at source or ingestion.';
COMMENT ON COLUMN linux_agent_telemetry.file_path IS 'Path of file involved if applicable (optional).';
COMMENT ON COLUMN linux_agent_telemetry.file_sha256 IS 'SHA-256 hash of file content when available (optional).';
COMMENT ON COLUMN linux_agent_telemetry.network_src_ip IS 'Source IP for network events (optional).';
COMMENT ON COLUMN linux_agent_telemetry.network_src_port IS 'Source port for network events (optional).';
COMMENT ON COLUMN linux_agent_telemetry.network_dst_ip IS 'Destination IP for network events (optional).';
COMMENT ON COLUMN linux_agent_telemetry.network_dst_port IS 'Destination port for network events (optional).';
COMMENT ON COLUMN linux_agent_telemetry.protocol IS 'L4/L7 protocol label if available (optional).';
COMMENT ON COLUMN linux_agent_telemetry.payload IS 'Optional structured payload for event-specific fields (JSONB justified for heterogeneity).';
COMMENT ON COLUMN linux_agent_telemetry.payload_sha256 IS 'SHA-256 digest of payload for integrity/deduplication (optional).';
COMMENT ON COLUMN linux_agent_telemetry.correlation_hint IS 'Optional hint key used by correlator (e.g., session id, trace id).';
COMMENT ON COLUMN linux_agent_telemetry.created_at IS 'Row creation timestamp.';

CREATE INDEX IF NOT EXISTS idx_linux_agent_telemetry_observed_at ON linux_agent_telemetry (observed_at);
CREATE INDEX IF NOT EXISTS idx_linux_agent_telemetry_received_at ON linux_agent_telemetry (received_at);
CREATE INDEX IF NOT EXISTS idx_linux_agent_telemetry_agent_id ON linux_agent_telemetry (agent_id);
CREATE INDEX IF NOT EXISTS idx_linux_agent_telemetry_agent_observed_at ON linux_agent_telemetry (agent_id, observed_at DESC);
CREATE UNIQUE INDEX IF NOT EXISTS idx_linux_agent_telemetry_source_message_id_uniq
  ON linux_agent_telemetry (source_message_id)
  WHERE source_message_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS windows_agent_telemetry (
  telemetry_id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  agent_id               uuid NOT NULL REFERENCES agents(agent_id) ON UPDATE RESTRICT ON DELETE RESTRICT,
  -- Signed envelope fields (agent protocol contract)
  source_message_id      uuid NULL,
  source_nonce           text NULL,
  source_component_identity text NULL,
  source_host_id         text NULL,
  source_signature_b64   text NULL,
  source_signature_alg   text NULL,
  source_data_hash_hex   text NULL,
  observed_at            timestamptz NOT NULL,
  received_at            timestamptz NOT NULL DEFAULT now(),
  boot_id                uuid NULL,
  pid                    integer NULL,
  ppid                   integer NULL,
  user_sid               text NULL,
  username               text NULL,
  image_path             text NULL,
  cmdline                text NULL,
  event_name             text NOT NULL,
  event_provider         text NULL,
  event_id               integer NULL,
  severity               severity_level NOT NULL DEFAULT 'info',
  -- Protocol-aligned typed fields
  file_operation         text NULL,
  file_process_id        integer NULL,
  registry_operation     text NULL,
  auth_user              text NULL,
  auth_source            text NULL,
  auth_success           boolean NULL,
  network_process_id     integer NULL,
  file_path              text NULL,
  file_sha256            bytea NULL,
  registry_key           text NULL,
  registry_value_name    text NULL,
  registry_value_data    text NULL,
  network_src_ip         inet NULL,
  network_src_port       integer NULL,
  network_dst_ip         inet NULL,
  network_dst_port       integer NULL,
  protocol               text NULL,
  payload                jsonb NULL,
  payload_sha256         bytea NULL,
  correlation_hint       text NULL,
  created_at             timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT windows_agent_nonce_hex_chk CHECK (source_nonce IS NULL OR source_nonce ~ '^[0-9a-f]{64}$'),
  CONSTRAINT windows_agent_data_hash_hex_chk CHECK (source_data_hash_hex IS NULL OR source_data_hash_hex ~ '^[0-9a-f]{64}$'),
  CONSTRAINT windows_agent_ports_chk CHECK (
    (network_src_port IS NULL OR (network_src_port BETWEEN 1 AND 65535)) AND
    (network_dst_port IS NULL OR (network_dst_port BETWEEN 1 AND 65535))
  )
);

COMMENT ON TABLE windows_agent_telemetry IS
'Purpose: High-fidelity Windows endpoint telemetry events (ETW/security/process/file/registry/network) used for ingestion, correlation, and enforcement.\n'
'Writing module(s): Windows Agent.\n'
'Reading module(s): Core Engine ingestion, Normalizer, Correlation Engine, Policy Engine, Forensics, AI/ML.\n'
'Retention expectation: long.';

COMMENT ON COLUMN windows_agent_telemetry.telemetry_id IS 'Primary key. UUID for telemetry record.';
COMMENT ON COLUMN windows_agent_telemetry.agent_id IS 'Foreign key to agents.agent_id (Windows agent identity).';
COMMENT ON COLUMN windows_agent_telemetry.observed_at IS 'Event time on the host (source timestamp).';
COMMENT ON COLUMN windows_agent_telemetry.received_at IS 'Ingestion receive time at server.';
COMMENT ON COLUMN windows_agent_telemetry.boot_id IS 'Host boot id associated with the event (optional).';
COMMENT ON COLUMN windows_agent_telemetry.pid IS 'Process ID that generated the event (optional).';
COMMENT ON COLUMN windows_agent_telemetry.ppid IS 'Parent process ID (optional).';
COMMENT ON COLUMN windows_agent_telemetry.user_sid IS 'Windows user SID associated with event (optional).';
COMMENT ON COLUMN windows_agent_telemetry.username IS 'Username associated with SID at observation time (optional).';
COMMENT ON COLUMN windows_agent_telemetry.image_path IS 'Executable image path (optional).';
COMMENT ON COLUMN windows_agent_telemetry.cmdline IS 'Full command line if available (optional).';
COMMENT ON COLUMN windows_agent_telemetry.event_name IS 'Event name/type emitted by agent.';
COMMENT ON COLUMN windows_agent_telemetry.event_provider IS 'ETW provider/source name (optional).';
COMMENT ON COLUMN windows_agent_telemetry.event_id IS 'Provider-specific event id (optional).';
COMMENT ON COLUMN windows_agent_telemetry.severity IS 'Severity classification at source or ingestion.';
COMMENT ON COLUMN windows_agent_telemetry.file_path IS 'Path of file involved if applicable (optional).';
COMMENT ON COLUMN windows_agent_telemetry.file_sha256 IS 'SHA-256 hash of file content when available (optional).';
COMMENT ON COLUMN windows_agent_telemetry.registry_key IS 'Registry key path involved (optional).';
COMMENT ON COLUMN windows_agent_telemetry.registry_value_name IS 'Registry value name involved (optional).';
COMMENT ON COLUMN windows_agent_telemetry.registry_value_data IS 'Registry value data as string when captured (optional).';
COMMENT ON COLUMN windows_agent_telemetry.network_src_ip IS 'Source IP for network events (optional).';
COMMENT ON COLUMN windows_agent_telemetry.network_src_port IS 'Source port for network events (optional).';
COMMENT ON COLUMN windows_agent_telemetry.network_dst_ip IS 'Destination IP for network events (optional).';
COMMENT ON COLUMN windows_agent_telemetry.network_dst_port IS 'Destination port for network events (optional).';
COMMENT ON COLUMN windows_agent_telemetry.protocol IS 'L4/L7 protocol label if available (optional).';
COMMENT ON COLUMN windows_agent_telemetry.payload IS 'Optional structured payload for event-specific fields (JSONB justified for heterogeneity).';
COMMENT ON COLUMN windows_agent_telemetry.payload_sha256 IS 'SHA-256 digest of payload for integrity/deduplication (optional).';
COMMENT ON COLUMN windows_agent_telemetry.correlation_hint IS 'Optional hint key used by correlator (e.g., session id, trace id).';
COMMENT ON COLUMN windows_agent_telemetry.created_at IS 'Row creation timestamp.';

CREATE INDEX IF NOT EXISTS idx_windows_agent_telemetry_observed_at ON windows_agent_telemetry (observed_at);
CREATE INDEX IF NOT EXISTS idx_windows_agent_telemetry_received_at ON windows_agent_telemetry (received_at);
CREATE INDEX IF NOT EXISTS idx_windows_agent_telemetry_agent_id ON windows_agent_telemetry (agent_id);
CREATE INDEX IF NOT EXISTS idx_windows_agent_telemetry_agent_observed_at ON windows_agent_telemetry (agent_id, observed_at DESC);
CREATE UNIQUE INDEX IF NOT EXISTS idx_windows_agent_telemetry_source_message_id_uniq
  ON windows_agent_telemetry (source_message_id)
  WHERE source_message_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS dpi_probe_telemetry (
  telemetry_id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  agent_id               uuid NOT NULL REFERENCES agents(agent_id) ON UPDATE RESTRICT ON DELETE RESTRICT,
  -- Signed envelope fields (probe protocol contract)
  source_message_id      uuid NULL,
  source_nonce           text NULL,
  source_component_identity text NULL,
  source_signature_b64   text NULL,
  source_signature_alg   text NULL,
  source_data_hash_hex   text NULL,
  observed_at            timestamptz NOT NULL,
  received_at            timestamptz NOT NULL DEFAULT now(),
  sensor_id              text NULL,
  iface_name             text NULL,
  flow_id                text NULL,
  src_ip                 inet NULL,
  src_port               integer NULL,
  dst_ip                 inet NULL,
  dst_port               integer NULL,
  protocol               text NULL,
  -- Protocol-aligned totals (DPI schema exposes totals, while bytes_in/out can also be captured)
  packet_count           bigint NULL,
  byte_count             bigint NULL,
  classification         text NULL,
  metadata               jsonb NULL,
  bytes_in               bigint NULL,
  bytes_out              bigint NULL,
  packets_in             bigint NULL,
  packets_out            bigint NULL,
  tls_sni                text NULL,
  http_host              text NULL,
  http_method            text NULL,
  http_path              text NULL,
  ja3                    text NULL,
  ja3s                   text NULL,
  severity               severity_level NOT NULL DEFAULT 'info',
  payload                jsonb NULL,
  payload_sha256         bytea NULL,
  correlation_hint       text NULL,
  created_at             timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT dpi_probe_nonce_hex_chk CHECK (source_nonce IS NULL OR source_nonce ~ '^[0-9a-f]{64}$'),
  CONSTRAINT dpi_probe_data_hash_hex_chk CHECK (source_data_hash_hex IS NULL OR source_data_hash_hex ~ '^[0-9a-f]{64}$'),
  CONSTRAINT dpi_probe_ports_chk CHECK (
    (src_port IS NULL OR (src_port BETWEEN 1 AND 65535)) AND
    (dst_port IS NULL OR (dst_port BETWEEN 1 AND 65535))
  )
);

COMMENT ON TABLE dpi_probe_telemetry IS
'Purpose: Network telemetry from DPI probe (flows, L7 hints, metadata) used for correlation and detection.\n'
'Writing module(s): DPI Probe.\n'
'Reading module(s): Core Engine ingestion, Normalizer, Correlation Engine, AI/ML, Policy Engine.\n'
'Retention expectation: long.';

COMMENT ON COLUMN dpi_probe_telemetry.telemetry_id IS 'Primary key. UUID for telemetry record.';
COMMENT ON COLUMN dpi_probe_telemetry.agent_id IS 'Foreign key to agents.agent_id (DPI probe identity).';
COMMENT ON COLUMN dpi_probe_telemetry.observed_at IS 'Event time at sensor (source timestamp).';
COMMENT ON COLUMN dpi_probe_telemetry.received_at IS 'Ingestion receive time at server.';
COMMENT ON COLUMN dpi_probe_telemetry.sensor_id IS 'Optional sensor identifier within probe deployment.';
COMMENT ON COLUMN dpi_probe_telemetry.iface_name IS 'Interface name used for capture (optional).';
COMMENT ON COLUMN dpi_probe_telemetry.flow_id IS 'Probe flow/session identifier (optional).';
COMMENT ON COLUMN dpi_probe_telemetry.src_ip IS 'Source IP observed.';
COMMENT ON COLUMN dpi_probe_telemetry.src_port IS 'Source port observed (optional for non-TCP/UDP).';
COMMENT ON COLUMN dpi_probe_telemetry.dst_ip IS 'Destination IP observed.';
COMMENT ON COLUMN dpi_probe_telemetry.dst_port IS 'Destination port observed (optional for non-TCP/UDP).';
COMMENT ON COLUMN dpi_probe_telemetry.protocol IS 'Protocol label (tcp/udp/icmp/http/tls/etc.) if known.';
COMMENT ON COLUMN dpi_probe_telemetry.bytes_in IS 'Ingress bytes for the flow/event (optional).';
COMMENT ON COLUMN dpi_probe_telemetry.bytes_out IS 'Egress bytes for the flow/event (optional).';
COMMENT ON COLUMN dpi_probe_telemetry.packets_in IS 'Ingress packets for the flow/event (optional).';
COMMENT ON COLUMN dpi_probe_telemetry.packets_out IS 'Egress packets for the flow/event (optional).';
COMMENT ON COLUMN dpi_probe_telemetry.tls_sni IS 'TLS SNI hostname if present (optional).';
COMMENT ON COLUMN dpi_probe_telemetry.http_host IS 'HTTP Host header if present (optional).';
COMMENT ON COLUMN dpi_probe_telemetry.http_method IS 'HTTP method if present (optional).';
COMMENT ON COLUMN dpi_probe_telemetry.http_path IS 'HTTP path if present (optional).';
COMMENT ON COLUMN dpi_probe_telemetry.ja3 IS 'JA3 client fingerprint if present (optional).';
COMMENT ON COLUMN dpi_probe_telemetry.ja3s IS 'JA3S server fingerprint if present (optional).';
COMMENT ON COLUMN dpi_probe_telemetry.severity IS 'Severity classification at source or ingestion.';
COMMENT ON COLUMN dpi_probe_telemetry.payload IS 'Optional structured payload for protocol-specific fields (JSONB justified for heterogeneity).';
COMMENT ON COLUMN dpi_probe_telemetry.payload_sha256 IS 'SHA-256 digest of payload for integrity/deduplication (optional).';
COMMENT ON COLUMN dpi_probe_telemetry.correlation_hint IS 'Optional hint key used by correlator (e.g., session id, trace id).';
COMMENT ON COLUMN dpi_probe_telemetry.created_at IS 'Row creation timestamp.';

CREATE INDEX IF NOT EXISTS idx_dpi_probe_telemetry_observed_at ON dpi_probe_telemetry (observed_at);
CREATE INDEX IF NOT EXISTS idx_dpi_probe_telemetry_received_at ON dpi_probe_telemetry (received_at);
CREATE INDEX IF NOT EXISTS idx_dpi_probe_telemetry_agent_id ON dpi_probe_telemetry (agent_id);
CREATE INDEX IF NOT EXISTS idx_dpi_probe_telemetry_agent_observed_at ON dpi_probe_telemetry (agent_id, observed_at DESC);
CREATE UNIQUE INDEX IF NOT EXISTS idx_dpi_probe_telemetry_source_message_id_uniq
  ON dpi_probe_telemetry (source_message_id)
  WHERE source_message_id IS NOT NULL;

-- ============================================================================
-- A0. Unified signed telemetry stream (public schema; required by Posture Engine)
-- ============================================================================
--
-- NOTE: Some modules (e.g., Posture Engine) connect without setting search_path and query
-- `FROM telemetry_events`. To remain code-compatible without modifying modules, this table
-- is created in `public` and is populated via triggers from the canonical per-producer
-- telemetry tables in schema `ransomeye`.
CREATE TABLE IF NOT EXISTS public.telemetry_events (
  event_id               text PRIMARY KEY,
  producer_id            text NOT NULL,
  producer_type          event_source_type NOT NULL,
  host_id                text NULL,
  "timestamp"            timestamptz NOT NULL,
  event_type             text NOT NULL,
  event_data             jsonb NOT NULL,
  signature              text NOT NULL,
  signature_algorithm    text NULL,
  signature_valid        boolean NULL,
  created_at             timestamptz NOT NULL DEFAULT now()
);

COMMENT ON TABLE public.telemetry_events IS
'Purpose: Unified signed telemetry stream for consumers that query without ransomeye search_path (e.g., Posture Engine).\n'
'Writing module(s): Populated by DB triggers from linux_agent_telemetry/windows_agent_telemetry/dpi_probe_telemetry.\n'
'Reading module(s): Posture Engine (signature verification), UI.\n'
'Retention expectation: long.';

CREATE INDEX IF NOT EXISTS idx_telemetry_events_timestamp ON public.telemetry_events ("timestamp");
CREATE INDEX IF NOT EXISTS idx_telemetry_events_producer_type_ts ON public.telemetry_events (producer_type, "timestamp" DESC);
CREATE INDEX IF NOT EXISTS idx_telemetry_events_host_id_ts ON public.telemetry_events (host_id, "timestamp" DESC);
CREATE INDEX IF NOT EXISTS idx_telemetry_events_producer_id_ts ON public.telemetry_events (producer_id, "timestamp" DESC);

CREATE OR REPLACE FUNCTION ransomeye.mirror_linux_telemetry_to_public_stream()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  INSERT INTO public.telemetry_events (
    event_id, producer_id, producer_type, host_id, "timestamp", event_type, event_data,
    signature, signature_algorithm, signature_valid
  )
  VALUES (
    COALESCE(NEW.source_message_id::text, NEW.telemetry_id::text),
    COALESCE(NEW.source_component_identity, NEW.agent_id::text),
    'linux_agent',
    NEW.source_host_id,
    NEW.observed_at,
    NEW.event_name,
    jsonb_build_object(
      'telemetry_id', NEW.telemetry_id,
      'agent_id', NEW.agent_id,
      'observed_at', NEW.observed_at,
      'received_at', NEW.received_at,
      'event_name', NEW.event_name,
      'event_category', NEW.event_category,
      'severity', NEW.severity,
      'pid', NEW.pid,
      'ppid', NEW.ppid,
      'uid', NEW.uid,
      'gid', NEW.gid,
      'username', NEW.username,
      'process_name', NEW.process_name,
      'process_path', NEW.process_path,
      'cmdline', NEW.cmdline,
      'file_path', NEW.file_path,
      'network_src_ip', NEW.network_src_ip,
      'network_src_port', NEW.network_src_port,
      'network_dst_ip', NEW.network_dst_ip,
      'network_dst_port', NEW.network_dst_port,
      'protocol', NEW.protocol,
      'payload', NEW.payload,
      'payload_sha256', encode(NEW.payload_sha256, 'hex'),
      'source_nonce', NEW.source_nonce,
      'source_data_hash', NEW.source_data_hash_hex
    ),
    COALESCE(NEW.source_signature_b64, ''),
    NEW.source_signature_alg,
    NULL
  )
  ON CONFLICT (event_id) DO NOTHING;
  RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION ransomeye.mirror_windows_telemetry_to_public_stream()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  INSERT INTO public.telemetry_events (
    event_id, producer_id, producer_type, host_id, "timestamp", event_type, event_data,
    signature, signature_algorithm, signature_valid
  )
  VALUES (
    COALESCE(NEW.source_message_id::text, NEW.telemetry_id::text),
    COALESCE(NEW.source_component_identity, NEW.agent_id::text),
    'windows_agent',
    NEW.source_host_id,
    NEW.observed_at,
    NEW.event_name,
    jsonb_build_object(
      'telemetry_id', NEW.telemetry_id,
      'agent_id', NEW.agent_id,
      'observed_at', NEW.observed_at,
      'received_at', NEW.received_at,
      'event_name', NEW.event_name,
      'event_provider', NEW.event_provider,
      'event_id', NEW.event_id,
      'severity', NEW.severity,
      'pid', NEW.pid,
      'ppid', NEW.ppid,
      'user_sid', NEW.user_sid,
      'username', NEW.username,
      'image_path', NEW.image_path,
      'cmdline', NEW.cmdline,
      'file_path', NEW.file_path,
      'registry_key', NEW.registry_key,
      'registry_value_name', NEW.registry_value_name,
      'registry_value_data', NEW.registry_value_data,
      'network_src_ip', NEW.network_src_ip,
      'network_src_port', NEW.network_src_port,
      'network_dst_ip', NEW.network_dst_ip,
      'network_dst_port', NEW.network_dst_port,
      'protocol', NEW.protocol,
      'payload', NEW.payload,
      'payload_sha256', encode(NEW.payload_sha256, 'hex'),
      'source_nonce', NEW.source_nonce,
      'source_data_hash', NEW.source_data_hash_hex
    ),
    COALESCE(NEW.source_signature_b64, ''),
    NEW.source_signature_alg,
    NULL
  )
  ON CONFLICT (event_id) DO NOTHING;
  RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION ransomeye.mirror_dpi_telemetry_to_public_stream()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  INSERT INTO public.telemetry_events (
    event_id, producer_id, producer_type, host_id, "timestamp", event_type, event_data,
    signature, signature_algorithm, signature_valid
  )
  VALUES (
    COALESCE(NEW.source_message_id::text, NEW.telemetry_id::text),
    COALESCE(NEW.source_component_identity, NEW.agent_id::text),
    'dpi_probe',
    NULL,
    NEW.observed_at,
    COALESCE(NEW.payload->>'event_type', 'flow'),
    jsonb_build_object(
      'telemetry_id', NEW.telemetry_id,
      'agent_id', NEW.agent_id,
      'observed_at', NEW.observed_at,
      'received_at', NEW.received_at,
      'sensor_id', NEW.sensor_id,
      'iface_name', NEW.iface_name,
      'flow_id', NEW.flow_id,
      'src_ip', NEW.src_ip,
      'src_port', NEW.src_port,
      'dst_ip', NEW.dst_ip,
      'dst_port', NEW.dst_port,
      'protocol', NEW.protocol,
      'bytes_in', NEW.bytes_in,
      'bytes_out', NEW.bytes_out,
      'packets_in', NEW.packets_in,
      'packets_out', NEW.packets_out,
      'tls_sni', NEW.tls_sni,
      'http_host', NEW.http_host,
      'http_method', NEW.http_method,
      'http_path', NEW.http_path,
      'ja3', NEW.ja3,
      'ja3s', NEW.ja3s,
      'severity', NEW.severity,
      'payload', NEW.payload,
      'payload_sha256', encode(NEW.payload_sha256, 'hex'),
      'source_nonce', NEW.source_nonce,
      'source_data_hash', NEW.source_data_hash_hex
    ),
    COALESCE(NEW.source_signature_b64, ''),
    NEW.source_signature_alg,
    NULL
  )
  ON CONFLICT (event_id) DO NOTHING;
  RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trg_linux_telemetry_to_public_stream ON linux_agent_telemetry;
CREATE TRIGGER trg_linux_telemetry_to_public_stream
AFTER INSERT ON linux_agent_telemetry
FOR EACH ROW EXECUTE FUNCTION ransomeye.mirror_linux_telemetry_to_public_stream();

DROP TRIGGER IF EXISTS trg_windows_telemetry_to_public_stream ON windows_agent_telemetry;
CREATE TRIGGER trg_windows_telemetry_to_public_stream
AFTER INSERT ON windows_agent_telemetry
FOR EACH ROW EXECUTE FUNCTION ransomeye.mirror_windows_telemetry_to_public_stream();

DROP TRIGGER IF EXISTS trg_dpi_telemetry_to_public_stream ON dpi_probe_telemetry;
CREATE TRIGGER trg_dpi_telemetry_to_public_stream
AFTER INSERT ON dpi_probe_telemetry
FOR EACH ROW EXECUTE FUNCTION ransomeye.mirror_dpi_telemetry_to_public_stream();

-- ============================================================================
-- A1. Retention policy configuration (required by DB Validator; enforcement comes later)
-- ============================================================================
CREATE TABLE IF NOT EXISTS retention_policies (
  table_name             text PRIMARY KEY,
  retention_days         integer NOT NULL,
  retention_enabled      boolean NOT NULL DEFAULT true,
  created_at             timestamptz NOT NULL DEFAULT now(),
  updated_at             timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT retention_days_max_chk CHECK (retention_days BETWEEN 1 AND 2555)
);

COMMENT ON TABLE retention_policies IS
'Purpose: Install-time retention configuration per table. This is configuration only (no purge logic).\n'
'Writing module(s): Installer/Validator.\n'
'Reading module(s): Retention engine, Global Validator.\n'
'Retention expectation: long.';

CREATE INDEX IF NOT EXISTS idx_retention_policies_enabled ON retention_policies (retention_enabled);

-- ============================================================================
-- Public-schema tables required by modules that create/query without ransomeye search_path
-- ============================================================================
CREATE TABLE IF NOT EXISTS public.scan_results (
  scan_id               varchar(36) PRIMARY KEY,
  "timestamp"           timestamptz NOT NULL,
  scanner_mode          varchar(20) NOT NULL,
  asset_ip              varchar(45) NOT NULL,
  asset_hostname        varchar(255),
  asset_mac             varchar(17),
  asset_vendor          varchar(255),
  open_ports            jsonb NOT NULL DEFAULT '[]'::jsonb,
  services              jsonb NOT NULL DEFAULT '[]'::jsonb,
  confidence_score      double precision NOT NULL,
  hash                  varchar(64) NOT NULL,
  signature             text NOT NULL,
  metadata              jsonb,
  created_at            timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS public.scan_assets (
  asset_id              serial PRIMARY KEY,
  ip                    varchar(45) NOT NULL UNIQUE,
  hostname              varchar(255),
  mac                   varchar(17),
  vendor                varchar(255),
  first_seen            timestamptz NOT NULL DEFAULT now(),
  last_seen             timestamptz NOT NULL DEFAULT now(),
  scan_count            integer NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS public.scan_port_services (
  mapping_id            serial PRIMARY KEY,
  asset_ip              varchar(45) NOT NULL,
  port                  integer NOT NULL,
  protocol              varchar(10) NOT NULL,
  service_name          varchar(255),
  service_version       varchar(255),
  first_seen            timestamptz NOT NULL DEFAULT now(),
  last_seen             timestamptz NOT NULL DEFAULT now(),
  UNIQUE(asset_ip, port, protocol)
);

CREATE TABLE IF NOT EXISTS public.scan_deltas (
  delta_id              serial PRIMARY KEY,
  scan_id               varchar(36) NOT NULL,
  asset_ip              varchar(45) NOT NULL,
  delta_type            varchar(20) NOT NULL,
  delta_data            jsonb NOT NULL,
  created_at            timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_scan_results_asset_ip ON public.scan_results(asset_ip);
CREATE INDEX IF NOT EXISTS idx_scan_results_timestamp ON public.scan_results("timestamp");
CREATE INDEX IF NOT EXISTS idx_scan_results_asset_ip_ts ON public.scan_results(asset_ip, "timestamp" DESC);
CREATE INDEX IF NOT EXISTS idx_scan_results_hash ON public.scan_results(hash);
CREATE INDEX IF NOT EXISTS idx_scan_assets_last_seen ON public.scan_assets(last_seen DESC);
CREATE INDEX IF NOT EXISTS idx_scan_port_services_asset_ip ON public.scan_port_services(asset_ip);
CREATE INDEX IF NOT EXISTS idx_scan_port_services_asset_ip_last_seen ON public.scan_port_services(asset_ip, last_seen DESC);
CREATE INDEX IF NOT EXISTS idx_scan_deltas_asset_ip ON public.scan_deltas(asset_ip);
CREATE INDEX IF NOT EXISTS idx_scan_deltas_asset_ip_created_at ON public.scan_deltas(asset_ip, created_at DESC);

CREATE TABLE IF NOT EXISTS public.playbook_executions (
  execution_id          varchar(36) PRIMARY KEY,
  playbook_id           varchar(36) NOT NULL,
  state                 varchar(20) NOT NULL,
  started_at            timestamptz NOT NULL,
  completed_at          timestamptz,
  current_step          integer NOT NULL DEFAULT 0,
  step_results          jsonb NOT NULL DEFAULT '{}'::jsonb,
  nonce                 varchar(36) NOT NULL UNIQUE,
  policy_decision_id    varchar(36),
  created_at            timestamptz NOT NULL DEFAULT now(),
  updated_at            timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS public.playbook_rollback_states (
  rollback_id           varchar(36) PRIMARY KEY,
  execution_id          varchar(36) NOT NULL,
  playbook_id           varchar(36) NOT NULL,
  started_at            timestamptz NOT NULL,
  completed_at          timestamptz,
  rollback_step_results jsonb NOT NULL DEFAULT '[]'::jsonb,
  status                varchar(20) NOT NULL,
  created_at            timestamptz NOT NULL DEFAULT now(),
  updated_at            timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS public.playbook_nonces (
  nonce                 varchar(36) PRIMARY KEY,
  used_at               timestamptz NOT NULL DEFAULT now(),
  execution_id          varchar(36)
);

CREATE TABLE IF NOT EXISTS public.playbook_audit_log (
  audit_id              serial PRIMARY KEY,
  execution_id          varchar(36),
  rollback_id           varchar(36),
  event_type            varchar(50) NOT NULL,
  event_data            jsonb NOT NULL,
  created_at            timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS public.playbook_safe_halt_state (
  halt_id               serial PRIMARY KEY,
  rollback_id           varchar(36) NOT NULL,
  error_message         text,
  entered_at            timestamptz NOT NULL DEFAULT now(),
  resolved_at           timestamptz,
  is_active             boolean NOT NULL DEFAULT true
);

CREATE INDEX IF NOT EXISTS idx_playbook_executions_playbook_id ON public.playbook_executions(playbook_id);
CREATE INDEX IF NOT EXISTS idx_playbook_executions_state ON public.playbook_executions(state);
CREATE INDEX IF NOT EXISTS idx_playbook_executions_started_at ON public.playbook_executions(started_at DESC);
CREATE INDEX IF NOT EXISTS idx_playbook_executions_updated_at ON public.playbook_executions(updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_playbook_executions_policy_decision_id ON public.playbook_executions(policy_decision_id);
CREATE INDEX IF NOT EXISTS idx_playbook_rollback_execution_id ON public.playbook_rollback_states(execution_id);
CREATE INDEX IF NOT EXISTS idx_playbook_rollback_status ON public.playbook_rollback_states(status);
CREATE INDEX IF NOT EXISTS idx_playbook_audit_created_at ON public.playbook_audit_log(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_playbook_audit_execution_id ON public.playbook_audit_log(execution_id);
CREATE INDEX IF NOT EXISTS idx_playbook_safe_halt_active ON public.playbook_safe_halt_state(is_active) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_playbook_safe_halt_entered_at ON public.playbook_safe_halt_state(entered_at DESC);

-- ============================================================================
-- B. Ingestion & Normalization (REQUIRED)
-- ============================================================================

CREATE TABLE IF NOT EXISTS raw_events (
  raw_event_id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  source_type            event_source_type NOT NULL,
  source_agent_id        uuid NULL REFERENCES agents(agent_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  source_component_id    uuid NULL REFERENCES components(component_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  ingestion_pipeline     text NULL,
  observed_at            timestamptz NULL,
  received_at            timestamptz NOT NULL DEFAULT now(),
  event_name             text NULL,
  transport_id           text NULL,
  trace_id               text NULL,
  payload_json           jsonb NULL,
  payload_bytes          bytea NULL,
  payload_sha256         bytea NOT NULL,
  schema_version         text NULL,
  is_replay              boolean NOT NULL DEFAULT false,
  replay_source          text NULL,
  created_at             timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT raw_events_payload_presence_chk CHECK (payload_json IS NOT NULL OR payload_bytes IS NOT NULL),
  CONSTRAINT raw_events_payload_sha256_len_chk CHECK (octet_length(payload_sha256) = 32)
);

COMMENT ON TABLE raw_events IS
'Purpose: Append-only ingestion buffer storing the original raw event payload as received (for forensic replay and deterministic normalization).\n'
'Writing module(s): Core Engine ingestion, Agent gateways, DPI gateway.\n'
'Reading module(s): Normalizer, Forensic Engine, Validator.\n'
'Retention expectation: long.';

COMMENT ON COLUMN raw_events.raw_event_id IS 'Primary key. UUID for raw event record.';
COMMENT ON COLUMN raw_events.source_type IS 'Origin classification (agent/probe/component).';
COMMENT ON COLUMN raw_events.source_agent_id IS 'If source is an agent/probe, FK to agents.agent_id.';
COMMENT ON COLUMN raw_events.source_component_id IS 'If source is a service component, FK to components.component_id.';
COMMENT ON COLUMN raw_events.ingestion_pipeline IS 'Pipeline name/stage identifier (optional).';
COMMENT ON COLUMN raw_events.observed_at IS 'Source timestamp if provided by emitter (optional).';
COMMENT ON COLUMN raw_events.received_at IS 'Timestamp when ingestion accepted the event.';
COMMENT ON COLUMN raw_events.event_name IS 'Emitter-provided event name/type (optional).';
COMMENT ON COLUMN raw_events.transport_id IS 'Transport/session identifier (optional).';
COMMENT ON COLUMN raw_events.trace_id IS 'Distributed trace identifier for cross-component linking (optional).';
COMMENT ON COLUMN raw_events.payload_json IS 'Raw payload in JSONB (preferred for structured events).';
COMMENT ON COLUMN raw_events.payload_bytes IS 'Raw payload bytes for non-JSON payloads (optional).';
COMMENT ON COLUMN raw_events.payload_sha256 IS 'SHA-256 digest of canonical payload representation for integrity/deduplication.';
COMMENT ON COLUMN raw_events.schema_version IS 'Optional emitter schema version tag.';
COMMENT ON COLUMN raw_events.is_replay IS 'True if this raw event was re-ingested from forensic replay.';
COMMENT ON COLUMN raw_events.replay_source IS 'Replay source identifier (e.g., bundle id, evidence id) if is_replay is true.';
COMMENT ON COLUMN raw_events.created_at IS 'Row creation timestamp.';

CREATE INDEX IF NOT EXISTS idx_raw_events_received_at ON raw_events (received_at);
CREATE INDEX IF NOT EXISTS idx_raw_events_observed_at ON raw_events (observed_at);
CREATE INDEX IF NOT EXISTS idx_raw_events_source_agent_id ON raw_events (source_agent_id);
CREATE INDEX IF NOT EXISTS idx_raw_events_source_component_id ON raw_events (source_component_id);
CREATE INDEX IF NOT EXISTS idx_raw_events_payload_sha256 ON raw_events (payload_sha256);

CREATE TABLE IF NOT EXISTS normalized_events (
  normalized_event_id    uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  raw_event_id           uuid NOT NULL REFERENCES raw_events(raw_event_id) ON UPDATE RESTRICT ON DELETE RESTRICT,
  normalized_at          timestamptz NOT NULL DEFAULT now(),
  observed_at            timestamptz NULL,
  source_type            event_source_type NOT NULL,
  source_agent_id        uuid NULL REFERENCES agents(agent_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  source_component_id    uuid NULL REFERENCES components(component_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  event_kind             text NOT NULL,
  event_subkind          text NULL,
  severity               severity_level NOT NULL DEFAULT 'info',
  primary_entity_id      uuid NULL REFERENCES entities(entity_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  secondary_entity_id    uuid NULL REFERENCES entities(entity_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  actor_entity_id        uuid NULL REFERENCES entities(entity_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  target_entity_id       uuid NULL REFERENCES entities(entity_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  attributes             jsonb NULL,
  deterministic_key      bytea NOT NULL,
  created_at             timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT normalized_events_det_key_len_chk CHECK (octet_length(deterministic_key) IN (16, 20, 32, 64))
);

COMMENT ON TABLE normalized_events IS
'Purpose: Canonical normalized event representation derived from raw_events, enabling deterministic correlation, policy evaluation, and replay.\n'
'Writing module(s): Normalizer (Core Engine pipeline).\n'
'Reading module(s): Correlation Engine, Policy Engine, AI/ML, Forensic Engine, UI.\n'
'Retention expectation: long.';

COMMENT ON COLUMN normalized_events.normalized_event_id IS 'Primary key. UUID for normalized event record.';
COMMENT ON COLUMN normalized_events.raw_event_id IS 'Foreign key to raw_events.raw_event_id.';
COMMENT ON COLUMN normalized_events.normalized_at IS 'Timestamp when normalization occurred.';
COMMENT ON COLUMN normalized_events.observed_at IS 'Best-effort event time after normalization (optional).';
COMMENT ON COLUMN normalized_events.source_type IS 'Origin classification copied/derived from raw event.';
COMMENT ON COLUMN normalized_events.source_agent_id IS 'Agent/probe FK if applicable.';
COMMENT ON COLUMN normalized_events.source_component_id IS 'Component FK if applicable.';
COMMENT ON COLUMN normalized_events.event_kind IS 'Canonical event kind (e.g., process_start, file_write, dns_query, net_flow).';
COMMENT ON COLUMN normalized_events.event_subkind IS 'Optional more specific subtype (e.g., syscall name, ETW event type).';
COMMENT ON COLUMN normalized_events.severity IS 'Severity classification post-normalization.';
COMMENT ON COLUMN normalized_events.primary_entity_id IS 'Optional FK to primary entity involved (e.g., process, flow, file).';
COMMENT ON COLUMN normalized_events.secondary_entity_id IS 'Optional FK to secondary entity involved.';
COMMENT ON COLUMN normalized_events.actor_entity_id IS 'Optional FK to actor entity (e.g., user, process) initiating action.';
COMMENT ON COLUMN normalized_events.target_entity_id IS 'Optional FK to target entity (e.g., file, host, service).';
COMMENT ON COLUMN normalized_events.attributes IS 'Structured normalized attributes (JSONB justified for variable event facets).';
COMMENT ON COLUMN normalized_events.deterministic_key IS 'Deterministic correlation key derived from normalized fields (digest) for dedup/correlation.';
COMMENT ON COLUMN normalized_events.created_at IS 'Row creation timestamp.';

CREATE INDEX IF NOT EXISTS idx_normalized_events_normalized_at ON normalized_events (normalized_at);
CREATE INDEX IF NOT EXISTS idx_normalized_events_observed_at ON normalized_events (observed_at);
CREATE INDEX IF NOT EXISTS idx_normalized_events_raw_event_id ON normalized_events (raw_event_id);
CREATE INDEX IF NOT EXISTS idx_normalized_events_source_agent_id ON normalized_events (source_agent_id);
CREATE INDEX IF NOT EXISTS idx_normalized_events_source_component_id ON normalized_events (source_component_id);
CREATE INDEX IF NOT EXISTS idx_normalized_events_primary_entity_id ON normalized_events (primary_entity_id);
CREATE INDEX IF NOT EXISTS idx_normalized_events_det_key ON normalized_events (deterministic_key);

-- ============================================================================
-- C. Correlation & Detection (REQUIRED)
-- ============================================================================

CREATE TABLE IF NOT EXISTS correlation_graph (
  correlation_edge_id    uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  correlation_run_id     uuid NOT NULL,
  created_at             timestamptz NOT NULL DEFAULT now(),
  observed_start_at      timestamptz NULL,
  observed_end_at        timestamptz NULL,
  src_entity_id          uuid NOT NULL REFERENCES entities(entity_id) ON UPDATE RESTRICT ON DELETE RESTRICT,
  dst_entity_id          uuid NOT NULL REFERENCES entities(entity_id) ON UPDATE RESTRICT ON DELETE RESTRICT,
  relationship_type      text NOT NULL,
  relationship_subtype   text NULL,
  direction              text NOT NULL DEFAULT 'directed',
  evidence_event_id      uuid NULL REFERENCES normalized_events(normalized_event_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  evidence_count         bigint NOT NULL DEFAULT 1,
  weight                double precision NOT NULL DEFAULT 1.0,
  confidence             double precision NOT NULL DEFAULT 0.5,
  attributes             jsonb NULL,
  deterministic_key      bytea NOT NULL,
  CONSTRAINT correlation_graph_conf_chk CHECK (confidence >= 0.0 AND confidence <= 1.0),
  CONSTRAINT correlation_graph_weight_chk CHECK (weight >= 0.0),
  CONSTRAINT correlation_graph_direction_chk CHECK (direction IN ('directed','undirected')),
  CONSTRAINT correlation_graph_det_key_len_chk CHECK (octet_length(deterministic_key) IN (16, 20, 32, 64))
);

COMMENT ON TABLE correlation_graph IS
'Purpose: Correlation graph edges connecting canonical entities with evidence and confidence for deterministic threat chaining.\n'
'Writing module(s): Correlation Engine.\n'
'Reading module(s): Detection Engine, Policy Engine, Forensics, UI, Validator.\n'
'Retention expectation: long.';

COMMENT ON COLUMN correlation_graph.correlation_edge_id IS 'Primary key. UUID for correlation edge.';
COMMENT ON COLUMN correlation_graph.correlation_run_id IS 'Correlation run/batch identifier (UUID chosen by correlator).';
COMMENT ON COLUMN correlation_graph.created_at IS 'Edge creation timestamp.';
COMMENT ON COLUMN correlation_graph.observed_start_at IS 'Optional start time window covered by this edge evidence.';
COMMENT ON COLUMN correlation_graph.observed_end_at IS 'Optional end time window covered by this edge evidence.';
COMMENT ON COLUMN correlation_graph.src_entity_id IS 'Source entity FK.';
COMMENT ON COLUMN correlation_graph.dst_entity_id IS 'Destination entity FK.';
COMMENT ON COLUMN correlation_graph.relationship_type IS 'Relationship label (e.g., spawned, connected_to, wrote_file, resolved_domain).';
COMMENT ON COLUMN correlation_graph.relationship_subtype IS 'Optional subtype for relationship.';
COMMENT ON COLUMN correlation_graph.direction IS 'Graph directionality: directed or undirected.';
COMMENT ON COLUMN correlation_graph.evidence_event_id IS 'Optional normalized event that directly evidences this edge.';
COMMENT ON COLUMN correlation_graph.evidence_count IS 'Number of evidence items aggregated into this edge.';
COMMENT ON COLUMN correlation_graph.weight IS 'Edge weight for downstream algorithms (not optimized here).';
COMMENT ON COLUMN correlation_graph.confidence IS 'Edge confidence score in [0,1].';
COMMENT ON COLUMN correlation_graph.attributes IS 'Optional structured metadata for edge (JSONB justified for heterogeneity).';
COMMENT ON COLUMN correlation_graph.deterministic_key IS 'Deterministic key derived from (run, src, dst, rel) for deduplication.';

CREATE INDEX IF NOT EXISTS idx_correlation_graph_created_at ON correlation_graph (created_at);
CREATE INDEX IF NOT EXISTS idx_correlation_graph_run_id ON correlation_graph (correlation_run_id);
CREATE INDEX IF NOT EXISTS idx_correlation_graph_src_entity ON correlation_graph (src_entity_id);
CREATE INDEX IF NOT EXISTS idx_correlation_graph_dst_entity ON correlation_graph (dst_entity_id);
CREATE INDEX IF NOT EXISTS idx_correlation_graph_evidence_event ON correlation_graph (evidence_event_id);
CREATE INDEX IF NOT EXISTS idx_correlation_graph_det_key ON correlation_graph (deterministic_key);

CREATE TABLE IF NOT EXISTS detection_results (
  detection_id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  created_at             timestamptz NOT NULL DEFAULT now(),
  detection_engine       text NOT NULL,
  correlation_run_id     uuid NULL,
  normalized_event_id    uuid NULL REFERENCES normalized_events(normalized_event_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  primary_entity_id      uuid NULL REFERENCES entities(entity_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  detection_name         text NOT NULL,
  detection_category     text NULL,
  mitre_tactic           text NULL,
  mitre_technique        text NULL,
  severity               severity_level NOT NULL DEFAULT 'warning',
  confidence             double precision NOT NULL DEFAULT 0.5,
  score                  double precision NULL,
  reasoning              text NULL,
  artifacts              jsonb NULL,
  deterministic_key      bytea NOT NULL,
  CONSTRAINT detection_results_conf_chk CHECK (confidence >= 0.0 AND confidence <= 1.0),
  CONSTRAINT detection_results_det_key_len_chk CHECK (octet_length(deterministic_key) IN (16, 20, 32, 64))
);

COMMENT ON TABLE detection_results IS
'Purpose: Detection outputs (rules/ML/correlation-based) tied to normalized events/entities with confidence and MITRE mapping.\n'
'Writing module(s): Correlation Engine, AI/ML pipeline, Alert Engine.\n'
'Reading module(s): Policy Engine, Response Engine, UI, Forensics, Validator.\n'
'Retention expectation: long.';

COMMENT ON COLUMN detection_results.detection_id IS 'Primary key. UUID for detection record.';
COMMENT ON COLUMN detection_results.created_at IS 'Creation timestamp.';
COMMENT ON COLUMN detection_results.detection_engine IS 'Engine name/identifier producing the detection.';
COMMENT ON COLUMN detection_results.correlation_run_id IS 'Optional correlation run id if detection derived from graph batch.';
COMMENT ON COLUMN detection_results.normalized_event_id IS 'Optional FK to a primary normalized event that triggered detection.';
COMMENT ON COLUMN detection_results.primary_entity_id IS 'Optional FK to primary entity for detection.';
COMMENT ON COLUMN detection_results.detection_name IS 'Human-friendly detection name.';
COMMENT ON COLUMN detection_results.detection_category IS 'Optional category (malware, lateral_movement, exfil, etc.).';
COMMENT ON COLUMN detection_results.mitre_tactic IS 'Optional MITRE ATT&CK tactic identifier/name.';
COMMENT ON COLUMN detection_results.mitre_technique IS 'Optional MITRE ATT&CK technique identifier/name.';
COMMENT ON COLUMN detection_results.severity IS 'Severity of the detection.';
COMMENT ON COLUMN detection_results.confidence IS 'Confidence in [0,1].';
COMMENT ON COLUMN detection_results.score IS 'Optional raw model/rule score for ranking.';
COMMENT ON COLUMN detection_results.reasoning IS 'Optional human-readable reasoning summary.';
COMMENT ON COLUMN detection_results.artifacts IS 'Optional structured artifacts (IOCs, paths, snippets) as JSONB.';
COMMENT ON COLUMN detection_results.deterministic_key IS 'Deterministic key derived from engine+event/entity+name for deduplication.';

CREATE INDEX IF NOT EXISTS idx_detection_results_created_at ON detection_results (created_at);
CREATE INDEX IF NOT EXISTS idx_detection_results_norm_event ON detection_results (normalized_event_id);
CREATE INDEX IF NOT EXISTS idx_detection_results_entity ON detection_results (primary_entity_id);
CREATE INDEX IF NOT EXISTS idx_detection_results_corr_run ON detection_results (correlation_run_id);
CREATE INDEX IF NOT EXISTS idx_detection_results_det_key ON detection_results (deterministic_key);

CREATE TABLE IF NOT EXISTS confidence_scores (
  confidence_score_id    uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  created_at             timestamptz NOT NULL DEFAULT now(),
  target_type            text NOT NULL,
  detection_id           uuid NULL REFERENCES detection_results(detection_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  correlation_edge_id    uuid NULL REFERENCES correlation_graph(correlation_edge_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  model_version_id       uuid NULL,
  score_name             text NOT NULL,
  score_value            double precision NOT NULL,
  score_min              double precision NULL,
  score_max              double precision NULL,
  calibration_version    text NULL,
  rationale              text NULL,
  attributes             jsonb NULL
);

COMMENT ON TABLE confidence_scores IS
'Purpose: Store one or more confidence scores for detections/graph edges with provenance and optional calibration metadata.\n'
'Writing module(s): Correlation Engine, AI/ML pipeline.\n'
'Reading module(s): Policy Engine, UI, Validator, Forensics.\n'
'Retention expectation: long.';

COMMENT ON COLUMN confidence_scores.confidence_score_id IS 'Primary key. UUID for confidence score record.';
COMMENT ON COLUMN confidence_scores.created_at IS 'Creation timestamp.';
COMMENT ON COLUMN confidence_scores.target_type IS 'Target class for score (e.g., detection, edge, inference).';
COMMENT ON COLUMN confidence_scores.detection_id IS 'Optional FK to detection_results if score is for detection.';
COMMENT ON COLUMN confidence_scores.correlation_edge_id IS 'Optional FK to correlation_graph if score is for an edge.';
COMMENT ON COLUMN confidence_scores.model_version_id IS 'Optional FK to model_versions if score produced by a model.';
COMMENT ON COLUMN confidence_scores.score_name IS 'Name of score (e.g., p_malicious, trust_score, anomaly_score).';
COMMENT ON COLUMN confidence_scores.score_value IS 'Score numeric value.';
COMMENT ON COLUMN confidence_scores.score_min IS 'Optional minimum possible score value.';
COMMENT ON COLUMN confidence_scores.score_max IS 'Optional maximum possible score value.';
COMMENT ON COLUMN confidence_scores.calibration_version IS 'Optional calibration identifier/version.';
COMMENT ON COLUMN confidence_scores.rationale IS 'Optional explanation or notes about the score.';
COMMENT ON COLUMN confidence_scores.attributes IS 'Optional structured metadata (JSONB justified for extensibility).';

CREATE INDEX IF NOT EXISTS idx_confidence_scores_created_at ON confidence_scores (created_at);
CREATE INDEX IF NOT EXISTS idx_confidence_scores_detection_id ON confidence_scores (detection_id);
CREATE INDEX IF NOT EXISTS idx_confidence_scores_edge_id ON confidence_scores (correlation_edge_id);
CREATE INDEX IF NOT EXISTS idx_confidence_scores_model_version_id ON confidence_scores (model_version_id);

-- ============================================================================
-- E. AI / ML / LLM (REQUIRED) - model tables defined here to satisfy FK above
-- ============================================================================

CREATE TABLE IF NOT EXISTS model_registry (
  model_id               uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  model_name             text NOT NULL,
  model_task             model_task_type NOT NULL,
  owner_component_id     uuid NULL REFERENCES components(component_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  description            text NULL,
  is_active              boolean NOT NULL DEFAULT true,
  created_at             timestamptz NOT NULL DEFAULT now(),
  updated_at             timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT model_registry_name_uniq UNIQUE (model_name)
);

COMMENT ON TABLE model_registry IS
'Purpose: Canonical model registry (stable identity) for AI/ML/LLM artifacts used across RansomEye.\n'
'Writing module(s): AI Core (model registry).\n'
'Reading module(s): AI/ML pipeline, Validator, UI.\n'
'Retention expectation: long.';

COMMENT ON COLUMN model_registry.model_id IS 'Primary key. Stable UUID identity for a model family.';
COMMENT ON COLUMN model_registry.model_name IS 'Unique model name.';
COMMENT ON COLUMN model_registry.model_task IS 'Model task category (classification, anomaly_detection, etc.).';
COMMENT ON COLUMN model_registry.owner_component_id IS 'Component that owns the model (optional).';
COMMENT ON COLUMN model_registry.description IS 'Optional human-readable description.';
COMMENT ON COLUMN model_registry.is_active IS 'Whether model is active/eligible for inference.';
COMMENT ON COLUMN model_registry.created_at IS 'Row creation timestamp.';
COMMENT ON COLUMN model_registry.updated_at IS 'Row last update timestamp (mutable table).';

CREATE INDEX IF NOT EXISTS idx_model_registry_active ON model_registry (is_active);
CREATE INDEX IF NOT EXISTS idx_model_registry_owner_component_id ON model_registry (owner_component_id);

CREATE TABLE IF NOT EXISTS model_versions (
  model_version_id       uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  model_id               uuid NOT NULL REFERENCES model_registry(model_id) ON UPDATE RESTRICT ON DELETE RESTRICT,
  version                text NOT NULL,
  artifact_type          model_artifact_type NOT NULL,
  artifact_uri           text NOT NULL,
  artifact_sha256        bytea NOT NULL,
  trained_on             text NULL,
  training_data_hash     bytea NULL,
  hyperparameters        jsonb NULL,
  shap_enabled           boolean NOT NULL DEFAULT true,
  shap_artifact_uri      text NULL,
  shap_artifact_sha256   bytea NULL,
  metadata_json          jsonb NULL,
  created_at             timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT model_versions_uniq UNIQUE (model_id, version),
  CONSTRAINT model_versions_art_hash_len_chk CHECK (octet_length(artifact_sha256) = 32),
  CONSTRAINT model_versions_shap_hash_len_chk CHECK (shap_artifact_sha256 IS NULL OR octet_length(shap_artifact_sha256) = 32)
);

COMMENT ON TABLE model_versions IS
'Purpose: Immutable model version artifacts and metadata, including SHAP artifact linkage for explainability.\n'
'Writing module(s): AI Core (model registry, artifact validation).\n'
'Reading module(s): AI/ML pipeline, Validator, UI.\n'
'Retention expectation: long.';

COMMENT ON COLUMN model_versions.model_version_id IS 'Primary key. UUID for model version record.';
COMMENT ON COLUMN model_versions.model_id IS 'Foreign key to model_registry.model_id.';
COMMENT ON COLUMN model_versions.version IS 'Model version string (semantic or hash-like).';
COMMENT ON COLUMN model_versions.artifact_type IS 'Model artifact format.';
COMMENT ON COLUMN model_versions.artifact_uri IS 'Artifact location URI/path as stored by deployment.';
COMMENT ON COLUMN model_versions.artifact_sha256 IS 'SHA-256 digest of the model artifact.';
COMMENT ON COLUMN model_versions.trained_on IS 'Human-readable training dataset provenance string (optional).';
COMMENT ON COLUMN model_versions.training_data_hash IS 'Optional digest of training dataset for provenance.';
COMMENT ON COLUMN model_versions.hyperparameters IS 'Optional structured hyperparameters/config (JSONB justified).';
COMMENT ON COLUMN model_versions.shap_enabled IS 'Whether SHAP explainability is required/enabled for this model version.';
COMMENT ON COLUMN model_versions.shap_artifact_uri IS 'SHAP artifact location URI/path (required when shap_enabled is true by policy).';
COMMENT ON COLUMN model_versions.shap_artifact_sha256 IS 'SHA-256 digest of SHAP artifact (optional but recommended).';
COMMENT ON COLUMN model_versions.metadata_json IS 'Optional structured metadata (hashes, versions, training info) as JSONB.';
COMMENT ON COLUMN model_versions.created_at IS 'Row creation timestamp (append-only by convention).';

CREATE INDEX IF NOT EXISTS idx_model_versions_model_id ON model_versions (model_id);
CREATE INDEX IF NOT EXISTS idx_model_versions_created_at ON model_versions (created_at);

ALTER TABLE confidence_scores
  ADD CONSTRAINT confidence_scores_model_version_id_fk
  FOREIGN KEY (model_version_id)
  REFERENCES model_versions(model_version_id)
  ON UPDATE RESTRICT
  ON DELETE SET NULL
  DEFERRABLE INITIALLY DEFERRED;

CREATE TABLE IF NOT EXISTS inference_results (
  inference_id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  created_at             timestamptz NOT NULL DEFAULT now(),
  model_version_id       uuid NOT NULL REFERENCES model_versions(model_version_id) ON UPDATE RESTRICT ON DELETE RESTRICT,
  normalized_event_id    uuid NULL REFERENCES normalized_events(normalized_event_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  detection_id           uuid NULL REFERENCES detection_results(detection_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  input_features         jsonb NULL,
  input_features_sha256  bytea NULL,
  output_label           text NULL,
  output_score           double precision NULL,
  output_json            jsonb NULL,
  latency_ms             integer NULL,
  deterministic_key      bytea NOT NULL,
  CONSTRAINT inference_results_det_key_len_chk CHECK (octet_length(deterministic_key) IN (16, 20, 32, 64)),
  CONSTRAINT inference_results_feat_hash_len_chk CHECK (input_features_sha256 IS NULL OR octet_length(input_features_sha256) = 32)
);

COMMENT ON TABLE inference_results IS
'Purpose: Store model inference outputs with provenance (model version), inputs, and linkage to events/detections for explainable AI.\n'
'Writing module(s): AI/ML pipeline.\n'
'Reading module(s): Policy Engine, UI, Forensics, Validator.\n'
'Retention expectation: long.';

COMMENT ON COLUMN inference_results.inference_id IS 'Primary key. UUID for inference record.';
COMMENT ON COLUMN inference_results.created_at IS 'Inference time (server-side).';
COMMENT ON COLUMN inference_results.model_version_id IS 'FK to model_versions indicating the exact model used.';
COMMENT ON COLUMN inference_results.normalized_event_id IS 'Optional FK to normalized event used as inference context.';
COMMENT ON COLUMN inference_results.detection_id IS 'Optional FK to detection tied to inference output.';
COMMENT ON COLUMN inference_results.input_features IS 'Optional structured representation of model input features (JSONB justified).';
COMMENT ON COLUMN inference_results.input_features_sha256 IS 'Optional digest of canonicalized input features for integrity/deduplication.';
COMMENT ON COLUMN inference_results.output_label IS 'Optional predicted label/class.';
COMMENT ON COLUMN inference_results.output_score IS 'Optional predicted probability/score.';
COMMENT ON COLUMN inference_results.output_json IS 'Optional structured output (multi-label, embeddings metadata, etc.).';
COMMENT ON COLUMN inference_results.latency_ms IS 'Optional inference latency in milliseconds.';
COMMENT ON COLUMN inference_results.deterministic_key IS 'Deterministic key derived from model+input hash for deduplication.';

CREATE INDEX IF NOT EXISTS idx_inference_results_created_at ON inference_results (created_at);
CREATE INDEX IF NOT EXISTS idx_inference_results_model_version_id ON inference_results (model_version_id);
CREATE INDEX IF NOT EXISTS idx_inference_results_norm_event_id ON inference_results (normalized_event_id);
CREATE INDEX IF NOT EXISTS idx_inference_results_detection_id ON inference_results (detection_id);
CREATE INDEX IF NOT EXISTS idx_inference_results_det_key ON inference_results (deterministic_key);

CREATE TABLE IF NOT EXISTS shap_explanations (
  shap_id                uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  inference_id           uuid NOT NULL REFERENCES inference_results(inference_id) ON UPDATE RESTRICT ON DELETE RESTRICT,
  created_at             timestamptz NOT NULL DEFAULT now(),
  explainer_name         text NULL,
  explainer_version      text NULL,
  baseline_description   text NULL,
  expected_value         double precision NULL,
  shap_values            jsonb NULL,
  shap_summary           jsonb NULL,
  sha256                 bytea NULL,
  CONSTRAINT shap_explanations_hash_len_chk CHECK (sha256 IS NULL OR octet_length(sha256) = 32)
);

COMMENT ON TABLE shap_explanations IS
'Purpose: Store SHAP explainability artifacts for a given inference (per-inference explainability contract).\n'
'Writing module(s): AI/ML pipeline (explainability engine).\n'
'Reading module(s): UI, Validator, Forensics, Incident Summarizer.\n'
'Retention expectation: long.';

COMMENT ON COLUMN shap_explanations.shap_id IS 'Primary key. UUID for SHAP explanation.';
COMMENT ON COLUMN shap_explanations.inference_id IS 'FK to inference_results for which this explanation was generated.';
COMMENT ON COLUMN shap_explanations.created_at IS 'Creation timestamp.';
COMMENT ON COLUMN shap_explanations.explainer_name IS 'Name of explainer implementation (optional).';
COMMENT ON COLUMN shap_explanations.explainer_version IS 'Version of explainer implementation (optional).';
COMMENT ON COLUMN shap_explanations.baseline_description IS 'Description of baseline/background used for SHAP (optional).';
COMMENT ON COLUMN shap_explanations.expected_value IS 'Expected value/baseline prediction if provided by explainer (optional).';
COMMENT ON COLUMN shap_explanations.shap_values IS 'SHAP values payload (JSONB justified for array/map representation).';
COMMENT ON COLUMN shap_explanations.shap_summary IS 'Optional compact summary (top features, totals) (JSONB).';
COMMENT ON COLUMN shap_explanations.sha256 IS 'Optional digest of canonical SHAP payload for integrity.';

CREATE INDEX IF NOT EXISTS idx_shap_explanations_created_at ON shap_explanations (created_at);
CREATE INDEX IF NOT EXISTS idx_shap_explanations_inference_id ON shap_explanations (inference_id);

CREATE TABLE IF NOT EXISTS feature_contributions (
  feature_contribution_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  shap_id                uuid NOT NULL REFERENCES shap_explanations(shap_id) ON UPDATE RESTRICT ON DELETE RESTRICT,
  created_at             timestamptz NOT NULL DEFAULT now(),
  feature_name           text NOT NULL,
  feature_value          text NULL,
  contribution_value     double precision NOT NULL,
  rank                  integer NULL,
  direction             text NULL,
  CONSTRAINT feature_contributions_direction_chk CHECK (direction IS NULL OR direction IN ('positive','negative','neutral'))
);

COMMENT ON TABLE feature_contributions IS
'Purpose: Normalized, query-friendly per-feature contribution rows for explainability (top-k SHAP features).\n'
'Writing module(s): AI/ML pipeline (explainability materializer).\n'
'Reading module(s): UI, Validator, Forensics, Reporting.\n'
'Retention expectation: long.';

COMMENT ON COLUMN feature_contributions.feature_contribution_id IS 'Primary key. UUID for contribution record.';
COMMENT ON COLUMN feature_contributions.shap_id IS 'FK to shap_explanations.';
COMMENT ON COLUMN feature_contributions.created_at IS 'Creation timestamp.';
COMMENT ON COLUMN feature_contributions.feature_name IS 'Feature name.';
COMMENT ON COLUMN feature_contributions.feature_value IS 'Optional stringified feature value for reporting.';
COMMENT ON COLUMN feature_contributions.contribution_value IS 'Numeric contribution value (e.g., SHAP value).';
COMMENT ON COLUMN feature_contributions.rank IS 'Optional rank among contributions (1 = most important).';
COMMENT ON COLUMN feature_contributions.direction IS 'Optional contribution direction label.';

CREATE INDEX IF NOT EXISTS idx_feature_contributions_created_at ON feature_contributions (created_at);
CREATE INDEX IF NOT EXISTS idx_feature_contributions_shap_id ON feature_contributions (shap_id);
CREATE INDEX IF NOT EXISTS idx_feature_contributions_feature_name ON feature_contributions (feature_name);

CREATE TABLE IF NOT EXISTS llm_requests (
  llm_request_id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  created_at             timestamptz NOT NULL DEFAULT now(),
  requester_component_id uuid NULL REFERENCES components(component_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  requester_agent_id     uuid NULL REFERENCES agents(agent_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  incident_id            uuid NULL,
  correlation_run_id     uuid NULL,
  model_name             text NOT NULL,
  model_version          text NULL,
  purpose                text NULL,
  prompt_template_id     text NULL,
  prompt_messages        jsonb NOT NULL,
  prompt_sha256          bytea NOT NULL,
  max_tokens             integer NULL,
  temperature            double precision NULL,
  top_p                  double precision NULL,
  stop_sequences         jsonb NULL,
  tool_spec              jsonb NULL,
  context_refs           jsonb NULL,
  offline_policy         text NULL,
  CONSTRAINT llm_requests_prompt_sha256_len_chk CHECK (octet_length(prompt_sha256) = 32)
);

COMMENT ON TABLE llm_requests IS
'Purpose: Store LLM request prompts and parameters for deterministic replay and auditability.\n'
'Writing module(s): LLM module, SOC Copilot (Assistant), Incident Summarizer.\n'
'Reading module(s): Forensics, Validator, UI, Audit.\n'
'Retention expectation: long.';

COMMENT ON COLUMN llm_requests.llm_request_id IS 'Primary key. UUID for LLM request.';
COMMENT ON COLUMN llm_requests.created_at IS 'Request creation timestamp.';
COMMENT ON COLUMN llm_requests.requester_component_id IS 'Optional FK to components for service-originated requests.';
COMMENT ON COLUMN llm_requests.requester_agent_id IS 'Optional FK to agents for agent-originated requests.';
COMMENT ON COLUMN llm_requests.incident_id IS 'Optional incident identifier (UUID chosen by incident pipeline).';
COMMENT ON COLUMN llm_requests.correlation_run_id IS 'Optional correlation batch identifier associated with the request.';
COMMENT ON COLUMN llm_requests.model_name IS 'LLM model name identifier.';
COMMENT ON COLUMN llm_requests.model_version IS 'Optional model version tag (if distinct from name).';
COMMENT ON COLUMN llm_requests.purpose IS 'Optional purpose label (summarize, triage, explain, recommend).';
COMMENT ON COLUMN llm_requests.prompt_template_id IS 'Optional template identifier used to produce prompt.';
COMMENT ON COLUMN llm_requests.prompt_messages IS 'Structured message list (role/content/tool) (JSONB justified).';
COMMENT ON COLUMN llm_requests.prompt_sha256 IS 'SHA-256 digest of canonical prompt for integrity and deduplication.';
COMMENT ON COLUMN llm_requests.max_tokens IS 'Optional token budget parameter.';
COMMENT ON COLUMN llm_requests.temperature IS 'Optional temperature parameter.';
COMMENT ON COLUMN llm_requests.top_p IS 'Optional nucleus sampling parameter.';
COMMENT ON COLUMN llm_requests.stop_sequences IS 'Optional stop sequences list (JSONB).';
COMMENT ON COLUMN llm_requests.tool_spec IS 'Optional tool/function-call spec (JSONB).';
COMMENT ON COLUMN llm_requests.context_refs IS 'Optional references to evidence/artifacts used as context (JSONB).';
COMMENT ON COLUMN llm_requests.offline_policy IS 'Optional label describing offline/air-gapped constraints enforced for this request.';

CREATE INDEX IF NOT EXISTS idx_llm_requests_created_at ON llm_requests (created_at);
CREATE INDEX IF NOT EXISTS idx_llm_requests_requester_component_id ON llm_requests (requester_component_id);
CREATE INDEX IF NOT EXISTS idx_llm_requests_requester_agent_id ON llm_requests (requester_agent_id);
CREATE INDEX IF NOT EXISTS idx_llm_requests_prompt_sha256 ON llm_requests (prompt_sha256);

CREATE TABLE IF NOT EXISTS llm_responses (
  llm_response_id        uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  llm_request_id         uuid NOT NULL REFERENCES llm_requests(llm_request_id) ON UPDATE RESTRICT ON DELETE RESTRICT,
  created_at             timestamptz NOT NULL DEFAULT now(),
  provider_name          text NULL,
  runtime_name           text NULL,
  finish_reason          text NULL,
  response_messages      jsonb NOT NULL,
  response_sha256        bytea NOT NULL,
  usage_json             jsonb NULL,
  safety_json            jsonb NULL,
  latency_ms             integer NULL,
  CONSTRAINT llm_responses_sha256_len_chk CHECK (octet_length(response_sha256) = 32)
);

COMMENT ON TABLE llm_responses IS
'Purpose: Store LLM outputs for reproducibility, audit, and forensic replay.\n'
'Writing module(s): LLM module, SOC Copilot (Assistant).\n'
'Reading module(s): UI, Forensics, Validator, Audit.\n'
'Retention expectation: long.';

COMMENT ON COLUMN llm_responses.llm_response_id IS 'Primary key. UUID for LLM response.';
COMMENT ON COLUMN llm_responses.llm_request_id IS 'FK to llm_requests.';
COMMENT ON COLUMN llm_responses.created_at IS 'Response creation timestamp.';
COMMENT ON COLUMN llm_responses.provider_name IS 'Optional provider label (local runtime, on-prem appliance, etc.).';
COMMENT ON COLUMN llm_responses.runtime_name IS 'Optional runtime/engine identifier.';
COMMENT ON COLUMN llm_responses.finish_reason IS 'Optional finish reason from runtime (stop, length, tool_call).';
COMMENT ON COLUMN llm_responses.response_messages IS 'Structured response message list (JSONB justified).';
COMMENT ON COLUMN llm_responses.response_sha256 IS 'SHA-256 digest of canonical response for integrity and deduplication.';
COMMENT ON COLUMN llm_responses.usage_json IS 'Optional usage metrics payload (JSONB).';
COMMENT ON COLUMN llm_responses.safety_json IS 'Optional safety/filtering metadata (JSONB).';
COMMENT ON COLUMN llm_responses.latency_ms IS 'Optional end-to-end latency in milliseconds.';

CREATE INDEX IF NOT EXISTS idx_llm_responses_created_at ON llm_responses (created_at);
CREATE INDEX IF NOT EXISTS idx_llm_responses_request_id ON llm_responses (llm_request_id);
CREATE INDEX IF NOT EXISTS idx_llm_responses_sha256 ON llm_responses (response_sha256);

-- ============================================================================
-- D. Policy & Enforcement (REQUIRED)
-- ============================================================================

CREATE TABLE IF NOT EXISTS policy_evaluations (
  policy_evaluation_id   uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  created_at             timestamptz NOT NULL DEFAULT now(),
  evaluated_at           timestamptz NOT NULL DEFAULT now(),
  policy_version_id      uuid NOT NULL REFERENCES policy_versions(policy_version_id) ON UPDATE RESTRICT ON DELETE RESTRICT,
  normalized_event_id    uuid NULL REFERENCES normalized_events(normalized_event_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  detection_id           uuid NULL REFERENCES detection_results(detection_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  agent_id               uuid NULL REFERENCES agents(agent_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  component_id           uuid NULL REFERENCES components(component_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  evaluation_result      jsonb NOT NULL,
  matched_rules          jsonb NULL,
  evaluation_sha256      bytea NULL,
  deterministic_key      bytea NOT NULL,
  CONSTRAINT policy_eval_hash_len_chk CHECK (evaluation_sha256 IS NULL OR octet_length(evaluation_sha256) = 32),
  CONSTRAINT policy_eval_det_key_len_chk CHECK (octet_length(deterministic_key) IN (16, 20, 32, 64))
);

COMMENT ON TABLE policy_evaluations IS
'Purpose: Record policy evaluations against events/detections with rule-match details for enforcement traceability.\n'
'Writing module(s): Policy Engine.\n'
'Reading module(s): Enforcement, UI, Forensics, Validator.\n'
'Retention expectation: long.';

COMMENT ON COLUMN policy_evaluations.policy_evaluation_id IS 'Primary key. UUID for evaluation record.';
COMMENT ON COLUMN policy_evaluations.created_at IS 'Creation timestamp.';
COMMENT ON COLUMN policy_evaluations.evaluated_at IS 'Evaluation time.';
COMMENT ON COLUMN policy_evaluations.policy_version_id IS 'FK to policy_versions.';
COMMENT ON COLUMN policy_evaluations.normalized_event_id IS 'Optional FK to normalized event evaluated.';
COMMENT ON COLUMN policy_evaluations.detection_id IS 'Optional FK to detection evaluated.';
COMMENT ON COLUMN policy_evaluations.agent_id IS 'Optional FK to agent context.';
COMMENT ON COLUMN policy_evaluations.component_id IS 'Optional FK to component context.';
COMMENT ON COLUMN policy_evaluations.evaluation_result IS 'Structured evaluation output (decision inputs, matches) (JSONB justified).';
COMMENT ON COLUMN policy_evaluations.matched_rules IS 'Optional list/detail of matched rules (JSONB).';
COMMENT ON COLUMN policy_evaluations.evaluation_sha256 IS 'Optional digest of canonical evaluation output for integrity.';
COMMENT ON COLUMN policy_evaluations.deterministic_key IS 'Deterministic key derived from policy_version + target for deduplication.';

CREATE INDEX IF NOT EXISTS idx_policy_evaluations_created_at ON policy_evaluations (created_at);
CREATE INDEX IF NOT EXISTS idx_policy_evaluations_evaluated_at ON policy_evaluations (evaluated_at);
CREATE INDEX IF NOT EXISTS idx_policy_evaluations_policy_version_id ON policy_evaluations (policy_version_id);
CREATE INDEX IF NOT EXISTS idx_policy_evaluations_norm_event_id ON policy_evaluations (normalized_event_id);
CREATE INDEX IF NOT EXISTS idx_policy_evaluations_detection_id ON policy_evaluations (detection_id);
CREATE INDEX IF NOT EXISTS idx_policy_evaluations_agent_id ON policy_evaluations (agent_id);
CREATE INDEX IF NOT EXISTS idx_policy_evaluations_component_id ON policy_evaluations (component_id);
CREATE INDEX IF NOT EXISTS idx_policy_evaluations_det_key ON policy_evaluations (deterministic_key);

CREATE TABLE IF NOT EXISTS enforcement_decisions (
  enforcement_decision_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  created_at             timestamptz NOT NULL DEFAULT now(),
  decided_at             timestamptz NOT NULL DEFAULT now(),
  policy_evaluation_id   uuid NOT NULL REFERENCES policy_evaluations(policy_evaluation_id) ON UPDATE RESTRICT ON DELETE RESTRICT,
  decision               enforcement_decision_type NOT NULL,
  decision_reason        text NULL,
  target_agent_id        uuid NULL REFERENCES agents(agent_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  target_entity_id       uuid NULL REFERENCES entities(entity_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  response_playbook_id   uuid NULL,
  decision_payload       jsonb NULL,
  decision_sha256        bytea NULL,
  signature_status       signature_status NOT NULL DEFAULT 'unknown',
  signed_by              text NULL,
  signature_alg          text NULL,
  signature_b64          text NULL,
  deterministic_key      bytea NOT NULL,
  CONSTRAINT enforcement_decisions_hash_len_chk CHECK (decision_sha256 IS NULL OR octet_length(decision_sha256) = 32),
  CONSTRAINT enforcement_decisions_det_key_len_chk CHECK (octet_length(deterministic_key) IN (16, 20, 32, 64))
);

COMMENT ON TABLE enforcement_decisions IS
'Purpose: Store final enforcement decisions derived from policy evaluations, including signatures for trust and replay.\n'
'Writing module(s): Policy Engine, Response Engine (approval/signature).\n'
'Reading module(s): Agents, UI, Forensics, Validator, Audit.\n'
'Retention expectation: long.';

COMMENT ON COLUMN enforcement_decisions.enforcement_decision_id IS 'Primary key. UUID for enforcement decision.';
COMMENT ON COLUMN enforcement_decisions.created_at IS 'Creation timestamp.';
COMMENT ON COLUMN enforcement_decisions.decided_at IS 'Decision time.';
COMMENT ON COLUMN enforcement_decisions.policy_evaluation_id IS 'FK to policy_evaluations.';
COMMENT ON COLUMN enforcement_decisions.decision IS 'Decision type (allow/block/quarantine/etc.).';
COMMENT ON COLUMN enforcement_decisions.decision_reason IS 'Optional human-readable reason for decision.';
COMMENT ON COLUMN enforcement_decisions.target_agent_id IS 'Optional FK to agent targeted by decision.';
COMMENT ON COLUMN enforcement_decisions.target_entity_id IS 'Optional FK to entity targeted by decision.';
COMMENT ON COLUMN enforcement_decisions.response_playbook_id IS 'Optional playbook UUID selected for response/execution.';
COMMENT ON COLUMN enforcement_decisions.decision_payload IS 'Optional structured payload containing parameters for enforcement (JSONB).';
COMMENT ON COLUMN enforcement_decisions.decision_sha256 IS 'Optional digest of canonical decision payload for integrity.';
COMMENT ON COLUMN enforcement_decisions.signature_status IS 'Validation status of decision signature.';
COMMENT ON COLUMN enforcement_decisions.signed_by IS 'Signer identity (key id/cert subject/principal).';
COMMENT ON COLUMN enforcement_decisions.signature_alg IS 'Signature algorithm identifier.';
COMMENT ON COLUMN enforcement_decisions.signature_b64 IS 'Base64-encoded signature over decision payload/hash.';
COMMENT ON COLUMN enforcement_decisions.deterministic_key IS 'Deterministic key derived from evaluation+decision for deduplication.';

CREATE INDEX IF NOT EXISTS idx_enforcement_decisions_created_at ON enforcement_decisions (created_at);
CREATE INDEX IF NOT EXISTS idx_enforcement_decisions_decided_at ON enforcement_decisions (decided_at);
CREATE INDEX IF NOT EXISTS idx_enforcement_decisions_eval_id ON enforcement_decisions (policy_evaluation_id);
CREATE INDEX IF NOT EXISTS idx_enforcement_decisions_target_agent_id ON enforcement_decisions (target_agent_id);
CREATE INDEX IF NOT EXISTS idx_enforcement_decisions_target_entity_id ON enforcement_decisions (target_entity_id);
CREATE INDEX IF NOT EXISTS idx_enforcement_decisions_det_key ON enforcement_decisions (deterministic_key);

CREATE TABLE IF NOT EXISTS actions_taken (
  action_id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  created_at             timestamptz NOT NULL DEFAULT now(),
  started_at             timestamptz NULL,
  completed_at           timestamptz NULL,
  enforcement_decision_id uuid NOT NULL REFERENCES enforcement_decisions(enforcement_decision_id) ON UPDATE RESTRICT ON DELETE RESTRICT,
  actor_component_id     uuid NULL REFERENCES components(component_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  target_agent_id        uuid NULL REFERENCES agents(agent_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  action_type            text NOT NULL,
  action_parameters      jsonb NULL,
  action_status          action_status_type NOT NULL DEFAULT 'requested',
  status_details         text NULL,
  result_payload         jsonb NULL,
  result_sha256          bytea NULL,
  deterministic_key      bytea NOT NULL,
  CONSTRAINT actions_taken_hash_len_chk CHECK (result_sha256 IS NULL OR octet_length(result_sha256) = 32),
  CONSTRAINT actions_taken_det_key_len_chk CHECK (octet_length(deterministic_key) IN (16, 20, 32, 64))
);

COMMENT ON TABLE actions_taken IS
'Purpose: Record concrete response/enforcement actions executed (or attempted) for a given enforcement decision.\n'
'Writing module(s): Response Engine, Agents (ack/status), Orchestrator.\n'
'Reading module(s): UI, Forensics, Validator, Audit.\n'
'Retention expectation: long.';

COMMENT ON COLUMN actions_taken.action_id IS 'Primary key. UUID for action.';
COMMENT ON COLUMN actions_taken.created_at IS 'Creation timestamp.';
COMMENT ON COLUMN actions_taken.started_at IS 'Action start time (optional).';
COMMENT ON COLUMN actions_taken.completed_at IS 'Action completion time (optional).';
COMMENT ON COLUMN actions_taken.enforcement_decision_id IS 'FK to enforcement_decisions.';
COMMENT ON COLUMN actions_taken.actor_component_id IS 'Optional FK to component executing action.';
COMMENT ON COLUMN actions_taken.target_agent_id IS 'Optional FK to agent where action executed.';
COMMENT ON COLUMN actions_taken.action_type IS 'Action type identifier (e.g., kill_process, isolate_host, block_ip).';
COMMENT ON COLUMN actions_taken.action_parameters IS 'Optional structured parameters for action execution (JSONB).';
COMMENT ON COLUMN actions_taken.action_status IS 'Current lifecycle status of action.';
COMMENT ON COLUMN actions_taken.status_details IS 'Optional detail message (errors, notes).';
COMMENT ON COLUMN actions_taken.result_payload IS 'Optional structured result payload (JSONB).';
COMMENT ON COLUMN actions_taken.result_sha256 IS 'Optional digest of canonical result payload for integrity.';
COMMENT ON COLUMN actions_taken.deterministic_key IS 'Deterministic key derived from decision+action_type for deduplication.';

CREATE INDEX IF NOT EXISTS idx_actions_taken_created_at ON actions_taken (created_at);
CREATE INDEX IF NOT EXISTS idx_actions_taken_enforcement_decision_id ON actions_taken (enforcement_decision_id);
CREATE INDEX IF NOT EXISTS idx_actions_taken_actor_component_id ON actions_taken (actor_component_id);
CREATE INDEX IF NOT EXISTS idx_actions_taken_target_agent_id ON actions_taken (target_agent_id);
CREATE INDEX IF NOT EXISTS idx_actions_taken_status ON actions_taken (action_status);
CREATE INDEX IF NOT EXISTS idx_actions_taken_det_key ON actions_taken (deterministic_key);

-- ============================================================================
-- F. Audit & Forensics (REQUIRED) - immutable/append-only enforced
-- ============================================================================

CREATE TABLE IF NOT EXISTS immutable_audit_log (
  audit_id               uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  created_at             timestamptz NOT NULL DEFAULT now(),
  actor_component_id     uuid NULL REFERENCES components(component_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  actor_agent_id         uuid NULL REFERENCES agents(agent_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  action                text NOT NULL,
  object_type           trust_object_type NOT NULL DEFAULT 'other',
  object_id             uuid NULL,
  event_time            timestamptz NULL,
  payload_json           jsonb NULL,
  payload_sha256         bytea NOT NULL,
  prev_audit_id          uuid NULL,
  prev_payload_sha256    bytea NULL,
  chain_hash_sha256      bytea NOT NULL,
  signature_status       signature_status NOT NULL DEFAULT 'unknown',
  signed_by              text NULL,
  signature_alg          text NULL,
  signature_b64          text NULL,
  CONSTRAINT immutable_audit_payload_sha256_len_chk CHECK (octet_length(payload_sha256) = 32),
  CONSTRAINT immutable_audit_chain_hash_len_chk CHECK (octet_length(chain_hash_sha256) = 32),
  CONSTRAINT immutable_audit_prev_hash_len_chk CHECK (prev_payload_sha256 IS NULL OR octet_length(prev_payload_sha256) = 32)
);

COMMENT ON TABLE immutable_audit_log IS
'Purpose: Tamper-evident append-only audit log with hash chaining and optional signatures.\n'
'Writing module(s): All components (Core, Agents, AI, LLM, Policy, Response, Validator).\n'
'Reading module(s): Forensics, Trust Verification, Validator, UI.\n'
'Retention expectation: immutable.';

COMMENT ON COLUMN immutable_audit_log.audit_id IS 'Primary key. UUID for audit entry.';
COMMENT ON COLUMN immutable_audit_log.created_at IS 'Audit entry creation time.';
COMMENT ON COLUMN immutable_audit_log.actor_component_id IS 'Optional FK to component producing the audit entry.';
COMMENT ON COLUMN immutable_audit_log.actor_agent_id IS 'Optional FK to agent producing the audit entry.';
COMMENT ON COLUMN immutable_audit_log.action IS 'Audit action label (e.g., insert_raw_event, validate_signature, enforce_block).';
COMMENT ON COLUMN immutable_audit_log.object_type IS 'Type of referenced object (raw_event, policy_version, model_version, etc.).';
COMMENT ON COLUMN immutable_audit_log.object_id IS 'Optional referenced object UUID.';
COMMENT ON COLUMN immutable_audit_log.event_time IS 'Optional time of the underlying action/event being logged.';
COMMENT ON COLUMN immutable_audit_log.payload_json IS 'Structured audit payload (JSONB justified for varied audit event shape).';
COMMENT ON COLUMN immutable_audit_log.payload_sha256 IS 'SHA-256 digest of canonical audit payload for integrity.';
COMMENT ON COLUMN immutable_audit_log.prev_audit_id IS 'Previous audit entry id for hash chain linkage (optional for chain start).';
COMMENT ON COLUMN immutable_audit_log.prev_payload_sha256 IS 'SHA-256 digest of previous audit payload (optional for chain start).';
COMMENT ON COLUMN immutable_audit_log.chain_hash_sha256 IS 'SHA-256 chain hash computed over (prev_chain_hash, payload_sha256, metadata).';
COMMENT ON COLUMN immutable_audit_log.signature_status IS 'Validation status for optional signature over payload/chain hash.';
COMMENT ON COLUMN immutable_audit_log.signed_by IS 'Signer identity for audit entry (optional).';
COMMENT ON COLUMN immutable_audit_log.signature_alg IS 'Signature algorithm identifier (optional).';
COMMENT ON COLUMN immutable_audit_log.signature_b64 IS 'Base64 signature for audit entry (optional).';

CREATE INDEX IF NOT EXISTS idx_immutable_audit_created_at ON immutable_audit_log (created_at);
CREATE INDEX IF NOT EXISTS idx_immutable_audit_object ON immutable_audit_log (object_type, object_id);
CREATE INDEX IF NOT EXISTS idx_immutable_audit_actor_component ON immutable_audit_log (actor_component_id);
CREATE INDEX IF NOT EXISTS idx_immutable_audit_actor_agent ON immutable_audit_log (actor_agent_id);

CREATE TRIGGER trg_immutable_audit_no_update
BEFORE UPDATE OR DELETE ON immutable_audit_log
FOR EACH ROW EXECUTE FUNCTION prevent_update_delete();

CREATE TABLE IF NOT EXISTS trust_verification_records (
  trust_record_id        uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  created_at             timestamptz NOT NULL DEFAULT now(),
  verifier_component_id  uuid NULL REFERENCES components(component_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  object_type            trust_object_type NOT NULL,
  object_id              uuid NOT NULL,
  verification_time      timestamptz NOT NULL DEFAULT now(),
  verification_method    text NOT NULL,
  expected_sha256        bytea NULL,
  observed_sha256        bytea NULL,
  signature_status       signature_status NOT NULL,
  signer_identity        text NULL,
  cert_fingerprint_sha256 bytea NULL,
  details_json           jsonb NULL,
  immutable_audit_id     uuid NULL REFERENCES immutable_audit_log(audit_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  CONSTRAINT trust_verif_expected_len_chk CHECK (expected_sha256 IS NULL OR octet_length(expected_sha256) = 32),
  CONSTRAINT trust_verif_observed_len_chk CHECK (observed_sha256 IS NULL OR octet_length(observed_sha256) = 32),
  CONSTRAINT trust_verif_cert_fp_len_chk CHECK (cert_fingerprint_sha256 IS NULL OR octet_length(cert_fingerprint_sha256) = 32)
);

COMMENT ON TABLE trust_verification_records IS
'Purpose: Append-only records of integrity and trust verification checks across objects (hash/signature/cert-chain).\n'
'Writing module(s): Trust & Verification, Global Validator.\n'
'Reading module(s): Forensics, UI, Audit.\n'
'Retention expectation: immutable.';

COMMENT ON COLUMN trust_verification_records.trust_record_id IS 'Primary key. UUID for trust verification record.';
COMMENT ON COLUMN trust_verification_records.created_at IS 'Creation timestamp.';
COMMENT ON COLUMN trust_verification_records.verifier_component_id IS 'Optional FK to component performing verification.';
COMMENT ON COLUMN trust_verification_records.object_type IS 'Object type being verified.';
COMMENT ON COLUMN trust_verification_records.object_id IS 'UUID of the object being verified.';
COMMENT ON COLUMN trust_verification_records.verification_time IS 'Timestamp when verification occurred.';
COMMENT ON COLUMN trust_verification_records.verification_method IS 'Method label (hash_check, signature_check, cert_chain_check).';
COMMENT ON COLUMN trust_verification_records.expected_sha256 IS 'Optional expected SHA-256 digest for integrity verification.';
COMMENT ON COLUMN trust_verification_records.observed_sha256 IS 'Optional observed SHA-256 digest computed at verification time.';
COMMENT ON COLUMN trust_verification_records.signature_status IS 'Signature/trust status outcome.';
COMMENT ON COLUMN trust_verification_records.signer_identity IS 'Signer identity extracted from signature/cert if available.';
COMMENT ON COLUMN trust_verification_records.cert_fingerprint_sha256 IS 'Optional cert fingerprint digest for signer certificate.';
COMMENT ON COLUMN trust_verification_records.details_json IS 'Optional structured verification details (JSONB justified).';
COMMENT ON COLUMN trust_verification_records.immutable_audit_id IS 'Optional FK to immutable_audit_log entry associated with this verification.';

CREATE INDEX IF NOT EXISTS idx_trust_verif_created_at ON trust_verification_records (created_at);
CREATE INDEX IF NOT EXISTS idx_trust_verif_object ON trust_verification_records (object_type, object_id);
CREATE INDEX IF NOT EXISTS idx_trust_verif_verifier_component ON trust_verification_records (verifier_component_id);
CREATE INDEX IF NOT EXISTS idx_trust_verif_audit_id ON trust_verification_records (immutable_audit_id);

CREATE TRIGGER trg_trust_verif_no_update
BEFORE UPDATE OR DELETE ON trust_verification_records
FOR EACH ROW EXECUTE FUNCTION prevent_update_delete();

CREATE TABLE IF NOT EXISTS signature_validation_events (
  signature_event_id     uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  created_at             timestamptz NOT NULL DEFAULT now(),
  validator_component_id uuid NULL REFERENCES components(component_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  object_type            trust_object_type NOT NULL,
  object_id              uuid NOT NULL,
  signature_alg          text NULL,
  signature_b64          text NULL,
  signature_status       signature_status NOT NULL,
  signer_identity        text NULL,
  cert_chain_json        jsonb NULL,
  error_details          text NULL,
  immutable_audit_id     uuid NULL REFERENCES immutable_audit_log(audit_id) ON UPDATE RESTRICT ON DELETE SET NULL
);

COMMENT ON TABLE signature_validation_events IS
'Purpose: Append-only signature validation events capturing validation results and certificate chain context.\n'
'Writing module(s): Trust & Verification, Policy Engine (policy sig), Response Engine (decision sig), Global Validator.\n'
'Reading module(s): Forensics, UI, Audit.\n'
'Retention expectation: immutable.';

COMMENT ON COLUMN signature_validation_events.signature_event_id IS 'Primary key. UUID for signature validation event.';
COMMENT ON COLUMN signature_validation_events.created_at IS 'Creation timestamp.';
COMMENT ON COLUMN signature_validation_events.validator_component_id IS 'Optional FK to component that performed validation.';
COMMENT ON COLUMN signature_validation_events.object_type IS 'Type of object whose signature was validated.';
COMMENT ON COLUMN signature_validation_events.object_id IS 'UUID of the object whose signature was validated.';
COMMENT ON COLUMN signature_validation_events.signature_alg IS 'Signature algorithm identifier (optional).';
COMMENT ON COLUMN signature_validation_events.signature_b64 IS 'Base64 signature (optional).';
COMMENT ON COLUMN signature_validation_events.signature_status IS 'Validation outcome status.';
COMMENT ON COLUMN signature_validation_events.signer_identity IS 'Signer identity extracted from signature/cert if available.';
COMMENT ON COLUMN signature_validation_events.cert_chain_json IS 'Optional certificate chain context (JSONB justified).';
COMMENT ON COLUMN signature_validation_events.error_details IS 'Optional error details if validation failed.';
COMMENT ON COLUMN signature_validation_events.immutable_audit_id IS 'Optional FK to immutable_audit_log entry for this validation.';

CREATE INDEX IF NOT EXISTS idx_sig_validation_created_at ON signature_validation_events (created_at);
CREATE INDEX IF NOT EXISTS idx_sig_validation_object ON signature_validation_events (object_type, object_id);
CREATE INDEX IF NOT EXISTS idx_sig_validation_validator_component ON signature_validation_events (validator_component_id);
CREATE INDEX IF NOT EXISTS idx_sig_validation_audit_id ON signature_validation_events (immutable_audit_id);

CREATE TRIGGER trg_sig_validation_no_update
BEFORE UPDATE OR DELETE ON signature_validation_events
FOR EACH ROW EXECUTE FUNCTION prevent_update_delete();

-- ============================================================================
-- G. System Health & Ops (REQUIRED)
-- ============================================================================

CREATE TABLE IF NOT EXISTS component_health (
  health_id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  created_at             timestamptz NOT NULL DEFAULT now(),
  component_id           uuid NOT NULL REFERENCES components(component_id) ON UPDATE RESTRICT ON DELETE RESTRICT,
  observed_at            timestamptz NOT NULL DEFAULT now(),
  status                 text NOT NULL,
  status_details         text NULL,
  metrics_json           jsonb NULL,
  CONSTRAINT component_health_status_chk CHECK (status IN ('healthy','degraded','unhealthy','unknown'))
);

COMMENT ON TABLE component_health IS
'Purpose: Time-series component health snapshots used for ops monitoring and validation.\n'
'Writing module(s): All components (heartbeat reporter), Master/Core.\n'
'Reading module(s): UI, Validator, Ops.\n'
'Retention expectation: short.';

COMMENT ON COLUMN component_health.health_id IS 'Primary key. UUID for health snapshot.';
COMMENT ON COLUMN component_health.created_at IS 'Creation timestamp.';
COMMENT ON COLUMN component_health.component_id IS 'FK to components.';
COMMENT ON COLUMN component_health.observed_at IS 'Timestamp when health observation was made.';
COMMENT ON COLUMN component_health.status IS 'Health status (healthy/degraded/unhealthy/unknown).';
COMMENT ON COLUMN component_health.status_details IS 'Optional details explaining status.';
COMMENT ON COLUMN component_health.metrics_json IS 'Optional health metrics (CPU/mem/queue depth) as JSONB.';

CREATE INDEX IF NOT EXISTS idx_component_health_observed_at ON component_health (observed_at);
CREATE INDEX IF NOT EXISTS idx_component_health_component_id ON component_health (component_id);

CREATE TABLE IF NOT EXISTS startup_events (
  startup_event_id       uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  created_at             timestamptz NOT NULL DEFAULT now(),
  component_id           uuid NOT NULL REFERENCES components(component_id) ON UPDATE RESTRICT ON DELETE RESTRICT,
  started_at             timestamptz NOT NULL,
  boot_reason            text NULL,
  config_sha256          bytea NULL,
  build_hash             text NULL,
  version                text NULL,
  env_fingerprint_sha256 bytea NULL,
  details_json           jsonb NULL,
  CONSTRAINT startup_events_cfg_hash_len_chk CHECK (config_sha256 IS NULL OR octet_length(config_sha256) = 32),
  CONSTRAINT startup_events_env_hash_len_chk CHECK (env_fingerprint_sha256 IS NULL OR octet_length(env_fingerprint_sha256) = 32)
);

COMMENT ON TABLE startup_events IS
'Purpose: Append-only service startup records for uptime, provenance, and forensic correlation.\n'
'Writing module(s): All components at startup.\n'
'Reading module(s): UI, Validator, Forensics.\n'
'Retention expectation: long.';

COMMENT ON COLUMN startup_events.startup_event_id IS 'Primary key. UUID for startup event.';
COMMENT ON COLUMN startup_events.created_at IS 'Creation timestamp.';
COMMENT ON COLUMN startup_events.component_id IS 'FK to components.';
COMMENT ON COLUMN startup_events.started_at IS 'Timestamp when component started.';
COMMENT ON COLUMN startup_events.boot_reason IS 'Optional reason for startup (boot, restart, deploy, crash-recovery).';
COMMENT ON COLUMN startup_events.config_sha256 IS 'Optional digest of effective config used at startup.';
COMMENT ON COLUMN startup_events.build_hash IS 'Optional build hash recorded at startup.';
COMMENT ON COLUMN startup_events.version IS 'Optional version recorded at startup.';
COMMENT ON COLUMN startup_events.env_fingerprint_sha256 IS 'Optional digest of environment variable fingerprint (no secrets, hashed).';
COMMENT ON COLUMN startup_events.details_json IS 'Optional structured details about startup context (JSONB).';

CREATE INDEX IF NOT EXISTS idx_startup_events_created_at ON startup_events (created_at);
CREATE INDEX IF NOT EXISTS idx_startup_events_started_at ON startup_events (started_at);
CREATE INDEX IF NOT EXISTS idx_startup_events_component_id ON startup_events (component_id);

CREATE TABLE IF NOT EXISTS error_events (
  error_event_id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  created_at             timestamptz NOT NULL DEFAULT now(),
  component_id           uuid NULL REFERENCES components(component_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  agent_id               uuid NULL REFERENCES agents(agent_id) ON UPDATE RESTRICT ON DELETE SET NULL,
  observed_at            timestamptz NOT NULL DEFAULT now(),
  severity               severity_level NOT NULL DEFAULT 'error',
  error_type             text NOT NULL,
  error_message          text NOT NULL,
  stacktrace             text NULL,
  context_json           jsonb NULL,
  trace_id               text NULL,
  correlation_hint       text NULL
);

COMMENT ON TABLE error_events IS
'Purpose: Centralized error/event log for components and agents to support triage and forensic replay.\n'
'Writing module(s): All components and agents.\n'
'Reading module(s): UI, Validator, Ops.\n'
'Retention expectation: short.';

COMMENT ON COLUMN error_events.error_event_id IS 'Primary key. UUID for error event.';
COMMENT ON COLUMN error_events.created_at IS 'Creation timestamp.';
COMMENT ON COLUMN error_events.component_id IS 'Optional FK to components (service error).';
COMMENT ON COLUMN error_events.agent_id IS 'Optional FK to agents (agent/probe error).';
COMMENT ON COLUMN error_events.observed_at IS 'Timestamp when error occurred.';
COMMENT ON COLUMN error_events.severity IS 'Severity level for error event.';
COMMENT ON COLUMN error_events.error_type IS 'Error classification/type label.';
COMMENT ON COLUMN error_events.error_message IS 'Error message summary.';
COMMENT ON COLUMN error_events.stacktrace IS 'Optional stacktrace text.';
COMMENT ON COLUMN error_events.context_json IS 'Optional structured context (JSONB justified).';
COMMENT ON COLUMN error_events.trace_id IS 'Optional distributed trace id.';
COMMENT ON COLUMN error_events.correlation_hint IS 'Optional correlation hint key.';

CREATE INDEX IF NOT EXISTS idx_error_events_created_at ON error_events (created_at);
CREATE INDEX IF NOT EXISTS idx_error_events_observed_at ON error_events (observed_at);
CREATE INDEX IF NOT EXISTS idx_error_events_component_id ON error_events (component_id);
CREATE INDEX IF NOT EXISTS idx_error_events_agent_id ON error_events (agent_id);

-- ============================================================================
-- Ownership & Grants (read/write ownership contract)
-- ============================================================================

ALTER TABLE agents OWNER TO ransomeye_owner;
ALTER TABLE components OWNER TO ransomeye_owner;
ALTER TABLE entities OWNER TO ransomeye_owner;
ALTER TABLE policies OWNER TO ransomeye_owner;
ALTER TABLE policy_versions OWNER TO ransomeye_owner;
ALTER TABLE linux_agent_telemetry OWNER TO ransomeye_owner;
ALTER TABLE windows_agent_telemetry OWNER TO ransomeye_owner;
ALTER TABLE dpi_probe_telemetry OWNER TO ransomeye_owner;
ALTER TABLE retention_policies OWNER TO ransomeye_owner;
ALTER TABLE raw_events OWNER TO ransomeye_owner;
ALTER TABLE normalized_events OWNER TO ransomeye_owner;
ALTER TABLE correlation_graph OWNER TO ransomeye_owner;
ALTER TABLE detection_results OWNER TO ransomeye_owner;
ALTER TABLE confidence_scores OWNER TO ransomeye_owner;
ALTER TABLE model_registry OWNER TO ransomeye_owner;
ALTER TABLE model_versions OWNER TO ransomeye_owner;
ALTER TABLE inference_results OWNER TO ransomeye_owner;
ALTER TABLE shap_explanations OWNER TO ransomeye_owner;
ALTER TABLE feature_contributions OWNER TO ransomeye_owner;
ALTER TABLE llm_requests OWNER TO ransomeye_owner;
ALTER TABLE llm_responses OWNER TO ransomeye_owner;
ALTER TABLE policy_evaluations OWNER TO ransomeye_owner;
ALTER TABLE enforcement_decisions OWNER TO ransomeye_owner;
ALTER TABLE actions_taken OWNER TO ransomeye_owner;
ALTER TABLE immutable_audit_log OWNER TO ransomeye_owner;
ALTER TABLE trust_verification_records OWNER TO ransomeye_owner;
ALTER TABLE signature_validation_events OWNER TO ransomeye_owner;
ALTER TABLE component_health OWNER TO ransomeye_owner;
ALTER TABLE startup_events OWNER TO ransomeye_owner;
ALTER TABLE error_events OWNER TO ransomeye_owner;

-- Public-schema compatibility tables (owned + granted explicitly)
ALTER TABLE public.telemetry_events OWNER TO ransomeye_owner;
ALTER TABLE public.scan_results OWNER TO ransomeye_owner;
ALTER TABLE public.scan_assets OWNER TO ransomeye_owner;
ALTER TABLE public.scan_port_services OWNER TO ransomeye_owner;
ALTER TABLE public.scan_deltas OWNER TO ransomeye_owner;
ALTER TABLE public.playbook_executions OWNER TO ransomeye_owner;
ALTER TABLE public.playbook_rollback_states OWNER TO ransomeye_owner;
ALTER TABLE public.playbook_nonces OWNER TO ransomeye_owner;
ALTER TABLE public.playbook_audit_log OWNER TO ransomeye_owner;
ALTER TABLE public.playbook_safe_halt_state OWNER TO ransomeye_owner;

GRANT USAGE ON SCHEMA ransomeye TO ransomeye_rw, ransomeye_ro, ransomeye_audit;

-- RW: full DML on operational tables, read-only on immutable audit tables (writes should go through audited pathways).
GRANT SELECT, INSERT, UPDATE, DELETE ON
  agents,
  components,
  entities,
  policies,
  policy_versions,
  linux_agent_telemetry,
  windows_agent_telemetry,
  dpi_probe_telemetry,
  retention_policies,
  raw_events,
  normalized_events,
  correlation_graph,
  detection_results,
  confidence_scores,
  model_registry,
  model_versions,
  inference_results,
  shap_explanations,
  feature_contributions,
  llm_requests,
  llm_responses,
  policy_evaluations,
  enforcement_decisions,
  actions_taken,
  component_health,
  startup_events,
  error_events
TO ransomeye_rw;

GRANT SELECT, INSERT ON immutable_audit_log, trust_verification_records, signature_validation_events TO ransomeye_rw;

-- RO: read-only across all tables.
GRANT SELECT ON ALL TABLES IN SCHEMA ransomeye TO ransomeye_ro;

-- AUDIT: read-only on everything, plus ability to insert into audit/trust tables if separated auditor component is used.
GRANT SELECT ON ALL TABLES IN SCHEMA ransomeye TO ransomeye_audit;
GRANT INSERT ON immutable_audit_log, trust_verification_records, signature_validation_events TO ransomeye_audit;

-- Public schema grants (modules that do not set search_path depend on these relations)
GRANT USAGE ON SCHEMA public TO ransomeye_rw, ransomeye_ro, ransomeye_audit;

GRANT SELECT, INSERT, UPDATE, DELETE ON
  public.telemetry_events,
  public.scan_results,
  public.scan_assets,
  public.scan_port_services,
  public.scan_deltas,
  public.playbook_executions,
  public.playbook_rollback_states,
  public.playbook_nonces,
  public.playbook_audit_log,
  public.playbook_safe_halt_state
TO ransomeye_rw;

GRANT SELECT ON
  public.telemetry_events,
  public.scan_results,
  public.scan_assets,
  public.scan_port_services,
  public.scan_deltas,
  public.playbook_executions,
  public.playbook_rollback_states,
  public.playbook_nonces,
  public.playbook_audit_log,
  public.playbook_safe_halt_state
TO ransomeye_ro, ransomeye_audit;

COMMIT;


