// Path and File Name : /home/ransomeye/rebuild/ransomeye_reporting/src/formats/pdf.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: PDF export - generates PDF reports with proper formatting, branding, and evidence references

#![cfg(feature = "future-reporting")]

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use printpdf::*;
use chrono::Utc;

use crate::errors::ReportingError;
use crate::report_builder::ForensicReport;

const FOOTER_TEXT: &str = "© RansomEye.Tech | Support: Gagan@RansomEye.Tech";

pub fn export_pdf(
    report: &ForensicReport,
    output_path: impl AsRef<Path>,
) -> Result<(), ReportingError> {
    let (doc, page1, layer1) = PdfDocument::new("RansomEye Forensic Report", Mm(210.0), Mm(297.0), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);
    
    // Set up fonts
    let font = doc.add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to add font: {:?}", e)))?;
    let font_regular = doc.add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to add font: {:?}", e)))?;
    
    let mut y_position = 280.0;
    let line_height = 12.0;
    let margin = 20.0;
    
    // Title
    current_layer.use_text(&report.title, 24.0, Mm(margin), Mm(y_position), &font);
    y_position -= 30.0;
    
    // Metadata
    current_layer.use_text(
        &format!("Report ID: {}", report.metadata.report_id),
        10.0,
        Mm(margin),
        Mm(y_position),
        &font_regular,
    );
    y_position -= line_height;
    
    current_layer.use_text(
        &format!("Created: {}", report.metadata.created_at.to_rfc3339()),
        10.0,
        Mm(margin),
        Mm(y_position),
        &font_regular,
    );
    y_position -= line_height;
    
    current_layer.use_text(
        &format!("Engine Version: {}", report.metadata.engine_version),
        10.0,
        Mm(margin),
        Mm(y_position),
        &font_regular,
    );
    y_position -= line_height;
    
    current_layer.use_text(
        &format!("Policy Version: {}", report.metadata.policy_version),
        10.0,
        Mm(margin),
        Mm(y_position),
        &font_regular,
    );
    y_position -= line_height;
    
    current_layer.use_text(
        &format!("Build Hash: {}", report.metadata.build_hash),
        10.0,
        Mm(margin),
        Mm(y_position),
        &font_regular,
    );
    y_position -= line_height * 2.0;
    
    // Description
    current_layer.use_text("Description:", 12.0, Mm(margin), Mm(y_position), &font);
    y_position -= line_height;
    current_layer.use_text(&report.description, 10.0, Mm(margin), Mm(y_position), &font_regular);
    y_position -= line_height * 2.0;
    
    // Summary
    current_layer.use_text("Summary:", 12.0, Mm(margin), Mm(y_position), &font);
    y_position -= line_height;
    current_layer.use_text(
        &format!("Total Evidence Items: {}", report.summary.total_evidence_items),
        10.0,
        Mm(margin),
        Mm(y_position),
        &font_regular,
    );
    y_position -= line_height;
    
    if let Some(start) = report.summary.time_range_start {
        current_layer.use_text(
            &format!("Time Range Start: {}", start.to_rfc3339()),
            10.0,
            Mm(margin),
            Mm(y_position),
            &font_regular,
        );
        y_position -= line_height;
    }
    
    if let Some(end) = report.summary.time_range_end {
        current_layer.use_text(
            &format!("Time Range End: {}", end.to_rfc3339()),
            10.0,
            Mm(margin),
            Mm(y_position),
            &font_regular,
        );
        y_position -= line_height;
    }
    
    y_position -= line_height;
    
    // Evidence Hashes
    current_layer.use_text("Evidence Bundle Hashes:", 12.0, Mm(margin), Mm(y_position), &font);
    y_position -= line_height;
    
    for (i, hash) in report.evidence_hashes.iter().enumerate() {
        if y_position < 30.0 {
            // Add new page if needed
            let (page, layer) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
            let current_layer = doc.get_page(page).get_layer(layer);
            y_position = 280.0;
        }
        
        current_layer.use_text(
            &format!("  {}. {}", i + 1, hash),
            9.0,
            Mm(margin),
            Mm(y_position),
            &font_regular,
        );
        y_position -= line_height;
    }
    
    y_position -= line_height;
    
    // Footer on last page
    let last_layer = doc.get_page(page1).get_layer(layer1);
    last_layer.use_text(
        FOOTER_TEXT,
        9.0,
        Mm(margin),
        Mm(10.0),
        &font_regular,
    );
    last_layer.use_text(
        &format!("Generated: {}", Utc::now().to_rfc3339()),
        9.0,
        Mm(margin),
        Mm(5.0),
        &font_regular,
    );
    
    // Save PDF
    doc.save(&mut BufWriter::new(File::create(output_path)?))
        .map_err(|e| ReportingError::ReportGenerationFailed(format!("Failed to save PDF: {:?}", e)))?;
    
    Ok(())
}

