# ETW → Envelope Example

**Path and File Name:** /home/ransomeye/rebuild/ransomeye_windows_agent/ETW_TO_ENVELOPE_EXAMPLE.md  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Complete example of ETW event processing to Phase-4 envelope

## Complete Flow Example

### Step 1: ETW Event Received

```rust
// ETW callback receives raw event from Windows kernel
let etw_event = EtwEvent {
    event_type: EtwEventType::ProcessStart,
    timestamp: 1706371200,  // Unix timestamp
    pid: 1234,              // Process ID
    tid: 5678,              // Thread ID
    data: EtwEventData::Process {
        image_name: "notepad.exe".to_string(),
        command_line: Some("notepad.exe C:\\test.txt".to_string()),
        ppid: Some(1000),   // Parent process ID
    },
};
```

### Step 2: Convert to Process Event

```rust
use ransomeye_windows_agent::process::{ProcessMonitor, ProcessEvent, ProcessEventType};

let process_monitor = ProcessMonitor::new(10000);
let process_event = process_monitor.get_process_create(etw_event.pid)?;

// Result:
// ProcessEvent {
//     event_type: ProcessEventType::Create,
//     pid: 1234,
//     ppid: Some(1000),
//     executable: Some("notepad.exe".to_string()),
//     command_line: Some("notepad.exe C:\\test.txt".to_string()),
//     timestamp: 1706371200,
// }
```

### Step 3: Extract Features

```rust
use ransomeye_windows_agent::features::Features;

let features = Features::from_process_event(&process_event);

// Result:
// Features {
//     event_type: "Create".to_string(),
//     process_activity: true,
//     filesystem_activity: false,
//     registry_activity: false,
//     network_activity: false,
//     path_count: 0,
//     has_command_line: true,
//     has_autorun: false,
//     has_persistence: false,
// }
```

### Step 4: Create Phase-4 Envelope

```rust
use ransomeye_windows_agent::envelope::{EnvelopeBuilder, EventEnvelope, EventData, ProcessData, FeaturesData};

let mut envelope_builder = EnvelopeBuilder::new(
    "windows_agent".to_string(),
    "windows-agent-uuid-1234-5678".to_string(),
);

let envelope = envelope_builder.build_from_process(
    &process_event,
    &features,
    "".to_string(),  // Signature added in next step
)?;

// Result (before signing):
// EventEnvelope {
//     event_id: "windows-agent-uuid-1234-5678-1",
//     timestamp: "2025-01-27T12:00:00Z",
//     component: "windows_agent",
//     component_id: "windows-agent-uuid-1234-5678",
//     event_type: "process_telemetry",
//     sequence: 1,
//     signature: "",  // To be signed
//     data: EventData {
//         event_category: "process",
//         pid: 1234,
//         process_data: Some(ProcessData {
//             event_type: "Create",
//             ppid: Some(1000),
//             executable: Some("notepad.exe"),
//             command_line: Some("notepad.exe C:\\test.txt"),
//         }),
//         filesystem_data: None,
//         registry_data: None,
//         network_data: None,
//         features: FeaturesData { ... },
//     },
// }
```

### Step 5: Sign Envelope

```rust
use ransomeye_windows_agent::security::signing::EventSigner;
use serde_json;

// Serialize envelope (without signature)
let envelope_json = serde_json::to_string(&envelope)?;

// Sign with Ed25519
let signer = EventSigner::new()?;
let signature = signer.sign(envelope_json.as_bytes())?;

// Update envelope with signature
let signed_envelope = EventEnvelope {
    signature: signature.clone(),
    ..envelope
};
```

### Step 6: Final Phase-4 Envelope

```json
{
  "event_id": "windows-agent-uuid-1234-5678-1",
  "timestamp": "2025-01-27T12:00:00Z",
  "component": "windows_agent",
  "component_id": "windows-agent-uuid-1234-5678",
  "event_type": "process_telemetry",
  "sequence": 1,
  "signature": "Ed25519_signature_base64_encoded...",
  "data": {
    "event_category": "process",
    "pid": 1234,
    "process_data": {
      "event_type": "Create",
      "ppid": 1000,
      "executable": "notepad.exe",
      "command_line": "notepad.exe C:\\test.txt"
    },
    "filesystem_data": null,
    "registry_data": null,
    "network_data": null,
    "features": {
      "event_type": "Create",
      "process_activity": true,
      "filesystem_activity": false,
      "registry_activity": false,
      "network_activity": false,
      "path_count": 0,
      "has_command_line": true,
      "has_autorun": false,
      "has_persistence": false
    }
  }
}
```

### Step 7: Send to Phase 4 Ingestion Pipeline

```rust
// Envelope is now ready for Phase 4 ingestion
// Transport via mTLS to Core API endpoint
// Phase 4 will:
// 1. Verify Ed25519 signature
// 2. Check sequence number (replay protection)
// 3. Validate envelope structure
// 4. Ingest into pipeline
```

## Key Points

1. **ETW → Structured Event:** Raw ETW event converted to structured ProcessEvent
2. **Feature Extraction:** Lightweight features extracted (no AI inference)
3. **Envelope Creation:** Phase-4 compliant envelope structure
4. **Signing:** Ed25519 signature includes sequence number (replay protection)
5. **No Enrichment:** Raw telemetry only, no inference or policy decisions
6. **Phase-4 Compliant:** Envelope structure matches Phase 4 ingestion requirements

---

**© RansomEye.Tech | Support: Gagan@RansomEye.Tech**

