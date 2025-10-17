// src/main.rs - CLI application
use anyhow::Result;
use clap::{Parser, Subcommand};
use perceptlog::{
    TransformerConfig,
    commands::{ConvertCommand, RunCommand, TransformCommand, ValidateCommand},
    config::OutputFormat,
};
use std::path::PathBuf;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

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
    let _config = if let Some(config_path) = &cli.config {
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
            TransformCommand::execute(
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
            use perceptlog::commands::WatchCommand;
            WatchCommand::execute(vrl_script, input, output, interval).await?;
        }

        Commands::Validate { vrl_script } => {
            ValidateCommand::execute(vrl_script).await?;
        }

        Commands::Convert {
            vector_config,
            output,
        } => {
            ConvertCommand::execute(vector_config, output).await?;
        }

        Commands::Run {
            config: config_path,
        } => {
            RunCommand::execute(config_path).await?;
        }

        #[cfg(feature = "metrics-support")]
        Commands::Metrics { port } => {
            use perceptlog::commands::MetricsCommand;
            MetricsCommand::execute(port).await?;
        }
    }

    Ok(())
}

fn setup_logging(level: &str) -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    Ok(())
}
