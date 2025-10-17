// src/vrl.rs - VRL execution and value conversion module
use anyhow::Result;
use serde_json;
use vrl::compiler::{Program, TimeZone};
use vrl::prelude::*;
use vrl::value::ObjectMap;

/// VRL runtime wrapper for executing transformations
pub struct VrlRuntime {
    #[allow(dead_code)] // Will be used when VRL execution is properly implemented
    program: Program,
    #[allow(dead_code)] // Will be used when VRL execution is properly implemented
    timezone: TimeZone,
}

impl VrlRuntime {
    /// Create a new VRL runtime with the given VRL script
    pub fn new(vrl_script: &str) -> Result<Self> {
        // Parse and compile the VRL program
        let program = vrl::compiler::compile(vrl_script, &vrl::stdlib::all())
            .map_err(|e| anyhow::anyhow!("Failed to compile VRL script: {e:?}"))?
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
pub fn log_event_to_vrl_value(event: crate::LogEvent) -> Value {
    let mut map = ObjectMap::new();
    map.insert("message".into(), Value::from(event.message));

    for (key, value) in event.metadata {
        map.insert(key.into(), serde_json_to_vrl_value(value));
    }

    Value::Object(map)
}

/// Convert serde_json::Value to VRL Value
pub fn serde_json_to_vrl_value(json: serde_json::Value) -> Value {
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
    fn test_vrl_runtime_creation() {
        let script = r#".message = "Hello, VRL!""#;
        let result = VrlRuntime::new(script);
        assert!(result.is_ok());
    }

    #[test]
    fn test_value_conversions() {
        let json = serde_json::json!({
            "test": "value",
            "number": 42
        });

        let vrl_value = serde_json_to_vrl_value(json.clone());
        let converted_back = vrl_value_to_serde_json(vrl_value);

        assert_eq!(json, converted_back);
    }
}
