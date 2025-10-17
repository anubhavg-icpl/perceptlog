// src/processing/runtime.rs - Fully compliant VRL runtime implementation
use anyhow::Result;
use serde_json;
use std::collections::BTreeMap;
use vrl::compiler::{state::RuntimeState, Context, Program, TargetValue, TimeZone};
use vrl::prelude::*;
use vrl::value::{ObjectMap, Secrets};

/// VRL runtime wrapper for executing transformations
pub struct VrlRuntime {
    program: Program,
    timezone: TimeZone,
}

impl VrlRuntime {
    /// Create a new VRL runtime with the given VRL script
    pub fn new(vrl_script: &str) -> Result<Self> {
        // Compile the VRL program with all standard library functions
        let fns = vrl::stdlib::all();
        let result = vrl::compiler::compile(vrl_script, &fns)
            .map_err(|diagnostics| {
                anyhow::anyhow!("Failed to compile script: {diagnostics:?}")
            })?;

        Ok(Self {
            program: result.program,
            timezone: TimeZone::default(),
        })
    }

    /// Execute the VRL program against an event and return the transformed result
    pub fn transform(&mut self, event: Value) -> Result<Value> {
        // Create a target from the input event
        let mut target = TargetValue {
            value: event,
            metadata: Value::Object(BTreeMap::new()),
            secrets: Secrets::default(),
        };

        // Create runtime state for local variables
        let mut state = RuntimeState::default();

        // Create execution context
        let mut ctx = Context::new(&mut target, &mut state, &self.timezone);

        // Execute the VRL program
        let result = self
            .program
            .resolve(&mut ctx)
            .map_err(|e| anyhow::anyhow!("VRL execution error: {e}"))?;

        Ok(result)
    }

    /// Get a reference to the compiled program
    pub fn program(&self) -> &Program {
        &self.program
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
        Value::Timestamp(ts) => {
            // Convert timestamp to ISO 8601 string
            serde_json::Value::String(ts.to_string())
        }
        Value::Regex(r) => serde_json::Value::String(r.to_string()),
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vrl_runtime_simple() {
        let script = r#".result = "Hello, VRL!""#;
        let mut runtime = VrlRuntime::new(script).unwrap();
        
        let input = Value::Object(ObjectMap::new());
        let output = runtime.transform(input).unwrap();
        
        // The script sets .result, so output could be the result value or modified object
        // VRL returns the last expression value
        assert!(matches!(output, Value::Bytes(_) | Value::Object(_)));
    }

    #[test]
    fn test_vrl_runtime_field_access() {
        let script = r#".message"#;
        let mut runtime = VrlRuntime::new(script).unwrap();
        
        let mut input_map = ObjectMap::new();
        input_map.insert("message".into(), Value::from("test"));
        let input = Value::Object(input_map);
        
        let output = runtime.transform(input).unwrap();
        assert_eq!(output, Value::from("test"));
    }

    #[test]
    fn test_value_conversions() {
        let json = serde_json::json!({
            "string": "test",
            "number": 42,
            "boolean": true,
            "array": [1, 2, 3],
            "nested": {
                "key": "value"
            }
        });

        let vrl_value = serde_json_to_vrl_value(json.clone());
        let converted_back = vrl_value_to_serde_json(vrl_value);

        assert_eq!(json, converted_back);
    }

    #[test]
    fn test_log_event_conversion() {
        let event = crate::LogEvent::new("test message")
            .with_metadata("key", serde_json::json!("value"));

        let vrl_value = log_event_to_vrl_value(event);
        
        assert!(vrl_value.is_object());
        if let Value::Object(map) = vrl_value {
            assert!(map.contains_key("message"));
            assert!(map.contains_key("key"));
        }
    }
}
