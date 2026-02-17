//! Fuzz target for prompt injection detection.
//!
//! Tests that arbitrary strings cannot cause panics in the prompt
//! injection scanner, and that it handles edge cases gracefully.

#![no_main]

use libfuzzer_sys::fuzz_target;
use veritas_sdr::security::PromptInjectionFilter;

fuzz_target!(|data: &str| {
    let filter = PromptInjectionFilter::new(true);

    // scan() should never panic on any input
    let (detected, severity, matches) = filter.scan(data);

    // Basic invariants that should always hold
    if matches.is_empty() {
        assert!(!detected, "detected=true but no matches found");
    }
    if detected {
        assert!(severity > 0, "detected=true but severity=0");
    }

    // sanitize() should never panic
    let (sanitized, _was_modified) = filter.sanitize(data);

    // Sanitized output should never be longer than input + some overhead
    assert!(
        sanitized.len() <= data.len() + 100,
        "sanitized output unexpectedly large"
    );
});
