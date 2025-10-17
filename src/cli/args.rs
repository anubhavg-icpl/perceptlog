// src/cli/args.rs - Command-line argument definitions
use crate::core::config::OutputFormat;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "perceptlog",
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = "Transform Linux authentication logs to OCSF format"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info", env = "PERCEPTLOG_LOG_LEVEL")]
    pub log_level: String,

    /// Configuration file path
    #[arg(short, long, env = "PERCEPTLOG_CONFIG")]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Transform log files to OCSF format
    Transform {
        /// Transform script file path (.perceptlog)
        #[arg(short = 's', long)]
        script: PathBuf,

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
        /// Transform script file path (.perceptlog)
        #[arg(short = 's', long)]
        script: PathBuf,

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

    /// Validate transform script syntax
    Validate {
        /// Transform script file path (.perceptlog)
        script: PathBuf,
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
        /// Port for metrics server
        #[arg(short, long, default_value = "9090")]
        port: u16,
    },
}