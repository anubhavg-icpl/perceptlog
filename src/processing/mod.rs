// src/processing/mod.rs - Data processing and transformation module
pub mod transformer;
pub mod vrl;
pub mod ocsf;

// Re-exports for convenience
pub use transformer::OcsfTransformer;
pub use vrl::{VrlRuntime, vrl_value_to_serde_json, log_event_to_vrl_value, serde_json_to_vrl_value};
pub use ocsf::{OcsfEvent, OcsfEventBuilder, OcsfMetadata, OcsfProduct, OcsfUser, OcsfActor, OcsfService, OcsfEndpoint, OcsfProcess, OcsfObservable};