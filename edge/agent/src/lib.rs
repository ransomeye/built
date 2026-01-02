#[cfg(target_os = "linux")]
pub use agent_linux::*;

#[cfg(target_os = "windows")]
pub use agent_windows::*;

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn placeholder() {}

