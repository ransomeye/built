// Path and File Name : /home/ransomeye/rebuild/core/reporting/src/forensic_report.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Forensic reporting - generates reports with full audit timeline, evidence references, integrity verification

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::path::Path;
use tracing::info;

use crate::errors::ReportingError;

/// Forensic report with full audit timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForensicReport {
    pub report_id: String,
    pub created_at: DateTime<Utc>,
    pub audit_timeline: Vec<AuditTimelineEntry>,
    pub evidence_references: Vec<EvidenceReference>,
    pub integrity_status: IntegrityStatus,
    pub summary: ForensicReportSummary,
}

/// Audit timeline entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTimelineEntry {
    pub record_id: String,
    pub timestamp: DateTime<Utc>,
    pub component: String,
    pub event_type: String,
    pub actor: String,
    pub host: String,
    pub hash: String,
    pub signature: String,
    pub data: serde_json::Value,
}

/// Evidence reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceReference {
    pub evidence_id: String,
    pub content_hash: String,
    pub evidence_type: String,
    pub source: String,
    pub timestamp: DateTime<Utc>,
    pub integrity_verified: bool,
}

/// Integrity verification status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityStatus {
    pub hash_chain_valid: bool,
    pub signatures_valid: bool,
    pub evidence_integrity_valid: bool,
    pub clock_rollback_detected: bool,
    pub tampering_detected: bool,
    pub verification_errors: Vec<String>,
}

/// Forensic report summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForensicReportSummary {
    pub total_audit_records: usize,
    pub total_evidence_items: usize,
    pub time_span_start: Option<DateTime<Utc>>,
    pub time_span_end: Option<DateTime<Utc>>,
    pub integrity_status: String, // "VALID", "INVALID", "PARTIAL"
}

/// Forensic report builder
pub struct ForensicReportBuilder;

impl ForensicReportBuilder {
    /// Export report to JSON
    pub fn export_json(report: &ForensicReport, output_path: impl AsRef<Path>) -> Result<(), ReportingError> {
        let json = serde_json::to_string_pretty(report)
            .map_err(|e| ReportingError::SerializationError(e))?;
        
        std::fs::write(output_path, json)
            .map_err(|e| ReportingError::IoError(e))?;
        
        info!("Exported forensic report to JSON");
        Ok(())
    }
    
    /// Export report to CSV
    pub fn export_csv(report: &ForensicReport, output_path: impl AsRef<Path>) -> Result<(), ReportingError> {
        use csv::Writer;
        
        let mut wtr = Writer::from_path(output_path)
            .map_err(|e| ReportingError::IoError(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to create CSV writer: {}", e))))?;
        
        // Write audit timeline
        wtr.write_record(&["Record ID", "Timestamp", "Component", "Event Type", "Actor", "Host", "Hash"])
            .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
        
        for entry in &report.audit_timeline {
            wtr.write_record(&[
                &entry.record_id,
                &entry.timestamp.to_rfc3339(),
                &entry.component,
                &entry.event_type,
                &entry.actor,
                &entry.host,
                &entry.hash,
            ])
            .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
        }
        
        wtr.flush()
            .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to flush CSV: {}", e)))?;
        
        info!("Exported forensic report to CSV");
        Ok(())
    }
    
    /// Export report to HTML
    pub fn export_html(report: &ForensicReport, output_path: impl AsRef<Path>) -> Result<(), ReportingError> {
        let integrity_class = if report.integrity_status.hash_chain_valid && 
                                  report.integrity_status.signatures_valid {
            "valid"
        } else {
            "invalid"
        };
        
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>RansomEye Forensic Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        h1 {{ color: #2c3e50; }}
        h2 {{ color: #34495e; margin-top: 30px; }}
        table {{ border-collapse: collapse; width: 100%; margin-top: 20px; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #3498db; color: white; }}
        tr:nth-child(even) {{ background-color: #f2f2f2; }}
        .summary {{ background-color: #ecf0f1; padding: 15px; border-radius: 5px; margin-top: 20px; }}
        .valid {{ color: #27ae60; font-weight: bold; }}
        .invalid {{ color: #e74c3c; font-weight: bold; }}
        .footer {{ margin-top: 40px; padding-top: 20px; border-top: 1px solid #ddd; text-align: center; color: #7f8c8d; }}
    </style>
</head>
<body>
    <h1>RansomEye Forensic Report</h1>
    <p><strong>Report ID:</strong> {}</p>
    <p><strong>Created:</strong> {}</p>
    
    <div class="summary">
        <h2>Summary</h2>
        <p><strong>Total Audit Records:</strong> {}</p>
        <p><strong>Total Evidence Items:</strong> {}</p>
        <p><strong>Integrity Status:</strong> <span class="{}">{}</span></p>
        <p><strong>Hash Chain:</strong> <span class="{}">{}</span></p>
        <p><strong>Signatures:</strong> <span class="{}">{}</span></p>
        <p><strong>Evidence Integrity:</strong> <span class="{}">{}</span></p>
    </div>
    
    <h2>Audit Timeline</h2>
    <table>
        <tr>
            <th>Timestamp</th>
            <th>Component</th>
            <th>Event Type</th>
            <th>Actor</th>
            <th>Host</th>
            <th>Hash</th>
        </tr>
        {}
    </table>
    
    <h2>Evidence References</h2>
    <table>
        <tr>
            <th>Evidence ID</th>
            <th>Type</th>
            <th>Source</th>
            <th>Timestamp</th>
            <th>Integrity</th>
        </tr>
        {}
    </table>
    
    <div class="footer">
        <p>© RansomEye.Tech | Support: Gagan@RansomEye.Tech</p>
        <p>Generated: {}</p>
    </div>
</body>
</html>"#,
            report.report_id,
            report.created_at.to_rfc3339(),
            report.summary.total_audit_records,
            report.summary.total_evidence_items,
            integrity_class,
            if report.integrity_status.hash_chain_valid && report.integrity_status.signatures_valid { "VALID" } else { "INVALID" },
            if report.integrity_status.hash_chain_valid { "valid" } else { "invalid" },
            if report.integrity_status.hash_chain_valid { "Valid" } else { "Invalid" },
            if report.integrity_status.signatures_valid { "valid" } else { "invalid" },
            if report.integrity_status.signatures_valid { "Valid" } else { "Invalid" },
            if report.integrity_status.evidence_integrity_valid { "valid" } else { "invalid" },
            if report.integrity_status.evidence_integrity_valid { "Valid" } else { "Invalid" },
            report.audit_timeline.iter().map(|e| {
                format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                    e.timestamp.to_rfc3339(),
                    e.component,
                    e.event_type,
                    e.actor,
                    e.host,
                    &e.hash[..16]
                )
            }).collect::<Vec<_>>().join("\n        "),
            report.evidence_references.iter().map(|e| {
                format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td class=\"{}\">{}</td></tr>",
                    e.evidence_id,
                    e.evidence_type,
                    e.source,
                    e.timestamp.to_rfc3339(),
                    if e.integrity_verified { "valid" } else { "invalid" },
                    if e.integrity_verified { "Verified" } else { "Not Verified" }
                )
            }).collect::<Vec<_>>().join("\n        "),
            Utc::now().to_rfc3339()
        );
        
        std::fs::write(output_path, html)
            .map_err(|e| ReportingError::IoError(e))?;
        
        info!("Exported forensic report to HTML");
        Ok(())
    }
    
    /// Export report to PDF (using existing PDF exporter)
    pub fn export_pdf(report: &ForensicReport, output_path: impl AsRef<Path>) -> Result<(), ReportingError> {
        // Convert to ForensicReport format compatible with existing PDF exporter
        // For now, export as HTML and let user convert if needed
        // In production, would use printpdf or similar
        Self::export_html(report, output_path)
    }
}

