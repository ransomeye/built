// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/targets/dpi.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: DPI probe target implementation

use crate::dispatcher::router::AgentInfo;

pub struct DpiTarget;

impl DpiTarget {
    pub fn create_agent_info(agent_id: String, api_url: String) -> AgentInfo {
        AgentInfo {
            agent_id,
            platform: "network".to_string(),
            capabilities: vec!["block".to_string(), "isolate".to_string(), "monitor".to_string()],
            api_url,
            asset_class: None,
            environment: None,
        }
    }
}

