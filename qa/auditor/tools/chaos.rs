// Path and File Name : /home/ransomeye/rebuild/ransomeye_validation/tools/chaos.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Standalone chaos engineering tool binary - injects faults for validation testing

use auditor::chaos::ChaosEngine;
use std::env;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: chaos <command> [args...]");
        eprintln!("Commands:");
        eprintln!("  crash <service> - Crash and restart a service");
        eprintln!("  partition <seconds> - Inject network partition");
        eprintln!("  memory <mb> - Exhaust memory");
        eprintln!("  disk <mb> - Exhaust disk");
        eprintln!("  clock <seconds> - Inject clock skew");
        eprintln!("  revoke <cert_path> - Revoke certificate");
        std::process::exit(1);
    }
    
    let chaos = ChaosEngine::new(true);
    let command = &args[1];
    
    match command.as_str() {
        "crash" => {
            if args.len() < 3 {
                eprintln!("Usage: chaos crash <service_name>");
                std::process::exit(1);
            }
            chaos.crash_service(&args[2]).await?;
        }
        "partition" => {
            if args.len() < 3 {
                eprintln!("Usage: chaos partition <seconds>");
                std::process::exit(1);
            }
            let seconds: u64 = args[2].parse()?;
            chaos.inject_network_partition(seconds).await?;
        }
        "memory" => {
            if args.len() < 3 {
                eprintln!("Usage: chaos memory <mb>");
                std::process::exit(1);
            }
            let mb: u64 = args[2].parse()?;
            chaos.exhaust_memory(mb).await?;
        }
        "disk" => {
            if args.len() < 3 {
                eprintln!("Usage: chaos disk <mb>");
                std::process::exit(1);
            }
            let mb: u64 = args[2].parse()?;
            chaos.exhaust_disk(mb).await?;
        }
        "clock" => {
            if args.len() < 3 {
                eprintln!("Usage: chaos clock <seconds>");
                std::process::exit(1);
            }
            let seconds: i64 = args[2].parse()?;
            chaos.inject_clock_skew(seconds).await?;
        }
        "revoke" => {
            if args.len() < 3 {
                eprintln!("Usage: chaos revoke <cert_path>");
                std::process::exit(1);
            }
            chaos.revoke_certificate(&args[2]).await?;
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            std::process::exit(1);
        }
    }
    
    Ok(())
}

