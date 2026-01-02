// Path and File Name : /home/ransomeye/rebuild/core/bus/src/acl.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Topic ACL enforcement with role-based access control

use thiserror::Error;
use tracing::{error, warn, debug};

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ComponentRole {
    Agent,
    DPI,
    UI,
    Governor,
    Core,
    Ingestion,
    Dispatcher,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MessageType {
    TelemetryPublish,
    CommandPublish,
    QueryOnly,
    AlertPublish,
    HeartbeatPublish,
}

#[derive(Debug, Error)]
pub enum AclError {
    #[error("Access denied: {0}")]
    AccessDenied(String),
    #[error("Invalid component role: {0}")]
    InvalidRole(String),
    #[error("Invalid message type: {0}")]
    InvalidMessageType(String),
}

/// Access Control List for message bus
/// 
/// Enforces strict role-based permissions:
/// - Agent: Can publish Telemetry, Heartbeat, Alert. CANNOT publish Commands
/// - DPI: Can publish Telemetry, Heartbeat, Alert. CANNOT publish Commands
/// - UI: QueryOnly. CANNOT publish anything
/// - Governor: Can publish Commands. CANNOT publish raw Telemetry
/// - Core: Can publish Commands, Alerts. Full access
pub struct Acl;

impl Acl {
    /// Check if component can publish message type
    /// 
    /// FAIL-CLOSED: Returns error on access denial
    pub fn can_publish(role: &ComponentRole, message_type: &MessageType) -> Result<bool, AclError> {
        let allowed = match (role, message_type) {
            // Agent permissions
            (ComponentRole::Agent, MessageType::TelemetryPublish) => true,
            (ComponentRole::Agent, MessageType::HeartbeatPublish) => true,
            (ComponentRole::Agent, MessageType::AlertPublish) => true,
            (ComponentRole::Agent, MessageType::CommandPublish) => {
                error!("SECURITY VIOLATION: Agent attempted to publish Command");
                return Err(AclError::AccessDenied(
                    "Agent cannot publish Commands (prevents compromised agent from controlling fleet)".to_string()
                ));
            }
            (ComponentRole::Agent, MessageType::QueryOnly) => false,
            
            // DPI permissions
            (ComponentRole::DPI, MessageType::TelemetryPublish) => true,
            (ComponentRole::DPI, MessageType::HeartbeatPublish) => true,
            (ComponentRole::DPI, MessageType::AlertPublish) => true,
            (ComponentRole::DPI, MessageType::CommandPublish) => {
                error!("SECURITY VIOLATION: DPI attempted to publish Command");
                return Err(AclError::AccessDenied(
                    "DPI cannot publish Commands".to_string()
                ));
            }
            (ComponentRole::DPI, MessageType::QueryOnly) => false,
            
            // UI permissions
            (ComponentRole::UI, MessageType::QueryOnly) => true,
            (ComponentRole::UI, _) => {
                error!("SECURITY VIOLATION: UI attempted to publish");
                return Err(AclError::AccessDenied(
                    "UI can only query, cannot publish".to_string()
                ));
            }
            
            // Governor permissions
            (ComponentRole::Governor, MessageType::CommandPublish) => true,
            (ComponentRole::Governor, MessageType::AlertPublish) => true,
            (ComponentRole::Governor, MessageType::TelemetryPublish) => {
                error!("SECURITY VIOLATION: Governor attempted to publish raw Telemetry");
                return Err(AclError::AccessDenied(
                    "Governor cannot publish raw Telemetry".to_string()
                ));
            }
            (ComponentRole::Governor, MessageType::HeartbeatPublish) => true,
            (ComponentRole::Governor, MessageType::QueryOnly) => true,
            
            // Core permissions (full access)
            (ComponentRole::Core, _) => true,
            (ComponentRole::Ingestion, MessageType::TelemetryPublish) => true,
            (ComponentRole::Ingestion, MessageType::AlertPublish) => true,
            (ComponentRole::Ingestion, MessageType::HeartbeatPublish) => true,
            (ComponentRole::Ingestion, MessageType::QueryOnly) => true,
            (ComponentRole::Ingestion, MessageType::CommandPublish) => false,
            (ComponentRole::Dispatcher, MessageType::CommandPublish) => true,
            (ComponentRole::Dispatcher, MessageType::AlertPublish) => true,
            (ComponentRole::Dispatcher, MessageType::QueryOnly) => true,
            (ComponentRole::Dispatcher, MessageType::TelemetryPublish) => false,
            (ComponentRole::Dispatcher, MessageType::HeartbeatPublish) => true,
        };
        
        if allowed {
            debug!("ACL check passed: {:?} can publish {:?}", role, message_type);
            Ok(true)
        } else {
            warn!("ACL check failed: {:?} cannot publish {:?}", role, message_type);
            Err(AclError::AccessDenied(
                format!("{:?} cannot publish {:?}", role, message_type)
            ))
        }
    }
    
    /// Check if component can subscribe to topic
    pub fn can_subscribe(role: &ComponentRole, topic: &str) -> bool {
        match role {
            ComponentRole::UI => topic.starts_with("query.") || topic.starts_with("telemetry."),
            ComponentRole::Governor => topic.starts_with("telemetry.") || topic.starts_with("alert."),
            ComponentRole::Core => true,
            ComponentRole::Dispatcher => topic.starts_with("command.") || topic.starts_with("alert."),
            _ => false, // Agents and DPI cannot subscribe
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_agent_cannot_publish_command() {
        let result = Acl::can_publish(&ComponentRole::Agent, &MessageType::CommandPublish);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AclError::AccessDenied(_)));
    }
    
    #[test]
    fn test_agent_can_publish_telemetry() {
        let result = Acl::can_publish(&ComponentRole::Agent, &MessageType::TelemetryPublish);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
    
    #[test]
    fn test_ui_cannot_publish() {
        let result = Acl::can_publish(&ComponentRole::UI, &MessageType::TelemetryPublish);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_governor_cannot_publish_raw_telemetry() {
        let result = Acl::can_publish(&ComponentRole::Governor, &MessageType::TelemetryPublish);
        assert!(result.is_err());
    }
}

