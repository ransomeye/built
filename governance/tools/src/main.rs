// Path and File Name : /home/ransomeye/rebuild/governance/tools/src/main.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Main entry point for governance tools (license_check, header_check)

use std::path::Path;

mod license_check;
mod header_check;

fn main() {
    // Determine which binary was invoked
    let binary_name = std::env::args()
        .next()
        .and_then(|path| {
            Path::new(&path)
                .file_name()
                .and_then(|name| name.to_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());
    
    match binary_name.as_str() {
        "license_check" => license_check::run(),
        "header_check" => header_check::run(),
        _ => {
            eprintln!("Unknown binary: {}", binary_name);
            eprintln!("Expected: license_check or header_check");
            std::process::exit(1);
        }
    }
}

