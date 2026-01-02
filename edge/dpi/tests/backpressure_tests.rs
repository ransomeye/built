// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/tests/backpressure_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Backpressure handling tests - DROP + SIGNAL, never block

use ransomeye_dpi_probe::backpressure::BackpressureManager;

#[test]
fn test_backpressure_activation() {
    let manager = BackpressureManager::new(1000);
    
    // Below threshold (80% = 800)
    assert!(!manager.should_drop(500));
    
    // At threshold
    assert!(manager.should_drop(800));
    
    // Above threshold
    assert!(manager.should_drop(900));
    assert!(manager.should_drop(1000));
}

#[test]
fn test_backpressure_deactivation() {
    let manager = BackpressureManager::new(1000);
    
    // Activate backpressure
    manager.should_drop(800);
    assert!(manager.stats().backpressure_active);
    
    // Deactivate (below 50% of threshold = 400)
    manager.should_drop(400);
    assert!(!manager.stats().backpressure_active);
}

#[test]
fn test_backpressure_never_blocks() {
    let manager = BackpressureManager::new(1000);
    
    // Should return immediately (non-blocking)
    for _ in 0..10000 {
        let _ = manager.should_drop(900);
    }
    
    // Should complete without blocking
    assert!(manager.stats().packets_dropped > 0);
}

#[test]
fn test_backpressure_stats() {
    let manager = BackpressureManager::new(1000);
    
    manager.update_queue_size(850);
    manager.should_drop(850);
    
    let stats = manager.stats();
    assert!(stats.backpressure_active);
    assert!(stats.packets_dropped > 0);
    assert_eq!(stats.current_queue_size, 850);
    assert_eq!(stats.drop_threshold, 800);
}
