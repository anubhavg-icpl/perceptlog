// tests/processing_ocsf_tests.rs - Tests for OCSF processing
use perceptlog::ocsf::OcsfEventBuilder;

#[test]
fn test_ocsf_event_builder() {
    let event = OcsfEventBuilder::new()
        .with_category(3, "Identity & Access Management")
        .with_class(3002, "Authentication")
        .with_time(1234567890)
        .with_message("Test authentication event")
        .build();

    assert_eq!(event.category_uid, 3);
    assert_eq!(event.category_name, "Identity & Access Management");
    assert_eq!(event.class_uid, 3002);
    assert_eq!(event.class_name, "Authentication");
    assert_eq!(event.time, 1234567890);
    assert_eq!(event.message, Some("Test authentication event".to_string()));
}

#[test]
fn test_ocsf_event_builder_minimal() {
    let event = OcsfEventBuilder::new()
        .with_category(1, "System Activity")
        .with_class(1001, "Process Activity")
        .with_time(1000000000)
        .build();

    assert_eq!(event.category_uid, 1);
    assert_eq!(event.class_uid, 1001);
    assert_eq!(event.message, None);
}

#[test]
fn test_ocsf_event_serialization() {
    let event = OcsfEventBuilder::new()
        .with_category(3, "IAM")
        .with_class(3002, "Auth")
        .with_time(12345)
        .build();

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("category_uid"));
    assert!(json.contains("class_uid"));
}
