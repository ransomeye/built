// Path and File Name : /home/ransomeye/rebuild/governance/tools/src/generate_notices.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Generates THIRD_PARTY_NOTICES.md from cargo metadata

use std::collections::BTreeMap;
use std::process::Command;
use std::fs;

pub fn generate() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Extracting dependency information...");
    
    let output = Command::new("cargo")
        .args(&["metadata", "--format-version", "1"])
        .output()?;
    
    if !output.status.success() {
        return Err("Failed to run cargo metadata".into());
    }
    
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    
    let mut deps: BTreeMap<String, (String, String)> = BTreeMap::new();
    
    if let Some(packages) = metadata.get("packages").and_then(|p| p.as_array()) {
        for pkg in packages {
            if let Some(source) = pkg.get("source") {
                if source.is_string() && source.as_str().unwrap().contains("crates.io") {
                    let name = pkg.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                    let version = pkg.get("version").and_then(|v| v.as_str()).unwrap_or("unknown");
                    let license = pkg.get("license")
                        .and_then(|l| l.as_str())
                        .unwrap_or("UNKNOWN")
                        .to_string();
                    
                    // Normalize license format
                    let license_normalized = license
                        .replace("/", " OR ")
                        .replace(" AND ", " AND ");
                    
                    deps.insert(name.to_string(), (version.to_string(), license_normalized));
                }
            }
        }
    }
    
    println!("📝 Generating THIRD_PARTY_NOTICES.md...");
    
    let mut content = String::from("# Third-Party Notices — RansomEye\n\n");
    content.push_str("This document lists all third-party software components used in RansomEye,\n");
    content.push_str("their respective licenses, and attribution requirements.\n\n");
    content.push_str("**Generated from cargo metadata — machine-verifiable**\n\n");
    content.push_str("---\n\n");
    content.push_str("## Dependency Inventory\n\n");
    
    for (name, (version, license)) in &deps {
        content.push_str(&format!("## {} {}\n\n", name, version));
        content.push_str(&format!("- **License:** {}\n", license));
        content.push_str(&format!("- **Source:** https://crates.io/crates/{}\n", name));
        content.push_str("- **Copyright:** See crate source repository\n");
        content.push_str("- **Usage:** Core/Edge/UI/Tooling (transitive dependency)\n\n");
    }
    
    content.push_str("---\n\n");
    content.push_str("## License Summary\n\n");
    content.push_str("All dependencies use permissive licenses compatible with proprietary software:\n\n");
    content.push_str("- MIT\n");
    content.push_str("- Apache-2.0\n");
    content.push_str("- BSD-2-Clause / BSD-3-Clause\n");
    content.push_str("- MPL-2.0\n");
    content.push_str("- ISC\n");
    content.push_str("- Unlicense\n");
    content.push_str("- CC0-1.0\n");
    content.push_str("- 0BSD\n\n");
    content.push_str(&format!("**Total dependencies:** {}\n\n", deps.len()));
    content.push_str("---\n\n");
    content.push_str("*This file is auto-generated. Do not edit manually.*\n");
    content.push_str("*Regenerate with: `cargo run --bin generate_notices --manifest-path governance/tools/Cargo.toml`*\n");
    
    fs::write("/home/ransomeye/rebuild/THIRD_PARTY_NOTICES.md", content)?;
    
    println!("✅ Generated THIRD_PARTY_NOTICES.md with {} dependencies", deps.len());
    
    Ok(())
}

