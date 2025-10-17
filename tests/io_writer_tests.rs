// tests/io_writer_tests.rs - Tests for file writing
use perceptlog::FileWriter;
use tempfile::TempDir;
use tokio::fs;

#[tokio::test]
async fn test_file_writer() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("test_output.txt");

    let content = "Test content\nLine 2\nLine 3";
    FileWriter::write_to_file(&output_path, content)
        .await
        .unwrap();

    // Verify file was created and contains correct content
    let read_content = fs::read_to_string(&output_path).await.unwrap();
    assert_eq!(read_content, content);
}

#[tokio::test]
async fn test_append_to_file() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("append_test.txt");

    // Write initial content
    FileWriter::write_to_file(&output_path, "Line 1\n")
        .await
        .unwrap();

    // Append more content
    FileWriter::append_to_file(&output_path, "Line 2\n")
        .await
        .unwrap();
    FileWriter::append_to_file(&output_path, "Line 3\n")
        .await
        .unwrap();

    // Verify all content is present
    let content = fs::read_to_string(&output_path).await.unwrap();
    assert_eq!(content, "Line 1\nLine 2\nLine 3\n");
}

#[tokio::test]
async fn test_write_to_invalid_path() {
    let result = FileWriter::write_to_file("/invalid/path/file.txt", "content").await;
    assert!(result.is_err());
}
