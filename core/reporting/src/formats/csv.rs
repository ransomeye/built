// Path and File Name : /home/ransomeye/rebuild/ransomeye_reporting/src/formats/csv.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: CSV export - generates machine-readable CSV reports with all evidence data

use std::fs::File;
use std::path::Path;
use csv::Writer;

use crate::errors::ReportingError;
use crate::report_builder::ForensicReport;

pub fn export_csv(
    report: &ForensicReport,
    output_path: impl AsRef<Path>,
) -> Result<(), ReportingError> {
    let mut wtr = Writer::from_path(output_path)
        .map_err(|e| ReportingError::IoError(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to create CSV writer: {}", e))))?;
    
    // Write metadata
    wtr.write_record(&["Field", "Value"])
        .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
    
    wtr.write_record(&["Report ID", &report.metadata.report_id])
        .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
    
    wtr.write_record(&["Title", &report.title])
        .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
    
    wtr.write_record(&["Created", &report.metadata.created_at.to_rfc3339()])
        .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
    
    wtr.write_record(&["Engine Version", &report.metadata.engine_version])
        .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
    
    wtr.write_record(&["Policy Version", &report.metadata.policy_version])
        .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
    
    wtr.write_record(&["Build Hash", &report.metadata.build_hash])
        .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
    
    if let Some(ref model_hash) = report.metadata.model_version_hash {
        wtr.write_record(&["Model Version Hash", model_hash])
            .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
    }
    
    wtr.write_record(&["Total Evidence Items", &report.summary.total_evidence_items.to_string()])
        .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
    
    // Empty row
    wtr.write_record(&["", ""])
        .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
    
    // Evidence Bundle IDs
    wtr.write_record(&["Evidence Bundle IDs", ""])
        .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
    
    for bundle_id in &report.evidence_bundle_ids {
        wtr.write_record(&["", bundle_id])
            .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
    }
    
    // Empty row
    wtr.write_record(&["", ""])
        .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
    
    // Evidence Hashes
    wtr.write_record(&["Evidence Bundle Hashes", ""])
        .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
    
    for hash in &report.evidence_hashes {
        wtr.write_record(&["", hash])
            .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
    }
    
    wtr.flush()
        .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to flush CSV: {}", e)))?;
    
    Ok(())
}

