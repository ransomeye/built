# Reporting Formats

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_reporting/docs/reporting_formats.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Reporting formats documentation - describes PDF, HTML, and CSV export formats

---

## Overview

RansomEye reports are exported in **multiple formats** to support different use cases:

- **PDF**: Formatted reports for printing and archival
- **HTML**: Interactive web reports for viewing
- **CSV**: Machine-readable data for analysis

---

## Common Requirements

All export formats must include:

1. **Report Metadata**: ID, creation time, engine version, policy version, build hash
2. **Evidence References**: Bundle IDs and hashes
3. **Footer**: "© RansomEye.Tech | Support: Gagan@RansomEye.Tech"
4. **Generation Timestamp**: When report was generated (UTC)
5. **Reproducibility**: Reports must be reproducible from stored evidence

---

## PDF Format

### Structure

- **Title Page**: Report title, metadata, creation date
- **Executive Summary**: High-level overview
- **Forensic Timeline**: Chronological event sequence
- **Evidence Bundles**: Details of each evidence bundle
- **Evidence Hashes**: SHA-256 hashes of all bundles
- **Footer**: Branding and support information

### Features

- **Formatted Layout**: Professional formatting for printing
- **Page Breaks**: Automatic page breaks for long content
- **Metadata Section**: Complete version and build information
- **Hash Display**: All evidence hashes included

---

## HTML Format

### Structure

- **Header**: RansomEye branding
- **Metadata Table**: Report metadata in table format
- **Description**: Report description
- **Summary**: Summary statistics
- **Evidence Hashes**: List of bundle hashes
- **Sections**: Report sections with subsections
- **Footer**: Branding and generation timestamp

### Features

- **Interactive**: Clickable links and navigation
- **Styled**: Professional CSS styling
- **Responsive**: Works on different screen sizes
- **Embedded Styles**: All styles embedded in HTML

---

## CSV Format

### Structure

- **Metadata Rows**: Report metadata as key-value pairs
- **Evidence Bundle IDs**: List of bundle IDs
- **Evidence Hashes**: List of bundle hashes
- **Machine-Readable**: Structured for programmatic processing

### Features

- **Tabular Format**: Easy to import into spreadsheets
- **Key-Value Pairs**: Metadata as field-value pairs
- **List Format**: Arrays as separate rows
- **UTF-8 Encoding**: Supports international characters

---

## Branding

All formats include:

- **Footer Text**: "© RansomEye.Tech | Support: Gagan@RansomEye.Tech"
- **Generation Timestamp**: UTC timestamp of report generation
- **Version Information**: Engine version, policy version, build hash

---

## Reproducibility

Reports are **reproducible**: given the same evidence bundles, the same report is generated. This ensures:

- **Audit Compliance**: Reports can be regenerated for audits
- **Verification**: Reports can be verified against evidence
- **Consistency**: Same evidence always produces same report

---

## Export Process

1. **Build Report**: Construct report from evidence bundles
2. **Verify Reproducibility**: Ensure report is reproducible
3. **Export Formats**: Generate PDF, HTML, and CSV
4. **Validate Exports**: Verify all exports are valid

All exports are generated **synchronously** and **atomically**: either all formats succeed or all fail.

