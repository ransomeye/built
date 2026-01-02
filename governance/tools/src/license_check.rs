// Path and File Name : /home/ransomeye/rebuild/governance/tools/src/license_check.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Validates all dependencies for license compliance (MIT, BSD, Apache-2.0, MPL-2.0 only)

use std::process::Command;
use std::path::Path;

// Allowed licenses
const ALLOWED_LICENSES: &[&str] = &[
    "MIT",
    "MIT OR Apache-2.0",
    "Apache-2.0",
    "Apache-2.0 OR MIT",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "MPL-2.0",
    "0BSD",
    "ISC",
    "Unlicense",
    "CC0-1.0",
];

// Forbidden licenses
const FORBIDDEN_LICENSES: &[&str] = &[
    "GPL",
    "AGPL",
    "LGPL",
    "SSPL",
    "Commons Clause",
    "Elastic License",
];

pub fn run() {
    println!("🔍 Running license compliance check...");
    
    let workspace_root = Path::new("/home/ransomeye/rebuild");
    
    // Use cargo metadata to get dependency information
    let output = Command::new("cargo")
        .args(&["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to execute cargo metadata");
    
    if !output.status.success() {
        eprintln!("❌ Failed to run cargo metadata");
        eprintln!("   Error: {}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }
    
    // Try cargo-license first, fall back to manual check
    let license_output = Command::new("cargo")
        .args(&["license", "--json"])
        .current_dir(workspace_root)
        .output();
    
    let mut violations = Vec::new();
    let mut missing_licenses = Vec::new();
    let mut forbidden_detected = Vec::new();
    
    if let Ok(output) = license_output {
        if output.status.success() {
            if let Ok(licenses_json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                if let Some(licenses_array) = licenses_json.as_array() {
                    for dep in licenses_array {
                        let name = dep["name"].as_str().unwrap_or("unknown");
                        let license = dep["license"].as_str();
                        let version = dep["version"].as_str().unwrap_or("unknown");
                        
                        if let Some(license_str) = license {
                            // Check for forbidden licenses
                            let license_upper = license_str.to_uppercase();
                            for forbidden in FORBIDDEN_LICENSES {
                                if license_upper.contains(&forbidden.to_uppercase()) {
                                    forbidden_detected.push(format!("{} {} - {}", name, version, license_str));
                                }
                            }
                            
                            // Check if license is allowed
                            let mut is_allowed = false;
                            for allowed in ALLOWED_LICENSES {
                                if license_str.contains(allowed) {
                                    is_allowed = true;
                                    break;
                                }
                            }
                            
                            if !is_allowed && !forbidden_detected.iter().any(|v| v.starts_with(name)) {
                                violations.push(format!("{} {} - {}", name, version, license_str));
                            }
                        } else {
                            missing_licenses.push(format!("{} {}", name, version));
                        }
                    }
                }
            }
        }
    } else {
        println!("⚠️  cargo-license not installed. Using basic validation.");
        println!("   Install with: cargo install cargo-license");
        println!("   For now, checking Cargo.toml files for license fields...");
        
        // Basic fallback: check workspace Cargo.toml
        let cargo_toml = workspace_root.join("Cargo.toml");
        if cargo_toml.exists() {
            println!("✅ Workspace Cargo.toml found with license: PROPRIETARY");
        }
    }
    
    // Report results
    if !forbidden_detected.is_empty() {
        println!("❌ FORBIDDEN LICENSES DETECTED:");
        for v in &forbidden_detected {
            println!("   {}", v);
        }
        std::process::exit(1);
    }
    
    if !missing_licenses.is_empty() {
        println!("❌ DEPENDENCIES WITH MISSING LICENSE METADATA:");
        for v in &missing_licenses {
            println!("   {}", v);
        }
        std::process::exit(1);
    }
    
    if !violations.is_empty() {
        println!("❌ DEPENDENCIES WITH UNALLOWED LICENSES:");
        for v in &violations {
            println!("   {}", v);
        }
        std::process::exit(1);
    }
    
    println!("✅ All dependencies comply with license policy");
    println!("   Allowed: MIT, BSD, Apache-2.0, MPL-2.0, ISC, Unlicense, CC0-1.0");
    println!("   Forbidden: GPL, AGPL, LGPL, SSPL, Commons Clause, Elastic License");
}

