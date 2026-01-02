# Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/docs/privacy_guarantees.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Privacy guarantees and data handling

# Privacy Guarantees

## Data Retention

- **No Payload Retention**: Payload parsed but not stored beyond parsing window
- **No Plaintext Storage**: All stored data is metadata only
- **Flow Data**: Only 5-tuple and statistics (no payload)

## Collected Data

### Metadata Only
- Source/destination IP addresses
- Source/destination ports
- Protocol type
- Packet size
- Flow statistics (count, bytes, duration)

### Not Collected
- Packet payload content
- Application-layer data
- User data
- Sensitive information

## Data Lifecycle

1. **Capture**: Packet captured from network
2. **Parse**: Extract metadata only
3. **Process**: Create event envelope with metadata
4. **Emit**: Send to Phase 4 pipeline
5. **Discard**: Payload discarded immediately

## Privacy Protection

- **Minimal Data**: Only network metadata collected
- **No Enrichment**: No external data sources
- **No Inference**: No AI/ML processing
- **No Policy Logic**: No decision-making

## Compliance

- **GDPR**: Minimal data collection
- **Privacy by Design**: No payload retention
- **Data Minimization**: Only necessary metadata

