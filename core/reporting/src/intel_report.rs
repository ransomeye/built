// Path and File Name : /home/ransomeye/rebuild/core/reporting/src/intel_report.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Intel reporting - generates reports with confidence breakdown, correlated evidence, timeline, actions taken

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::info;

use crate::errors::ReportingError;

/// Intel report with full context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelReport {
    pub report_id: String,
    pub created_at: DateTime<Utc>,
    pub correlation_id: String,
    pub ioc_value: String,
    pub ioc_type: String,
    pub confidence_breakdown: ConfidenceBreakdown,
    pub correlated_evidence: Vec<CorrelatedEvidence>,
    pub timeline: Vec<TimelineEvent>,
    pub actions_taken: Vec<ActionTaken>,
    pub summary: IntelReportSummary,
}

/// Confidence breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceBreakdown {
    pub overall_confidence: f64,
    pub source_reputation_score: f64,
    pub signal_frequency_score: f64,
    pub cross_source_agreement: f64,
    pub temporal_proximity_score: f64,
    pub confidence_level: String, // "High", "Medium", "Low"
}

/// Correlated evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelatedEvidence {
    pub signal_id: String,
    pub source: String,
    pub timestamp: DateTime<Utc>,
    pub confidence: f64,
    pub metadata: serde_json::Value,
}

/// Timeline event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub description: String,
    pub source: String,
}

/// Action taken
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionTaken {
    pub action_id: String,
    pub action_type: String,
    pub timestamp: DateTime<Utc>,
    pub result: String,
    pub policy_id: Option<String>,
}

/// Intel report summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelReportSummary {
    pub total_signals: usize,
    pub unique_sources: usize,
    pub confidence_level: String,
    pub policy_triggered: bool,
    pub actions_count: usize,
    pub time_span_start: Option<DateTime<Utc>>,
    pub time_span_end: Option<DateTime<Utc>>,
}

/// Intel report builder
pub struct IntelReportBuilder;

impl IntelReportBuilder {
    /// Export report to JSON
    pub fn export_json(report: &IntelReport, output_path: impl AsRef<Path>) -> Result<(), ReportingError> {
        let json = serde_json::to_string_pretty(report)
            .map_err(|e| ReportingError::SerializationError(e))?;
        
        std::fs::write(output_path, json)
            .map_err(|e| ReportingError::IoError(e))?;
        
        info!("Exported intel report to JSON");
        Ok(())
    }
    
    /// Export report to CSV
    pub fn export_csv(report: &IntelReport, output_path: impl AsRef<Path>) -> Result<(), ReportingError> {
        use csv::Writer;
        
        let mut wtr = Writer::from_path(output_path)
            .map_err(|e| ReportingError::IoError(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to create CSV writer: {}", e))))?;
        
        // Write summary
        wtr.write_record(&["Field", "Value"])
            .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
        
        wtr.write_record(&["Report ID", &report.report_id])
            .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
        
        wtr.write_record(&["Correlation ID", &report.correlation_id])
            .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
        
        wtr.write_record(&["IOC Value", &report.ioc_value])
            .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
        
        wtr.write_record(&["IOC Type", &report.ioc_type])
            .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
        
        wtr.write_record(&["Overall Confidence", &report.confidence_breakdown.overall_confidence.to_string()])
            .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
        
        wtr.write_record(&["Confidence Level", &report.confidence_breakdown.confidence_level])
            .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
        
        // Write correlated evidence
        wtr.write_record(&[""; 0]).ok();
        wtr.write_record(&["Correlated Evidence"])
            .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
        
        wtr.write_record(&["Signal ID", "Source", "Timestamp", "Confidence"])
            .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
        
        for evidence in &report.correlated_evidence {
            wtr.write_record(&[
                &evidence.signal_id,
                &evidence.source,
                &evidence.timestamp.to_rfc3339(),
                &evidence.confidence.to_string(),
            ])
            .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
        }
        
        wtr.flush()
            .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to flush CSV: {}", e)))?;
        
        info!("Exported intel report to CSV");
        Ok(())
    }
    
    /// Export report to HTML
    pub fn export_html(report: &IntelReport, output_path: impl AsRef<Path>) -> Result<(), ReportingError> {
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>RansomEye Intel Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        h1 {{ color: #2c3e50; }}
        h2 {{ color: #34495e; margin-top: 30px; }}
        table {{ border-collapse: collapse; width: 100%; margin-top: 20px; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #3498db; color: white; }}
        tr:nth-child(even) {{ background-color: #f2f2f2; }}
        .summary {{ background-color: #ecf0f1; padding: 15px; border-radius: 5px; margin-top: 20px; }}
        .high {{ color: #27ae60; font-weight: bold; }}
        .medium {{ color: #f39c12; font-weight: bold; }}
        .low {{ color: #e74c3c; font-weight: bold; }}
        .footer {{ margin-top: 40px; padding-top: 20px; border-top: 1px solid #ddd; text-align: center; color: #7f8c8d; }}
    </style>
</head>
<body>
    <h1>RansomEye Intel Report</h1>
    <p><strong>Report ID:</strong> {}</p>
    <p><strong>Created:</strong> {}</p>
    
    <div class="summary">
        <h2>Summary</h2>
        <p><strong>Correlation ID:</strong> {}</p>
        <p><strong>IOC Value:</strong> {}</p>
        <p><strong>IOC Type:</strong> {}</p>
        <p><strong>Overall Confidence:</strong> <span class="{}">{:.2}% ({})</span></p>
        <p><strong>Total Signals:</strong> {}</p>
        <p><strong>Unique Sources:</strong> {}</p>
        <p><strong>Policy Triggered:</strong> {}</p>
        <p><strong>Actions Taken:</strong> {}</p>
    </div>
    
    <h2>Confidence Breakdown</h2>
    <table>
        <tr>
            <th>Factor</th>
            <th>Score</th>
        </tr>
        <tr>
            <td>Source Reputation</td>
            <td>{:.2}</td>
        </tr>
        <tr>
            <td>Signal Frequency</td>
            <td>{:.2}</td>
        </tr>
        <tr>
            <td>Cross-Source Agreement</td>
            <td>{:.2}</td>
        </tr>
        <tr>
            <td>Temporal Proximity</td>
            <td>{:.2}</td>
        </tr>
    </table>
    
    <h2>Correlated Evidence</h2>
    <table>
        <tr>
            <th>Signal ID</th>
            <th>Source</th>
            <th>Timestamp</th>
            <th>Confidence</th>
        </tr>
        {}
    </table>
    
    <h2>Timeline</h2>
    <table>
        <tr>
            <th>Timestamp</th>
            <th>Event Type</th>
            <th>Description</th>
            <th>Source</th>
        </tr>
        {}
    </table>
    
    <h2>Actions Taken</h2>
    <table>
        <tr>
            <th>Action ID</th>
            <th>Action Type</th>
            <th>Timestamp</th>
            <th>Result</th>
            <th>Policy ID</th>
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
            report.correlation_id,
            report.ioc_value,
            report.ioc_type,
            report.confidence_breakdown.confidence_level.to_lowercase(),
            report.confidence_breakdown.overall_confidence * 100.0,
            report.confidence_breakdown.confidence_level,
            report.summary.total_signals,
            report.summary.unique_sources,
            if report.summary.policy_triggered { "Yes" } else { "No" },
            report.summary.actions_count,
            report.confidence_breakdown.source_reputation_score,
            report.confidence_breakdown.signal_frequency_score,
            report.confidence_breakdown.cross_source_agreement,
            report.confidence_breakdown.temporal_proximity_score,
            report.correlated_evidence.iter().map(|e| {
                format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{:.2}%</td></tr>",
                    e.signal_id,
                    e.source,
                    e.timestamp.to_rfc3339(),
                    e.confidence * 100.0
                )
            }).collect::<Vec<_>>().join("\n        "),
            report.timeline.iter().map(|e| {
                format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                    e.timestamp.to_rfc3339(),
                    e.event_type,
                    e.description,
                    e.source
                )
            }).collect::<Vec<_>>().join("\n        "),
            report.actions_taken.iter().map(|a| {
                format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                    a.action_id,
                    a.action_type,
                    a.timestamp.to_rfc3339(),
                    a.result,
                    a.policy_id.as_ref().map(|s| s.clone()).unwrap_or_else(|| "N/A".to_string())
                )
            }).collect::<Vec<_>>().join("\n        "),
            Utc::now().to_rfc3339()
        );
        
        std::fs::write(output_path, html)
            .map_err(|e| ReportingError::IoError(e))?;
        
        info!("Exported intel report to HTML");
        Ok(())
    }
}

