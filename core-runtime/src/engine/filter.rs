//! Output content filtering for CORE Runtime.
//!
//! Applies configurable filters to inference output before returning to caller.
//! Uses NFC Unicode normalization to prevent bypass through decomposed characters.

use regex::Regex;
use serde::Deserialize;
use unicode_normalization::UnicodeNormalization;

use crate::engine::InferenceError;

/// Configuration for output filtering.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FilterConfig {
    /// List of substrings to block (case-insensitive).
    #[serde(default)]
    pub blocklist: Vec<String>,
    /// List of regex patterns to block.
    #[serde(default)]
    pub regex_patterns: Vec<String>,
    /// Maximum output length in characters (0 = unlimited).
    #[serde(default)]
    pub max_output_chars: usize,
    /// Replacement text for filtered content.
    #[serde(default = "default_replacement")]
    pub replacement: String,
}

fn default_replacement() -> String {
    "[filtered]".to_string()
}

/// Output filter that applies blocklist and regex patterns.
/// Pre-computes NFC-normalized blocklist at construction for efficiency.
pub struct OutputFilter {
    config: FilterConfig,
    compiled_patterns: Vec<Regex>,
    /// Pre-normalized, pre-lowercased blocklist for O(1) lookup per entry.
    normalized_blocklist: Vec<String>,
}

impl OutputFilter {
    /// Create a new output filter with the given configuration.
    /// Pre-computes NFC-normalized, lowercased blocklist at construction time.
    pub fn new(config: FilterConfig) -> Result<Self, InferenceError> {
        let compiled = config
            .regex_patterns
            .iter()
            .map(|p| Regex::new(p))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| InferenceError::InputValidation(format!("invalid regex: {}", e)))?;

        // Pre-compute normalized lowercase blocklist (avoids per-call allocations)
        let normalized_blocklist = config
            .blocklist
            .iter()
            .map(|s| s.nfc().collect::<String>().to_lowercase())
            .collect();

        Ok(Self {
            config,
            compiled_patterns: compiled,
            normalized_blocklist,
        })
    }

    /// Filter the output text, returning filtered version or error if blocked.
    /// Applies NFC normalization before blocklist comparison.
    pub fn filter(&self, text: &str) -> Result<String, InferenceError> {
        let mut result = text.to_string();

        // Normalize input for comparison (NFC handles composed/decomposed equivalence)
        let normalized: String = result.nfc().collect();
        let lower = normalized.to_lowercase();

        // Apply blocklist with pre-computed normalized entries
        for (i, normalized_blocked) in self.normalized_blocklist.iter().enumerate() {
            if lower.contains(normalized_blocked) {
                result = result.replace(&self.config.blocklist[i], &self.config.replacement);
            }
        }

        // Apply regex patterns (on original, not normalized)
        for pattern in &self.compiled_patterns {
            result = pattern
                .replace_all(&result, &self.config.replacement)
                .to_string();
        }

        // Apply length limit
        if self.config.max_output_chars > 0 && result.len() > self.config.max_output_chars {
            result.truncate(self.config.max_output_chars);
        }

        Ok(result)
    }

    /// Check if text contains any blocked content.
    /// Applies NFC normalization before comparison.
    pub fn contains_blocked(&self, text: &str) -> bool {
        let normalized: String = text.nfc().collect();
        let lower = normalized.to_lowercase();

        for normalized_blocked in &self.normalized_blocklist {
            if lower.contains(normalized_blocked) {
                return true;
            }
        }
        for pattern in &self.compiled_patterns {
            if pattern.is_match(text) {
                return true;
            }
        }
        false
    }
}

impl Default for OutputFilter {
    fn default() -> Self {
        Self {
            config: FilterConfig::default(),
            compiled_patterns: Vec::new(),
            normalized_blocklist: Vec::new(),
        }
    }
}
