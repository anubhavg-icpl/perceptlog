// src/commands.rs - CLI command handlers module
use crate::{
    core::config::{OutputFormat, TransformerConfig},
    io::{FileReader, FileWriter},
    output::OutputFormatter,
    processing::transformer::OcsfTransformer,
    utils::validation::InputValidator,
};
use anyhow::Result;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{error, info, warn};

/// Transform command handler
pub struct TransformCommand;

impl TransformCommand {
    /// Execute the transform command
    pub async fn execute(
        vrl_script: PathBuf,
        input: PathBuf,
        output: PathBuf,
        format: OutputFormat,
        pretty: bool,
        skip_errors: bool,
        _batch_size: usize,
    ) -> Result<()> {
        info!("Starting transformation process");
        info!("VRL Script: {}", vrl_script.display());
        info!("Input: {}", input.display());
        info!("Output: {}", output.display());
        info!("Format: {:?}", format);

        // Validate inputs
        InputValidator::validate_vrl_script_file(&vrl_script)?;
        InputValidator::validate_input_path(&input)?;

        // Create transformer
        let transformer = OcsfTransformer::new(&vrl_script).await?;

        // Create output directory
        FileWriter::ensure_directory_exists(&output).await?;

        // Process based on input type
        if input.is_file() {
            Self::process_single_file(&transformer, &input, &output, format, pretty).await?;
        } else if input.is_dir() {
            Self::process_directory(&transformer, &input, &output, format, pretty, skip_errors)
                .await?;
        } else {
            return Err(anyhow::anyhow!(
                "Input path is neither a file nor a directory"
            ));
        }

        info!("Transformation completed successfully");
        Ok(())
    }

    /// Process a single file
    async fn process_single_file(
        transformer: &OcsfTransformer,
        input: &Path,
        output_dir: &Path,
        format: OutputFormat,
        pretty: bool,
    ) -> Result<()> {
        info!("Processing file: {}", input.display());

        let events = FileReader::read_file_to_events(input).await?;
        let results: Vec<_> = transformer
            .transform_batch(events.into_iter().map(|e| e.message).collect())
            .await;

        let successful_events: Vec<_> = results.into_iter().filter_map(|r| r.ok()).collect();

        if !successful_events.is_empty() {
            let output_filename = OutputFormatter::create_output_filename(
                input
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("output"),
                format,
                false,
            );
            let output_file = output_dir.join(output_filename);
            Self::write_events(&successful_events, &output_file, format, pretty).await?;
            info!("Processed {} events from file", successful_events.len());
        }

        Ok(())
    }

    /// Process a directory
    async fn process_directory(
        transformer: &OcsfTransformer,
        input_dir: &Path,
        output_dir: &Path,
        format: OutputFormat,
        pretty: bool,
        skip_errors: bool,
    ) -> Result<()> {
        info!("Processing directory: {}", input_dir.display());

        let log_files = FileReader::get_log_files_from_directory(
            input_dir,
            &[], // TODO: Get from config
            &[], // TODO: Get from config
        )
        .await?;

        let _total_events = 0;
        let mut processed_files = 0;
        let mut failed_files = 0;

        for file_path in log_files {
            match Self::process_single_file(transformer, &file_path, output_dir, format, pretty)
                .await
            {
                Ok(_) => {
                    processed_files += 1;
                    info!("Successfully processed: {}", file_path.display());
                }
                Err(e) => {
                    failed_files += 1;
                    if skip_errors {
                        warn!(
                            "Failed to process {} (skipping): {}",
                            file_path.display(),
                            e
                        );
                    } else {
                        error!("Failed to process {}: {}", file_path.display(), e);
                        return Err(e);
                    }
                }
            }
        }

        info!(
            "Directory processing completed. Processed: {}, Failed: {}",
            processed_files, failed_files
        );
        Ok(())
    }

    /// Write events to output file
    async fn write_events(
        events: &[crate::processing::ocsf::OcsfEvent],
        output_file: &Path,
        format: OutputFormat,
        pretty: bool,
    ) -> Result<()> {
        let content = OutputFormatter::format_events(events, format, pretty)?;
        FileWriter::write_to_file(output_file, &content).await?;
        info!("Wrote {} events to {}", events.len(), output_file.display());
        Ok(())
    }
}

/// Validate command handler
pub struct ValidateCommand;

impl ValidateCommand {
    /// Execute the validate command
    pub async fn execute(vrl_script: PathBuf) -> Result<()> {
        info!("Validating VRL script: {}", vrl_script.display());

        let script_content = fs::read_to_string(&vrl_script).await?;

        match OcsfTransformer::validate_script(&script_content) {
            Ok(_) => {
                info!("✓ VRL script is valid");
                Ok(())
            }
            Err(e) => {
                error!("✗ VRL script validation failed: {}", e);
                Err(anyhow::anyhow!("VRL script validation failed: {e}"))
            }
        }
    }
}

/// Convert command handler
pub struct ConvertCommand;

impl ConvertCommand {
    /// Execute the convert command
    pub async fn execute(vector_config: PathBuf, output: Option<PathBuf>) -> Result<()> {
        info!("Converting Vector config: {}", vector_config.display());

        let content = fs::read_to_string(&vector_config).await?;
        let vector_cfg = crate::core::config::VectorConfig::from_toml(&content)?;
        let transformer_cfg = vector_cfg.to_transformer_config();

        let output_path = output.unwrap_or_else(|| PathBuf::from("transformer_config.toml"));
        let config_content = toml::to_string_pretty(&transformer_cfg)?;

        FileWriter::write_to_file(&output_path, &config_content).await?;
        info!("Converted config saved to: {}", output_path.display());
        Ok(())
    }
}

/// Run command handler
pub struct RunCommand;

impl RunCommand {
    /// Execute the run command
    pub async fn execute(config_path: PathBuf) -> Result<()> {
        info!("Loading configuration from: {}", config_path.display());

        let config = TransformerConfig::from_file(&config_path)?;
        config
            .validate()
            .map_err(|e| anyhow::anyhow!("Configuration validation failed: {e}"))?;

        let transformer = OcsfTransformer::with_config(config.clone()).await?;

        // Create output directory
        fs::create_dir_all(&config.output_path).await?;

        // Process based on input type
        if config.input_path.is_file() {
            TransformCommand::process_single_file(
                &transformer,
                &config.input_path,
                &config.output_path,
                config.output_format,
                config.pretty_print,
            )
            .await?;
        } else if config.input_path.is_dir() {
            TransformCommand::process_directory(
                &transformer,
                &config.input_path,
                &config.output_path,
                config.output_format,
                config.pretty_print,
                config.skip_errors,
            )
            .await?;
        } else {
            return Err(anyhow::anyhow!(
                "Input path is neither a file nor a directory"
            ));
        }

        info!("Processing completed successfully");
        Ok(())
    }
}

/// Watch command handler
#[cfg(feature = "watch-mode")]
pub struct WatchCommand;

#[cfg(feature = "watch-mode")]
impl WatchCommand {
    /// Execute the watch command
    pub async fn execute(
        vrl_script: PathBuf,
        input: PathBuf,
        output: PathBuf,
        interval: u64,
    ) -> Result<()> {
        use crate::io::watcher::FileWatcher;

        info!("Starting watch mode");
        info!("VRL Script: {}", vrl_script.display());
        info!("Input: {}", input.display());
        info!("Output: {}", output.display());
        info!("Interval: {} seconds", interval);

        let transformer = OcsfTransformer::new(&vrl_script).await?;
        let watcher = FileWatcher::new(transformer, input, output, interval)?;
        watcher.start().await?;

        Ok(())
    }
}

/// Metrics command handler
#[cfg(feature = "metrics-support")]
pub struct MetricsCommand;

#[cfg(feature = "metrics-support")]
impl MetricsCommand {
    /// Execute the metrics command
    pub async fn execute(port: u16) -> Result<()> {
        use crate::utils::metrics::start_metrics_server;

        info!("Starting metrics server on port {}", port);
        start_metrics_server(port)
            .await
            .map_err(|e| anyhow::anyhow!("Metrics server error: {e}"))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_validate_command() {
        // Create a temporary VRL script file
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, ". = .").unwrap();
        temp_file.flush().unwrap();

        let result = ValidateCommand::execute(temp_file.path().to_path_buf()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_convert_command() {
        // Create a temporary Vector config file
        let mut temp_file = NamedTempFile::with_suffix(".toml").unwrap();
        write!(
            temp_file,
            r#"
            data_dir = "/tmp"
            [sources]
            [transforms]
            [sinks]
        "#
        )
        .unwrap();
        temp_file.flush().unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let result = ConvertCommand::execute(
            temp_file.path().to_path_buf(),
            Some(output_file.path().to_path_buf()),
        )
        .await;

        assert!(result.is_ok());
    }
}
