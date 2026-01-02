// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/security/revocation.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Policy revocation checking with persistent storage

use std::collections::HashSet;
use std::path::Path;
use std::fs;
use std::io::{BufRead, BufWriter, Write};
use tracing::{debug, warn};
use parking_lot::RwLock;
use once_cell::sync::Lazy;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
struct RevokedPolicy {
    policy_id: String,
    revoked_at: DateTime<Utc>,
    reason: String,
}

static REVOCATION_LIST: Lazy<Arc<RwLock<RevocationList>>> = Lazy::new(|| {
    Arc::new(RwLock::new(RevocationList::new()))
});

pub struct RevocationList {
    revoked_policies: HashSet<String>,
    revocation_records: Vec<RevokedPolicy>,
}

impl RevocationList {
    fn new() -> Self {
        Self {
            revoked_policies: HashSet::new(),
            revocation_records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = Path::new(path);
        if !file_path.exists() {
            debug!("Revocation list file not found, starting with empty list");
            return Ok(());
        }

        let file = fs::File::open(file_path)?;
        let reader = std::io::BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 1 {
                let policy_id = parts[0].trim().to_string();
                self.revoked_policies.insert(policy_id);
            }
        }

        debug!("Loaded {} revoked policies from {}", self.revoked_policies.len(), path);
        Ok(())
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = fs::File::create(path)?;
        let mut writer = BufWriter::new(file);

        writeln!(writer, "# Policy Revocation List")?;
        writeln!(writer, "# Format: policy_id,revoked_at,reason")?;

        for record in &self.revocation_records {
            writeln!(writer, "{},{},{}", 
                record.policy_id, 
                record.revoked_at.to_rfc3339(),
                record.reason.replace(',', ";")
            )?;
        }

        writer.flush()?;
        debug!("Saved revocation list to {}", path);
        Ok(())
    }

    pub fn revoke_policy(&mut self, policy_id: &str, reason: &str) {
        self.revoked_policies.insert(policy_id.to_string());
        self.revocation_records.push(RevokedPolicy {
            policy_id: policy_id.to_string(),
            revoked_at: Utc::now(),
            reason: reason.to_string(),
        });
        debug!("Revoked policy: {} (reason: {})", policy_id, reason);
    }

    pub fn is_revoked(&self, policy_id: &str) -> bool {
        self.revoked_policies.contains(policy_id)
    }
}

pub struct PolicyRevocationChecker {
    revocation_list_path: String,
}

impl PolicyRevocationChecker {
    pub fn new(revocation_list_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut list = REVOCATION_LIST.write();
        list.load_from_file(revocation_list_path)?;

        Ok(Self {
            revocation_list_path: revocation_list_path.to_string(),
        })
    }

    pub fn is_revoked(&self, policy_id: &str) -> bool {
        let list = REVOCATION_LIST.read();
        let revoked = list.is_revoked(policy_id);
        if revoked {
            warn!("Policy is revoked: {}", policy_id);
        }
        revoked
    }

    pub fn revoke_policy(&self, policy_id: &str, reason: &str) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut list = REVOCATION_LIST.write();
            list.revoke_policy(policy_id, reason);
        }

        let list = REVOCATION_LIST.read();
        list.save_to_file(&self.revocation_list_path)?;
        Ok(())
    }
}

use std::sync::Arc;

