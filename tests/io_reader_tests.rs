// tests/io_reader_tests.rs - Tests for file reading
use perceptlog::FileReader;
use std::io::Write;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_file_reader() {
    // Create temporary file with test content
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "Test log line 1").unwrap();
    writeln!(temp_file, "Test log line 2").unwrap();
    writeln!(temp_file).unwrap(); // Empty line should be skipped
    writeln!(temp_file, "Test log line 3").unwrap();
    temp_file.flush().unwrap();

    let events = FileReader::read_file_to_events(temp_file.path())
        .await
        .unwrap();

    // Should have 3 non-empty log lines
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].message, "Test log line 1");
    assert_eq!(events[1].message, "Test log line 2");
    assert_eq!(events[2].message, "Test log line 3");
}

#[tokio::test]
async fn test_file_reader_empty_file() {
    let temp_file = NamedTempFile::new().unwrap();
    
    let events = FileReader::read_file_to_events(temp_file.path())
        .await
        .unwrap();
    
    assert_eq!(events.len(), 0);
}

#[tokio::test]
async fn test_file_reader_nonexistent() {
    let result = FileReader::read_file_to_events("/nonexistent/file.log").await;
    assert!(result.is_err());
}

#[test]
fn test_file_pattern_matching() {
    let patterns = vec!["*.log".to_string(), "auth*".to_string()];
    
    // Test that patterns are valid
    for pattern in patterns {
        assert!(glob::Pattern::new(&pattern).is_ok());
    }
}
