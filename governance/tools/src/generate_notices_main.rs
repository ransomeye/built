// Path and File Name : /home/ransomeye/rebuild/governance/tools/src/generate_notices_main.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Main entry point for generate_notices binary

mod generate_notices;

fn main() {
    if let Err(e) = generate_notices::generate() {
        eprintln!("❌ Error: {}", e);
        std::process::exit(1);
    }
}

