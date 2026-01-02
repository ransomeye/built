// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/targets/windows_agent.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Windows agent target implementation

use crate::dispatcher::router::AgentInfo;

pub struct WindowsAgentTarget;

impl WindowsAgentTarget {
    pub fn create_agent_info(agent_id: String, api_url: String) -> AgentInfo {
        AgentInfo {
            agent_id,
            platform: "windows".to_string(),
            capabilities: vec!["block".to_string(), "isolate".to_string(), "quarantine".to_string(), "monitor".to_string()],
            api_url,
            asset_class: None,
            environment: None,
        }
    }
}

