// Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/tests/process_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Process monitoring tests

use ransomeye_linux_agent::process::{ProcessMonitor, ProcessEventType};

#[test]
fn test_process_exec_event() {
    let monitor = ProcessMonitor::new(1000);
    
    let event = monitor.record_exec(
        1234,
        Some(1000),
        1000,
        1000,
        "/usr/bin/test".to_string(),
        Some("test --arg".to_string()),
    ).unwrap();
    
    assert_eq!(event.event_type, ProcessEventType::Exec);
    assert_eq!(event.pid, 1234);
    assert_eq!(event.ppid, Some(1000));
    assert_eq!(event.executable, Some("/usr/bin/test".to_string()));
    assert_eq!(event.command_line, Some("test --arg".to_string()));
}

#[test]
fn test_process_fork_event() {
    let monitor = ProcessMonitor::new(1000);
    
    let event = monitor.record_fork(
        1000, // parent
        1234, // child
        1000,
        1000,
    ).unwrap();
    
    assert_eq!(event.event_type, ProcessEventType::Fork);
    assert_eq!(event.pid, 1234);
    assert_eq!(event.ppid, Some(1000));
}

#[test]
fn test_process_mmap_event() {
    let monitor = ProcessMonitor::new(1000);
    
    let event = monitor.record_mmap(
        1234,
        0x400000,
        4096,
    ).unwrap();
    
    assert_eq!(event.event_type, ProcessEventType::Mmap);
    assert_eq!(event.pid, 1234);
    assert_eq!(event.mmap_address, Some(0x400000));
    assert_eq!(event.mmap_size, Some(4096));
}

#[test]
fn test_process_tracking_bounded_memory() {
    let monitor = ProcessMonitor::new(100);
    
    // Add processes up to limit
    for i in 0..100 {
        monitor.record_exec(
            i as u32,
            None,
            1000,
            1000,
            format!("/usr/bin/proc{}", i),
            None,
        ).unwrap();
    }
    
    assert_eq!(monitor.process_count(), 100);
}

#[test]
fn test_process_eviction() {
    let monitor = ProcessMonitor::new(100);
    
    // Fill beyond eviction threshold
    for i in 0..95 {
        monitor.record_exec(
            i as u32,
            None,
            1000,
            1000,
            format!("/usr/bin/proc{}", i),
            None,
        ).unwrap();
    }
    
    // Eviction should have occurred (target: 80% = 80 processes)
    let count = monitor.process_count();
    assert!(count <= 80, "Process count {} should be <= 80 after eviction", count);
}

