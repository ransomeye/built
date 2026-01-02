// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/targets/linux_agent.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Linux agent target implementation

use crate::dispatcher::router::AgentInfo;

pub struct LinuxAgentTarget;

impl LinuxAgentTarget {
    pub fn create_agent_info(agent_id: String, api_url: String) -> AgentInfo {
        AgentInfo {
            agent_id,
            platform: "linux".to_string(),
            capabilities: vec!["block".to_string(), "isolate".to_string(), "quarantine".to_string(), "monitor".to_string()],
            api_url,
            asset_class: None,
            environment: None,
        }
    }
}

