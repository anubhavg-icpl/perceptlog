// src/main.rs - CLI application
use anyhow::Result;
use clap::Parser;
use perceptlog::{
    TransformerConfig,
    commands::{ConvertCommand, RunCommand, TransformCommand, ValidateCommand},
    Cli, Commands,
};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

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
