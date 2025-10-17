// src/core/mod.rs - Core types and fundamental structures
pub mod types;
pub mod error;
pub mod config;

// Re-exports for convenience
pub use error::{TransformError, TransformResult};
pub use types::{LogEvent};
pub use config::{TransformerConfig, OutputFormat, VectorConfig};