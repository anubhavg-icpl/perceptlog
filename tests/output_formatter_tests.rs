// tests/output_formatter_tests.rs - Tests for output formatting
use perceptlog::{config::OutputFormat, ocsf::OcsfEventBuilder, output::OutputFormatter};
use perceptlog::output::StreamingOutputFormatter;

fn create_test_event() -> perceptlog::ocsf::OcsfEvent {
    OcsfEventBuilder::new()
        .with_category(1, "System Activity")
        .with_class(1001, "Process Activity")
        .with_time(1234567890)
        .with_message("Test event")
        .build()
}

#[test]
fn test_json_formatting() {
    let event = create_test_event();
    let events = vec![event];

    let json_output =
        OutputFormatter::format_events(&events, OutputFormat::Json, false).unwrap();
    assert!(json_output.contains("category_uid"));
    assert!(json_output.contains("class_uid"));

    // Test pretty formatting
    let pretty_json =
        OutputFormatter::format_events(&events, OutputFormat::JsonPretty, false).unwrap();
    assert!(pretty_json.contains("category_uid"));
    assert!(pretty_json.len() > json_output.len()); // Pretty print should be longer
}

#[test]
fn test_ndjson_formatting() {
    let event = create_test_event();
    let events = vec![event.clone(), event];

    let ndjson_output =
        OutputFormatter::format_events(&events, OutputFormat::Ndjson, false).unwrap();
    let lines: Vec<&str> = ndjson_output.lines().collect();
    assert_eq!(lines.len(), 2); // Should have 2 lines for 2 events
}

#[test]
fn test_yaml_formatting() {
    let event = create_test_event();
    let events = vec![event];

    let yaml_output =
        OutputFormatter::format_events(&events, OutputFormat::Yaml, false).unwrap();
    assert!(yaml_output.contains("category_uid: 1"));
}

#[test]
fn test_filename_generation() {
    let filename = OutputFormatter::create_output_filename("test", OutputFormat::Json, false);
    assert_eq!(filename, "test.json");

    let filename_with_timestamp =
        OutputFormatter::create_output_filename("test", OutputFormat::Ndjson, true);
    assert!(filename_with_timestamp.starts_with("test_"));
    assert!(filename_with_timestamp.ends_with(".ndjson"));
}

#[test]
fn test_streaming_formatter() {
    let mut formatter = StreamingOutputFormatter::new(OutputFormat::Ndjson, false);
    let event = create_test_event();

    formatter.add_event(&event).unwrap();
    formatter.add_event(&event).unwrap();

    let buffer = formatter.get_buffer();
    let lines: Vec<&str> = buffer.lines().collect();
    assert_eq!(lines.len(), 2);
}

#[test]
fn test_empty_events() {
    let events: Vec<perceptlog::ocsf::OcsfEvent> = vec![];
    let result = OutputFormatter::format_events(&events, OutputFormat::Json, false).unwrap();
    assert_eq!(result, "[]");
}
