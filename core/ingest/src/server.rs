// Path and File Name : /home/ransomeye/rebuild/ransomeye_ingestion/src/server.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Main ingestion server - coordinates all ingestion components

/*
 * Ingestion Server
 * 
 * Coordinates authentication, signature verification, schema validation,
 * rate limiting, backpressure, and event dispatch.
 */

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error, warn};

use crate::listener::EventListener;
use crate::auth::Authenticator;
use crate::signature::SignatureVerifier;
use crate::schema::SchemaValidator;
use crate::rate_limit::RateLimiter;
use crate::backpressure::BackpressureController;
use crate::buffer::EventBuffer;
use crate::ordering::OrderingManager;
use crate::dedupe::ContentDeduplicator;
use crate::dispatcher::EventDispatcher;
use crate::config::Config;
use crate::security::{TrustStore, IdentityVerifier, TrustChainValidator, RevocationChecker, ReplayProtector};

pub struct IngestionServer {
    config: Config,
    listener: Arc<EventListener>,
    authenticator: Arc<Authenticator>,
    signature_verifier: Arc<SignatureVerifier>,
    schema_validator: Arc<SchemaValidator>,
    rate_limiter: Arc<RateLimiter>,
    backpressure: Arc<BackpressureController>,
    buffer: Arc<EventBuffer>,
    ordering: Arc<OrderingManager>,
    dispatcher: Arc<EventDispatcher>,
    shutdown: Arc<RwLock<bool>>,
}

impl IngestionServer {
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Initializing Ingestion Server");
        
        // Initialize security components
        let trust_store = Arc::new(TrustStore::new(&config.trust_store_path)?);
        trust_store.initialize()?;
        
        let trust_chain_validator = Arc::new(TrustChainValidator::new(trust_store.clone())?);
        let revocation_checker = Arc::new(RevocationChecker::new(trust_store.clone(), config.crl_path.clone())?);
        let replay_protector = Arc::new(ReplayProtector::new()?);
        
        let identity_verifier = Arc::new(IdentityVerifier::new(
            trust_store.clone(),
            trust_chain_validator.clone(),
            revocation_checker.clone(),
        )?);
        
        // Initialize components
        let authenticator = Arc::new(Authenticator::new(&config, identity_verifier.clone(), replay_protector.clone())?);
        let signature_verifier = Arc::new(SignatureVerifier::new(&config)?);
        let schema_validator = Arc::new(SchemaValidator::new(&config)?);
        let rate_limiter = Arc::new(RateLimiter::new(&config)?);
        let backpressure = Arc::new(BackpressureController::new(&config)?);
        let buffer = Arc::new(EventBuffer::new(&config)?);
        let ordering = Arc::new(OrderingManager::new(&config)?);
        let deduplicator = Arc::new(ContentDeduplicator::new(&config)?);
        let dispatcher = Arc::new(EventDispatcher::new(&config)?);
        
        // Initialize listener
        let listener = Arc::new(EventListener::new(
            config.clone(),
            authenticator.clone(),
            signature_verifier.clone(),
            schema_validator.clone(),
            rate_limiter.clone(),
            backpressure.clone(),
            buffer.clone(),
            ordering.clone(),
            deduplicator.clone(),
            dispatcher.clone(),
        ).await?);
        
        Ok(Self {
            config,
            listener,
            authenticator,
            signature_verifier,
            schema_validator,
            rate_limiter,
            backpressure,
            buffer,
            ordering,
            dispatcher,
            shutdown: Arc::new(RwLock::new(false)),
        })
    }
    
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting Ingestion Server on {}", self.config.listen_address);
        
        // Start listener
        self.listener.start().await?;
        
        // Wait for shutdown
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            let shutdown = *self.shutdown.read().await;
            if shutdown {
                break;
            }
        }
        
        Ok(())
    }
    
    pub async fn shutdown(&self) {
        info!("Shutting down Ingestion Server");
        *self.shutdown.write().await = true;
        
        // Shutdown listener
        self.listener.shutdown().await;
        
        // Flush buffer
        self.buffer.flush().await;
        
        info!("Ingestion Server shutdown complete");
    }
}

