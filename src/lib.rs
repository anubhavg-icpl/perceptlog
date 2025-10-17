// src/lib.rs - Core library implementation
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// Public modules
pub mod commands;
pub mod config;
pub mod error;
pub mod io;
pub mod metrics;
pub mod ocsf;
pub mod output;
pub mod transformer;
pub mod validation;
pub mod vrl;
pub mod watcher;

// Re-exports for convenience
pub use config::TransformerConfig;
pub use error::TransformError;
pub use ocsf::OcsfEvent;
pub use transformer::OcsfTransformer;

/// Result type alias for library operations
pub type TransformResult<T> = std::result::Result<T, TransformError>;

/// Represents a single log event to be transformed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    pub message: String,
    #[serde(flatten)]
    pub metadata: BTreeMap<String, serde_json::Value>,
}

impl LogEvent {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            metadata: BTreeMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_event_creation() {
        let event = LogEvent::new("test message")
            .with_metadata("source", serde_json::json!("sshd"))
            .with_metadata("severity", serde_json::json!(3));

        assert_eq!(event.message, "test message");
        assert_eq!(event.metadata.len(), 2);
    }

    #[test]
    fn test_vrl_runtime_creation() {
        let script = r#".message = "Hello, VRL!""#;
        let result = crate::vrl::VrlRuntime::new(script);
        assert!(result.is_ok());
    }
}
