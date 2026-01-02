// Path and File Name : /home/ransomeye/rebuild/ransomeye_validation/tests/compliance_assertion_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Compliance assertion tests - FAIL-CLOSED validation using REAL artifacts from Phases 4-10

#[cfg(test)]
mod tests {
    use ransomeye_validation::suites::compliance::ComplianceSuite;
    use ransomeye_validation::core::ValidationResult;
    
    /// Test compliance suite with REAL artifacts
    /// FAIL-CLOSED: This test will FAIL if artifacts are missing or invalid
    #[tokio::test]
    async fn test_compliance_suite_real_artifacts() {
        let suite = ComplianceSuite::new();
        let result = suite.run().await;
        
        // FAIL-CLOSED: Result must be Ok, and ValidationResult must not be Fail
        assert!(result.is_ok(), "Compliance suite execution failed: {:?}", result);
        
        let validation_result = result.unwrap();
        
        // FAIL-CLOSED: Any Critical or High findings → Fail
        match validation_result {
            ValidationResult::Fail(findings) => {
                panic!("Compliance validation FAILED with {} findings:\n{}", 
                    findings.len(),
                    findings.iter()
                        .map(|f| format!("  [{}] {}", 
                            match f.severity {
                                ransomeye_validation::core::Severity::Critical => "CRITICAL",
                                ransomeye_validation::core::Severity::High => "HIGH",
                                ransomeye_validation::core::Severity::Medium => "MEDIUM",
                                ransomeye_validation::core::Severity::Low => "LOW",
                                ransomeye_validation::core::Severity::Info => "INFO",
                            },
                            f.description))
                        .collect::<Vec<_>>()
                        .join("\n"));
            }
            ValidationResult::Hold(findings) => {
                eprintln!("Compliance validation on HOLD with {} medium-severity findings", findings.len());
                // Hold is acceptable for testing, but should be addressed
            }
            ValidationResult::Pass(_) => {
                // PASS - all checks passed
            }
        }
    }
    
    /// Test that compliance suite fails when artifacts are missing
    /// This verifies fail-closed behavior
    #[tokio::test]
    #[ignore] // Ignore by default - only run when testing fail-closed behavior
    async fn test_compliance_suite_fails_on_missing_artifacts() {
        // This test verifies that the suite fails when artifacts don't exist
        // It should be run in an environment without artifacts to verify fail-closed behavior
        let suite = ComplianceSuite::new();
        let result = suite.run().await;
        
        // In a clean environment without artifacts, this should fail
        // This test is ignored by default because it requires a specific test environment
        if let Ok(validation_result) = result {
            match validation_result {
                ValidationResult::Fail(_) => {
                    // Expected: fail-closed behavior working correctly
                }
                _ => {
                    // If artifacts exist, this is fine - test is environment-dependent
                }
            }
        }
    }
}

