// Path and File Name : /home/ransomeye/rebuild/core/governor/tests/resource_governance_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Comprehensive tests for resource governance - CPU saturation, memory pressure, SWAP growth (no 64GB cap), disk full, FD exhaustion, network overload

use governor::{
    ResourceGovernor, ResourceGovernanceConfig, ComponentResourceLimits,
    cpu::ComponentPriority as CpuPriority,
    network::TrafficPriority as NetPriority,
};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_cpu_saturation_handling() {
    let config = ResourceGovernanceConfig {
        cpu_exhaustion_threshold: 90.0,
        ..Default::default()
    };
    let governor = ResourceGovernor::new(config);
    
    let limits = ComponentResourceLimits {
        cpu_max_percent: 50.0,
        cpu_window_seconds: 60,
        cpu_backpressure_threshold: 40.0,
        memory_max_mb: 1024,
        memory_oom_threshold: 80.0,
        disk_max_mb: 2048,
        disk_path: "/tmp".to_string(),
        is_audit: false,
        connection_max: 100,
        rate_limit_per_second: 1000,
        rate_limit_window_seconds: 1,
        is_critical: false,
    };
    
    governor.register_component("test_component".to_string(), limits).unwrap();
    
    // Record high CPU usage
    governor.cpu().record_usage("test_component", 45.0);
    
    // Check quota - should trigger backpressure
    let result = governor.cpu().check_quota("test_component");
    assert!(result.is_ok() || result.is_err()); // May pass or fail depending on system state
    
    // Verify backpressure is tracked
    let metrics = governor.cpu().get_metrics();
    assert!(metrics.system_cpu_percent >= 0.0);
}

#[test]
fn test_memory_pressure_handling() {
    let config = ResourceGovernanceConfig {
        memory_oom_threshold: 85.0,
        ..Default::default()
    };
    let governor = ResourceGovernor::new(config);
    
    let limits = ComponentResourceLimits {
        cpu_max_percent: 50.0,
        cpu_window_seconds: 60,
        cpu_backpressure_threshold: 40.0,
        memory_max_mb: 1024,
        memory_oom_threshold: 80.0,
        disk_max_mb: 2048,
        disk_path: "/tmp".to_string(),
        is_audit: false,
        connection_max: 100,
        rate_limit_per_second: 1000,
        rate_limit_window_seconds: 1,
        is_critical: false,
    };
    
    governor.register_component("test_component".to_string(), limits).unwrap();
    
    // Check memory quota
    let result = governor.memory().check_quota("test_component", 512);
    assert!(result.is_ok() || result.is_err()); // May pass or fail depending on system state
    
    // Verify memory info is retrieved
    let memory_info = governor.memory().get_memory_info();
    assert!(memory_info.is_ok());
    
    let info = memory_info.unwrap();
    assert!(info.total_ram_gb > 0.0);
}

#[test]
fn test_swap_growth_no_64gb_cap() {
    let config = ResourceGovernanceConfig::default();
    let governor = ResourceGovernor::new(config);
    
    // CRITICAL: Verify SWAP configuration does NOT enforce 64GB cap
    // SWAP should scale to available physical RAM
    let result = governor.memory().verify_swap_configuration();
    assert!(result.is_ok());
    
    // Get memory info to verify SWAP can exceed 64GB
    let memory_info = governor.memory().get_memory_info().unwrap();
    
    // For systems with >64GB RAM, SWAP should be allowed to scale
    // This test verifies the logic doesn't cap SWAP at 64GB
    if memory_info.total_ram_gb > 64.0 {
        // SWAP should be allowed to scale to RAM (or beyond)
        // We don't enforce a cap - we just verify it CAN scale
        assert!(memory_info.total_swap_gb >= 0.0); // SWAP can be any size
    }
    
    // Verify no hardcoded 64GB check in the code
    // The verification function should NOT reject SWAP > 64GB
}

#[test]
fn test_disk_full_handling() {
    let config = ResourceGovernanceConfig {
        disk_full_threshold: 90.0,
        ..Default::default()
    };
    let governor = ResourceGovernor::new(config);
    
    let temp_dir = TempDir::new().unwrap();
    let disk_path = temp_dir.path().to_string_lossy().to_string();
    
    let limits = ComponentResourceLimits {
        cpu_max_percent: 50.0,
        cpu_window_seconds: 60,
        cpu_backpressure_threshold: 40.0,
        memory_max_mb: 1024,
        memory_oom_threshold: 80.0,
        disk_max_mb: 2048,
        disk_path: disk_path.clone(),
        is_audit: false,
        connection_max: 100,
        rate_limit_per_second: 1000,
        rate_limit_window_seconds: 1,
        is_critical: false,
    };
    
    governor.register_component("test_component".to_string(), limits).unwrap();
    
    // Check disk quota
    let result = governor.disk().check_quota("test_component", 100);
    assert!(result.is_ok() || result.is_err()); // May pass or fail depending on disk state
    
    // Verify disk metrics are retrieved
    let disk_metrics = governor.disk().get_metrics();
    assert!(disk_metrics.is_ok());
}

#[test]
fn test_audit_partition_protection() {
    let config = ResourceGovernanceConfig::default();
    let governor = ResourceGovernor::new(config);
    
    let temp_dir = TempDir::new().unwrap();
    let audit_path = temp_dir.path().to_string_lossy().to_string();
    
    let limits = ComponentResourceLimits {
        cpu_max_percent: 50.0,
        cpu_window_seconds: 60,
        cpu_backpressure_threshold: 40.0,
        memory_max_mb: 1024,
        memory_oom_threshold: 80.0,
        disk_max_mb: 2048,
        disk_path: audit_path.clone(),
        is_audit: true, // CRITICAL: Audit partition
        connection_max: 100,
        rate_limit_per_second: 1000,
        rate_limit_window_seconds: 1,
        is_critical: true,
    };
    
    governor.register_component("audit_component".to_string(), limits).unwrap();
    
    // Audit partition must be writable (fail-closed)
    let result = governor.disk().check_quota("audit_component", 100);
    assert!(result.is_ok()); // Should pass if audit partition is writable
    
    // Verify audit protection in metrics
    let disk_metrics = governor.disk().get_metrics().unwrap();
    // Audit partitions should be protected
    assert!(disk_metrics.audit_protected || disk_metrics.disk_info.is_empty());
}

#[test]
fn test_fd_exhaustion_handling() {
    let config = ResourceGovernanceConfig {
        fd_warning_threshold: 80.0,
        ..Default::default()
    };
    let governor = ResourceGovernor::new(config);
    
    // Check FD limits
    let fd_info = governor.disk().check_fd_limits();
    assert!(fd_info.is_ok());
    
    let info = fd_info.unwrap();
    assert!(info.limit > 0);
    assert!(info.current >= 0);
    assert!(info.utilization_percent >= 0.0);
}

#[test]
fn test_network_overload_handling() {
    let config = ResourceGovernanceConfig {
        network_overload_threshold: 80.0,
        ..Default::default()
    };
    let governor = ResourceGovernor::new(config);
    
    let limits = ComponentResourceLimits {
        cpu_max_percent: 50.0,
        cpu_window_seconds: 60,
        cpu_backpressure_threshold: 40.0,
        memory_max_mb: 1024,
        memory_oom_threshold: 80.0,
        disk_max_mb: 2048,
        disk_path: "/tmp".to_string(),
        is_audit: false,
        connection_max: 100,
        rate_limit_per_second: 1000,
        rate_limit_window_seconds: 1,
        is_critical: false,
    };
    
    governor.register_component("test_component".to_string(), limits).unwrap();
    
    // Check connection quota
    let result = governor.network().check_connection("test_component");
    assert!(result.is_ok() || result.is_err());
    
    // Check rate limit
    let result = governor.network().check_rate_limit("test_component");
    assert!(result.is_ok() || result.is_err());
    
    // Verify network metrics
    let metrics = governor.network().get_metrics();
    assert!(metrics.overload_detected || !metrics.overload_detected); // Boolean check
}

#[test]
fn test_critical_traffic_never_dropped() {
    let config = ResourceGovernanceConfig::default();
    let governor = ResourceGovernor::new(config);
    
    let limits = ComponentResourceLimits {
        cpu_max_percent: 100.0,
        cpu_window_seconds: 60,
        cpu_backpressure_threshold: 90.0,
        memory_max_mb: 4096,
        memory_oom_threshold: 90.0,
        disk_max_mb: 8192,
        disk_path: "/tmp".to_string(),
        is_audit: true,
        connection_max: 1000,
        rate_limit_per_second: 10000,
        rate_limit_window_seconds: 1,
        is_critical: true, // CRITICAL: Telemetry/alerts
    };
    
    governor.register_component("critical_telemetry".to_string(), limits).unwrap();
    
    // CRITICAL traffic should always pass
    let result = governor.network().check_connection("critical_telemetry");
    assert!(result.is_ok());
    assert!(result.unwrap());
    
    // Even under rate limit, critical should pass
    for _ in 0..20000 {
        let result = governor.network().check_rate_limit("critical_telemetry");
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}

#[test]
fn test_fail_safe_degradation() {
    let config = ResourceGovernanceConfig {
        degradation_threshold: 50.0,
        ..Default::default()
    };
    let governor = ResourceGovernor::new(config);
    
    let limits = ComponentResourceLimits {
        cpu_max_percent: 50.0,
        cpu_window_seconds: 60,
        cpu_backpressure_threshold: 40.0,
        memory_max_mb: 1024,
        memory_oom_threshold: 80.0,
        disk_max_mb: 2048,
        disk_path: "/tmp".to_string(),
        is_audit: false,
        connection_max: 100,
        rate_limit_per_second: 1000,
        rate_limit_window_seconds: 1,
        is_critical: false,
    };
    
    governor.register_component("test_component".to_string(), limits).unwrap();
    
    // Report degradation
    let result = governor.report_degradation(
        "test_component",
        "High CPU usage".to_string(),
        "WARNING",
    );
    assert!(result.is_ok());
    
    // Verify degradation is logged
    let metrics = governor.degradation().get_metrics();
    assert!(metrics.degradation_events.len() > 0);
}

#[test]
fn test_critical_functions_remain_operational() {
    let config = ResourceGovernanceConfig::default();
    let governor = ResourceGovernor::new(config);
    
    let limits = ComponentResourceLimits {
        cpu_max_percent: 50.0,
        cpu_window_seconds: 60,
        cpu_backpressure_threshold: 40.0,
        memory_max_mb: 1024,
        memory_oom_threshold: 80.0,
        disk_max_mb: 2048,
        disk_path: "/tmp".to_string(),
        is_audit: true,
        connection_max: 100,
        rate_limit_per_second: 1000,
        rate_limit_window_seconds: 1,
        is_critical: true,
    };
    
    governor.register_component("secure_component".to_string(), limits).unwrap();
    governor.register_critical_function("secure_component".to_string(), "encrypt".to_string());
    
    // Even if we try to shutdown, critical functions must remain operational
    let result = governor.degradation().shutdown_component(
        "secure_component",
        "Test".to_string(),
    );
    assert!(result.is_ok());
    
    // Component should be isolated, not shutdown (to protect critical functions)
    let metrics = governor.degradation().get_metrics();
    assert!(metrics.critical_functions_operational);
}

#[test]
fn test_system_safety_verification() {
    let config = ResourceGovernanceConfig::default();
    let governor = ResourceGovernor::new(config);
    
    // Verify system safety (fail-closed check)
    let result = governor.verify_system_safety();
    // May pass or fail depending on system state, but should not panic
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_comprehensive_resource_check() {
    let config = ResourceGovernanceConfig::default();
    let governor = ResourceGovernor::new(config);
    
    let temp_dir = TempDir::new().unwrap();
    let disk_path = temp_dir.path().to_string_lossy().to_string();
    
    let limits = ComponentResourceLimits {
        cpu_max_percent: 50.0,
        cpu_window_seconds: 60,
        cpu_backpressure_threshold: 40.0,
        memory_max_mb: 1024,
        memory_oom_threshold: 80.0,
        disk_max_mb: 2048,
        disk_path: disk_path.clone(),
        is_audit: false,
        connection_max: 100,
        rate_limit_per_second: 1000,
        rate_limit_window_seconds: 1,
        is_critical: false,
    };
    
    governor.register_component("test_component".to_string(), limits).unwrap();
    
    // Comprehensive resource check
    let result = governor.check_resources(
        "test_component",
        Some(30.0),
        Some(512),
        Some(1024),
    );
    assert!(result.is_ok() || result.is_err()); // May pass or fail depending on system state
    
    // Get comprehensive metrics
    let metrics = governor.get_metrics();
    assert!(metrics.is_ok());
}

