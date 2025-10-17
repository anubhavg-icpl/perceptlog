// src/config.rs - Configuration handling
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for the OCSF transformer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformerConfig {
    /// Path to VRL script file
    pub vrl_script_path: PathBuf,

    /// Input file or directory path
    pub input_path: PathBuf,

    /// Output directory for transformed events
    pub output_path: PathBuf,

    /// Output format (json, ndjson, yaml)
    #[serde(default = "default_output_format")]
    pub output_format: OutputFormat,

    /// Batch size for processing
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,

    /// Skip errors and continue processing
    #[serde(default = "default_skip_errors")]
    pub skip_errors: bool,

    /// Enable watch mode for continuous monitoring
    #[serde(default)]
    pub watch_mode: bool,

    /// Watch interval in seconds
    #[serde(default = "default_watch_interval")]
    pub watch_interval: u64,

    /// Enable metrics collection
    #[serde(default)]
    pub enable_metrics: bool,

    /// Metrics port
    #[serde(default = "default_metrics_port")]
    pub metrics_port: u16,

    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Enable pretty printing for JSON output
    #[serde(default)]
    pub pretty_print: bool,

    /// Maximum number of worker threads
    #[serde(default = "default_max_workers")]
    pub max_workers: usize,

    /// File glob patterns to include
    #[serde(default)]
    pub include_patterns: Vec<String>,

    /// File glob patterns to exclude
    #[serde(default)]
    pub exclude_patterns: Vec<String>,

    /// Enable hot reload of VRL script
    #[serde(default)]
    pub hot_reload: bool,

    /// Additional VRL functions directory
    #[serde(default)]
    pub vrl_functions_dir: Option<PathBuf>,
}

impl Default for TransformerConfig {
    fn default() -> Self {
        Self {
            vrl_script_path: PathBuf::from("remap.vrl"),
            input_path: PathBuf::from("/var/log/auth.log"),
            output_path: PathBuf::from("./ocsf_output"),
            output_format: default_output_format(),
            batch_size: default_batch_size(),
            skip_errors: default_skip_errors(),
            watch_mode: false,
            watch_interval: default_watch_interval(),
            enable_metrics: false,
            metrics_port: default_metrics_port(),
            log_level: default_log_level(),
            pretty_print: false,
            max_workers: default_max_workers(),
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            hot_reload: false,
            vrl_functions_dir: None,
        }
    }
}

impl TransformerConfig {
    /// Load configuration from a TOML file
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, config::ConfigError> {
        let settings = config::Config::builder()
            .add_source(config::File::from(path.as_ref()))
            .add_source(config::Environment::with_prefix("VRL_OCSF"))
            .build()?;

        settings.try_deserialize()
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if !self.vrl_script_path.exists() {
            return Err(format!(
                "VRL script file not found: {}",
                self.vrl_script_path.display()
            ));
        }

        if !self.input_path.exists() {
            return Err(format!(
                "Input path not found: {}",
                self.input_path.display()
            ));
        }

        if self.batch_size == 0 {
            return Err("Batch size must be greater than 0".to_string());
        }

        if self.max_workers == 0 {
            return Err("Max workers must be greater than 0".to_string());
        }

        Ok(())
    }
}

/// Output format for transformed events
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Json,
    Ndjson,
    Yaml,
    #[serde(rename = "json-pretty")]
    JsonPretty,
}

impl OutputFormat {
    pub fn extension(&self) -> &str {
        match self {
            OutputFormat::Json | OutputFormat::JsonPretty => "json",
            OutputFormat::Ndjson => "ndjson",
            OutputFormat::Yaml => "yaml",
        }
    }
}

// Default value functions
fn default_output_format() -> OutputFormat {
    OutputFormat::Ndjson
}

fn default_batch_size() -> usize {
    100
}

fn default_skip_errors() -> bool {
    true
}

fn default_watch_interval() -> u64 {
    5
}

fn default_metrics_port() -> u16 {
    9090
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_max_workers() -> usize {
    num_cpus::get()
}

/// Vector-compatible configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorConfig {
    pub data_dir: PathBuf,
    pub sources: toml::Table,
    pub transforms: toml::Table,
    pub sinks: toml::Table,
}

impl VectorConfig {
    /// Parse Vector TOML configuration
    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    /// Convert to TransformerConfig
    pub fn to_transformer_config(&self) -> TransformerConfig {
        let mut config = TransformerConfig::default();

        // Extract VRL script path from transforms
        if let Some(transform) = self.transforms.get("ocsf_transform") {
            if let Some(file) = transform.get("file") {
                if let Some(path) = file.as_str() {
                    config.vrl_script_path = PathBuf::from(path);
                }
            }
        }

        // Extract input path from sources
        if let Some(source) = self.sources.values().next() {
            if let Some(include) = source.get("include") {
                if let Some(arr) = include.as_array() {
                    if let Some(first) = arr.first() {
                        if let Some(path) = first.as_str() {
                            config.input_path = PathBuf::from(path);
                        }
                    }
                }
            }
        }

        // Extract output path from sinks
        if let Some(sink) = self.sinks.get("ocsf_output") {
            if let Some(path) = sink.get("path") {
                if let Some(p) = path.as_str() {
                    config.output_path = PathBuf::from(p)
                        .parent()
                        .unwrap_or(&PathBuf::from("."))
                        .to_path_buf();
                }
            }
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = TransformerConfig::default();
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.output_format, OutputFormat::Ndjson);
        assert!(config.skip_errors);
    }

    #[test]
    fn test_config_from_toml() {
        let toml_content = r#"
            vrl_script_path = "custom.vrl"
            input_path = "/var/log/secure"
            output_path = "./output"
            output_format = "json"
            batch_size = 50
            skip_errors = false
            enable_metrics = true
            metrics_port = 8080
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", toml_content).unwrap();
        temp_file.flush().unwrap();

        let config = TransformerConfig::from_file(temp_file.path()).unwrap();

        assert_eq!(config.vrl_script_path, PathBuf::from("custom.vrl"));
        assert_eq!(config.batch_size, 50);
        assert_eq!(config.output_format, OutputFormat::Json);
        assert!(!config.skip_errors);
        assert!(config.enable_metrics);
        assert_eq!(config.metrics_port, 8080);
    }

    #[test]
    fn test_output_format_extension() {
        assert_eq!(OutputFormat::Json.extension(), "json");
        assert_eq!(OutputFormat::JsonPretty.extension(), "json");
        assert_eq!(OutputFormat::Ndjson.extension(), "ndjson");
        assert_eq!(OutputFormat::Yaml.extension(), "yaml");
    }
}
