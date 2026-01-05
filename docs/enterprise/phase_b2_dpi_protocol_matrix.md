# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/phase_b2_dpi_protocol_matrix.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Phase B2 - DPI Probe Protocol Validation Matrix

# Phase B2 - DPI Probe Protocol Validation Matrix

**Date:** 2026-01-28  
**Phase:** PROMPT-53 — BLOCKER 3 Resolution  
**Status:** ✅ **FRAMEWORK COMPLETE**

---

## Objective

Validate DPI Probe protocol decoding (no decryption) for:
- SMB
- DNS
- HTTP
- HTTPS (SNI, JA3)
- RDP

Confirm output schema consistency across all protocols.

---

## Current Implementation Status

### Layer 2-4 (Implemented)
- ✅ **Ethernet**: Full support (MAC address extraction)
- ✅ **IPv4**: Full support (IP extraction, fragment detection)
- ✅ **IPv6**: Full support (IPv6 extraction, extension headers)
- ✅ **TCP**: Full support (port extraction, flags)
- ✅ **UDP**: Full support (port extraction)
- ✅ **ICMP**: Basic support (type/code extraction)

### Layer 7 (Required, Not Yet Implemented)
- ❌ **SMB**: Not implemented
- ❌ **DNS**: Not implemented
- ❌ **HTTP**: Not implemented
- ❌ **HTTPS (SNI)**: Not implemented
- ❌ **HTTPS (JA3)**: Not implemented
- ❌ **RDP**: Not implemented

---

## Protocol Validation Matrix

| Protocol | Layer | Decoding Method | Output Fields | Status |
|----------|-------|-----------------|---------------|--------|
| **SMB** | L7 | Parse SMB header (no decryption) | `smb_command`, `smb_version`, `smb_negotiate_flags` | PENDING |
| **DNS** | L7 | Parse DNS header (no decryption) | `dns_query`, `dns_type`, `dns_response_code` | PENDING |
| **HTTP** | L7 | Parse HTTP headers (no decryption) | `http_method`, `http_host`, `http_path`, `http_user_agent` | PENDING |
| **HTTPS (SNI)** | L7 | Parse TLS ClientHello (no decryption) | `tls_sni`, `tls_version`, `tls_cipher_suites` | PENDING |
| **HTTPS (JA3)** | L7 | Compute JA3 fingerprint from TLS handshake | `ja3`, `ja3s` | PENDING |
| **RDP** | L7 | Parse RDP header (no decryption) | `rdp_version`, `rdp_requested_protocols` | PENDING |

---

## Output Schema Consistency

### Required Schema Fields (All Protocols)

All protocol decodings must populate the following base schema:

```json
{
  "flow_id": "string",
  "src_ip": "string",
  "dst_ip": "string",
  "src_port": "integer",
  "dst_port": "integer",
  "protocol": "string",
  "packet_count": "integer",
  "byte_count": "integer",
  "classification": "string",
  "metadata": {
    "protocol_specific": {}
  },
  "tls_sni": "string|null",
  "http_host": "string|null",
  "http_method": "string|null",
  "http_path": "string|null",
  "ja3": "string|null",
  "ja3s": "string|null"
}
```

### Protocol-Specific Metadata

#### SMB Metadata
```json
{
  "metadata": {
    "smb_command": "string",
    "smb_version": "string",
    "smb_negotiate_flags": "integer"
  }
}
```

#### DNS Metadata
```json
{
  "metadata": {
    "dns_query": "string",
    "dns_type": "string",
    "dns_response_code": "integer"
  }
}
```

#### HTTP Metadata
```json
{
  "metadata": {
    "http_method": "string",
    "http_host": "string",
    "http_path": "string",
    "http_user_agent": "string"
  },
  "http_method": "string",
  "http_host": "string",
  "http_path": "string"
}
```

#### HTTPS Metadata
```json
{
  "metadata": {
    "tls_sni": "string",
    "tls_version": "string",
    "tls_cipher_suites": "array"
  },
  "tls_sni": "string",
  "ja3": "string",
  "ja3s": "string"
}
```

#### RDP Metadata
```json
{
  "metadata": {
    "rdp_version": "string",
    "rdp_requested_protocols": "array"
  }
}
```

---

## Validation Test Cases

### Test Case 1: SMB Protocol Decoding
**Objective:** Validate SMB header parsing without decryption

**Test Procedure:**
1. Capture SMB traffic (port 445)
2. Verify SMB command extraction
3. Verify SMB version extraction
4. Verify SMB negotiate flags extraction
5. Confirm no decryption occurs

**Expected Output:**
- `metadata.smb_command` populated
- `metadata.smb_version` populated
- `metadata.smb_negotiate_flags` populated
- No decrypted payload in output

**Status:** PENDING (implementation required)

---

### Test Case 2: DNS Protocol Decoding
**Objective:** Validate DNS header parsing without decryption

**Test Procedure:**
1. Capture DNS traffic (port 53)
2. Verify DNS query extraction
3. Verify DNS type extraction
4. Verify DNS response code extraction
5. Confirm no decryption occurs

**Expected Output:**
- `metadata.dns_query` populated
- `metadata.dns_type` populated
- `metadata.dns_response_code` populated
- No decrypted payload in output

**Status:** PENDING (implementation required)

---

### Test Case 3: HTTP Protocol Decoding
**Objective:** Validate HTTP header parsing without decryption

**Test Procedure:**
1. Capture HTTP traffic (port 80)
2. Verify HTTP method extraction
3. Verify HTTP host extraction
4. Verify HTTP path extraction
5. Verify HTTP user agent extraction
6. Confirm no decryption occurs

**Expected Output:**
- `http_method` populated
- `http_host` populated
- `http_path` populated
- `metadata.http_user_agent` populated
- No decrypted payload in output

**Status:** PENDING (implementation required)

---

### Test Case 4: HTTPS SNI Extraction
**Objective:** Validate TLS SNI extraction from ClientHello without decryption

**Test Procedure:**
1. Capture HTTPS traffic (port 443)
2. Parse TLS ClientHello handshake
3. Extract SNI (Server Name Indication)
4. Extract TLS version
5. Extract cipher suites
6. Confirm no decryption occurs

**Expected Output:**
- `tls_sni` populated
- `metadata.tls_version` populated
- `metadata.tls_cipher_suites` populated
- No decrypted payload in output

**Status:** PENDING (implementation required)

---

### Test Case 5: HTTPS JA3 Fingerprinting
**Objective:** Validate JA3/JA3S fingerprint computation from TLS handshake

**Test Procedure:**
1. Capture HTTPS traffic (port 443)
2. Parse TLS ClientHello handshake
3. Compute JA3 fingerprint
4. Parse TLS ServerHello handshake
5. Compute JA3S fingerprint
6. Confirm no decryption occurs

**Expected Output:**
- `ja3` populated (ClientHello fingerprint)
- `ja3s` populated (ServerHello fingerprint)
- No decrypted payload in output

**Status:** PENDING (implementation required)

---

### Test Case 6: RDP Protocol Decoding
**Objective:** Validate RDP header parsing without decryption

**Test Procedure:**
1. Capture RDP traffic (port 3389)
2. Verify RDP version extraction
3. Verify RDP requested protocols extraction
4. Confirm no decryption occurs

**Expected Output:**
- `metadata.rdp_version` populated
- `metadata.rdp_requested_protocols` populated
- No decrypted payload in output

**Status:** PENDING (implementation required)

---

## Schema Consistency Validation

### Validation Rules

1. **All protocols must populate base schema:**
   - `flow_id`, `src_ip`, `dst_ip`, `src_port`, `dst_port`, `protocol`
   - `packet_count`, `byte_count`, `classification`
   - `metadata` object (protocol-specific fields)

2. **Protocol-specific fields must be consistent:**
   - HTTP: `http_method`, `http_host`, `http_path` at root AND in metadata
   - HTTPS: `tls_sni`, `ja3`, `ja3s` at root AND in metadata
   - Other protocols: Only in metadata

3. **No decryption:**
   - All fields extracted from headers/handshakes only
   - No decrypted payload in output
   - No decryption keys required

4. **Null handling:**
   - Fields not applicable to protocol must be `null`
   - No empty strings for missing fields

---

## Implementation Requirements

### Required Libraries
- **SMB**: `smb-parser` or custom parser
- **DNS**: `dns-parser` or custom parser
- **HTTP**: `httparse` or custom parser
- **HTTPS/TLS**: `rustls` or `openssl` for handshake parsing
- **RDP**: Custom parser (RDP protocol specification)

### Integration Points
- **Parser Module**: `edge/dpi/probe/src/parser.rs`
- **Flow Assembler**: `edge/dpi/probe/src/flow.rs`
- **Feature Extractor**: `edge/dpi/probe/src/features.rs`

### Performance Requirements
- **Zero allocation in hot path** (current requirement)
- **Deterministic parsing** (same packet → same result)
- **No payload retention** (headers only)

---

## Test Execution Framework

### Test Script Location
`tests/dpi_protocol_validation.sh`

### Test Execution
```bash
# Run protocol validation tests
./tests/dpi_protocol_validation.sh

# Expected output:
# - Protocol decoding results
# - Schema consistency validation
# - Performance metrics
```

---

## Conclusion

**Phase B2 Status:** ✅ **FRAMEWORK COMPLETE**

Protocol validation matrix is complete with:
- ✅ All required protocols defined
- ✅ Output schema consistency rules defined
- ✅ Test cases specified
- ✅ Implementation requirements documented
- ❌ L7 protocol parsing not yet implemented (pending)

**Next Steps:**
1. Implement L7 protocol parsers (SMB, DNS, HTTP, HTTPS, RDP)
2. Integrate parsers into DPI Probe flow assembler
3. Execute protocol validation tests
4. Validate schema consistency
5. Confirm no decryption occurs

**Blocking Issues:** L7 protocol parsing implementation required

