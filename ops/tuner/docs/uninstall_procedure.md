# Uninstall Procedure

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_operations/docs/uninstall_procedure.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Uninstall procedures for RansomEye with evidence preservation options

---

## Overview

RansomEye uninstallation provides clean removal with options for evidence preservation or secure deletion.

---

## Uninstallation Flow

1. **Verification**: Verify install state is valid
2. **Confirmation**: Require explicit confirmation
3. **Service Stop**: Stop all services
4. **Cleanup**: Remove services, configs, and optionally evidence
5. **Logging**: Log cleanup operation

---

## Uninstallation Options

### Preserve Evidence (Default)

```bash
ransomeye_operations uninstall --confirm
```

- Removes services and configs
- Preserves evidence for audit/compliance
- Logs cleanup operation

### Remove Evidence

```bash
ransomeye_operations uninstall --confirm --remove-evidence
```

- Removes services, configs, and evidence
- Standard file deletion
- Logs cleanup operation

### Secure Delete Evidence

```bash
ransomeye_operations uninstall --confirm --remove-evidence --secure-delete
```

- Removes services, configs, and evidence
- 3-pass overwrite secure deletion
- Logs cleanup operation with destruction certificate

---

## Cleanup Components

### Services Removed

- All systemd service units
- Service configurations
- Service logs

### Configs Removed

- Retention configuration
- Install state
- Cryptographic keys (if not preserved)

### Evidence Removed (Optional)

- Forensic evidence bundles
- Reporting data
- Timeline data

---

## Cleanup Log

Cleanup operation is logged with:
- Timestamp
- Removed services
- Removed configs
- Evidence destruction status
- Secure deletion status

Log location: `/home/ransomeye/rebuild/logs/uninstall_YYYYMMDD_HHMMSS.json`

---

## Secure Deletion

Secure deletion uses 3-pass overwrite:
1. **Pass 1**: Overwrite with random data
2. **Pass 2**: Overwrite with zeros
3. **Pass 3**: Overwrite with random data again

After overwrite, files are deleted and directory is removed.

---

## Destruction Certificate

When secure deletion is used, a destruction certificate is created:
- Timestamp
- Files destroyed
- Secure deletion method
- Cryptographic signature

---

## Uninstallation Requirements

- **Confirmation**: `--confirm` flag is mandatory
- **Valid State**: Install state must be valid
- **Permissions**: Root or sudo privileges required

---

## Post-Uninstallation

After uninstallation:
1. Systemd units removed from `/etc/systemd/system/`
2. Configurations removed
3. Evidence removed (if requested)
4. Cleanup log created

---

## Rollback After Uninstallation

Uninstallation is **irreversible** unless:
- Evidence was preserved
- Backups were created
- Install state was backed up

---

## Best Practices

1. **Backup Before Uninstall**: Always backup before uninstalling
2. **Preserve Evidence**: Preserve evidence unless explicitly required to destroy
3. **Secure Delete When Required**: Use secure deletion for compliance
4. **Verify Cleanup**: Verify cleanup was successful
5. **Keep Logs**: Keep cleanup logs for audit

---

## Compliance Considerations

- **GDPR**: Secure deletion may be required
- **HIPAA**: Secure deletion with destruction certificate
- **Audit**: Cleanup logs provide audit trail

---

## Support

Contact: Gagan@RansomEye.Tech  
Support: Gagan@RansomEye.Tech

