# Directive Protocol Versioning

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_dispatcher/protocol/versioning.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Protocol versioning documentation for Phase 6 → Phase 7 directive envelope

## Version History

### Version 1.0.0 (Current)
- Initial directive envelope protocol
- UUIDv7 directive IDs
- TTL-based expiration
- Nonce-based replay protection
- Audit receipt integration

## Compatibility

- Phase 6 MUST produce directives compatible with version 1.0.0
- Phase 7 MUST reject directives with incompatible versions
- Version mismatch → REJECT

## Future Versions

Version changes will be documented here with:
- Breaking changes
- Migration path
- Backward compatibility policy

