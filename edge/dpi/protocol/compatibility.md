# DPI Probe Protocol Compatibility

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_dpi_probe/protocol/compatibility.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Protocol version compatibility and migration guide for DPI Probe telemetry events

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

## Backward Compatibility

The DPI Probe protocol is designed to be forward-compatible. Core must accept events from any DPI Probe version and handle gracefully:

- Unknown fields are ignored
- Missing optional fields use defaults
- Signature verification is version-agnostic

## Forward Compatibility

DPI Probe will:
- Reject responses from Core that contain unknown required fields
- Accept responses with unknown optional fields
- Log compatibility warnings for schema mismatches

## Migration Notes

### Upgrading DPI Probe

When upgrading DPI Probe:
1. Verify Core supports current protocol version
2. Test event transmission
3. Monitor signature verification
4. Check replay protection nonce format

### Upgrading Core

When upgrading Core:
1. Ensure backward compatibility with v1.0 events
2. Test event reception from existing DPI Probes
3. Validate signature verification
4. Verify flow data parsing

## Breaking Changes

Breaking changes require protocol version bump:

- Signature algorithm changes
- Required field additions/removals
- Data type changes
- Nonce format changes

## Non-Breaking Changes

These changes do not require version bump:

- Optional field additions
- Metadata field expansions
- New protocol enum values
- Documentation updates
