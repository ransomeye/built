// Path and File Name : /home/ransomeye/rebuild/ransomeye_architecture/enforcement/src/plane_classifier.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Classifies components into architectural planes for enforcement

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Plane {
    DataPlane,
    ControlPlane,
    IntelligencePlane,
    ManagementPlane,
}

pub struct PlaneClassifier {
    component_to_plane: HashMap<String, Plane>,
}

impl PlaneClassifier {
    pub fn new() -> Self {
        let mut classifier = PlaneClassifier {
            component_to_plane: HashMap::new(),
        };
        
        // Data Plane components
        classifier.register_component("ransomeye_dpi_probe", Plane::DataPlane);
        classifier.register_component("ransomeye_linux_agent", Plane::DataPlane);
        classifier.register_component("ransomeye_windows_agent", Plane::DataPlane);
        
        // Control Plane components
        classifier.register_component("ransomeye_threat_correlation", Plane::ControlPlane);
        classifier.register_component("ransomeye_alert_engine", Plane::ControlPlane);
        classifier.register_component("ransomeye_master_core", Plane::ControlPlane);
        classifier.register_component("ransomeye_response", Plane::ControlPlane);
        
        // Intelligence Plane components
        classifier.register_component("ransomeye_ai_core", Plane::IntelligencePlane);
        classifier.register_component("ransomeye_ai_assistant", Plane::IntelligencePlane);
        classifier.register_component("ransomeye_threat_intel_engine", Plane::IntelligencePlane);
        classifier.register_component("ransomeye_llm", Plane::IntelligencePlane);
        
        // Management Plane components
        classifier.register_component("ransomeye_installer", Plane::ManagementPlane);
        classifier.register_component("ransomeye_ui", Plane::ManagementPlane);
        classifier.register_component("ransomeye_forensic", Plane::ManagementPlane);
        classifier.register_component("ransomeye_reporting", Plane::ManagementPlane);
        
        classifier
    }
    
    pub fn register_component(&mut self, component: &str, plane: Plane) {
        self.component_to_plane.insert(component.to_string(), plane);
    }
    
    pub fn classify(&self, component: &str) -> Option<Plane> {
        self.component_to_plane.get(component).copied()
    }
    
    pub fn is_data_plane(&self, component: &str) -> bool {
        self.classify(component) == Some(Plane::DataPlane)
    }
    
    pub fn is_control_plane(&self, component: &str) -> bool {
        self.classify(component) == Some(Plane::ControlPlane)
    }
    
    pub fn is_intelligence_plane(&self, component: &str) -> bool {
        self.classify(component) == Some(Plane::IntelligencePlane)
    }
    
    pub fn is_management_plane(&self, component: &str) -> bool {
        self.classify(component) == Some(Plane::ManagementPlane)
    }
}

impl Default for PlaneClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_data_plane_classification() {
        let classifier = PlaneClassifier::new();
        assert!(classifier.is_data_plane("ransomeye_dpi_probe"));
        assert!(classifier.is_data_plane("ransomeye_linux_agent"));
        assert!(classifier.is_data_plane("ransomeye_windows_agent"));
    }
    
    #[test]
    fn test_control_plane_classification() {
        let classifier = PlaneClassifier::new();
        assert!(classifier.is_control_plane("ransomeye_alert_engine"));
        assert!(classifier.is_control_plane("ransomeye_threat_correlation"));
    }
    
    #[test]
    fn test_intelligence_plane_classification() {
        let classifier = PlaneClassifier::new();
        assert!(classifier.is_intelligence_plane("ransomeye_ai_core"));
        assert!(classifier.is_intelligence_plane("ransomeye_ai_assistant"));
    }
}

