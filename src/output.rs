// src/output.rs - Output formatting module
use crate::{config::OutputFormat, ocsf::OcsfEvent};
use anyhow::Result;
use serde_json;

/// Output formatter for different formats
pub struct OutputFormatter;

impl OutputFormatter {
    /// Format events according to the specified output format
    pub fn format_events(
        events: &[OcsfEvent],
        format: OutputFormat,
        pretty: bool,
    ) -> Result<String> {
        match format {
            OutputFormat::Json => Self::format_json(events, pretty),
            OutputFormat::JsonPretty => Self::format_json(events, true),
            OutputFormat::Ndjson => Self::format_ndjson(events),
            OutputFormat::Yaml => Self::format_yaml(events),
        }
    }

    /// Format a single event according to the specified output format
    pub fn format_single_event(
        event: &OcsfEvent,
        format: OutputFormat,
        pretty: bool,
    ) -> Result<String> {
        match format {
            OutputFormat::Json => Self::format_single_json(event, pretty),
            OutputFormat::JsonPretty => Self::format_single_json(event, true),
            OutputFormat::Ndjson => Self::format_single_ndjson(event),
            OutputFormat::Yaml => Self::format_single_yaml(event),
        }
    }

    /// Format events as JSON
    fn format_json(events: &[OcsfEvent], pretty: bool) -> Result<String> {
        if pretty {
            Ok(serde_json::to_string_pretty(events)?)
        } else {
            Ok(serde_json::to_string(events)?)
        }
    }

    /// Format a single event as JSON
    fn format_single_json(event: &OcsfEvent, pretty: bool) -> Result<String> {
        if pretty {
            Ok(serde_json::to_string_pretty(event)?)
        } else {
            Ok(serde_json::to_string(event)?)
        }
    }

    /// Format events as NDJSON (newline-delimited JSON)
    fn format_ndjson(events: &[OcsfEvent]) -> Result<String> {
        let mut result = String::new();
        for event in events {
            result.push_str(&serde_json::to_string(event)?);
            result.push('\n');
        }
        Ok(result)
    }

    /// Format a single event as NDJSON
    fn format_single_ndjson(event: &OcsfEvent) -> Result<String> {
        Ok(format!("{}\n", serde_json::to_string(event)?))
    }

    /// Format events as YAML
    fn format_yaml(events: &[OcsfEvent]) -> Result<String> {
        Ok(serde_yaml::to_string(events)?)
    }

    /// Format a single event as YAML
    fn format_single_yaml(event: &OcsfEvent) -> Result<String> {
        Ok(serde_yaml::to_string(event)?)
    }

    /// Get the appropriate file extension for the output format
    pub fn get_file_extension(format: OutputFormat) -> String {
        format.extension().to_string()
    }

    /// Create output filename with appropriate extension
    pub fn create_output_filename(
        base_name: &str,
        format: OutputFormat,
        include_timestamp: bool,
    ) -> String {
        let extension = format.extension();

        if include_timestamp {
            let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
            format!("{base_name}_{timestamp}.{extension}")
        } else {
            format!("{base_name}.{extension}")
        }
    }

    /// Validate that the events can be serialized in the given format
    pub fn validate_format_compatibility(events: &[OcsfEvent], format: OutputFormat) -> Result<()> {
        match format {
            OutputFormat::Json | OutputFormat::JsonPretty | OutputFormat::Ndjson => {
                // Test JSON serialization
                for event in events {
                    serde_json::to_string(event)?;
                }
            }
            OutputFormat::Yaml => {
                // Test YAML serialization
                for event in events {
                    serde_yaml::to_string(event)?;
                }
            }
        }
        Ok(())
    }
}

/// Streaming output formatter for processing large files
pub struct StreamingOutputFormatter {
    format: OutputFormat,
    pretty: bool,
    buffer: String,
}

impl StreamingOutputFormatter {
    /// Create a new streaming formatter
    pub fn new(format: OutputFormat, pretty: bool) -> Self {
        Self {
            format,
            pretty,
            buffer: String::new(),
        }
    }

    /// Add an event to the streaming buffer
    pub fn add_event(&mut self, event: &OcsfEvent) -> Result<()> {
        let formatted = OutputFormatter::format_single_event(event, self.format, self.pretty)?;

        match self.format {
            OutputFormat::Json | OutputFormat::JsonPretty => {
                // For JSON arrays, we need to handle comma separation
                if !self.buffer.is_empty() && !self.buffer.ends_with('[') {
                    self.buffer.push(',');
                    if self.pretty {
                        self.buffer.push('\n');
                    }
                }
                // Remove the newline from single event formatting for JSON arrays
                let trimmed = formatted.trim_end();
                self.buffer.push_str(trimmed);
            }
            OutputFormat::Ndjson => {
                // NDJSON is naturally streaming-friendly
                self.buffer.push_str(&formatted);
            }
            OutputFormat::Yaml => {
                // YAML documents can be separated by ---
                if !self.buffer.is_empty() {
                    self.buffer.push_str("---\n");
                }
                self.buffer.push_str(&formatted);
            }
        }

        Ok(())
    }

    /// Get the current buffer content
    pub fn get_buffer(&self) -> &str {
        &self.buffer
    }

    /// Clear the buffer
    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }

    /// Finalize the output (add closing brackets for JSON arrays, etc.)
    pub fn finalize(&mut self) -> Result<String> {
        match self.format {
            OutputFormat::Json | OutputFormat::JsonPretty => {
                if self.buffer.is_empty() {
                    self.buffer = "[]".to_string();
                } else {
                    self.buffer = format!("[{}]", self.buffer);
                }
            }
            OutputFormat::Ndjson | OutputFormat::Yaml => {
                // These formats don't need finalization
            }
        }

        Ok(self.buffer.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ocsf::{OcsfEventBuilder, OcsfMetadata, OcsfProduct};

    fn create_test_event() -> OcsfEvent {
        let metadata = OcsfMetadata {
            uid: "test-123".to_string(),
            version: "1.6.0".to_string(),
            product: OcsfProduct {
                vendor_name: "Test".to_string(),
                name: "Test Product".to_string(),
                version: "1.0".to_string(),
            },
            logged_time: 1234567890,
            log_name: "test.log".to_string(),
            log_provider: "test".to_string(),
            event_code: "TEST001".to_string(),
            profiles: vec!["test".to_string()],
            log_version: "1.0".to_string(),
            log_level: None,
            original_time: None,
        };

        OcsfEventBuilder::new()
            .with_metadata(metadata)
            .with_category(1, "Test Category")
            .with_class(1001, "Test Class")
            .with_time(1234567890)
            .with_message("Test message")
            .build()
    }

    #[test]
    fn test_json_formatting() {
        let event = create_test_event();
        let events = vec![event];

        let json_output =
            OutputFormatter::format_events(&events, OutputFormat::Json, false).unwrap();
        assert!(json_output.starts_with('['));
        assert!(json_output.ends_with(']'));

        let pretty_output =
            OutputFormatter::format_events(&events, OutputFormat::Json, true).unwrap();
        assert!(pretty_output.contains('\n')); // Pretty formatting should have newlines
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
}
