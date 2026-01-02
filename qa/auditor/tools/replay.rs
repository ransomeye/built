// Path and File Name : /home/ransomeye/rebuild/ransomeye_validation/tools/replay.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Standalone replay tool binary - replays events for determinism validation

use auditor::replay::ReplayEngine;
use std::env;
use std::path::PathBuf;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: replay <log_path> <command>");
        eprintln!("Commands:");
        eprintln!("  replay - Replay all events");
        eprintln!("  validate - Validate determinism");
        std::process::exit(1);
    }
    
    let log_path = PathBuf::from(&args[1]);
    let command = &args[2];
    
    let mut replay_engine = ReplayEngine::new();
    replay_engine.load_events(&log_path)?;
    
    match command.as_str() {
        "replay" => {
            let results = replay_engine.replay_all().await?;
            println!("Replayed {} events", results.len());
            for result in &results {
                if !result.matches {
                    eprintln!("Mismatch for event {}: {:?}", result.event_id, result.divergence_point);
                }
            }
        }
        "validate" => {
            let results = replay_engine.replay_all().await?;
            match replay_engine.validate_determinism(&results) {
                Ok(_) => {
                    println!("Determinism validation: PASS");
                }
                Err(e) => {
                    eprintln!("Determinism validation: FAIL - {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            std::process::exit(1);
        }
    }
    
    Ok(())
}

