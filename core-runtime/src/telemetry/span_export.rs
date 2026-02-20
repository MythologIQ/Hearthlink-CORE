// Copyright 2024-2026 GG-CORE Contributors
// SPDX-License-Identifier: Apache-2.0

//! Span export for OpenTelemetry-compatible trace collection.
//!
//! Collects completed spans and exports them via IPC for external
//! trace aggregation (Jaeger, Zipkin, etc.).

use std::collections::{HashMap, VecDeque};
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// Maximum spans retained in the collector buffer.
const MAX_SPAN_BUFFER: usize = 1000;

/// Attribute value types for span attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SpanAttributeValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

/// Exportable span in OpenTelemetry-compatible format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportableSpan {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub name: String,
    pub start_time_unix_ns: u64,
    pub end_time_unix_ns: u64,
    pub status: SpanStatus,
    pub attributes: HashMap<String, SpanAttributeValue>,
}

/// Span completion status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpanStatus {
    Unset,
    Ok,
    Error,
}

/// Thread-safe span collector for IPC export.
pub struct SpanCollector {
    spans: RwLock<VecDeque<ExportableSpan>>,
    max_buffer: usize,
}

impl SpanCollector {
    /// Create a new span collector with default buffer size.
    pub fn new() -> Self {
        Self::with_capacity(MAX_SPAN_BUFFER)
    }

    /// Create a collector with custom buffer capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            spans: RwLock::new(VecDeque::with_capacity(capacity)),
            max_buffer: capacity,
        }
    }

    /// Record a completed span.
    pub fn record(&self, span: ExportableSpan) {
        let mut spans = self.spans.write().unwrap();
        if spans.len() >= self.max_buffer {
            spans.pop_front(); // Drop oldest span
        }
        spans.push_back(span);
    }

    /// Drain up to `max_count` spans for export.
    pub fn drain(&self, max_count: usize) -> Vec<ExportableSpan> {
        let mut spans = self.spans.write().unwrap();
        let count = max_count.min(spans.len());
        spans.drain(..count).collect()
    }

    /// Get current span count.
    pub fn len(&self) -> usize {
        self.spans.read().unwrap().len()
    }

    /// Check if buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.spans.read().unwrap().is_empty()
    }
}

impl Default for SpanCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to get current time in nanoseconds since Unix epoch.
pub fn now_unix_ns() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

/// Generate a random 16-character hex trace ID.
pub fn generate_trace_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:032x}", now)
}

/// Generate a random 8-character hex span ID.
pub fn generate_span_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:016x}", now & 0xFFFFFFFFFFFFFFFF)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_collector() {
        let collector = SpanCollector::with_capacity(3);

        for i in 0..5 {
            collector.record(ExportableSpan {
                trace_id: format!("trace_{i}"),
                span_id: format!("span_{i}"),
                parent_span_id: None,
                name: format!("op_{i}"),
                start_time_unix_ns: 0,
                end_time_unix_ns: 100,
                status: SpanStatus::Ok,
                attributes: HashMap::new(),
            });
        }

        // Buffer should only keep last 3
        assert_eq!(collector.len(), 3);

        let drained = collector.drain(2);
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].trace_id, "trace_2");
    }
}
