// tests/processing_runtime_tests.rs - Tests for transform runtime
use perceptlog::runtime::VrlRuntime;

#[test]
fn test_vrl_runtime_creation() {
    let script = r#".message = "Hello, VRL!""#;
    let result = VrlRuntime::new(script);
    assert!(result.is_ok());
}

#[test]
fn test_vrl_runtime_invalid_script() {
    let script = r#"invalid vrl syntax {{"#;
    let result = VrlRuntime::new(script);
    assert!(result.is_err());
}

#[test]
fn test_value_conversions() {
    let json = serde_json::json!({
        "test": "value",
        "number": 42,
        "nested": {
            "field": "data"
        }
    });

    // Test that we can create a valid JSON value
    assert_eq!(json["test"], "value");
    assert_eq!(json["number"], 42);
    assert_eq!(json["nested"]["field"], "data");
}

#[test]
fn test_vrl_runtime_simple_transform() {
    let script = r#". = .message"#;
    let result = VrlRuntime::new(script);
    assert!(result.is_ok(), "Simple transform script should compile");
}
