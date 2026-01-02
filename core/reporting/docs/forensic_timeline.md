# Forensic Timeline

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_reporting/docs/forensic_timeline.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Forensic timeline documentation - describes deterministic chronological event ordering

---

## Overview

Forensic timelines provide **deterministic, chronologically ordered** event sequences with **source attribution** and **kill-chain annotations**.

---

## Principles

1. **Deterministic Ordering**: Events are sorted by timestamp (UTC), then by evidence ID
2. **Source Attribution**: Every event includes explicit source and source type
3. **Kill-Chain Annotations**: Events are tagged with MITRE ATT&CK kill-chain stages
4. **UTC Timestamps**: All timestamps are in UTC (explicit timezone)

---

## Timeline Event Structure

Each timeline event contains:

- **Timestamp**: Event timestamp (UTC, ISO 8601)
- **Source**: Source system identifier
- **Source Type**: Type of source (dpi_probe, linux_agent, etc.)
- **Event Type**: Type of event (evidence, alert, correlation)
- **Kill Chain Stage**: MITRE ATT&CK stage (optional)
- **Description**: Human-readable event description
- **Evidence ID**: UUID of associated evidence item
- **Metadata**: Additional event metadata (JSON)

---

## Ordering Algorithm

Events are sorted using:

1. **Primary Sort**: Timestamp (ascending)
2. **Secondary Sort**: Evidence ID (ascending, for deterministic tie-breaking)

This ensures:
- **Reproducibility**: Same events always produce same timeline
- **Chronological Order**: Events appear in time order
- **Determinism**: No non-deterministic ordering

---

## Source Attribution

Every event must include:

- **Source**: Which system produced the event
- **Source Type**: Category of source (agent, probe, engine, etc.)

This enables:
- **Traceability**: Track events back to source
- **Filtering**: Filter by source or source type
- **Audit**: Verify event provenance

---

## Kill-Chain Integration

Events can be annotated with MITRE ATT&CK kill-chain stages:

- **Reconnaissance**: Initial information gathering
- **Weaponization**: Creating attack tools
- **Delivery**: Delivering attack to target
- **Exploitation**: Exploiting vulnerabilities
- **Installation**: Installing persistent access
- **Command and Control**: Establishing C2 channel
- **Actions on Objectives**: Achieving attack goals

This enables:
- **Attack Progression**: Track attack through kill chain
- **Stage Filtering**: Filter events by kill-chain stage
- **Threat Analysis**: Understand attack progression

---

## Timeline Construction

Timelines are constructed from:

1. **Evidence Bundles**: Extract events from sealed evidence bundles
2. **Manual Events**: Add custom events with explicit timestamps
3. **Correlation Events**: Events from correlation engine

All events are merged and sorted chronologically.

---

## Query Operations

Timelines support:

- **Time Range**: Get events in time range
- **Kill-Chain Stage**: Get events by stage
- **Source**: Get events by source
- **Export**: Export timeline as JSON

---

## Reproducibility

Timelines are **reproducible**: given the same evidence bundles, the same timeline is always generated. This ensures:

- **Audit Compliance**: Timelines can be regenerated for audits
- **Verification**: Timelines can be verified against evidence
- **Consistency**: Same evidence always produces same timeline

