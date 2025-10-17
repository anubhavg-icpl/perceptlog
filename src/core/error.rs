// src/error.rs - Error types and handling
use std::fmt;
use thiserror::Error;

/// Main error type for transformation operations
#[derive(Error, Debug)]
pub enum TransformError {
    #[error("VRL compilation error: {0}")]
    CompileError(String),

    #[error("Transformation error: {0}")]
    TransformError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Watch error: {0}")]
    WatchError(String),

    #[error("Metrics error: {0}")]
    MetricsError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Result type used across the crate for operations that may return a TransformError
pub type TransformResult<T> = Result<T, TransformError>;

impl From<std::io::Error> for TransformError {
    fn from(err: std::io::Error) -> Self {
        TransformError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for TransformError {
    fn from(err: serde_json::Error) -> Self {
        TransformError::ParseError(err.to_string())
    }
}

impl From<serde_yaml::Error> for TransformError {
    fn from(err: serde_yaml::Error) -> Self {
        TransformError::ParseError(err.to_string())
    }
}

impl From<config::ConfigError> for TransformError {
    fn from(err: config::ConfigError) -> Self {
        TransformError::ConfigError(err.to_string())
    }
}

impl From<anyhow::Error> for TransformError {
    fn from(err: anyhow::Error) -> Self {
        TransformError::Unknown(err.to_string())
    }
}

/// Result type for batch operations
pub type BatchResult<T> = Result<Vec<T>, BatchError>;

/// Error type for batch processing
#[derive(Debug)]
pub struct BatchError {
    pub successful: usize,
    pub failed: usize,
    pub errors: Vec<(usize, TransformError)>,
}

impl fmt::Display for BatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Batch processing failed: {} successful, {} failed",
            self.successful, self.failed
        )?;

        if !self.errors.is_empty() {
            writeln!(f, "\nErrors:")?;
            for (index, error) in &self.errors {
                writeln!(f, "  Line {index}: {error}")?;
            }
        }

        Ok(())
    }
}

impl std::error::Error for BatchError {}

/// Validation error details
#[derive(Debug, Clone)]
pub struct ValidationErrorDetail {
    pub field: String,
    pub message: String,
    pub line_number: Option<usize>,
}

/// Collection of validation errors
#[derive(Debug)]
pub struct ValidationErrors {
    pub errors: Vec<ValidationErrorDetail>,
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Validation failed with {} errors:", self.errors.len())?;
        for error in &self.errors {
            if let Some(line) = error.line_number {
                writeln!(f, "  Line {}: {} - {}", line, error.field, error.message)?;
            } else {
                writeln!(f, "  {} - {}", error.field, error.message)?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for ValidationErrors {}

/// Helper trait for error context
pub trait ErrorContext<T> {
    fn context(self, msg: &str) -> Result<T, TransformError>;
    fn with_context<F>(self, f: F) -> Result<T, TransformError>
    where
        F: FnOnce() -> String;
}

impl<T, E> ErrorContext<T> for Result<T, E>
where
    E: Into<TransformError>,
{
    fn context(self, msg: &str) -> Result<T, TransformError> {
        self.map_err(|e| {
            let base_error = e.into();
            TransformError::Unknown(format!("{msg}: {base_error}"))
        })
    }

    fn with_context<F>(self, f: F) -> Result<T, TransformError>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let base_error = e.into();
            TransformError::Unknown(format!("{}: {}", f(), base_error))
        })
    }
}

