# Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/docs/protocols_supported.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Supported protocol parsing documentation

# Supported Protocols

## Layer 2 (Data Link)

- **Ethernet**: Full support
  - MAC address extraction
  - EtherType detection

## Layer 3 (Network)

- **IPv4**: Full support
  - Source/destination IP extraction
  - Fragment detection
  - Protocol identification
  
- **IPv6**: Full support
  - Source/destination IPv6 extraction
  - Extension header handling

## Layer 4 (Transport)

- **TCP**: Full support
  - Source/destination port extraction
  - Flag extraction (when available)
  
- **UDP**: Full support
  - Source/destination port extraction
  
- **ICMP**: Basic support
  - Type/code extraction

## Parsing Characteristics

- **Zero Allocation**: Hot path uses zero-copy parsing
- **Deterministic**: Same packet → same parse result
- **No Payload Retention**: Payload parsed but not stored beyond parsing window

## Protocol Identification

Protocols are identified by:
- EtherType (L2 → L3)
- IP Protocol field (L3 → L4)
- Port numbers (L4 application hints)

## Error Handling

- Invalid packet → Parse error
- Unsupported protocol → Marked as Unknown
- Truncated packet → Partial parse (if possible)

