//! Enterprise Security Audit Module
//!
//! Provides comprehensive security audit logging and compliance features:
//! - Structured audit events with severity levels
//! - Configurable retention policies
//! - Integration with SIEM systems
//! - Compliance reporting (SOC2, HIPAA, GDPR)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Audit event severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum AuditSeverity {
    /// Informational events (normal operations)
    Info = 0,
    /// Warning events (potential issues)
    Warning = 1,
    /// Error events (failures)
    Error = 2,
    /// Critical security events (breaches, attacks)
    Critical = 3,
}

impl std::fmt::Display for AuditSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditSeverity::Info => write!(f, "INFO"),
            AuditSeverity::Warning => write!(f, "WARNING"),
            AuditSeverity::Error => write!(f, "ERROR"),
            AuditSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Audit event categories for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditCategory {
    /// Authentication events (login, logout, token refresh)
    Authentication,
    /// Authorization events (access granted/denied)
    Authorization,
    /// Data access events (read, write, delete)
    DataAccess,
    /// Configuration changes
    Configuration,
    /// Encryption operations
    Encryption,
    /// Network/IPC events
    Network,
    /// Model operations
    ModelOperation,
    /// System events (startup, shutdown)
    System,
}

impl std::fmt::Display for AuditCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditCategory::Authentication => write!(f, "AUTHENTICATION"),
            AuditCategory::Authorization => write!(f, "AUTHORIZATION"),
            AuditCategory::DataAccess => write!(f, "DATA_ACCESS"),
            AuditCategory::Configuration => write!(f, "CONFIGURATION"),
            AuditCategory::Encryption => write!(f, "ENCRYPTION"),
            AuditCategory::Network => write!(f, "NETWORK"),
            AuditCategory::ModelOperation => write!(f, "MODEL_OPERATION"),
            AuditCategory::System => write!(f, "SYSTEM"),
        }
    }
}

/// Audit event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event identifier
    pub id: String,
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// Event severity
    pub severity: AuditSeverity,
    /// Event category
    pub category: AuditCategory,
    /// Event type (specific action)
    pub event_type: String,
    /// Human-readable message
    pub message: String,
    /// Source component
    pub source: String,
    /// User or session ID (if applicable)
    pub actor: Option<String>,
    /// Resource being accessed (if applicable)
    pub resource: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Request/correlation ID for tracing
    pub correlation_id: Option<String>,
    /// Whether the event was successful
    pub success: bool,
}

impl AuditEvent {
    /// Create a new audit event builder
    pub fn builder() -> AuditEventBuilder {
        AuditEventBuilder::default()
    }

    /// Convert to JSON for logging/transmission
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Convert to log-friendly string
    pub fn to_log_string(&self) -> String {
        format!(
            "[{}] {} [{}] {} - {} (actor={:?}, resource={:?}, success={})",
            self.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            self.severity,
            self.category,
            self.event_type,
            self.message,
            self.actor,
            self.resource,
            self.success
        )
    }
}

/// Builder for audit events
#[derive(Debug, Default)]
pub struct AuditEventBuilder {
    severity: Option<AuditSeverity>,
    category: Option<AuditCategory>,
    event_type: Option<String>,
    message: Option<String>,
    source: Option<String>,
    actor: Option<String>,
    resource: Option<String>,
    metadata: HashMap<String, String>,
    correlation_id: Option<String>,
    success: bool,
}

impl AuditEventBuilder {
    pub fn severity(mut self, severity: AuditSeverity) -> Self {
        self.severity = Some(severity);
        self
    }

    pub fn category(mut self, category: AuditCategory) -> Self {
        self.category = Some(category);
        self
    }

    pub fn event_type(mut self, event_type: impl Into<String>) -> Self {
        self.event_type = Some(event_type.into());
        self
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn actor(mut self, actor: impl Into<String>) -> Self {
        self.actor = Some(actor.into());
        self
    }

    pub fn resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    pub fn success(mut self, success: bool) -> Self {
        self.success = success;
        self
    }

    /// Build the audit event
    pub fn build(self) -> Result<AuditEvent, &'static str> {
        Ok(AuditEvent {
            id: generate_event_id(),
            timestamp: Utc::now(),
            severity: self.severity.ok_or("severity is required")?,
            category: self.category.ok_or("category is required")?,
            event_type: self.event_type.ok_or("event_type is required")?,
            message: self.message.ok_or("message is required")?,
            source: self.source.ok_or("source is required")?,
            actor: self.actor,
            resource: self.resource,
            metadata: self.metadata,
            correlation_id: self.correlation_id,
            success: self.success,
        })
    }
}

/// Generate a unique event ID
fn generate_event_id() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut bytes[..]);
    hex::encode(bytes)
}

/// Audit log configuration
#[derive(Debug, Clone)]
pub struct AuditConfig {
    /// Minimum severity to log
    pub min_severity: AuditSeverity,
    /// Maximum events to keep in memory
    pub max_events: usize,
    /// Whether to log to stdout
    pub log_to_stdout: bool,
    /// Whether to include sensitive data (for debugging only)
    pub include_sensitive: bool,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            min_severity: AuditSeverity::Info,
            max_events: 10000,
            log_to_stdout: true,
            include_sensitive: false,
        }
    }
}

/// Audit logger for enterprise security compliance
pub struct AuditLogger {
    config: AuditConfig,
    events: Arc<RwLock<Vec<AuditEvent>>>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(config: AuditConfig) -> Self {
        Self {
            config,
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Log an audit event
    pub async fn log(&self, event: AuditEvent) {
        // Check severity threshold
        if event.severity < self.config.min_severity {
            return;
        }

        // Log to stdout if configured
        if self.config.log_to_stdout {
            println!("{}", event.to_log_string());
        }

        // Store event
        let mut events = self.events.write().await;
        events.push(event);

        // Enforce max events limit
        if events.len() > self.config.max_events {
            let excess = events.len() - self.config.max_events;
            events.drain(0..excess);
        }
    }

    /// Log a quick event (convenience method)
    pub async fn log_event(
        &self,
        severity: AuditSeverity,
        category: AuditCategory,
        event_type: &str,
        message: &str,
        source: &str,
    ) {
        if let Ok(event) = AuditEvent::builder()
            .severity(severity)
            .category(category)
            .event_type(event_type)
            .message(message)
            .source(source)
            .build()
        {
            self.log(event).await;
        }
    }

    /// Get all events (for compliance reporting)
    pub async fn get_events(&self) -> Vec<AuditEvent> {
        self.events.read().await.clone()
    }

    /// Get events by category
    pub async fn get_events_by_category(&self, category: AuditCategory) -> Vec<AuditEvent> {
        self.events
            .read()
            .await
            .iter()
            .filter(|e| e.category == category)
            .cloned()
            .collect()
    }

    /// Get events by severity
    pub async fn get_events_by_severity(&self, severity: AuditSeverity) -> Vec<AuditEvent> {
        self.events
            .read()
            .await
            .iter()
            .filter(|e| e.severity >= severity)
            .cloned()
            .collect()
    }

    /// Get events within a time range
    pub async fn get_events_by_time(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<AuditEvent> {
        self.events
            .read()
            .await
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .cloned()
            .collect()
    }

    /// Clear all events (use with caution)
    pub async fn clear(&self) {
        self.events.write().await.clear();
    }

    /// Export events as JSON (for SIEM integration)
    pub async fn export_json(&self) -> Result<String, serde_json::Error> {
        let events = self.events.read().await;
        serde_json::to_string_pretty(&*events)
    }

    /// Get event count
    pub async fn event_count(&self) -> usize {
        self.events.read().await.len()
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new(AuditConfig::default())
    }
}

/// Global audit logger instance
static AUDIT_LOGGER: std::sync::OnceLock<Arc<AuditLogger>> = std::sync::OnceLock::new();

/// Initialize the global audit logger
pub fn init_audit_logger(config: AuditConfig) {
    let _ = AUDIT_LOGGER.get_or_init(|| Arc::new(AuditLogger::new(config)));
}

/// Get the global audit logger
pub fn audit_logger() -> Option<Arc<AuditLogger>> {
    AUDIT_LOGGER.get().cloned()
}

/// Convenience macro for audit logging
#[macro_export]
macro_rules! audit_log {
    ($severity:expr, $category:expr, $event_type:expr, $message:expr, $source:expr) => {
        if let Some(logger) = $crate::security::audit::audit_logger() {
            tokio::spawn(async move {
                logger
                    .log_event($severity, $category, $event_type, $message, $source)
                    .await;
            });
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_severity_ordering() {
        assert!(AuditSeverity::Critical > AuditSeverity::Error);
        assert!(AuditSeverity::Error > AuditSeverity::Warning);
        assert!(AuditSeverity::Warning > AuditSeverity::Info);
    }

    #[test]
    fn test_audit_event_builder() {
        let event = AuditEvent::builder()
            .severity(AuditSeverity::Info)
            .category(AuditCategory::Authentication)
            .event_type("login")
            .message("User logged in")
            .source("auth_module")
            .actor("user123")
            .success(true)
            .build()
            .unwrap();

        assert_eq!(event.severity, AuditSeverity::Info);
        assert_eq!(event.category, AuditCategory::Authentication);
        assert_eq!(event.event_type, "login");
        assert!(event.success);
    }

    #[test]
    fn test_audit_event_builder_missing_fields() {
        let result = AuditEvent::builder().severity(AuditSeverity::Info).build();
        assert!(result.is_err());
    }

    #[test]
    fn test_audit_event_to_json() {
        let event = AuditEvent::builder()
            .severity(AuditSeverity::Warning)
            .category(AuditCategory::DataAccess)
            .event_type("read")
            .message("Data accessed")
            .source("api")
            .build()
            .unwrap();

        let json = event.to_json().unwrap();
        // Serde serializes enum variants as their names, not Display output
        assert!(json.contains("Warning"));
        assert!(json.contains("DataAccess"));
    }

    #[test]
    fn test_audit_event_to_log_string() {
        let event = AuditEvent::builder()
            .severity(AuditSeverity::Error)
            .category(AuditCategory::Encryption)
            .event_type("decrypt_failed")
            .message("Decryption failed")
            .source("encryption")
            .actor("session123")
            .resource("model.gguf")
            .success(false)
            .build()
            .unwrap();

        let log = event.to_log_string();
        assert!(log.contains("ERROR"));
        assert!(log.contains("ENCRYPTION"));
        assert!(log.contains("Decryption failed"));
        assert!(log.contains("session123"));
        assert!(log.contains("model.gguf"));
        assert!(log.contains("success=false"));
    }

    #[tokio::test]
    async fn test_audit_logger() {
        let logger = AuditLogger::new(AuditConfig::default());

        let event = AuditEvent::builder()
            .severity(AuditSeverity::Info)
            .category(AuditCategory::System)
            .event_type("startup")
            .message("System started")
            .source("main")
            .build()
            .unwrap();

        logger.log(event).await;
        assert_eq!(logger.event_count().await, 1);
    }

    #[tokio::test]
    async fn test_audit_logger_severity_filter() {
        let config = AuditConfig {
            min_severity: AuditSeverity::Warning,
            ..Default::default()
        };
        let logger = AuditLogger::new(config);

        // Info should be filtered out
        let info_event = AuditEvent::builder()
            .severity(AuditSeverity::Info)
            .category(AuditCategory::System)
            .event_type("test")
            .message("Info event")
            .source("test")
            .build()
            .unwrap();

        logger.log(info_event).await;
        assert_eq!(logger.event_count().await, 0);

        // Warning should be logged
        let warning_event = AuditEvent::builder()
            .severity(AuditSeverity::Warning)
            .category(AuditCategory::System)
            .event_type("test")
            .message("Warning event")
            .source("test")
            .build()
            .unwrap();

        logger.log(warning_event).await;
        assert_eq!(logger.event_count().await, 1);
    }

    #[tokio::test]
    async fn test_audit_logger_max_events() {
        let config = AuditConfig {
            max_events: 5,
            ..Default::default()
        };
        let logger = AuditLogger::new(config);

        // Add 10 events
        for i in 0..10 {
            let event = AuditEvent::builder()
                .severity(AuditSeverity::Info)
                .category(AuditCategory::System)
                .event_type("test")
                .message(format!("Event {}", i))
                .source("test")
                .build()
                .unwrap();
            logger.log(event).await;
        }

        // Should only keep 5
        assert_eq!(logger.event_count().await, 5);
    }

    #[tokio::test]
    async fn test_get_events_by_category() {
        let logger = AuditLogger::new(AuditConfig::default());

        for i in 0..5 {
            let event = AuditEvent::builder()
                .severity(AuditSeverity::Info)
                .category(if i % 2 == 0 {
                    AuditCategory::Authentication
                } else {
                    AuditCategory::DataAccess
                })
                .event_type("test")
                .message(format!("Event {}", i))
                .source("test")
                .build()
                .unwrap();
            logger.log(event).await;
        }

        let auth_events = logger
            .get_events_by_category(AuditCategory::Authentication)
            .await;
        assert_eq!(auth_events.len(), 3); // 0, 2, 4

        let data_events = logger
            .get_events_by_category(AuditCategory::DataAccess)
            .await;
        assert_eq!(data_events.len(), 2); // 1, 3
    }

    #[tokio::test]
    async fn test_export_json() {
        let logger = AuditLogger::new(AuditConfig::default());

        let event = AuditEvent::builder()
            .severity(AuditSeverity::Info)
            .category(AuditCategory::System)
            .event_type("test")
            .message("Test event")
            .source("test")
            .build()
            .unwrap();

        logger.log(event).await;

        let json = logger.export_json().await.unwrap();
        assert!(json.starts_with("["));
        assert!(json.contains("Test event"));
    }

    #[test]
    fn test_generate_event_id() {
        let id1 = generate_event_id();
        let id2 = generate_event_id();

        // IDs should be 32 characters (16 bytes hex-encoded)
        assert_eq!(id1.len(), 32);
        assert_eq!(id2.len(), 32);

        // IDs should be unique
        assert_ne!(id1, id2);

        // IDs should be valid hex
        assert!(id1.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
