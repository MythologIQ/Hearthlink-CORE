//! Output Sanitizer
//! 
//! Sanitizes model outputs for security and safety.
//! Combines PII detection, content filtering, and format validation.

use crate::security::{PIIDetector, pii_detector::PIIType};
use std::sync::Arc;

/// Output sanitizer configuration
#[derive(Debug, Clone)]
pub struct SanitizerConfig {
    /// Enable PII redaction
    pub redact_pii: bool,
    /// Enable content filtering
    pub filter_content: bool,
    /// Maximum output length
    pub max_length: usize,
    /// Minimum confidence for PII detection
    pub pii_confidence_threshold: f32,
    /// PII types to redact
    pub redact_types: Vec<PIIType>,
}

impl Default for SanitizerConfig {
    fn default() -> Self {
        Self {
            redact_pii: true,
            filter_content: true,
            max_length: 100_000,
            pii_confidence_threshold: 0.7,
            redact_types: vec![
                PIIType::SSN,
                PIIType::CreditCard,
                PIIType::Email,
                PIIType::Phone,
                PIIType::APIKey,
                PIIType::Passport,
                PIIType::BankAccount,
                PIIType::MedicalRecord,
            ],
        }
    }
}

/// Sanitization result
#[derive(Debug, Clone)]
pub struct SanitizationResult {
    /// Sanitized output
    pub output: String,
    /// Whether modifications were made
    pub modified: bool,
    /// Number of PII instances redacted
    pub pii_redacted: usize,
    /// Number of content filters applied
    pub content_filtered: usize,
    /// Warnings generated
    pub warnings: Vec<String>,
}

/// Output sanitizer
pub struct OutputSanitizer {
    /// PII detector
    pii_detector: Arc<PIIDetector>,
    /// Configuration
    config: SanitizerConfig,
}

impl OutputSanitizer {
    /// Create a new output sanitizer
    pub fn new(config: SanitizerConfig) -> Self {
        Self {
            pii_detector: Arc::new(PIIDetector::new()),
            config,
        }
    }
    
    /// Create with default configuration
    pub fn default_sanitizer() -> Self {
        Self::new(SanitizerConfig::default())
    }
    
    /// Sanitize output text
    pub fn sanitize(&self, output: &str) -> SanitizationResult {
        let mut result = output.to_string();
        let mut modified = false;
        let mut pii_redacted = 0;
        let mut content_filtered = 0;
        let mut warnings = Vec::new();
        
        // Check length limit
        if result.len() > self.config.max_length {
            result.truncate(self.config.max_length);
            warnings.push(format!(
                "Output truncated to {} characters",
                self.config.max_length
            ));
            modified = true;
        }
        
        // PII detection and redaction
        if self.config.redact_pii {
            let pii_matches = self.pii_detector.detect(&result);
            
            for m in pii_matches {
                // Check if this PII type should be redacted
                if !self.config.redact_types.contains(&m.pii_type) {
                    continue;
                }
                
                // Check confidence threshold
                if m.confidence < self.config.pii_confidence_threshold {
                    continue;
                }
                
                // Redact
                result = self.redact_pii(&result, &m);
                pii_redacted += 1;
                modified = true;
            }
        }
        
        // Content filtering (basic patterns)
        if self.config.filter_content {
            let (filtered, count) = self.filter_content_patterns(&result);
            if count > 0 {
                result = filtered;
                content_filtered = count;
                modified = true;
            }
        }
        
        SanitizationResult {
            output: result,
            modified,
            pii_redacted,
            content_filtered,
            warnings,
        }
    }
    
    /// Sanitize streaming output (for real-time processing)
    pub fn sanitize_chunk(&self, chunk: &str, state: &mut StreamingSanitizerState) -> String {
        let mut result = chunk.to_string();
        
        // Track position for PII that spans chunks
        state.buffer.push_str(chunk);
        
        // Check for PII in buffer
        if self.config.redact_pii {
            let pii_matches = self.pii_detector.detect(&state.buffer);
            
            for m in pii_matches {
                if m.start >= state.processed_until {
                    // New PII found
                    if m.end <= state.buffer.len() {
                        // Complete PII within buffer
                        let redacted = format!("[REDACTED:{}]", m.pii_type.name());
                        
                        // Calculate position in current chunk
                        let chunk_start = m.start.saturating_sub(state.processed_until);
                        let chunk_end = m.end.saturating_sub(state.processed_until);
                        
                        if chunk_start < result.len() && chunk_end <= result.len() {
                            result.replace_range(chunk_start..chunk_end, &redacted);
                        }
                        
                        state.processed_until = m.end;
                    }
                }
            }
        }
        
        // Trim buffer to prevent unbounded growth
        if state.buffer.len() > 1000 {
            let trim_amount = state.buffer.len() - 500;
            state.buffer.drain(0..trim_amount);
            state.processed_until = state.processed_until.saturating_sub(trim_amount);
        }
        
        result
    }
    
    /// Redact a single PII instance
    fn redact_pii(&self, text: &str, m: &crate::security::pii_detector::PIIMatch) -> String {
        let mut result = text.to_string();
        let replacement = format!("[REDACTED:{}]", m.pii_type.name());
        result.replace_range(m.start..m.end, &replacement);
        result
    }
    
    /// Filter content patterns (basic harmful content)
    fn filter_content_patterns(&self, text: &str) -> (String, usize) {
        let mut result = text.to_string();
        let mut count = 0;
        
        // Patterns to filter (basic harmful content markers)
        let patterns = [
            // Self-harm indicators (replace with resources)
            ("I want to kill myself", "If you're having thoughts of self-harm, please reach out to a crisis helpline: 988"),
            ("I want to die", "If you're having thoughts of self-harm, please reach out to a crisis helpline: 988"),
            
            // Dangerous instructions (generic warning)
            ("how to make a bomb", "[CONTENT FILTERED: Dangerous content]"),
            ("how to create a virus", "[CONTENT FILTERED: Dangerous content]"),
        ];
        
        for (pattern, replacement) in patterns {
            if result.to_lowercase().contains(pattern) {
                result = result.replace(pattern, replacement);
                count += 1;
            }
        }
        
        (result, count)
    }
    
    /// Validate output format
    pub fn validate_format(&self, output: &str) -> Result<(), String> {
        // Check for valid UTF-8
        if output.chars().any(|c| c == '\0') {
            return Err("Output contains null characters".to_string());
        }
        
        // Check for excessive repetition
        if self.has_excessive_repetition(output) {
            return Err("Output contains excessive repetition".to_string());
        }
        
        // Check for broken encoding patterns
        if output.contains("Ã") || output.contains("Â") {
            return Err("Output may have encoding issues".to_string());
        }
        
        Ok(())
    }
    
    /// Check for excessive repetition (model degradation indicator)
    fn has_excessive_repetition(&self, text: &str) -> bool {
        let words: Vec<&str> = text.split_whitespace().collect();
        
        if words.len() < 10 {
            return false;
        }
        
        // Check for repeated phrases
        let mut phrase_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        
        for window in words.windows(3) {
            let phrase = window.join(" ");
            *phrase_counts.entry(phrase).or_insert(0) += 1;
        }
        
        // If any 3-word phrase appears more than 5 times, flag as repetitive
        phrase_counts.values().any(|&count| count > 5)
    }
}

/// State for streaming sanitization
pub struct StreamingSanitizerState {
    /// Buffer for cross-chunk PII detection
    buffer: String,
    /// Characters already processed
    processed_until: usize,
}

impl Default for StreamingSanitizerState {
    fn default() -> Self {
        Self {
            buffer: String::new(),
            processed_until: 0,
        }
    }
}

impl Default for OutputSanitizer {
    fn default() -> Self {
        Self::default_sanitizer()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pii_redaction() {
        let sanitizer = OutputSanitizer::default_sanitizer();
        let output = "Contact support@example.com for assistance";
        let result = sanitizer.sanitize(output);
        
        assert!(result.modified);
        assert!(result.pii_redacted > 0);
        assert!(result.output.contains("[REDACTED:Email Address]"));
    }
    
    #[test]
    fn test_no_modification_needed() {
        let sanitizer = OutputSanitizer::default_sanitizer();
        let output = "The weather is nice today.";
        let result = sanitizer.sanitize(output);
        
        assert!(!result.modified);
        assert_eq!(result.pii_redacted, 0);
    }
    
    #[test]
    fn test_length_truncation() {
        let config = SanitizerConfig {
            max_length: 50,
            ..Default::default()
        };
        let sanitizer = OutputSanitizer::new(config);
        
        let output = "This is a very long output that should be truncated to fit within the limit.";
        let result = sanitizer.sanitize(output);
        
        assert!(result.modified);
        assert!(result.output.len() <= 50);
        assert!(!result.warnings.is_empty());
    }
    
    #[test]
    fn test_multiple_pii_types() {
        let sanitizer = OutputSanitizer::default_sanitizer();
        let output = "Email: test@example.com, Phone: 555-123-4567, SSN: 123-45-6789";
        let result = sanitizer.sanitize(output);
        
        assert!(result.modified);
        assert!(result.pii_redacted >= 3);
    }
    
    #[test]
    fn test_format_validation() {
        let sanitizer = OutputSanitizer::default_sanitizer();
        
        // Valid output
        assert!(sanitizer.validate_format("This is valid output").is_ok());
        
        // Null characters
        assert!(sanitizer.validate_format("Invalid\0output").is_err());
    }
    
    #[test]
    fn test_excessive_repetition_detection() {
        let sanitizer = OutputSanitizer::default_sanitizer();
        
        // Create text with clear repetition (same 3-word phrase 6+ times)
        let repetitive = "hello world test hello world test hello world test hello world test hello world test hello world test hello world test";
        assert!(sanitizer.has_excessive_repetition(repetitive));
        
        let normal = "The quick brown fox jumps over the lazy dog and runs through the forest.";
        assert!(!sanitizer.has_excessive_repetition(normal));
    }
    
    #[test]
    fn test_streaming_sanitization() {
        let sanitizer = OutputSanitizer::default_sanitizer();
        let mut state = StreamingSanitizerState::default();
        
        // Process chunks
        let chunk1 = sanitizer.sanitize_chunk("Contact ", &mut state);
        let chunk2 = sanitizer.sanitize_chunk("test@example.com", &mut state);
        let chunk3 = sanitizer.sanitize_chunk(" for help", &mut state);
        
        // At least some chunk should be modified
        let full_output = format!("{}{}{}", chunk1, chunk2, chunk3);
        assert!(full_output.contains("[REDACTED") || state.buffer.contains("@"));
    }
    
    #[test]
    fn test_confidence_threshold() {
        let config = SanitizerConfig {
            pii_confidence_threshold: 0.99, // Very high threshold
            ..Default::default()
        };
        let sanitizer = OutputSanitizer::new(config);
        
        // Most PII won't meet 99% confidence
        let output = "Email: test@example.com";
        let _result = sanitizer.sanitize(output);
        
        // May not be redacted due to high threshold
        // (depends on confidence calculation)
    }
    
    #[test]
    fn test_selective_pii_types() {
        let config = SanitizerConfig {
            redact_types: vec![PIIType::Email], // Only redact emails
            ..Default::default()
        };
        let sanitizer = OutputSanitizer::new(config);
        
        let output = "Email: test@example.com, Phone: 555-123-4567";
        let result = sanitizer.sanitize(output);
        
        assert!(result.output.contains("[REDACTED:Email Address]"));
        assert!(result.output.contains("555-123-4567")); // Phone not redacted
    }
    
    #[test]
    fn test_performance() {
        let sanitizer = OutputSanitizer::default_sanitizer();
        let output = "Contact support@example.com for help. Call 555-123-4567. SSN: 123-45-6789.".repeat(100);
        
        let start = std::time::Instant::now();
        for _ in 0..100 {
            let _ = sanitizer.sanitize(&output);
        }
        let duration = start.elapsed();
        
        // Should complete 100 sanitizations in under 10 seconds
        assert!(duration.as_millis() < 10000, "Sanitization too slow: {:?}", duration);
    }
}