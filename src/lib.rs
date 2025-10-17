// src/lib.rs - Core library implementation
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, warn};
use vrl::compiler::{CompileConfig, Program, TargetValueRef, TimeZone, state::RuntimeState};
use vrl::prelude::*;
use vrl::value::ObjectMap;

pub mod config;
pub mod error;
pub mod metrics;
pub mod transformer;
pub mod watcher;

pub use config::TransformerConfig;
pub use error::TransformError;
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

/// OCSF (Open Cybersecurity Schema Framework) event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcsfEvent {
    pub metadata: OcsfMetadata,
    pub category_uid: i32,
    pub category_name: String,
    pub class_uid: i32,
    pub class_name: String,
    pub time: i64,
    pub type_uid: i32,
    pub type_name: String,
    pub activity_id: i32,
    pub activity_name: String,
    pub status: String,
    pub status_id: i32,
    pub severity: String,
    pub severity_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<OcsfUser>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<OcsfActor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<OcsfService>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_endpoint: Option<OcsfEndpoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dst_endpoint: Option<OcsfEndpoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_protocol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_protocol_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logon_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logon_type_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logon_process: Option<OcsfProcess>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_remote: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_mfa: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_cleartext: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observables: Option<Vec<OcsfObservable>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unmapped: Option<BTreeMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone_offset: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcsfMetadata {
    pub uid: String,
    pub version: String,
    pub product: OcsfProduct,
    pub logged_time: i64,
    pub log_name: String,
    pub log_provider: String,
    pub event_code: String,
    pub profiles: Vec<String>,
    pub log_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcsfProduct {
    pub vendor_name: String,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcsfUser {
    pub name: String,
    pub uid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcsfActor {
    pub user: OcsfUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcsfService {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcsfEndpoint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcsfProcess {
    pub name: String,
    pub cmd_line: String,
    pub uid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcsfObservable {
    pub name: String,
    #[serde(rename = "type")]
    pub observable_type: String,
    pub type_id: i32,
    pub value: String,
}

/// VRL runtime wrapper for executing transformations
pub struct VrlRuntime {
    program: Program,
    timezone: TimeZone,
}

impl VrlRuntime {
    /// Create a new VRL runtime with the given VRL script
    pub fn new(vrl_script: &str) -> Result<Self> {
        // Parse and compile the VRL program
        let program = vrl::compiler::compile(vrl_script, &vrl::stdlib::all())
            .map_err(|e| anyhow::anyhow!("Failed to compile VRL script: {:?}", e))?
            .program;

        Ok(Self {
            program,
            timezone: TimeZone::default(),
        })
    }

    /// Execute the VRL program against an event
    pub fn transform(&mut self, event: Value) -> Result<Value> {
        // For now, return the event as-is until we can properly implement VRL execution
        // This is a temporary workaround for the API compatibility issues
        Ok(event)
    }
}

/// Convert LogEvent to VRL Value
impl From<LogEvent> for Value {
    fn from(event: LogEvent) -> Self {
        let mut map = ObjectMap::new();
        map.insert("message".into(), Value::from(event.message));

        for (key, value) in event.metadata {
            map.insert(key.into(), serde_json_to_vrl_value(value));
        }

        Value::Object(map)
    }
}

/// Convert serde_json::Value to VRL Value
fn serde_json_to_vrl_value(json: serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Boolean(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Integer(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(NotNan::new(f).unwrap_or_else(|_| NotNan::new(0.0).unwrap()))
            } else {
                Value::Null
            }
        }
        serde_json::Value::String(s) => Value::Bytes(s.into()),
        serde_json::Value::Array(arr) => {
            Value::Array(arr.into_iter().map(serde_json_to_vrl_value).collect())
        }
        serde_json::Value::Object(obj) => {
            let map: ObjectMap = obj
                .into_iter()
                .map(|(k, v)| (k.into(), serde_json_to_vrl_value(v)))
                .collect();
            Value::Object(map)
        }
    }
}

/// Convert VRL Value back to serde_json::Value
pub fn vrl_value_to_serde_json(value: Value) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Boolean(b) => serde_json::Value::Bool(b),
        Value::Integer(i) => serde_json::Value::Number(serde_json::Number::from(i)),
        Value::Float(f) => serde_json::Number::from_f64(f.into_inner())
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Value::Bytes(b) => serde_json::Value::String(String::from_utf8_lossy(&b).to_string()),
        Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(vrl_value_to_serde_json).collect())
        }
        Value::Object(map) => {
            let obj: serde_json::Map<String, serde_json::Value> = map
                .into_iter()
                .map(|(k, v)| (k.to_string(), vrl_value_to_serde_json(v)))
                .collect();
            serde_json::Value::Object(obj)
        }
        _ => serde_json::Value::Null,
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
        let runtime = VrlRuntime::new(script);
        assert!(runtime.is_ok());
    }
}
