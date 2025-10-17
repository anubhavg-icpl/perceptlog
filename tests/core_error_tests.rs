// tests/core_error_tests.rs - Tests for error handling
use perceptlog::error::{TransformError, BatchError, ValidationErrors, ValidationErrorDetail};

#[test]
fn test_error_display() {
    let err = TransformError::CompileError("Invalid PerceptLog syntax".to_string());
    assert_eq!(err.to_string(), "Script compilation error: Invalid PerceptLog syntax");

    let err = TransformError::ParseError("Invalid JSON".to_string());
    assert_eq!(err.to_string(), "Parse error: Invalid JSON");
}

#[test]
fn test_batch_error_display() {
    let errors = vec![
        (10, TransformError::ParseError("Invalid format".to_string())),
        (15, TransformError::ValidationError("Missing field".to_string())),
    ];

    let batch_err = BatchError {
        successful: 100,
        failed: 2,
        errors,
    };

    let display = batch_err.to_string();
    assert!(display.contains("100 successful"));
    assert!(display.contains("2 failed"));
    assert!(display.contains("Line 10"));
    assert!(display.contains("Line 15"));
}

#[test]
fn test_validation_errors_display() {
    let errors = vec![
        ValidationErrorDetail {
            field: "user".to_string(),
            message: "Required field missing".to_string(),
            line_number: Some(42),
        },
        ValidationErrorDetail {
            field: "timestamp".to_string(),
            message: "Invalid format".to_string(),
            line_number: None,
        },
    ];

    let validation_err = ValidationErrors { errors };
    let display = validation_err.to_string();

    assert!(display.contains("2 errors"));
    assert!(display.contains("Line 42: user"));
    assert!(display.contains("timestamp - Invalid format"));
}

#[test]
fn test_transform_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let transform_err: TransformError = io_err.into();
    
    match transform_err {
        TransformError::IoError(_) => (),
        _ => panic!("Expected IoError"),
    }
}
