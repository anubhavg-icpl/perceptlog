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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_file_writer() {
        let temp_file = NamedTempFile::new().unwrap();
        let content = "Test content";

        FileWriter::write_to_file(temp_file.path(), content).await.unwrap();
        
        let read_content = fs::read_to_string(temp_file.path()).await.unwrap();
        assert_eq!(read_content, content);
    }

    #[tokio::test]
    async fn test_append_to_file() {
        let temp_file = NamedTempFile::new().unwrap();
        
        FileWriter::append_to_file(temp_file.path(), "Line 1\n").await.unwrap();
        FileWriter::append_to_file(temp_file.path(), "Line 2\n").await.unwrap();
        
        let content = fs::read_to_string(temp_file.path()).await.unwrap();
        assert_eq!(content, "Line 1\nLine 2\n");
    }
}