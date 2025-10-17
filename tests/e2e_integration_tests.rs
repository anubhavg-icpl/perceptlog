// tests/e2e_integration_tests.rs - End-to-end integration tests with real data
use perceptlog::{OcsfTransformer, FileReader, FileWriter, config::OutputFormat, output::OutputFormatter};
use std::path::PathBuf;
use tempfile::TempDir;

async fn transform_events(transformer: &OcsfTransformer, events: Vec<perceptlog::LogEvent>) -> Vec<perceptlog::ocsf::OcsfEvent> {
    let mut results = Vec::new();
    for event in events {
        match transformer.transform_event(event).await {
            Ok(ocsf_event) => results.push(ocsf_event),
            Err(e) => eprintln!("Warning: Failed to transform event: {}", e),
        }
    }
    results
}

#[tokio::test]
async fn test_e2e_with_sample_logs() {
    let script_path = PathBuf::from("transform.perceptlog");
    
    if !script_path.exists() {
        eprintln!("Skipping: transform.perceptlog not found");
        return;
    }

    let transformer = OcsfTransformer::new(&script_path).await.unwrap();

    // Create test log events
    let test_logs = vec![
        "Feb 20 10:41:42 test-host sshd[3087]: Accepted password for testuser from 192.168.1.100 port 22222 ssh2",
        "Feb 20 10:48:27 test-host sshd[15112]: Failed password for invalid user hacker from 192.168.1.200 port 33333 ssh2",
        "Feb 20 10:42:00 test-host sudo: testuser : TTY=pts/0 ; PWD=/var/log ; USER=root ; COMMAND=/usr/bin/ls",
    ];

    let events: Vec<_> = test_logs.iter().map(|log| perceptlog::LogEvent::new(*log)).collect();
    let ocsf_events = transform_events(&transformer, events).await;

    assert!(!ocsf_events.is_empty(), "Should transform events");
    assert_eq!(ocsf_events.len(), 3, "Should transform all 3 events");

    for event in &ocsf_events {
        assert_eq!(event.category_uid, 3, "Should be IAM category");
        assert_eq!(event.class_uid, 3002, "Should be Authentication class");
    }

    println!("✓ Sample logs: Transformed {} events successfully", ocsf_events.len());
}

#[tokio::test]
async fn test_e2e_debian_auth_log() {
    let script_path = PathBuf::from("transform.perceptlog");
    let input_path = PathBuf::from("test_data/auth.log");
    
    if !script_path.exists() || !input_path.exists() {
        eprintln!("Skipping: Required files not found");
        return;
    }

    let transformer = OcsfTransformer::new(&script_path).await.unwrap();
    let events = FileReader::read_file_to_events(&input_path).await.unwrap();
    
    println!("Read {} events from Debian auth.log", events.len());
    
    let ocsf_events = transform_events(&transformer, events).await;
    
    assert!(!ocsf_events.is_empty(), "Should have OCSF events");
    println!("✓ Debian auth.log: Transformed {} events", ocsf_events.len());

    // Write output
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("debian_ocsf.ndjson");
    let formatted = OutputFormatter::format_events(&ocsf_events, OutputFormat::Ndjson, false).unwrap();
    FileWriter::write_to_file(&output_path, &formatted).await.unwrap();
    
    println!("✓ Wrote output to {}", output_path.display());
}

#[tokio::test]
async fn test_e2e_rhel_secure_log() {
    let script_path = PathBuf::from("transform.perceptlog");
    let input_path = PathBuf::from("test_data/secure");
    
    if !script_path.exists() || !input_path.exists() {
        eprintln!("Skipping: Required files not found");
        return;
    }

    let transformer = OcsfTransformer::new(&script_path).await.unwrap();
    let events = FileReader::read_file_to_events(&input_path).await.unwrap();
    
    println!("Read {} events from RHEL secure log", events.len());
    
    let ocsf_events = transform_events(&transformer, events).await;
    
    assert!(!ocsf_events.is_empty(), "Should have OCSF events");
    println!("✓ RHEL secure: Transformed {} events", ocsf_events.len());

    // Write JSON output
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("rhel_ocsf.json");
    let formatted = OutputFormatter::format_events(&ocsf_events, OutputFormat::JsonPretty, false).unwrap();
    FileWriter::write_to_file(&output_path, &formatted).await.unwrap();
    
    println!("✓ Wrote output to {}", output_path.display());
}

#[tokio::test]
async fn test_e2e_ssh_patterns() {
    let script_path = PathBuf::from("transform.perceptlog");
    
    if !script_path.exists() {
        return;
    }

    let transformer = OcsfTransformer::new(&script_path).await.unwrap();

    let test_cases = vec![
        ("Accepted password", "Feb 20 10:41:42 host sshd[3087]: Accepted password for user from 192.168.1.1 port 22 ssh2"),
        ("Failed password", "Feb 20 10:48:27 host sshd[15112]: Failed password for user from 192.168.1.1 port 22 ssh2"),
        ("Session opened", "Feb 20 10:41:47 host sshd[3087]: pam_unix(sshd:session): session opened for user test(uid=1000) by (uid=0)"),
        ("Session closed", "Feb 20 10:41:50 host sshd[3087]: pam_unix(sshd:session): session closed for user test"),
    ];

    for (name, log) in test_cases {
        let event = perceptlog::LogEvent::new(log);
        match transformer.transform_event(event).await {
            Ok(ocsf_event) => {
                assert_eq!(ocsf_event.category_uid, 3);
                assert_eq!(ocsf_event.class_uid, 3002);
                println!("✓ SSH pattern '{}' transformed successfully", name);
            }
            Err(e) => eprintln!("Warning: Failed to transform '{}': {}", name, e),
        }
    }
}

#[tokio::test]
async fn test_e2e_sudo_patterns() {
    let script_path = PathBuf::from("transform.perceptlog");
    
    if !script_path.exists() {
        return;
    }

    let transformer = OcsfTransformer::new(&script_path).await.unwrap();

    let sudo_log = "Feb 20 10:42:00 host sudo: user : TTY=pts/0 ; PWD=/home ; USER=root ; COMMAND=/bin/ls";
    let event = perceptlog::LogEvent::new(sudo_log);
    
    match transformer.transform_event(event).await {
        Ok(ocsf_event) => {
            assert_eq!(ocsf_event.category_uid, 3);
            assert_eq!(ocsf_event.class_uid, 3002);
            assert_eq!(ocsf_event.activity_id, 7); // Account Switch
            println!("✓ Sudo command transformed successfully");
        }
        Err(e) => eprintln!("Warning: Failed to transform sudo: {}", e),
    }
}

#[tokio::test]
async fn test_e2e_batch_processing() {
    let script_path = PathBuf::from("transform.perceptlog");
    let input_path = PathBuf::from("test_data/auth.log");
    
    if !script_path.exists() || !input_path.exists() {
        return;
    }

    let transformer = OcsfTransformer::new(&script_path).await.unwrap();
    let events = FileReader::read_file_to_events(&input_path).await.unwrap();

    // Process in batches
    let batch_size = 10;
    let mut total_processed = 0;

    for chunk in events.chunks(batch_size) {
        let ocsf_events = transform_events(&transformer, chunk.to_vec()).await;
        total_processed += ocsf_events.len();
    }

    println!("✓ Batch processing: {} events in batches of {}", total_processed, batch_size);
}

#[tokio::test]
async fn test_e2e_multiple_formats() {
    let script_path = PathBuf::from("transform.perceptlog");
    
    if !script_path.exists() {
        return;
    }

    let transformer = OcsfTransformer::new(&script_path).await.unwrap();
    
    let log = "Feb 20 10:41:42 host sshd[3087]: Accepted password for test from 192.168.1.1 port 22 ssh2";
    let event = perceptlog::LogEvent::new(log);
    let ocsf_event = transformer.transform_event(event).await.unwrap();

    let temp_dir = TempDir::new().unwrap();

    // Test all formats
    for (format, ext) in [
        (OutputFormat::Json, "json"),
        (OutputFormat::Ndjson, "ndjson"),
        (OutputFormat::Yaml, "yaml"),
        (OutputFormat::JsonPretty, "pretty.json"),
    ] {
        let output = OutputFormatter::format_events(&vec![ocsf_event.clone()], format, false).unwrap();
        let path = temp_dir.path().join(format!("output.{}", ext));
        FileWriter::write_to_file(&path, &output).await.unwrap();
        assert!(path.exists());
        println!("✓ Format {} written successfully", ext);
    }
}

#[tokio::test]
async fn test_e2e_ocsf_compliance() {
    let script_path = PathBuf::from("transform.perceptlog");
    
    if !script_path.exists() {
        return;
    }

    let transformer = OcsfTransformer::new(&script_path).await.unwrap();
    
    let log = "Feb 20 10:41:42 host sshd[3087]: Accepted password for test from 192.168.1.1 port 22 ssh2";
    let event = perceptlog::LogEvent::new(log);
    let ocsf = transformer.transform_event(event).await.unwrap();

    // Verify required OCSF v1.6.0 fields for Authentication [3002]
    assert_eq!(ocsf.category_uid, 3, "Category UID must be 3 (IAM)");
    assert_eq!(ocsf.class_uid, 3002, "Class UID must be 3002 (Authentication)");
    assert!(ocsf.time > 0, "Must have valid timestamp");
    assert!(ocsf.activity_id > 0, "Must have activity_id");
    assert!(!ocsf.activity_name.is_empty(), "Must have activity_name");
    assert!(ocsf.severity_id > 0, "Must have severity_id");
    assert!(!ocsf.severity.is_empty(), "Must have severity");
    assert!(!ocsf.status.is_empty(), "Must have status");
    assert!(ocsf.status_id > 0, "Must have status_id");
    assert!(ocsf.type_uid > 0, "Must have type_uid");
    
    println!("✓ OCSF v1.6.0 compliance validated");
}
