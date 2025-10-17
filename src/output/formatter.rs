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

