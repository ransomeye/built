// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/tests/core_unavailable_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests for Core unavailability handling and disk buffering

use ransomeye_dpi_probe::buffer::DiskBuffer;
use ransomeye_dpi_probe::signing::SignedEvent;
use tempfile::TempDir;

#[test]
fn test_buffer_event_to_disk() {
    let temp_dir = TempDir::new().unwrap();
    let buffer_dir = temp_dir.path().join("buffer");
    
    let buffer = DiskBuffer::new(&buffer_dir, 10).unwrap();
    
    // Create a test event
    let test_event = create_test_event();
    
    // Write event to buffer
    buffer.write_event(&test_event).unwrap();
    
    // Verify event can be read back
    let read_event = buffer.read_oldest_event().unwrap();
    assert!(read_event.is_some());
    let read_event = read_event.unwrap();
    assert_eq!(read_event.message_id, test_event.message_id);
}

#[test]
fn test_buffer_fifo_order() {
    let temp_dir = TempDir::new().unwrap();
    let buffer_dir = temp_dir.path().join("buffer");
    
    let buffer = DiskBuffer::new(&buffer_dir, 10).unwrap();
    
    // Write multiple events
    let event1 = create_test_event();
    std::thread::sleep(std::time::Duration::from_millis(10));
    let event2 = create_test_event();
    std::thread::sleep(std::time::Duration::from_millis(10));
    let event3 = create_test_event();
    
    buffer.write_event(&event1).unwrap();
    buffer.write_event(&event2).unwrap();
    buffer.write_event(&event3).unwrap();
    
    // Read events back in order (oldest first)
    let read1 = buffer.read_oldest_event().unwrap().unwrap();
    assert_eq!(read1.message_id, event1.message_id);
    
    // Remove first event
    buffer.remove_event(&read1).unwrap();
    
    // Read next event
    let read2 = buffer.read_oldest_event().unwrap().unwrap();
    assert_eq!(read2.message_id, event2.message_id);
}

#[test]
fn test_buffer_drop_oldest_when_full() {
    let temp_dir = TempDir::new().unwrap();
    let buffer_dir = temp_dir.path().join("buffer");
    
    // Create small buffer (1MB)
    let buffer = DiskBuffer::new(&buffer_dir, 1).unwrap();
    
    // Fill buffer with events
    let mut events = vec![];
    for _ in 0..100 {
        let event = create_test_event();
        events.push(event.clone());
        if buffer.write_event(&event).is_err() {
            break; // Buffer full
        }
    }
    
    // Buffer should have dropped oldest events
    let current_size = buffer.get_current_size();
    assert!(current_size <= buffer.get_max_size());
    
    // Count buffered events
    let count = buffer.count_buffered_events().unwrap();
    assert!(count > 0);
}

#[test]
fn test_buffer_size_tracking() {
    let temp_dir = TempDir::new().unwrap();
    let buffer_dir = temp_dir.path().join("buffer");
    
    let buffer = DiskBuffer::new(&buffer_dir, 10).unwrap();
    
    // Initial size should be zero or small
    let initial_size = buffer.get_current_size();
    
    // Write events
    let event = create_test_event();
    buffer.write_event(&event).unwrap();
    
    // Size should increase
    let new_size = buffer.get_current_size();
    assert!(new_size > initial_size);
}

#[test]
fn test_buffer_empty_when_no_events() {
    let temp_dir = TempDir::new().unwrap();
    let buffer_dir = temp_dir.path().join("buffer");
    
    let buffer = DiskBuffer::new(&buffer_dir, 10).unwrap();
    
    // No events in buffer
    let event = buffer.read_oldest_event().unwrap();
    assert!(event.is_none());
    
    // Count should be zero
    assert_eq!(buffer.count_buffered_events().unwrap(), 0);
}

#[test]
fn test_buffer_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let buffer_dir = temp_dir.path().join("buffer");
    
    // Write event to buffer
    {
        let buffer = DiskBuffer::new(&buffer_dir, 10).unwrap();
        let event = create_test_event();
        buffer.write_event(&event).unwrap();
    }
    
    // Create new buffer instance (simulates restart)
    let buffer = DiskBuffer::new(&buffer_dir, 10).unwrap();
    
    // Should be able to read events from previous instance
    let count = buffer.count_buffered_events().unwrap();
    assert!(count > 0);
}

fn create_test_event() -> SignedEvent {
    use chrono::Utc;
    use uuid::Uuid;
    
    SignedEvent {
        message_id: Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        nonce: "a".repeat(64),
        component_identity: "dpi_probe_test".to_string(),
        data: serde_json::json!({
            "flow_id": "test-flow",
            "src_ip": "192.168.1.1",
        }),
        signature: "test_signature".to_string(),
        data_hash: "a".repeat(64),
    }
}
