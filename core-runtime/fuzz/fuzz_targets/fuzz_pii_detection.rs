//! Fuzz target for PII detection.
//!
//! Tests that arbitrary strings cannot cause panics in the PII
//! detector, and that redaction produces valid output.

#![no_main]

use libfuzzer_sys::fuzz_target;
use gg_core::security::PIIDetector;

fuzz_target!(|data: &str| {
    let detector = PIIDetector::new();

    // detect() should never panic on any input
    let matches = detector.detect(data);

    // contains_pii() should be consistent with detect()
    let has_pii = detector.contains_pii(data);
    assert_eq!(
        has_pii,
        !matches.is_empty(),
        "contains_pii inconsistent with detect"
    );

    // redact() should never panic
    let redacted = detector.redact(data);

    // Redacted output should not be longer than original + redaction markers
    // Each redaction adds "[REDACTED:TYPE]" which is max ~20 chars
    let max_growth = matches.len() * 25;
    assert!(
        redacted.len() <= data.len() + max_growth,
        "redacted output unexpectedly large"
    );

    // If PII was detected, redacted should differ from original
    if has_pii {
        assert_ne!(
            redacted, data,
            "PII detected but redacted output unchanged"
        );
    }
});
