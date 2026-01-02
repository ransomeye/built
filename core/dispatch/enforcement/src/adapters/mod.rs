// Path and File Name : /home/ransomeye/rebuild/ransomeye_enforcement/src/adapters/mod.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Platform adapter module exports

pub mod linux_agent;
pub mod windows_agent;
pub mod network;

pub use linux_agent::LinuxAgentAdapter;
pub use windows_agent::WindowsAgentAdapter;
pub use network::NetworkAdapter;

