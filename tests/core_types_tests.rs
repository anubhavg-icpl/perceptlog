// tests/core_types_tests.rs - Tests for core types
use perceptlog::LogEvent;

#[test]
fn test_log_event_creation() {
    let event = LogEvent::new("test message")
        .with_metadata("source", serde_json::json!("sshd"))
        .with_metadata("severity", serde_json::json!(3));

    assert_eq!(event.message, "test message");
    assert_eq!(event.metadata.len(), 2);
}

#[test]
fn test_log_event_empty() {
    let event = LogEvent::new("empty event");
    assert_eq!(event.message, "empty event");
    assert_eq!(event.metadata.len(), 0);
}

#[test]
fn test_log_event_multiple_metadata() {
    let event = LogEvent::new("test")
        .with_metadata("key1", serde_json::json!("value1"))
        .with_metadata("key2", serde_json::json!(42))
        .with_metadata("key3", serde_json::json!(true));

    assert_eq!(event.metadata.len(), 3);
    assert_eq!(event.metadata.get("key1").unwrap(), &serde_json::json!("value1"));
    assert_eq!(event.metadata.get("key2").unwrap(), &serde_json::json!(42));
    assert_eq!(event.metadata.get("key3").unwrap(), &serde_json::json!(true));
}
