# Policy Versioning

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_policy/schema/versioning.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Policy versioning and compatibility rules

## Version Format

Semantic versioning: `MAJOR.MINOR.PATCH`

- **MAJOR**: Breaking changes
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes, backward compatible

## Version Compatibility

### Policy Engine Version

Engine supports policies up to its own major version.

- Engine 1.x supports policies 1.x
- Engine 2.x supports policies 2.x (and may support 1.x with compatibility mode)

### Policy Version Updates

- Policy updates require new signature
- Old policy versions remain valid until revoked
- Version changes must be explicit in policy file

## Migration

When upgrading policies:

1. Update version number
2. Re-sign with new key (if key rotation)
3. Update signature_hash
4. Test with engine version
5. Deploy new policy
6. Revoke old policy version

