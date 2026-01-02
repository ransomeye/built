// Path and File Name : /home/ransomeye/rebuild/core/reporting/src/deception_report.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Deception event reporting - generates reports with timeline, host sequence, artifact touched, confidence score, attacker ID

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::info;

use crate::errors::ReportingError;

/// Deception event for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeceptionReportEvent {
    pub event_id: String,
    pub event_type: String,
    pub artifact_id: String,
    pub artifact_path: String,
    pub artifact_type: String, // "honeyfile", "honeycredential", "fake_service"
    pub process_id: i32,
    pub process_name: String,
    pub user_id: u32,
    pub host_id: String,
    pub timestamp: DateTime<Utc>,
    pub severity: String,
    pub confidence_score: f64,
    pub attacker_session_id: Option<String>,
}

/// Deception report with full context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeceptionReport {
    pub report_id: String,
    pub created_at: DateTime<Utc>,
    pub timeline: Vec<DeceptionReportEvent>,
    pub host_sequence: Vec<String>,
    pub artifacts_touched: Vec<String>,
    pub attacker_sessions: Vec<AttackerSession>,
    pub confidence_scores: HashMap<String, f64>,
    pub summary: DeceptionReportSummary,
}

/// Attacker session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackerSession {
    pub session_id: String,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub hosts: Vec<String>,
    pub artifacts: Vec<String>,
    pub event_count: u64,
}

/// Deception report summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeceptionReportSummary {
    pub total_events: usize,
    pub unique_hosts: usize,
    pub unique_artifacts: usize,
    pub unique_attackers: usize,
    pub high_severity_count: usize,
    pub critical_severity_count: usize,
    pub average_confidence: f64,
    pub time_span_start: Option<DateTime<Utc>>,
    pub time_span_end: Option<DateTime<Utc>>,
}

/// Deception report builder
pub struct DeceptionReportBuilder {
    events: Vec<DeceptionReportEvent>,
}

impl DeceptionReportBuilder {
    /// Create new deception report builder
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }
    
    /// Add deception event
    pub fn add_event(&mut self, event: DeceptionReportEvent) {
        self.events.push(event);
    }
    
    /// Build deception report
    pub fn build(self) -> DeceptionReport {
        // Sort events by timestamp
        let mut events = self.events;
        events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        // Extract host sequence
        let mut host_sequence = Vec::new();
        for event in &events {
            if !host_sequence.contains(&event.host_id) {
                host_sequence.push(event.host_id.clone());
            }
        }
        
        // Extract artifacts touched
        let mut artifacts_touched = Vec::new();
        for event in &events {
            if !artifacts_touched.contains(&event.artifact_id) {
                artifacts_touched.push(event.artifact_id.clone());
            }
        }
        
        // Build attacker sessions
        let mut sessions_map: HashMap<String, Vec<&DeceptionReportEvent>> = HashMap::new();
        for event in &events {
            if let Some(ref session_id) = event.attacker_session_id {
                sessions_map.entry(session_id.clone())
                    .or_insert_with(Vec::new)
                    .push(event);
            }
        }
        
        let mut attacker_sessions = Vec::new();
        for (session_id, session_events) in sessions_map {
            let event_count = session_events.len();
            let mut hosts = Vec::new();
            let mut artifacts = Vec::new();
            let mut first_seen = session_events[0].timestamp;
            let mut last_seen = session_events[0].timestamp;
            
            for event in &session_events {
                if !hosts.contains(&event.host_id) {
                    hosts.push(event.host_id.clone());
                }
                if !artifacts.contains(&event.artifact_id) {
                    artifacts.push(event.artifact_id.clone());
                }
                if event.timestamp < first_seen {
                    first_seen = event.timestamp;
                }
                if event.timestamp > last_seen {
                    last_seen = event.timestamp;
                }
            }
            
            attacker_sessions.push(AttackerSession {
                session_id: session_id.clone(),
                first_seen,
                last_seen,
                hosts,
                artifacts,
                event_count: event_count as u64,
            });
        }
        
        // Calculate confidence scores
        let mut confidence_scores: HashMap<String, f64> = HashMap::new();
        for event in &events {
            let key = format!("{}_{}", event.artifact_id, event.event_type);
            confidence_scores.insert(key, event.confidence_score);
        }
        
        // Calculate summary
        let high_severity_count = events.iter()
            .filter(|e| e.severity == "HIGH")
            .count();
        let critical_severity_count = events.iter()
            .filter(|e| e.severity == "CRITICAL")
            .count();
        let average_confidence = if events.is_empty() {
            0.0
        } else {
            events.iter().map(|e| e.confidence_score).sum::<f64>() / events.len() as f64
        };
        
        let summary = DeceptionReportSummary {
            total_events: events.len(),
            unique_hosts: host_sequence.len(),
            unique_artifacts: artifacts_touched.len(),
            unique_attackers: attacker_sessions.len(),
            high_severity_count,
            critical_severity_count,
            average_confidence,
            time_span_start: events.first().map(|e| e.timestamp),
            time_span_end: events.last().map(|e| e.timestamp),
        };
        
        DeceptionReport {
            report_id: format!("deception_{}", uuid::Uuid::new_v4().to_string()),
            created_at: Utc::now(),
            timeline: events,
            host_sequence,
            artifacts_touched,
            attacker_sessions,
            confidence_scores,
            summary,
        }
    }
    
    /// Export report to JSON
    pub fn export_json(report: &DeceptionReport, output_path: impl AsRef<Path>) -> Result<(), ReportingError> {
        let json = serde_json::to_string_pretty(report)
            .map_err(|e| ReportingError::SerializationError(e))?;
        
        std::fs::write(output_path, json)
            .map_err(|e| ReportingError::IoError(e))?;
        
        info!("Exported deception report to JSON");
        Ok(())
    }
    
    /// Export report to CSV
    pub fn export_csv(report: &DeceptionReport, output_path: impl AsRef<Path>) -> Result<(), ReportingError> {
        use csv::Writer;
        
        let mut wtr = Writer::from_path(output_path)
            .map_err(|e| ReportingError::IoError(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to create CSV writer: {}", e))))?;
        
        // Write header
        wtr.write_record(&[
            "Event ID",
            "Timestamp",
            "Event Type",
            "Artifact ID",
            "Artifact Path",
            "Artifact Type",
            "Host ID",
            "Process Name",
            "User ID",
            "Severity",
            "Confidence Score",
            "Attacker Session ID",
        ])
        .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
        
        // Write events
        for event in &report.timeline {
            wtr.write_record(&[
                &event.event_id,
                &event.timestamp.to_rfc3339(),
                &event.event_type,
                &event.artifact_id,
                &event.artifact_path,
                &event.artifact_type,
                &event.host_id,
                &event.process_name,
                &event.user_id.to_string(),
                &event.severity,
                &event.confidence_score.to_string(),
                &event.attacker_session_id.as_ref().map(|s| s.clone()).unwrap_or_else(|| "N/A".to_string()),
            ])
            .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to write CSV: {}", e)))?;
        }
        
        wtr.flush()
            .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to flush CSV: {}", e)))?;
        
        info!("Exported deception report to CSV");
        Ok(())
    }
    
    /// Export report to HTML
    pub fn export_html(report: &DeceptionReport, output_path: impl AsRef<Path>) -> Result<(), ReportingError> {
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>RansomEye Deception Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        h1 {{ color: #2c3e50; }}
        h2 {{ color: #34495e; margin-top: 30px; }}
        table {{ border-collapse: collapse; width: 100%; margin-top: 20px; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #3498db; color: white; }}
        tr:nth-child(even) {{ background-color: #f2f2f2; }}
        .summary {{ background-color: #ecf0f1; padding: 15px; border-radius: 5px; margin-top: 20px; }}
        .critical {{ color: #e74c3c; font-weight: bold; }}
        .high {{ color: #e67e22; font-weight: bold; }}
        .footer {{ margin-top: 40px; padding-top: 20px; border-top: 1px solid #ddd; text-align: center; color: #7f8c8d; }}
    </style>
</head>
<body>
    <h1>RansomEye Deception Report</h1>
    <p><strong>Report ID:</strong> {}</p>
    <p><strong>Created:</strong> {}</p>
    
    <div class="summary">
        <h2>Summary</h2>
        <p><strong>Total Events:</strong> {}</p>
        <p><strong>Unique Hosts:</strong> {}</p>
        <p><strong>Unique Artifacts:</strong> {}</p>
        <p><strong>Unique Attackers:</strong> {}</p>
        <p><strong>High Severity:</strong> {}</p>
        <p><strong>Critical Severity:</strong> {}</p>
        <p><strong>Average Confidence:</strong> {:.2}%</p>
    </div>
    
    <h2>Host Sequence</h2>
    <ol>
        {}
    </ol>
    
    <h2>Artifacts Touched</h2>
    <ul>
        {}
    </ul>
    
    <h2>Timeline</h2>
    <table>
        <tr>
            <th>Timestamp</th>
            <th>Event Type</th>
            <th>Artifact</th>
            <th>Host</th>
            <th>Process</th>
            <th>Severity</th>
            <th>Confidence</th>
        </tr>
        {}
    </table>
    
    <h2>Attacker Sessions</h2>
    <table>
        <tr>
            <th>Session ID</th>
            <th>First Seen</th>
            <th>Last Seen</th>
            <th>Hosts</th>
            <th>Artifacts</th>
            <th>Event Count</th>
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
            report.summary.total_events,
            report.summary.unique_hosts,
            report.summary.unique_artifacts,
            report.summary.unique_attackers,
            report.summary.high_severity_count,
            report.summary.critical_severity_count,
            report.summary.average_confidence * 100.0,
            report.host_sequence.iter().map(|h| format!("<li>{}</li>", h)).collect::<Vec<_>>().join("\n        "),
            report.artifacts_touched.iter().map(|a| format!("<li>{}</li>", a)).collect::<Vec<_>>().join("\n        "),
            report.timeline.iter().map(|e| {
                let severity_class = if e.severity == "CRITICAL" { "critical" } else { "high" };
                format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td class=\"{}\">{}</td><td>{:.2}%</td></tr>",
                    e.timestamp.to_rfc3339(),
                    e.event_type,
                    e.artifact_id,
                    e.host_id,
                    e.process_name,
                    severity_class,
                    e.severity,
                    e.confidence_score * 100.0
                )
            }).collect::<Vec<_>>().join("\n        "),
            report.attacker_sessions.iter().map(|s| {
                format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                    s.session_id,
                    s.first_seen.to_rfc3339(),
                    s.last_seen.to_rfc3339(),
                    s.hosts.join(", "),
                    s.artifacts.join(", "),
                    s.event_count
                )
            }).collect::<Vec<_>>().join("\n        "),
            Utc::now().to_rfc3339()
        );
        
        std::fs::write(output_path, html)
            .map_err(|e| ReportingError::IoError(e))?;
        
        info!("Exported deception report to HTML");
        Ok(())
    }
}

impl Default for DeceptionReportBuilder {
    fn default() -> Self {
        Self::new()
    }
}

