// src/validation.rs - Input validation module
use anyhow::{Result, anyhow};
use std::path::Path;

/// Input validator for various types of inputs
pub struct InputValidator;

impl InputValidator {
    /// Validate VRL script file exists and is readable
    pub fn validate_vrl_script_file(script_path: &Path) -> Result<()> {
        if !script_path.exists() {
            return Err(anyhow!(
                "VRL script file not found: {}",
                script_path.display()
            ));
        }

        if !script_path.is_file() {
            return Err(anyhow!(
                "VRL script path is not a file: {}",
                script_path.display()
            ));
        }

        // Check if file is readable by attempting to read metadata
        std::fs::metadata(script_path).map_err(|e| {
            anyhow!(
                "Cannot read VRL script file {}: {}",
                script_path.display(),
                e
            )
        })?;

        Ok(())
    }

    /// Validate input path (file or directory) exists and is accessible
    pub fn validate_input_path(input_path: &Path) -> Result<()> {
        if !input_path.exists() {
            return Err(anyhow!("Input path not found: {}", input_path.display()));
        }

        if !input_path.is_file() && !input_path.is_dir() {
            return Err(anyhow!(
                "Input path is neither a file nor a directory: {}",
                input_path.display()
            ));
        }

        // Check if path is readable
        std::fs::metadata(input_path)
            .map_err(|e| anyhow!("Cannot access input path {}: {}", input_path.display(), e))?;

        Ok(())
    }

    /// Validate output directory can be created or is writable
    pub fn validate_output_path(output_path: &Path) -> Result<()> {
        if output_path.exists() {
            if !output_path.is_dir() {
                return Err(anyhow!(
                    "Output path exists but is not a directory: {}",
                    output_path.display()
                ));
            }

            // Check if directory is writable by attempting to create a temporary file
            let temp_file = output_path.join(".write_test");
            match std::fs::write(&temp_file, b"test") {
                Ok(_) => {
                    // Clean up test file
                    let _ = std::fs::remove_file(&temp_file);
                }
                Err(e) => {
                    return Err(anyhow!(
                        "Output directory is not writable {}: {}",
                        output_path.display(),
                        e
                    ));
                }
            }
        } else {
            // Try to create the directory
            std::fs::create_dir_all(output_path).map_err(|e| {
                anyhow!(
                    "Cannot create output directory {}: {}",
                    output_path.display(),
                    e
                )
            })?;
        }

        Ok(())
    }

    /// Validate configuration file exists and is readable
    pub fn validate_config_file(config_path: &Path) -> Result<()> {
        if !config_path.exists() {
            return Err(anyhow!(
                "Configuration file not found: {}",
                config_path.display()
            ));
        }

        if !config_path.is_file() {
            return Err(anyhow!(
                "Configuration path is not a file: {}",
                config_path.display()
            ));
        }

        // Check file extension
        if let Some(extension) = config_path.extension() {
            let ext_str = extension.to_string_lossy().to_lowercase();
            if !["toml", "yaml", "yml", "json"].contains(&ext_str.as_str()) {
                return Err(anyhow!(
                    "Unsupported configuration file format: {ext_str}. Supported formats: toml, yaml, yml, json"
                ));
            }
        } else {
            return Err(anyhow!(
                "Configuration file has no extension: {}",
                config_path.display()
            ));
        }

        Ok(())
    }

    /// Validate VRL script content syntax
    pub fn validate_vrl_script_content(script_content: &str) -> Result<()> {
        if script_content.trim().is_empty() {
            return Err(anyhow!("VRL script content is empty"));
        }

        // Try to compile the VRL script to validate syntax
        crate::vrl::VrlRuntime::new(script_content)
            .map_err(|e| anyhow!("VRL script syntax error: {e}"))?;

        Ok(())
    }

    /// Validate batch size parameter
    pub fn validate_batch_size(batch_size: usize) -> Result<()> {
        if batch_size == 0 {
            return Err(anyhow!("Batch size must be greater than 0"));
        }

        if batch_size > 10000 {
            return Err(anyhow!(
                "Batch size too large (max: 10000), got: {batch_size}"
            ));
        }

        Ok(())
    }

    /// Validate watch interval parameter
    pub fn validate_watch_interval(interval: u64) -> Result<()> {
        if interval == 0 {
            return Err(anyhow!("Watch interval must be greater than 0 seconds"));
        }

        if interval > 3600 {
            return Err(anyhow!(
                "Watch interval too large (max: 3600 seconds), got: {interval}"
            ));
        }

        Ok(())
    }

    /// Validate metrics port parameter
    pub fn validate_metrics_port(port: u16) -> Result<()> {
        if port < 1024 {
            return Err(anyhow!("Port number too low (min: 1024), got: {port}"));
        }

        Ok(())
    }

    /// Validate log level parameter
    pub fn validate_log_level(level: &str) -> Result<()> {
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        let level_lower = level.to_lowercase();

        if !valid_levels.contains(&level_lower.as_str()) {
            return Err(anyhow!(
                "Invalid log level: {}. Valid levels: {}",
                level,
                valid_levels.join(", ")
            ));
        }

        Ok(())
    }

    /// Validate file patterns (glob patterns)
    pub fn validate_file_patterns(patterns: &[String]) -> Result<()> {
        for pattern in patterns {
            glob::Pattern::new(pattern)
                .map_err(|e| anyhow!("Invalid file pattern '{pattern}': {e}"))?;
        }
        Ok(())
    }

    /// Comprehensive validation for transform command parameters
    pub fn validate_transform_params(
        vrl_script: &Path,
        input: &Path,
        output: &Path,
        batch_size: usize,
    ) -> Result<()> {
        Self::validate_vrl_script_file(vrl_script)?;
        Self::validate_input_path(input)?;
        Self::validate_output_path(output)?;
        Self::validate_batch_size(batch_size)?;
        Ok(())
    }

    /// Comprehensive validation for watch command parameters
    #[cfg(feature = "watch-mode")]
    pub fn validate_watch_params(
        vrl_script: &Path,
        input: &Path,
        output: &Path,
        interval: u64,
    ) -> Result<()> {
        Self::validate_vrl_script_file(vrl_script)?;
        Self::validate_input_path(input)?;
        Self::validate_output_path(output)?;
        Self::validate_watch_interval(interval)?;
        Ok(())
    }

    /// Validate that a file has expected content type based on extension
    pub fn validate_file_type(file_path: &Path, expected_extensions: &[&str]) -> Result<()> {
        if let Some(extension) = file_path.extension() {
            let ext_str = extension.to_string_lossy().to_lowercase();
            if !expected_extensions.contains(&ext_str.as_str()) {
                return Err(anyhow!(
                    "Unexpected file type: {}. Expected one of: {}",
                    ext_str,
                    expected_extensions.join(", ")
                ));
            }
        } else {
            return Err(anyhow!(
                "File has no extension: {}. Expected one of: {}",
                file_path.display(),
                expected_extensions.join(", ")
            ));
        }
        Ok(())
    }
}

/// Validation result with detailed error information
#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Create a new validation result
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Add an error to the validation result
    pub fn add_error(&mut self, error: impl Into<String>) {
        self.errors.push(error.into());
        self.is_valid = false;
    }

    /// Add a warning to the validation result
    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }

    /// Check if validation passed
    pub fn is_ok(&self) -> bool {
        self.is_valid
    }

    /// Get all errors as a single string
    pub fn error_summary(&self) -> String {
        self.errors.join("; ")
    }

    /// Get all warnings as a single string
    pub fn warning_summary(&self) -> String {
        self.warnings.join("; ")
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

