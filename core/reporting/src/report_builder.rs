// Path and File Name : /home/ransomeye/rebuild/ransomeye_reporting/src/report_builder.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Report builder - constructs reproducible reports with evidence references, engine versions, and policy versions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::errors::ReportingError;
use crate::evidence_store::EvidenceBundle;
use crate::timeline::ForensicTimeline;

/// Report metadata - version information and build hashes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub report_id: String,
    pub created_at: DateTime<Utc>,
    pub engine_version: String,
    pub policy_version: String,
    pub build_hash: String,
    pub model_version_hash: Option<String>,
}

/// Forensic report - complete report with evidence references
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForensicReport {
    pub metadata: ReportMetadata,
    pub title: String,
    pub description: String,
    pub evidence_bundle_ids: Vec<String>,
    pub timeline: Option<ForensicTimeline>,
    pub summary: ReportSummary,
    pub sections: Vec<ReportSection>,
    pub evidence_hashes: Vec<String>,
    pub reproducible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total_evidence_items: usize,
    pub time_range_start: Option<DateTime<Utc>>,
    pub time_range_end: Option<DateTime<Utc>>,
    pub kill_chain_stages: Vec<String>,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    pub title: String,
    pub content: String,
    pub evidence_references: Vec<String>,
    pub subsections: Vec<ReportSection>,
}

/// Report builder - constructs reproducible reports
pub struct ReportBuilder {
    engine_version: String,
    policy_version: String,
    build_hash: String,
    model_version_hash: Option<String>,
}

impl ReportBuilder {
    pub fn new(
        engine_version: &str,
        policy_version: &str,
        build_hash: &str,
        model_version_hash: Option<&str>,
    ) -> Self {
        Self {
            engine_version: engine_version.to_string(),
            policy_version: policy_version.to_string(),
            build_hash: build_hash.to_string(),
            model_version_hash: model_version_hash.map(|s| s.to_string()),
        }
    }
    
    /// Build report from evidence bundles
    pub fn build_report(
        &self,
        title: &str,
        description: &str,
        bundles: &[EvidenceBundle],
        timeline: Option<ForensicTimeline>,
    ) -> Result<ForensicReport, ReportingError> {
        let report_id = uuid::Uuid::new_v4().to_string();
        let created_at = Utc::now();
        
        // Collect evidence bundle IDs and hashes
        let evidence_bundle_ids: Vec<String> = bundles.iter()
            .map(|b| b.bundle_id.clone())
            .collect();
        
        let evidence_hashes: Vec<String> = bundles.iter()
            .map(|b| b.bundle_hash.clone())
            .collect();
        
        // Build summary
        let mut time_ranges: Vec<(DateTime<Utc>, DateTime<Utc>)> = bundles.iter()
            .map(|b| (b.created_at, b.sealed_at.unwrap_or(b.created_at)))
            .collect();
        
        time_ranges.sort_by_key(|(start, _)| *start);
        
        let time_range_start = time_ranges.first().map(|(start, _)| *start);
        let time_range_end = time_ranges.last().map(|(_, end)| *end);
        
        let mut kill_chain_stages = std::collections::HashSet::new();
        let mut sources = std::collections::HashSet::new();
        let mut total_evidence_items = 0;
        
        for bundle in bundles {
            for evidence in &bundle.evidence_items {
                total_evidence_items += 1;
                sources.insert(evidence.source.clone());
                if let Some(stage) = &evidence.kill_chain_stage {
                    kill_chain_stages.insert(stage.clone());
                }
            }
        }
        
        let summary = ReportSummary {
            total_evidence_items,
            time_range_start,
            time_range_end,
            kill_chain_stages: kill_chain_stages.into_iter().collect(),
            sources: sources.into_iter().collect(),
        };
        
        // Build sections
        let sections = self.build_sections(bundles, &timeline)?;
        
        let metadata = ReportMetadata {
            report_id: report_id.clone(),
            created_at,
            engine_version: self.engine_version.clone(),
            policy_version: self.policy_version.clone(),
            build_hash: self.build_hash.clone(),
            model_version_hash: self.model_version_hash.clone(),
        };
        
        Ok(ForensicReport {
            metadata,
            title: title.to_string(),
            description: description.to_string(),
            evidence_bundle_ids,
            timeline,
            summary,
            sections,
            evidence_hashes,
            reproducible: true,
        })
    }
    
    /// Build report sections
    fn build_sections(
        &self,
        bundles: &[EvidenceBundle],
        timeline: &Option<ForensicTimeline>,
    ) -> Result<Vec<ReportSection>, ReportingError> {
        let mut sections = Vec::new();
        
        // Executive Summary
        sections.push(ReportSection {
            title: "Executive Summary".to_string(),
            content: format!(
                "This report contains {} evidence items from {} bundles. ",
                bundles.iter().map(|b| b.evidence_items.len()).sum::<usize>(),
                bundles.len()
            ),
            evidence_references: bundles.iter().map(|b| b.bundle_id.clone()).collect(),
            subsections: Vec::new(),
        });
        
        // Timeline Section
        if let Some(timeline) = timeline {
            let timeline_summary = timeline.get_summary();
            sections.push(ReportSection {
                title: "Forensic Timeline".to_string(),
                content: format!(
                    "Timeline contains {} events spanning from {} to {}. ",
                    timeline_summary.total_events,
                    timeline_summary.time_span_start
                        .map(|t| t.to_rfc3339())
                        .unwrap_or_else(|| "N/A".to_string()),
                    timeline_summary.time_span_end
                        .map(|t| t.to_rfc3339())
                        .unwrap_or_else(|| "N/A".to_string()),
                ),
                evidence_references: Vec::new(),
                subsections: Vec::new(),
            });
        }
        
        // Evidence Bundles Section
        let mut bundle_subsections = Vec::new();
        for bundle in bundles {
            bundle_subsections.push(ReportSection {
                title: format!("Bundle {}", bundle.bundle_id),
                content: format!(
                    "Bundle created at {} with {} evidence items. Hash: {}",
                    bundle.created_at.to_rfc3339(),
                    bundle.evidence_items.len(),
                    bundle.bundle_hash
                ),
                evidence_references: vec![bundle.bundle_id.clone()],
                subsections: Vec::new(),
            });
        }
        
        sections.push(ReportSection {
            title: "Evidence Bundles".to_string(),
            content: format!("Total bundles: {}", bundles.len()),
            evidence_references: bundles.iter().map(|b| b.bundle_id.clone()).collect(),
            subsections: bundle_subsections,
        });
        
        Ok(sections)
    }
    
    /// Verify report reproducibility
    /// A report is reproducible if all evidence bundles are sealed and verifiable
    pub fn verify_reproducibility(
        &self,
        report: &ForensicReport,
        bundles: &[EvidenceBundle],
    ) -> Result<bool, ReportingError> {
        // All bundles must be sealed
        for bundle in bundles {
            if !bundle.is_sealed {
                return Ok(false);
            }
        }
        
        // All evidence hashes must match
        let report_hashes: Vec<&String> = report.evidence_hashes.iter().collect();
        let bundle_hashes: Vec<&String> = bundles.iter()
            .map(|b| &b.bundle_hash)
            .collect();
        
        if report_hashes.len() != bundle_hashes.len() {
            return Ok(false);
        }
        
        for (report_hash, bundle_hash) in report_hashes.iter().zip(bundle_hashes.iter()) {
            if report_hash != bundle_hash {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
}

