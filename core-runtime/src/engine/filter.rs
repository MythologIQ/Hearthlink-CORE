//! Output content filtering for CORE Runtime.
//!
//! Applies configurable filters to inference output before returning to caller.

use regex::Regex;
use serde::Deserialize;

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
pub struct OutputFilter {
    config: FilterConfig,
    compiled_patterns: Vec<Regex>,
}

impl OutputFilter {
    /// Create a new output filter with the given configuration.
    pub fn new(config: FilterConfig) -> Result<Self, InferenceError> {
        let compiled = config
            .regex_patterns
            .iter()
            .map(|p| Regex::new(p))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| InferenceError::InputValidation(format!("invalid regex: {}", e)))?;

        Ok(Self {
            config,
            compiled_patterns: compiled,
        })
    }

    /// Filter the output text, returning filtered version or error if blocked.
    pub fn filter(&self, text: &str) -> Result<String, InferenceError> {
        let mut result = text.to_string();

        // Apply blocklist (case-insensitive substring match)
        let lower = result.to_lowercase();
        for blocked in &self.config.blocklist {
            if lower.contains(&blocked.to_lowercase()) {
                result = result.replace(blocked, &self.config.replacement);
            }
        }

        // Apply regex patterns
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
    pub fn contains_blocked(&self, text: &str) -> bool {
        let lower = text.to_lowercase();
        for blocked in &self.config.blocklist {
            if lower.contains(&blocked.to_lowercase()) {
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
        }
    }
}
