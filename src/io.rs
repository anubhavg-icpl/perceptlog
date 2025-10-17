// src/io.rs - File I/O operations module
use crate::{LogEvent, TransformResult, error::TransformError};
use anyhow::Result;
use std::path::Path;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_stream::wrappers::LinesStream;
use tracing::{debug, info};

/// File reader for processing log files
pub struct FileReader;

impl FileReader {
    /// Read all lines from a file and convert them to LogEvents
    pub async fn read_file_to_events(
        file_path: impl AsRef<Path>,
    ) -> TransformResult<Vec<LogEvent>> {
        let file_path = file_path.as_ref();
        info!("Reading file: {}", file_path.display());

        let file = fs::File::open(file_path)
            .await
            .map_err(|e| TransformError::IoError(e.to_string()))?;

        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut events = Vec::new();

        while let Some(line) = lines
            .next_line()
            .await
            .map_err(|e| TransformError::IoError(e.to_string()))?
        {
            if !line.trim().is_empty() {
                events.push(LogEvent::new(line));
            }
        }

        debug!("Read {} events from file", events.len());
        Ok(events)
    }

    /// Create a stream of LogEvents from a file
    pub async fn stream_file_events(
        file_path: impl AsRef<Path>,
    ) -> TransformResult<impl futures::Stream<Item = TransformResult<LogEvent>>> {
        let file_path = file_path.as_ref();
        info!("Streaming file: {}", file_path.display());

        let file = fs::File::open(file_path)
            .await
            .map_err(|e| TransformError::IoError(e.to_string()))?;

        let reader = BufReader::new(file);
        let lines = reader.lines();
        let stream = LinesStream::new(lines);

        Ok(futures::StreamExt::map(
            stream,
            |line_result| match line_result {
                Ok(line) => {
                    if line.trim().is_empty() {
                        Err(TransformError::ParseError("Empty line".to_string()))
                    } else {
                        Ok(LogEvent::new(line))
                    }
                }
                Err(e) => Err(TransformError::IoError(e.to_string())),
            },
        ))
    }

    /// Read directory and get all log files
    pub async fn get_log_files_from_directory(
        dir_path: impl AsRef<Path>,
        include_patterns: &[String],
        exclude_patterns: &[String],
    ) -> TransformResult<Vec<std::path::PathBuf>> {
        let dir_path = dir_path.as_ref();
        info!("Scanning directory: {}", dir_path.display());

        let mut entries = fs::read_dir(dir_path)
            .await
            .map_err(|e| TransformError::IoError(e.to_string()))?;

        let mut log_files = Vec::new();

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| TransformError::IoError(e.to_string()))?
        {
            let path = entry.path();

            if path.is_file()
                && Self::should_process_file(&path, include_patterns, exclude_patterns)
            {
                log_files.push(path);
            }
        }

        info!("Found {} log files in directory", log_files.len());
        Ok(log_files)
    }

    /// Check if a file should be processed based on include/exclude patterns
    fn should_process_file(
        path: &Path,
        include_patterns: &[String],
        exclude_patterns: &[String],
    ) -> bool {
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Check exclude patterns first
        for pattern in exclude_patterns {
            if glob::Pattern::new(pattern)
                .map(|p| p.matches(file_name))
                .unwrap_or(false)
            {
                debug!(
                    "Excluding file {} (matches pattern: {})",
                    file_name, pattern
                );
                return false;
            }
        }

        // If no include patterns specified, include all files (that don't match exclude)
        if include_patterns.is_empty() {
            return true;
        }

        // Check include patterns
        for pattern in include_patterns {
            if glob::Pattern::new(pattern)
                .map(|p| p.matches(file_name))
                .unwrap_or(false)
            {
                debug!(
                    "Including file {} (matches pattern: {})",
                    file_name, pattern
                );
                return true;
            }
        }

        debug!("Excluding file {} (no include pattern matches)", file_name);
        false
    }
}

/// File writer for saving processed events
pub struct FileWriter;

impl FileWriter {
    /// Write content to a file
    pub async fn write_to_file(file_path: impl AsRef<Path>, content: &str) -> Result<()> {
        let file_path = file_path.as_ref();
        debug!("Writing to file: {}", file_path.display());

        // Create parent directories if they don't exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        fs::write(file_path, content).await?;
        debug!("Successfully wrote {} bytes to file", content.len());
        Ok(())
    }

    /// Append content to a file
    pub async fn append_to_file(file_path: impl AsRef<Path>, content: &str) -> Result<()> {
        let file_path = file_path.as_ref();
        debug!("Appending to file: {}", file_path.display());

        // Create parent directories if they don't exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .await?;

        use tokio::io::AsyncWriteExt;
        file.write_all(content.as_bytes()).await?;
        file.flush().await?;

        debug!("Successfully appended {} bytes to file", content.len());
        Ok(())
    }

    /// Ensure directory exists
    pub async fn ensure_directory_exists(dir_path: impl AsRef<Path>) -> Result<()> {
        let dir_path = dir_path.as_ref();
        if !dir_path.exists() {
            info!("Creating directory: {}", dir_path.display());
            fs::create_dir_all(dir_path).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_file_reader() {
        // Create temporary file with test content
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test log line 1").unwrap();
        writeln!(temp_file, "Test log line 2").unwrap();
        writeln!(temp_file, "").unwrap(); // Empty line should be skipped
        writeln!(temp_file, "Test log line 3").unwrap();
        temp_file.flush().unwrap();

        let events = FileReader::read_file_to_events(temp_file.path())
            .await
            .unwrap();

        assert_eq!(events.len(), 3);
        assert_eq!(events[0].message, "Test log line 1");
        assert_eq!(events[1].message, "Test log line 2");
        assert_eq!(events[2].message, "Test log line 3");
    }

    #[tokio::test]
    async fn test_file_writer() {
        let temp_file = NamedTempFile::new().unwrap();
        let content = "Test content";

        FileWriter::write_to_file(temp_file.path(), content)
            .await
            .unwrap();

        let read_content = fs::read_to_string(temp_file.path()).await.unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_file_pattern_matching() {
        let include_patterns = vec!["*.log".to_string(), "auth*".to_string()];
        let exclude_patterns = vec!["*.tmp".to_string()];

        assert!(FileReader::should_process_file(
            Path::new("test.log"),
            &include_patterns,
            &exclude_patterns
        ));

        assert!(FileReader::should_process_file(
            Path::new("auth.txt"),
            &include_patterns,
            &exclude_patterns
        ));

        assert!(!FileReader::should_process_file(
            Path::new("test.tmp"),
            &include_patterns,
            &exclude_patterns
        ));

        assert!(!FileReader::should_process_file(
            Path::new("other.txt"),
            &include_patterns,
            &exclude_patterns
        ));
    }
}
