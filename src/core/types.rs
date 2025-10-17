// src/core/types.rs - Fundamental data types
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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