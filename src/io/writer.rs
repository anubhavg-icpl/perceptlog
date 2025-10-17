// src/io/writer.rs - File writing operations
use anyhow::Result;
use std::path::Path;
use tokio::fs;
use tracing::debug;

/// File writer for saving processed events
pub struct FileWriter;

impl FileWriter {
    /// Write content to a file
    pub async fn write_to_file(
        file_path: impl AsRef<Path>,
        content: &str,
    ) -> Result<()> {
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
    pub async fn append_to_file(
        file_path: impl AsRef<Path>,
        content: &str,
    ) -> Result<()> {
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
            debug!("Creating directory: {}", dir_path.display());
            fs::create_dir_all(dir_path).await?;
        }
        Ok(())
    }
}

