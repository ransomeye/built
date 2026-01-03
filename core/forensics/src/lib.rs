// Path and File Name : /home/ransomeye/rebuild/core/forensics/src/lib.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Forensics library root - exports evidence collection and preservation modules

#[cfg(feature = "future-forensics")]
pub mod evidence;
#[cfg(feature = "future-forensics")]
pub mod store;
pub mod integrity;
pub mod errors;

#[cfg(feature = "future-forensics")]
pub use evidence::EvidenceCollector;
#[cfg(feature = "future-forensics")]
pub use store::EvidenceStore;
pub use integrity::EvidenceIntegrity;
pub use errors::ForensicsError;
