/// Integration test for history JSON format compatibility with TUI
///
/// Verifies that the Rust HistoryManager produces JSON that matches
/// the TypeScript interface expected by the TUI.
use onevox::config::HistoryConfig;
use onevox::history::{HistoryEntry, HistoryManager};
use std::fs;
use tempfile::TempDir;

#[tokio::test]
async fn test_history_json_format_matches_tui_expectations() {
    // Create a temporary directory for the test
    let temp_dir = TempDir::new().unwrap();
    let history_path = temp_dir.path().join("history.json");

    // Set environment variable to use temp directory
    unsafe {
        std::env::set_var("ONEVOX_DATA_DIR", temp_dir.path().to_str().unwrap());
    }

    // Create history manager with test config
    let config = HistoryConfig {
        enabled: true,
        max_entries: 100,
        auto_save: true,
    };
    let manager = HistoryManager::new(config).unwrap();

    // Add test entries
    let entry1 = HistoryEntry::new(
        "Hello world".to_string(),
        "ggml-base.en".to_string(),
        150,
        Some(0.95),
    );
    manager.add_entry(entry1).await.unwrap();

    let entry2 = HistoryEntry::new(
        "This is a test transcription".to_string(),
        "ggml-tiny.en".to_string(),
        85,
        None, // Some entries might not have confidence
    );
    manager.add_entry(entry2).await.unwrap();

    // Read the JSON file
    let json_content = fs::read_to_string(&history_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_content).unwrap();

    // Verify it's an array
    assert!(parsed.is_array(), "History file should contain an array");

    let entries = parsed.as_array().unwrap();
    assert_eq!(entries.len(), 2, "Should have 2 entries");

    // Verify first entry has all required fields with correct types
    let entry = &entries[0];
    assert!(entry.get("id").is_some(), "Entry should have 'id' field");
    assert!(entry.get("id").unwrap().is_u64(), "'id' should be a number");

    assert!(
        entry.get("timestamp").is_some(),
        "Entry should have 'timestamp' field"
    );
    assert!(
        entry.get("timestamp").unwrap().is_u64(),
        "'timestamp' should be a number"
    );

    assert!(
        entry.get("text").is_some(),
        "Entry should have 'text' field"
    );
    assert!(
        entry.get("text").unwrap().is_string(),
        "'text' should be a string"
    );
    assert_eq!(entry.get("text").unwrap().as_str().unwrap(), "Hello world");

    assert!(
        entry.get("model").is_some(),
        "Entry should have 'model' field"
    );
    assert!(
        entry.get("model").unwrap().is_string(),
        "'model' should be a string"
    );
    assert_eq!(
        entry.get("model").unwrap().as_str().unwrap(),
        "ggml-base.en"
    );

    assert!(
        entry.get("duration_ms").is_some(),
        "Entry should have 'duration_ms' field"
    );
    assert!(
        entry.get("duration_ms").unwrap().is_u64(),
        "'duration_ms' should be a number"
    );
    assert_eq!(entry.get("duration_ms").unwrap().as_u64().unwrap(), 150);

    assert!(
        entry.get("confidence").is_some(),
        "Entry should have 'confidence' field"
    );
    assert!(
        entry.get("confidence").unwrap().is_f64(),
        "'confidence' should be a number"
    );
    assert!((entry.get("confidence").unwrap().as_f64().unwrap() - 0.95).abs() < 0.001);

    // Verify second entry with null confidence
    let entry2 = &entries[1];
    assert!(
        entry2.get("confidence").is_some(),
        "Entry should have 'confidence' field"
    );
    assert!(
        entry2.get("confidence").unwrap().is_null(),
        "'confidence' can be null"
    );

    // Verify JSON is pretty-printed (has newlines)
    assert!(json_content.contains('\n'), "JSON should be pretty-printed");

    println!("✅ History JSON format is compatible with TUI expectations");
    println!("\nSample JSON output:");
    println!("{}", json_content);
}

#[tokio::test]
async fn test_history_can_be_read_by_tui_logic() {
    // Simulate what the TUI does when reading history

    let temp_dir = TempDir::new().unwrap();
    let history_path = temp_dir.path().join("history.json");
    unsafe {
        std::env::set_var("ONEVOX_DATA_DIR", temp_dir.path().to_str().unwrap());
    }

    // Create some history entries via Rust
    let config = HistoryConfig {
        enabled: true,
        max_entries: 100,
        auto_save: true,
    };
    let manager = HistoryManager::new(config).unwrap();

    let entry1 = HistoryEntry::new(
        "Test entry 1".to_string(),
        "model-a".to_string(),
        100,
        Some(0.9),
    );
    manager.add_entry(entry1).await.unwrap();

    let entry2 = HistoryEntry::new("Test entry 2".to_string(), "model-b".to_string(), 200, None);
    manager.add_entry(entry2).await.unwrap();

    // Now read it back as the TUI would (parsing JSON manually)
    let json_str = fs::read_to_string(&history_path).unwrap();

    // Parse using serde (simulating TypeScript JSON.parse)
    let entries: Vec<HistoryEntry> = serde_json::from_str(&json_str).unwrap();

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].text, "Test entry 1");
    assert_eq!(entries[0].model, "model-a");
    assert_eq!(entries[0].duration_ms, 100);
    assert_eq!(entries[0].confidence, Some(0.9));

    assert_eq!(entries[1].text, "Test entry 2");
    assert_eq!(entries[1].model, "model-b");
    assert_eq!(entries[1].duration_ms, 200);
    assert_eq!(entries[1].confidence, None);

    println!("✅ History can be successfully parsed using TUI logic");
}
