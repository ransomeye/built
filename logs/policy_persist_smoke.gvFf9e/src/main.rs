use std::path::Path;

fn main() {
    tracing_subscriber::fmt::init();

    // Force the legacy path to prove the runtime remaps into the hardened-writable directory.
    std::env::set_var("RANSOMEYE_POLICY_VERSION_STATE_PATH", "/var/lib/ransomeye/policy_versions.json");

    let policies_dir = "/home/ransomeye/rebuild/core/policy/policies";
    let trust_store_dir = "/etc/ransomeye/trust_store";

    let engine = policy::PolicyEngine::new(
        policies_dir,
        "1.0.0",
        Some(trust_store_dir),
        None,
        None,
    );

    match engine {
        Ok(_) => {
            let new_path = Path::new("/var/lib/ransomeye/policy/policy_versions.json");
            let legacy_path = Path::new("/var/lib/ransomeye/policy_versions.json");

            if !new_path.exists() {
                eprintln!("FAIL: expected version state file not found at {}", new_path.display());
                std::process::exit(2);
            }
            if legacy_path.exists() {
                eprintln!("FAIL: legacy version state file should not be written: {}", legacy_path.display());
                std::process::exit(3);
            }

            let content = std::fs::read_to_string(new_path)
                .unwrap_or_else(|e| {
                    eprintln!("FAIL: could not read {}: {}", new_path.display(), e);
                    std::process::exit(4);
                });

            println!("OK: policy engine initialized");
            println!("OK: version state persisted at {}", new_path.display());
            println!("OK: version state bytes={}", content.len());
        }
        Err(e) => {
            eprintln!("FAIL: policy engine init error: {e}");
            std::process::exit(1);
        }
    }
}
