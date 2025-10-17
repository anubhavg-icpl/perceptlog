// tests/utils_validation_tests.rs - Tests for input validation
use perceptlog::utils::{InputValidator, ValidationResult};
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

#[test]
fn test_validate_script_file() {
    // Test with valid file
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, ". = .").unwrap();
    temp_file.flush().unwrap();

    assert!(InputValidator::validate_script_file(temp_file.path()).is_ok());

    // Test with non-existent file
    let non_existent = std::path::PathBuf::from("/non/existent/file.perceptlog");
    assert!(InputValidator::validate_script_file(&non_existent).is_err());
}

#[test]
fn test_validate_input_path() {
    // Test with valid file
    let temp_file = NamedTempFile::new().unwrap();
    assert!(InputValidator::validate_input_path(temp_file.path()).is_ok());

    // Test with valid directory
    let temp_dir = TempDir::new().unwrap();
    assert!(InputValidator::validate_input_path(temp_dir.path()).is_ok());

    // Test with non-existent path
    let non_existent = std::path::PathBuf::from("/non/existent/path");
    assert!(InputValidator::validate_input_path(&non_existent).is_err());
}

#[test]
fn test_validate_batch_size() {
    assert!(InputValidator::validate_batch_size(100).is_ok());
    assert!(InputValidator::validate_batch_size(1).is_ok());
    assert!(InputValidator::validate_batch_size(10000).is_ok());
    
    assert!(InputValidator::validate_batch_size(0).is_err());
    assert!(InputValidator::validate_batch_size(20000).is_err());
}

#[test]
fn test_validate_log_level() {
    assert!(InputValidator::validate_log_level("info").is_ok());
    assert!(InputValidator::validate_log_level("debug").is_ok());
    assert!(InputValidator::validate_log_level("warn").is_ok());
    assert!(InputValidator::validate_log_level("error").is_ok());
    assert!(InputValidator::validate_log_level("trace").is_ok());
    assert!(InputValidator::validate_log_level("DEBUG").is_ok()); // Case insensitive
    
    assert!(InputValidator::validate_log_level("invalid").is_err());
    assert!(InputValidator::validate_log_level("").is_err());
}

#[test]
fn test_validate_file_patterns() {
    let valid_patterns = vec!["*.log".to_string(), "auth*".to_string(), "**/*.txt".to_string()];
    assert!(InputValidator::validate_file_patterns(&valid_patterns).is_ok());

    let invalid_patterns = vec!["[invalid".to_string()];
    assert!(InputValidator::validate_file_patterns(&invalid_patterns).is_err());
    
    let empty_patterns: Vec<String> = vec![];
    assert!(InputValidator::validate_file_patterns(&empty_patterns).is_ok());
}

#[test]
fn test_validation_result() {
    let mut result = ValidationResult::new();
    assert!(result.is_ok());
    assert!(result.errors.is_empty());
    assert!(result.warnings.is_empty());

    result.add_warning("This is a warning");
    assert!(result.is_ok()); // Warnings don't affect validity
    assert_eq!(result.warnings.len(), 1);

    result.add_error("This is an error");
    assert!(!result.is_ok()); // Errors affect validity
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.warnings.len(), 1);
}

#[test]
fn test_validation_result_multiple_errors() {
    let mut result = ValidationResult::new();
    
    result.add_error("Error 1");
    result.add_error("Error 2");
    result.add_error("Error 3");
    
    assert!(!result.is_ok());
    assert_eq!(result.errors.len(), 3);
}
