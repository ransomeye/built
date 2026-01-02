// Path and File Name : /home/ransomeye/rebuild/ransomeye_ingestion/src/listener.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Event listener - receives events from producers and processes them through the ingestion pipeline

/*
 * Event Listener
 * 
 * Receives events from producers (DPI Probe, Linux Agent, Windows Agent).
 * Processes events through authentication, signature verification, schema validation,
 * rate limiting, backpressure, and dispatch pipeline.
 */

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, error, warn, debug};

use crate::auth::Authenticator;
use crate::signature::SignatureVerifier;
use crate::schema::SchemaValidator;
use crate::rate_limit::RateLimiter;
use crate::backpressure::BackpressureController;
use crate::buffer::EventBuffer;
use crate::ordering::OrderingManager;
use crate::dedupe::ContentDeduplicator;
use crate::dispatcher::EventDispatcher;
use crate::protocol::event_envelope::EventEnvelope;
use crate::config::Config;

pub struct EventListener {
    config: Config,
    authenticator: Arc<Authenticator>,
    signature_verifier: Arc<SignatureVerifier>,
    schema_validator: Arc<SchemaValidator>,
    rate_limiter: Arc<RateLimiter>,
    backpressure: Arc<BackpressureController>,
    buffer: Arc<EventBuffer>,
    ordering: Arc<OrderingManager>,
    deduplicator: Arc<ContentDeduplicator>,
    dispatcher: Arc<EventDispatcher>,
    listener: Arc<RwLock<Option<TcpListener>>>,
    shutdown: Arc<RwLock<bool>>,
}

impl EventListener {
    pub async fn new(
        config: Config,
        authenticator: Arc<Authenticator>,
        signature_verifier: Arc<SignatureVerifier>,
        schema_validator: Arc<SchemaValidator>,
        rate_limiter: Arc<RateLimiter>,
        backpressure: Arc<BackpressureController>,
        buffer: Arc<EventBuffer>,
        ordering: Arc<OrderingManager>,
        deduplicator: Arc<ContentDeduplicator>,
        dispatcher: Arc<EventDispatcher>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            config,
            authenticator,
            signature_verifier,
            schema_validator,
            rate_limiter,
            backpressure,
            buffer,
            ordering,
            deduplicator,
            dispatcher,
            listener: Arc::new(RwLock::new(None)),
            shutdown: Arc::new(RwLock::new(false)),
        })
    }
    
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(&self.config.listen_address).await?;
        *self.listener.write().await = Some(listener);
        
        let listener_guard = self.listener.read().await;
        let listener = listener_guard.as_ref().unwrap();
        
        info!("Event listener started on {}", self.config.listen_address);
        
        loop {
            // Check shutdown
            if *self.shutdown.read().await {
                break;
            }
            
            // Accept connection
            match listener.accept().await {
                Ok((stream, addr)) => {
                    debug!("New connection from {}", addr);
                    
                    // Spawn handler
                    let authenticator = self.authenticator.clone();
                    let signature_verifier = self.signature_verifier.clone();
                    let schema_validator = self.schema_validator.clone();
                    let rate_limiter = self.rate_limiter.clone();
                    let backpressure = self.backpressure.clone();
                    let buffer = self.buffer.clone();
                    let ordering = self.ordering.clone();
                    let deduplicator = self.deduplicator.clone();
                    let dispatcher = self.dispatcher.clone();
                    
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(
                            stream,
                            addr,
                            authenticator,
                            signature_verifier,
                            schema_validator,
                            rate_limiter,
                            backpressure,
                            buffer,
                            ordering,
                            deduplicator,
                            dispatcher,
                        ).await {
                            error!("Connection handler error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_connection(
        mut stream: tokio::net::TcpStream,
        addr: std::net::SocketAddr,
        authenticator: Arc<Authenticator>,
        signature_verifier: Arc<SignatureVerifier>,
        schema_validator: Arc<SchemaValidator>,
        rate_limiter: Arc<RateLimiter>,
        backpressure: Arc<BackpressureController>,
        buffer: Arc<EventBuffer>,
        ordering: Arc<OrderingManager>,
        deduplicator: Arc<ContentDeduplicator>,
        dispatcher: Arc<EventDispatcher>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Read event data
        let mut data = Vec::new();
        stream.read_to_end(&mut data).await?;
        
        // Parse event envelope
        let envelope: EventEnvelope = serde_json::from_slice(&data)?;
        
        // Step 1: Authenticate producer
        let producer_id = authenticator.authenticate(&envelope, &addr).await?;
        
        // Step 2: Verify signature
        signature_verifier.verify(&envelope).await?;
        
        // Step 3: Validate schema
        schema_validator.validate(&envelope).await?;
        
        // Step 3.5: Content hash deduplication (before normalization side effects)
        if !deduplicator.should_process(&envelope).await? {
            // Duplicate content - drop deterministically
            let response = b"DUPLICATE_CONTENT\n";
            stream.write_all(response).await?;
            return Err("Duplicate content detected".into());
        }
        
        // Step 4: Check rate limit (priority-aware, including component quota)
        if !rate_limiter.check_limit(&producer_id, &envelope.component_type, &envelope.priority).await? {
            // Rate limit exceeded - send backpressure signal
            backpressure.signal_backpressure(&producer_id).await;
            let response = b"RATE_LIMIT_EXCEEDED\n";
            stream.write_all(response).await?;
            return Err("Rate limit exceeded".into());
        }
        
        // Step 5: Check backpressure
        if !backpressure.can_accept(&producer_id).await {
            // Backpressure active - reject event
            let response = b"BACKPRESSURE_ACTIVE\n";
            stream.write_all(response).await?;
            return Err("Backpressure active".into());
        }
        
        // Step 6: Check buffer capacity
        if !buffer.has_capacity().await {
            // Buffer full - reject event
            backpressure.signal_backpressure(&producer_id).await;
            let response = b"BUFFER_FULL\n";
            stream.write_all(response).await?;
            return Err("Buffer full".into());
        }
        
        // Step 7: Check ordering
        if !ordering.check_ordering(&producer_id, &envelope).await? {
            // Out of order or replay - reject
            let response = b"ORDERING_VIOLATION\n";
            stream.write_all(response).await?;
            return Err("Ordering violation".into());
        }
        
        // Step 8: Add to buffer
        buffer.add(&envelope).await?;
        
        // Step 9: Dispatch to Control Plane
        dispatcher.dispatch(&envelope).await?;
        
        // Success response
        let response = b"ACCEPTED\n";
        stream.write_all(response).await?;
        
        Ok(())
    }
    
    pub async fn shutdown(&self) {
        *self.shutdown.write().await = true;
        info!("Event listener shutdown");
    }
}

