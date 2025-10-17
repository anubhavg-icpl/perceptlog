// src/lib.rs - Core library implementation

// Public modules organized by functionality
pub mod cli;
pub mod core;
pub mod io;
pub mod output;
pub mod processing;
pub mod utils;

// Re-exports for convenience and backward compatibility
pub use core::{
    TransformError, TransformResult, TransformerConfig, OutputFormat, LogEvent
};
pub use processing::{
    OcsfEvent, OcsfTransformer, OcsfEventBuilder, VrlRuntime
};
pub use io::{FileReader, FileWriter, FileWatcher};
pub use output::{OutputFormatter, StreamingOutputFormatter};
pub use utils::{InputValidator, ValidationResult};
pub use cli::{Cli, Commands};

// Make commonly used submodules available at the crate root for a cleaner API
pub use cli::commands as commands;
pub use core::config as config;
pub use processing::vrl as vrl;

// Backwards-compatible module aliases used by internal paths
pub use core::error as error;
pub use processing::ocsf as ocsf;
pub use utils::metrics as metrics;

