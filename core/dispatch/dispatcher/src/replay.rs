// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/dispatcher/src/replay.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Replay protection using directive ID and nonce tracking

use super::replay_protection::ReplayProtector;
use super::nonce::NonceTracker;
use super::directive_envelope::DirectiveEnvelope;
use super::errors::DispatcherError;
use tracing::debug;

pub struct ReplayGuard {
    replay_protector: ReplayProtector,
    nonce_tracker: NonceTracker,
}

impl ReplayGuard {
    pub fn new(replay_protector: ReplayProtector, nonce_tracker: NonceTracker) -> Self {
        Self {
            replay_protector,
            nonce_tracker,
        }
    }
    
    /// Check if directive is a replay
    pub fn check_replay(&self, directive: &DirectiveEnvelope) -> Result<(), DispatcherError> {
        // Check directive ID replay
        if !self.replay_protector.is_new(&directive.directive_id) {
            return Err(DispatcherError::ReplayDetected(directive.directive_id.clone()));
        }
        
        // Check nonce replay
        if !self.nonce_tracker.is_fresh(&directive.nonce) {
            return Err(DispatcherError::NonceReplay(directive.nonce.clone()));
        }
        
        debug!("Replay check passed for directive {}", directive.directive_id);
        Ok(())
    }
}
