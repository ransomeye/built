# Linux Agent Protocol Compatibility

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_linux_agent/protocol/compatibility.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Protocol version compatibility and migration guide for Linux Agent telemetry events

---

## Protocol Version

Current version: **v1.0**

## Event Schema Evolution

### Version 1.0 (Current)

- Initial release
- RSA-4096-PSS-SHA256 signatures
- JSON format
- ISO-8601 timestamps
- UUID v4 message IDs
- 64-character hex nonces
- Host ID included in events

## Backward Compatibility

The Linux Agent protocol is designed to be forward-compatible. Core must accept events from any Linux Agent version and handle gracefully:

- Unknown fields are ignored
- Missing optional fields use defaults
- Signature verification is version-agnostic

## Forward Compatibility

Linux Agent will:
- Reject responses from Core that contain unknown required fields
- Accept responses with unknown optional fields
- Log compatibility warnings for schema mismatches

## Migration Notes

### Upgrading Linux Agent

When upgrading Linux Agent:
1. Verify Core supports current protocol version
2. Test event transmission
3. Monitor signature verification
4. Check replay protection nonce format
5. Verify host ID format

### Upgrading Core

When upgrading Core:
1. Ensure backward compatibility with v1.0 events
2. Test event reception from existing Linux Agents
3. Validate signature verification
4. Verify telemetry data parsing

## Breaking Changes

Breaking changes require protocol version bump:

- Signature algorithm changes
- Required field additions/removals
- Data type changes
- Nonce format changes
- Host ID format changes

## Non-Breaking Changes

These changes do not require version bump:

- Optional field additions
- New event type additions
- Metadata field expansions
- Documentation updates
