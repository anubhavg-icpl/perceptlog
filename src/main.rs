// src/main.rs - CLI application
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tokio::fs;
use tracing::{error, info, warn};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use vrl_ocsf_transformer::{
    config::{OutputFormat, TransformerConfig},
    OcsfTransformer,
};

#[derive(Parser)]
#[command(
    name = "perceptlog",
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = "Transform Linux authentication logs to OCSF format using VRL"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info", env = "VRL_OCSF_LOG_LEVEL")]
    log_level: String,

    /// Configuration file path
    #[arg(short, long, env = "VRL_OCSF_CONFIG")]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Transform log files to OCSF format
    Transform {
        /// VRL script file path
        #[arg(short = 's', long)]
        vrl_script: PathBuf,

        /// Input log file or directory
        #[arg(short, long)]
        input: PathBuf,

        /// Output directory for OCSF events
        #[arg(short, long, default_value = "./ocsf_output")]
        output: PathBuf,

        /// Output format (json, ndjson, yaml, json-pretty)
        #[arg(short = 'f', long, default_value = "ndjson")]
        format: OutputFormat,

        /// Enable pretty printing for JSON
        #[arg(short = 'p', long)]
        pretty: bool,

        /// Skip errors and continue processing
        #[arg(short = 'e', long)]
        skip_errors: bool,

        /// Batch size for processing
        #[arg(short = 'b', long, default_value = "100")]
        batch_size: usize,
    },

    /// Watch files for changes and continuously transform
    #[cfg(feature = "watch-mode")]
    Watch {
        /// VRL script file path
        #[arg(short = 's', long)]
        vrl_script: PathBuf,

        /// Input log file or directory to watch
        #[arg(short, long)]
        input: PathBuf,

        /// Output directory for OCSF events
        #[arg(short, long, default_value = "./ocsf_output")]
        output: PathBuf,

        /// Watch interval in seconds
        #[arg(short = 'n', long, default_value = "5")]
        interval: u64,
    },

    /// Validate VRL script syntax
    Validate {
        /// VRL script file path
        vrl_script: PathBuf,
    },

    /// Convert Vector TOML config to transformer config
    Convert {
        /// Vector TOML configuration file
        vector_config: PathBuf,

        /// Output configuration file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Run with full configuration file
    Run {
        /// Configuration file path
        config: PathBuf,
    },

    /// Start metrics server
    #[cfg(feature = "metrics-support")]
    Metrics {
        /// Port to listen on
        #[arg(short, long, default_value = "9090")]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    setup_logging(&cli.log_level)?;

    // Load configuration if provided
    let config = if let Some(config_path) = &cli.config {
        Some(TransformerConfig::from_file(config_path)?)
    } else {
        None
    };

    match cli.command {
        Commands::Transform {
            vrl_script,
            input,
            output,
            format,
            pretty,
            skip_errors,
            batch_size,
        } => {
            transform_command(
                vrl_script,
                input,
                output,
                format,
                pretty,
                skip_errors,
                batch_size,
            )
            .await?;
        }

        #[cfg(feature = "watch-mode")]
        Commands::Watch {
            vrl_script,
            input,
            output,
            interval,
        } => {
            watch_command(vrl_script, input, output, interval).await?;
        }

        Commands::Validate { vrl_script } => {
            validate_command(vrl_script).await?;
        }

        Commands::Convert {
            vector_config,
            output,
        } => {
            convert_command(vector_config, output).await?;
        }

        Commands::Run { config: config_path } => {
            run_command(config_path).await?;
        }

        #[cfg(feature = "metrics-support")]
        Commands::Metrics { port } => {
            metrics_command(port).await?;
        }
    }

    Ok(())
}

async fn transform_command(
    vrl_script: PathBuf,
    input: PathBuf,
    output: PathBuf,
    format: OutputFormat,
    pretty: bool,
    skip_errors: bool,
    batch_size: usize,
) -> Result<()> {
    info!("Starting transformation process");
    info!("VRL Script: {}", vrl_script.display());
    info!("Input: {}", input.display());
    info!("Output: {}", output.display());
    info!("Format: {:?}", format);

    // Create output directory if it doesn't exist
    fs::create_dir_all(&output).await?;

    // Create transformer
    let transformer = OcsfTransformer::new(&vrl_script).await?;

    // Process files based on input type
    if input.is_file() {
        process_single_file(&transformer, &input, &output, format, pretty).await?;
    } else if input.is_dir() {
        process_directory(&transformer, &input, &output, format, pretty, skip_errors).await?;
    } else {
        return Err(anyhow::anyhow!(
            "Input path does not exist: {}",
            input.display()
        ));
    }

    info!("Transformation completed successfully");
    Ok(())
}

async fn process_single_file(
    transformer: &OcsfTransformer,
    input: &PathBuf,
    output_dir: &PathBuf,
    format: OutputFormat,
    pretty: bool,
) -> Result<()> {
    info!("Processing file: {}", input.display());

    let events = transformer.process_file(input).await?;
    let output_file = output_dir.join(format!(
        "{}.{}",
        input.file_stem().unwrap().to_string_lossy(),
        format.extension()
    ));

    write_events(&events, &output_file, format, pretty).await?;
    info!(
        "Wrote {} events to {}",
        events.len(),
        output_file.display()
    );

    Ok(())
}

async fn process_directory(
    transformer: &OcsfTransformer,
    input_dir: &PathBuf,
    output_dir: &PathBuf,
    format: OutputFormat,
    pretty: bool,
    skip_errors: bool,
) -> Result<()> {
    info!("Processing directory: {}", input_dir.display());

    let mut entries = fs::read_dir(input_dir).await?;
    let mut total_events = 0;
    let mut processed_files = 0;
    let mut failed_files = 0;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |e| e == "log") {
            match process_single_file(transformer, &path, output_dir, format, pretty).await {
                Ok(_) => {
                    processed_files += 1;
                }
                Err(e) => {
                    error!("Failed to process {}: {}", path.display(), e);
                    failed_files += 1;
                    if !skip_errors {
                        return Err(e);
                    }
                }
            }
        }
    }

    info!(
        "Processed {} files successfully, {} failed",
        processed_files, failed_files
    );

    Ok(())
}

async fn write_events(
    events: &[vrl_ocsf_transformer::OcsfEvent],
    output_file: &PathBuf,
    format: OutputFormat,
    pretty: bool,
) -> Result<()> {
    let content = match format {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            if pretty || format == OutputFormat::JsonPretty {
                serde_json::to_string_pretty(events)?
            } else {
                serde_json::to_string(events)?
            }
        }
        OutputFormat::Ndjson => {
            events
                .iter()
                .map(|e| serde_json::to_string(e))
                .collect::<Result<Vec<_>, _>>()?
                .join("\n")
        }
        OutputFormat::Yaml => serde_yaml::to_string(events)?,
    };

    fs::write(output_file, content).await?;
    Ok(())
}

async fn validate_command(vrl_script: PathBuf) -> Result<()> {
    info!("Validating VRL script: {}", vrl_script.display());

    let script_content = fs::read_to_string(&vrl_script).await?;

    match OcsfTransformer::validate_script(&script_content) {
        Ok(_) => {
            info!("✓ VRL script is valid");
            Ok(())
        }
        Err(e) => {
            error!("✗ VRL script validation failed: {}", e);
            std::process::exit(1);
        }
    }
}

async fn convert_command(vector_config: PathBuf, output: Option<PathBuf>) -> Result<()> {
    info!("Converting Vector config: {}", vector_config.display());

    let content = fs::read_to_string(&vector_config).await?;
    let vector_cfg = vrl_ocsf_transformer::config::VectorConfig::from_toml(&content)?;
    let transformer_cfg = vector_cfg.to_transformer_config();

    let toml_output = toml::to_string_pretty(&transformer_cfg)?;

    if let Some(output_path) = output {
        fs::write(&output_path, toml_output).await?;
        info!("Wrote configuration to {}", output_path.display());
    } else {
        println!("{}", toml_output);
    }

    Ok(())
}

async fn run_command(config_path: PathBuf) -> Result<()> {
    info!("Loading configuration from: {}", config_path.display());

    let config = TransformerConfig::from_file(&config_path)?;
    config.validate().context("Configuration validation failed")?;

    let transformer = OcsfTransformer::with_config(config.clone()).await?;

    // Create output directory
    fs::create_dir_all(&config.output_path).await?;

    // Process based on input type
    if config.input_path.is_file() {
        process_single_file(
            &transformer,
            &config.input_path,
            &config.output_path,
            config.output_format,
            config.pretty_print,
        )
        .await?;
    } else if config.input_path.is_dir() {
        process_directory(
            &transformer,
            &config.input_path,
            &config.output_path,
            config.output_format,
            config.pretty_print,
            config.skip_errors,
        )
        .await?;
    }

    info!("Processing completed");
    Ok(())
}

#[cfg(feature = "watch-mode")]
async fn watch_command(
    vrl_script: PathBuf,
    input: PathBuf,
    output: PathBuf,
    interval: u64,
) -> Result<()> {
    use vrl_ocsf_transformer::watcher::FileWatcher;

    info!("Starting watch mode");
    info!("Watching: {}", input.display());
    info!("Interval: {} seconds", interval);

    let transformer = OcsfTransformer::new(&vrl_script).await?;
    let watcher = FileWatcher::new(transformer, input, output, interval)?;

    watcher.start().await?;

    Ok(())
}

#[cfg(feature = "metrics-support")]
async fn metrics_command(port: u16) -> Result<()> {
    use vrl_ocsf_transformer::metrics::start_metrics_server;

    info!("Starting metrics server on port {}", port);
    start_metrics_server(port).await?;

    Ok(())
}

fn setup_logging(level: &str) -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level));

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    Ok(())
}
