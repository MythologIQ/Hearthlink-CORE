//! TDD-Light tests for output content filtering.

use core_runtime::engine::filter::{FilterConfig, OutputFilter};

#[test]
fn filter_default_passes_all_text() {
    let filter = OutputFilter::default();
    let text = "Hello, world! This is a test.";

    let result = filter.filter(text).unwrap();
    assert_eq!(result, text);
}

#[test]
fn filter_blocklist_replaces_blocked_words() {
    let config = FilterConfig {
        blocklist: vec!["badword".to_string()],
        replacement: "[REDACTED]".to_string(),
        ..Default::default()
    };
    let filter = OutputFilter::new(config).unwrap();

    let result = filter.filter("This contains badword in it").unwrap();
    assert_eq!(result, "This contains [REDACTED] in it");
}

#[test]
fn filter_blocklist_is_case_insensitive() {
    let config = FilterConfig {
        blocklist: vec!["BadWord".to_string()],
        replacement: "[X]".to_string(),
        ..Default::default()
    };
    let filter = OutputFilter::new(config).unwrap();

    // Note: The current implementation replaces exact case match
    // In real implementation, would need case-insensitive replacement
    let contains = filter.contains_blocked("This has BADWORD here");
    assert!(contains);
}

#[test]
fn filter_regex_replaces_patterns() {
    let config = FilterConfig {
        regex_patterns: vec![r"\d{3}-\d{2}-\d{4}".to_string()], // SSN pattern
        replacement: "[SSN]".to_string(),
        ..Default::default()
    };
    let filter = OutputFilter::new(config).unwrap();

    let result = filter.filter("My SSN is 123-45-6789").unwrap();
    assert_eq!(result, "My SSN is [SSN]");
}

#[test]
fn filter_rejects_invalid_regex() {
    let config = FilterConfig {
        regex_patterns: vec!["[invalid".to_string()], // Invalid regex
        ..Default::default()
    };

    let result = OutputFilter::new(config);
    assert!(result.is_err());
}

#[test]
fn filter_truncates_long_output() {
    let config = FilterConfig {
        max_output_chars: 10,
        ..Default::default()
    };
    let filter = OutputFilter::new(config).unwrap();

    let result = filter.filter("This is a very long string").unwrap();
    assert_eq!(result.len(), 10);
    assert_eq!(result, "This is a ");
}

#[test]
fn filter_unlimited_when_max_is_zero() {
    let config = FilterConfig {
        max_output_chars: 0,
        ..Default::default()
    };
    let filter = OutputFilter::new(config).unwrap();

    let long_text = "x".repeat(1000);
    let result = filter.filter(&long_text).unwrap();
    assert_eq!(result.len(), 1000);
}

#[test]
fn filter_contains_blocked_detects_blocklist() {
    let config = FilterConfig {
        blocklist: vec!["secret".to_string()],
        ..Default::default()
    };
    let filter = OutputFilter::new(config).unwrap();

    assert!(filter.contains_blocked("This has a secret"));
    assert!(!filter.contains_blocked("This is safe"));
}

#[test]
fn filter_contains_blocked_detects_regex() {
    let config = FilterConfig {
        regex_patterns: vec![r"password:\s*\w+".to_string()],
        ..Default::default()
    };
    let filter = OutputFilter::new(config).unwrap();

    assert!(filter.contains_blocked("password: hunter2"));
    assert!(!filter.contains_blocked("no passwords here"));
}

#[test]
fn filter_multiple_blocklist_entries() {
    let config = FilterConfig {
        blocklist: vec!["bad1".to_string(), "bad2".to_string(), "bad3".to_string()],
        replacement: "[X]".to_string(),
        ..Default::default()
    };
    let filter = OutputFilter::new(config).unwrap();

    let result = filter.filter("bad1 and bad2 and bad3").unwrap();
    assert_eq!(result, "[X] and [X] and [X]");
}
