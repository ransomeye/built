// Path and File Name : /home/ransomeye/rebuild/ransomeye_reporting/src/exporter.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Report exporter - exports reports in multiple formats (PDF, HTML, CSV) with proper branding and metadata

use std::path::Path;
use tracing::{debug, error};

use crate::errors::ReportingError;
use crate::report_builder::ForensicReport;
use crate::formats;

/// Report exporter - handles export to multiple formats
pub struct ReportExporter;

impl ReportExporter {
    pub fn new() -> Self {
        Self
    }
    
    /// Export report to PDF
    pub fn export_pdf(
        &self,
        report: &ForensicReport,
        output_path: impl AsRef<Path>,
    ) -> Result<(), ReportingError> {
        debug!("Exporting report {} to PDF", report.metadata.report_id);
        formats::pdf::export_pdf(report, output_path)
    }
    
    /// Export report to HTML
    pub fn export_html(
        &self,
        report: &ForensicReport,
        output_path: impl AsRef<Path>,
    ) -> Result<(), ReportingError> {
        debug!("Exporting report {} to HTML", report.metadata.report_id);
        formats::html::export_html(report, output_path)
    }
    
    /// Export report to CSV
    pub fn export_csv(
        &self,
        report: &ForensicReport,
        output_path: impl AsRef<Path>,
    ) -> Result<(), ReportingError> {
        debug!("Exporting report {} to CSV", report.metadata.report_id);
        formats::csv::export_csv(report, output_path)
    }
    
    /// Export report to all formats
    pub fn export_all(
        &self,
        report: &ForensicReport,
        base_path: impl AsRef<Path>,
    ) -> Result<Vec<String>, ReportingError> {
        let base = base_path.as_ref();
        let report_id = &report.metadata.report_id;
        
        let mut exported_files = Vec::new();
        
        // Export PDF
        let pdf_path = base.join(format!("{}_report.pdf", report_id));
        match self.export_pdf(report, &pdf_path) {
            Ok(()) => {
                exported_files.push(pdf_path.to_string_lossy().to_string());
            }
            Err(e) => {
                error!("Failed to export PDF: {}", e);
                return Err(e);
            }
        }
        
        // Export HTML
        let html_path = base.join(format!("{}_report.html", report_id));
        match self.export_html(report, &html_path) {
            Ok(()) => {
                exported_files.push(html_path.to_string_lossy().to_string());
            }
            Err(e) => {
                error!("Failed to export HTML: {}", e);
                return Err(e);
            }
        }
        
        // Export CSV
        let csv_path = base.join(format!("{}_report.csv", report_id));
        match self.export_csv(report, &csv_path) {
            Ok(()) => {
                exported_files.push(csv_path.to_string_lossy().to_string());
            }
            Err(e) => {
                error!("Failed to export CSV: {}", e);
                return Err(e);
            }
        }
        
        debug!("Exported report {} to {} formats", report_id, exported_files.len());
        Ok(exported_files)
    }
}

impl Default for ReportExporter {
    fn default() -> Self {
        Self::new()
    }
}

