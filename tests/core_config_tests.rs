// tests/core_config_tests.rs - Tests for configuration
use perceptlog::config::{TransformerConfig, OutputFormat};
use std::io::Write;
use std::path::PathBuf;
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
        script_path = "custom.perceptlog"
        input_path = "/var/log/secure"
        output_path = "./output"
        output_format = "json"
        batch_size = 50
        skip_errors = false
        enable_metrics = true
        metrics_port = 8080
    "#;

    let mut temp_file = NamedTempFile::with_suffix(".toml").unwrap();
    write!(temp_file, "{toml_content}").unwrap();
    temp_file.flush().unwrap();

    let config = TransformerConfig::from_file(temp_file.path()).unwrap();

    assert_eq!(config.script_path, PathBuf::from("custom.perceptlog"));
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
