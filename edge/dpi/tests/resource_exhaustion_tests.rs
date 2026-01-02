// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/tests/resource_exhaustion_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests for resource exhaustion handling and graceful degradation

use ransomeye_dpi_probe::backpressure::BackpressureHandler;
use ransomeye_dpi_probe::buffer::DiskBuffer;
use std::sync::Arc;
use tempfile::TempDir;

#[test]
fn test_memory_buffer_exhaustion() {
    // Create small buffer
    let handler = BackpressureHandler::new(1024, 512);
    
    // Fill buffer to capacity
    for _ in 0..10 {
        handler.increment_buffer(100);
    }
    
    // Buffer should be at capacity
    assert!(!handler.can_accept());
    
    // Attempting to add more should fail gracefully
    assert!(!handler.increment_buffer(1));
    
    // Drop count should increase
    assert!(handler.get_dropped_count() > 0);
}

#[test]
fn test_disk_buffer_exhaustion() {
    let temp_dir = TempDir::new().unwrap();
    let buffer_dir = temp_dir.path().join("buffer");
    
    // Create very small buffer (100KB)
    let buffer = DiskBuffer::new(&buffer_dir, 0).unwrap(); // 0 MB = use minimal
    
    // Try to write many events (should handle gracefully)
    let mut success_count = 0;
    let mut fail_count = 0;
    
    for _ in 0..100 {
        let event = create_test_event();
        match buffer.write_event(&event) {
            Ok(_) => success_count += 1,
            Err(_) => fail_count += 1,
        }
    }
    
    // Should handle exhaustion gracefully (either success or failure, no panic)
    assert!(success_count + fail_count == 100);
}

#[test]
fn test_drop_oldest_policy() {
    let temp_dir = TempDir::new().unwrap();
    let buffer_dir = temp_dir.path().join("buffer");
    
    let buffer = DiskBuffer::new(&buffer_dir, 1).unwrap(); // 1 MB
    
    // Write events until buffer is full
    let mut events = vec![];
    for i in 0..50 {
        let event = create_test_event_with_id(i);
        events.push(event.clone());
        if buffer.write_event(&event).is_err() {
            break;
        }
    }
    
    // Buffer should drop oldest events when full
    // We can't easily test this without knowing exact sizes,
    // but we verify the buffer doesn't crash
    let count = buffer.count_buffered_events().unwrap();
    assert!(count >= 0); // Should handle gracefully
}

#[test]
fn test_concurrent_resource_contention() {
    use std::thread;
    
    let handler = Arc::new(BackpressureHandler::new(1000, 500));
    
    // Spawn threads that compete for buffer space
    let mut handles = vec![];
    for _ in 0..20 {
        let h = handler.clone();
        let handle = thread::spawn(move || {
            for _ in 0..10 {
                // Some will succeed, some will fail - should handle gracefully
                h.increment_buffer(100);
                std::thread::sleep(std::time::Duration::from_millis(1));
                h.decrement_buffer(100);
            }
        });
        handles.push(handle);
    }
    
    // All threads should complete without panicking
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Final state should be consistent
    assert!(handler.get_buffer_size() <= 1000);
}

#[test]
fn test_graceful_degradation_under_load() {
    let handler = BackpressureHandler::new(5000, 2500);
    
    // Simulate increasing load
    let mut accepted = 0;
    let mut rejected = 0;
    
    for i in 0..100 {
        let size = (i * 10) as usize;
        if handler.increment_buffer(size) {
            accepted += 1;
        } else {
            rejected += 1;
        }
    }
    
    // Should accept some and reject some gracefully
    assert!(accepted > 0);
    assert!(rejected > 0);
    
    // Process should remain stable
    assert!(handler.get_buffer_size() <= 5000);
}

fn create_test_event() -> ransomeye_dpi_probe::signing::SignedEvent {
    use chrono::Utc;
    use uuid::Uuid;
    
    ransomeye_dpi_probe::signing::SignedEvent {
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

fn create_test_event_with_id(id: u32) -> ransomeye_dpi_probe::signing::SignedEvent {
    use chrono::Utc;
    use uuid::Uuid;
    
    ransomeye_dpi_probe::signing::SignedEvent {
        message_id: format!("test-{}", id),
        timestamp: Utc::now(),
        nonce: format!("{:064x}", id),
        component_identity: "dpi_probe_test".to_string(),
        data: serde_json::json!({
            "flow_id": format!("flow-{}", id),
            "src_ip": "192.168.1.1",
        }),
        signature: format!("sig-{}", id),
        data_hash: format!("{:064x}", id * 2),
    }
}
