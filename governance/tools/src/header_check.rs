// Path and File Name : /home/ransomeye/rebuild/governance/tools/src/header_check.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Validates mandatory file headers across all source files

use std::fs;
use std::path::Path;
use walkdir::WalkDir;

const REQUIRED_EXTENSIONS: &[&str] = &[
    ".rs", ".py", ".sh", ".ts", ".tsx", ".yaml", ".json", ".service",
];

const EXCLUDED_PATTERNS: &[&str] = &[
    "target",
    ".git",
    "node_modules",
    "venv",
    ".venv",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    "dist",
    "build",
    "Cargo.lock",
    "package-lock.json",
    ".md",
];

const REQUIRED_AUTHOR: &str = "nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU";

pub fn run() {
    println!("🔍 Running file header compliance check...");
    
    let workspace_root = Path::new("/home/ransomeye/rebuild");
    let mut violations = Vec::new();
    
    for entry in WalkDir::new(workspace_root)
        .into_iter()
        .filter_entry(|e| {
            let path_str = e.path().to_string_lossy();
            !EXCLUDED_PATTERNS.iter().any(|pattern| path_str.contains(pattern))
        })
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        
        let path = entry.path();
        
        if !path.is_file() {
            continue;
        }
        
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        if !REQUIRED_EXTENSIONS.contains(&extension) {
            continue;
        }
        
        // Read first few lines to check header
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        
        let lines: Vec<&str> = content.lines().take(5).collect();
        
        if lines.len() < 3 {
            violations.push(format!("{} - Missing header (file too short)", path.display()));
            continue;
        }
        
        // Check for required header format
        let has_path = lines.iter().any(|l| {
            l.trim_start().starts_with("// Path and File Name") || 
            l.trim_start().starts_with("# Path and File Name")
        });
        
        let has_author = lines.iter().any(|l| {
            l.contains(REQUIRED_AUTHOR)
        });
        
        let has_details = lines.iter().any(|l| {
            l.trim_start().starts_with("// Details of functionality") ||
            l.trim_start().starts_with("# Details of functionality")
        });
        
        if !has_path || !has_author || !has_details {
            let mut issues = Vec::new();
            if !has_path { issues.push("missing Path and File Name"); }
            if !has_author { issues.push("missing or incorrect Author"); }
            if !has_details { issues.push("missing Details of functionality"); }
            
            violations.push(format!("{} - {}", path.display(), issues.join(", ")));
        }
    }
    
    if !violations.is_empty() {
        println!("❌ FILES WITH MISSING OR INVALID HEADERS:");
        for v in &violations {
            println!("   {}", v);
        }
        std::process::exit(1);
    }
    
    println!("✅ All source files have valid headers");
}

