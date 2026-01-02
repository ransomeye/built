// Path and File Name : /home/ransomeye/rebuild/ransomeye_reporting/src/formats/html.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: HTML export - generates interactive HTML reports with proper styling and evidence references

use std::fs;
use std::path::Path;
use chrono::Utc;

use crate::errors::ReportingError;
use crate::report_builder::ForensicReport;

const FOOTER_TEXT: &str = "© RansomEye.Tech | Support: Gagan@RansomEye.Tech";

pub fn export_html(
    report: &ForensicReport,
    output_path: impl AsRef<Path>,
) -> Result<(), ReportingError> {
    let mut html = String::new();
    
    html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str("<title>RansomEye Forensic Report</title>\n");
    html.push_str("<style>\n");
    html.push_str(include_str!("html_styles.css"));
    html.push_str("</style>\n");
    html.push_str("</head>\n<body>\n");
    
    // Header
    html.push_str("<div class=\"header\">\n");
    html.push_str("<h1>RansomEye Forensic Report</h1>\n");
    html.push_str("</div>\n");
    
    // Report Title
    html.push_str(&format!("<h2>{}</h2>\n", escape_html(&report.title)));
    
    // Metadata
    html.push_str("<div class=\"metadata\">\n");
    html.push_str("<h3>Report Metadata</h3>\n");
    html.push_str("<table>\n");
    html.push_str(&format!("<tr><td>Report ID:</td><td>{}</td></tr>\n", escape_html(&report.metadata.report_id)));
    html.push_str(&format!("<tr><td>Created:</td><td>{}</td></tr>\n", report.metadata.created_at.to_rfc3339()));
    html.push_str(&format!("<tr><td>Engine Version:</td><td>{}</td></tr>\n", escape_html(&report.metadata.engine_version)));
    html.push_str(&format!("<tr><td>Policy Version:</td><td>{}</td></tr>\n", escape_html(&report.metadata.policy_version)));
    html.push_str(&format!("<tr><td>Build Hash:</td><td><code>{}</code></td></tr>\n", escape_html(&report.metadata.build_hash)));
    if let Some(ref model_hash) = report.metadata.model_version_hash {
        html.push_str(&format!("<tr><td>Model Version Hash:</td><td><code>{}</code></td></tr>\n", escape_html(model_hash)));
    }
    html.push_str("</table>\n");
    html.push_str("</div>\n");
    
    // Description
    html.push_str("<div class=\"section\">\n");
    html.push_str("<h3>Description</h3>\n");
    html.push_str(&format!("<p>{}</p>\n", escape_html(&report.description)));
    html.push_str("</div>\n");
    
    // Summary
    html.push_str("<div class=\"section\">\n");
    html.push_str("<h3>Summary</h3>\n");
    html.push_str("<table>\n");
    html.push_str(&format!("<tr><td>Total Evidence Items:</td><td>{}</td></tr>\n", report.summary.total_evidence_items));
    if let Some(start) = report.summary.time_range_start {
        html.push_str(&format!("<tr><td>Time Range Start:</td><td>{}</td></tr>\n", start.to_rfc3339()));
    }
    if let Some(end) = report.summary.time_range_end {
        html.push_str(&format!("<tr><td>Time Range End:</td><td>{}</td></tr>\n", end.to_rfc3339()));
    }
    html.push_str("</table>\n");
    html.push_str("</div>\n");
    
    // Evidence Hashes
    html.push_str("<div class=\"section\">\n");
    html.push_str("<h3>Evidence Bundle Hashes</h3>\n");
    html.push_str("<ul>\n");
    for hash in &report.evidence_hashes {
        html.push_str(&format!("<li><code>{}</code></li>\n", escape_html(hash)));
    }
    html.push_str("</ul>\n");
    html.push_str("</div>\n");
    
    // Sections
    for section in &report.sections {
        html.push_str("<div class=\"section\">\n");
        html.push_str(&format!("<h3>{}</h3>\n", escape_html(&section.title)));
        html.push_str(&format!("<p>{}</p>\n", escape_html(&section.content)));
        html.push_str("</div>\n");
    }
    
    // Footer
    html.push_str("<div class=\"footer\">\n");
    html.push_str(&format!("<p>{}</p>\n", FOOTER_TEXT));
    html.push_str(&format!("<p>Generated: {}</p>\n", Utc::now().to_rfc3339()));
    html.push_str("</div>\n");
    
    html.push_str("</body>\n</html>\n");
    
    fs::write(output_path, html)
        .map_err(|e| ReportingError::IoError(e))?;
    
    Ok(())
}

fn escape_html(s: &str) -> String {
    s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#x27;")
}

