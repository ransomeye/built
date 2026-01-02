// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/src/health.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Health reporter - reports sensor health status

use std::sync::Arc;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::backpressure::BackpressureHandler;
use crate::transport::TransportClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub timestamp: DateTime<Utc>,
    pub component_identity: String,
    pub status: String,
    pub buffer_size: usize,
    pub dropped_count: usize,
    pub core_connectivity: bool,
}

pub struct HealthReporter {
    backpressure: Arc<BackpressureHandler>,
    transport: Arc<TransportClient>,
    component_identity: String,
}

impl HealthReporter {
    pub fn new(
        backpressure: Arc<BackpressureHandler>,
        transport: Arc<TransportClient>,
        component_identity: String,
    ) -> Self {
        Self {
            backpressure,
            transport,
            component_identity,
        }
    }
    
    pub fn generate_report(&self) -> HealthReport {
        HealthReport {
            timestamp: Utc::now(),
            component_identity: self.component_identity.clone(),
            status: if self.backpressure.is_backpressure_active() {
                "BACKPRESSURE".to_string()
            } else {
                "HEALTHY".to_string()
            },
            buffer_size: self.backpressure.get_buffer_size(),
            dropped_count: self.backpressure.get_dropped_count(),
            core_connectivity: false, // Will be updated asynchronously
        }
    }
    
    pub async fn check_core_connectivity(&self) -> bool {
        self.transport.health_check().await
    }
}

