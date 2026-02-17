//! Fuzz target for output sanitization.
//!
//! Tests that arbitrary strings cannot cause panics in the output
//! sanitizer, and that sanitization produces valid UTF-8 output.

#![no_main]

use libfuzzer_sys::fuzz_target;
use veritas_sdr::security::OutputSanitizer;

fuzz_target!(|data: &str| {
    let sanitizer = OutputSanitizer::default_sanitizer();

    // sanitize() should never panic on any input
    let result = sanitizer.sanitize(data);

    // Output should be valid UTF-8 (guaranteed by String type)
    // Check that length is reasonable
    assert!(
        result.sanitized.len() <= data.len() * 2 + 100,
        "sanitized output unexpectedly large"
    );

    // If modifications were made, the output should differ
    if result.modifications_made > 0 {
        // Note: output could still equal input if modifications cancel out
        // but that's an edge case we don't need to assert on
    }

    // validate_format() should never panic
    let _ = sanitizer.validate_format(data);
    let _ = sanitizer.validate_format(&result.sanitized);
});
