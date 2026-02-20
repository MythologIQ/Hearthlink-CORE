// Copyright 2024-2026 GG-CORE Contributors
// SPDX-License-Identifier: Apache-2.0

//! Deployment debugging CLI commands.
//!
//! Provides diagnostic commands for troubleshooting canary and blue-green
//! deployments. These commands interact with the Kubernetes API and local
//! runtime to gather deployment state information.
//!
//! ## Usage
//!
//! ```bash
//! GG-CORE deployment status
//! GG-CORE canary inspect
//! GG-CORE bluegreen inspect
//! GG-CORE rollback --canary
//! GG-CORE rollback --bluegreen
//! GG-CORE rollback --force
//! ```

use std::fmt;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Exit codes for deployment commands.
pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_ERROR: i32 = 1;
pub const EXIT_DEGRADED: i32 = 2;

/// Deployment debug errors.
#[derive(Error, Debug)]
pub enum DeploymentError {
    #[error("Kubernetes API error: {0}")]
    KubernetesApi(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Timeout waiting for operation")]
    Timeout,

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Rollback failed: {0}")]
    RollbackFailed(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

/// Deployment phase for canary deployments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanaryPhase {
    Pending,
    Progressing,
    Paused,
    Promoting,
    Complete,
    Failed,
    RollingBack,
}

impl fmt::Display for CanaryPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Progressing => write!(f, "Progressing"),
            Self::Paused => write!(f, "Paused"),
            Self::Promoting => write!(f, "Promoting"),
            Self::Complete => write!(f, "Complete"),
            Self::Failed => write!(f, "Failed"),
            Self::RollingBack => write!(f, "RollingBack"),
        }
    }
}

/// Deployment phase for blue-green environments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlueGreenPhase {
    Stable,
    Switching,
    Verifying,
    RollingBack,
    Failed,
}

impl fmt::Display for BlueGreenPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stable => write!(f, "Stable"),
            Self::Switching => write!(f, "Switching"),
            Self::Verifying => write!(f, "Verifying"),
            Self::RollingBack => write!(f, "RollingBack"),
            Self::Failed => write!(f, "Failed"),
        }
    }
}

/// Active environment in blue-green deployment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActiveEnvironment {
    Blue,
    Green,
}

impl fmt::Display for ActiveEnvironment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blue => write!(f, "blue"),
            Self::Green => write!(f, "green"),
        }
    }
}

/// Canary deployment status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryStatus {
    pub phase: CanaryPhase,
    pub stable_version: String,
    pub canary_version: String,
    pub traffic_weight: u8,
    pub stable_replicas: u32,
    pub canary_replicas: u32,
    pub analysis_runs: u32,
    pub successful_analyses: u32,
    pub failed_analyses: u32,
}

/// Blue-green environment status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueGreenStatus {
    pub phase: BlueGreenPhase,
    pub active: ActiveEnvironment,
    pub blue_version: String,
    pub green_version: String,
    pub blue_ready_replicas: u32,
    pub green_ready_replicas: u32,
    pub blue_healthy: bool,
    pub green_healthy: bool,
}

/// Overall deployment status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentStatus {
    pub healthy: bool,
    pub canary: Option<CanaryStatus>,
    pub bluegreen: Option<BlueGreenStatus>,
    pub total_pods: u32,
    pub ready_pods: u32,
    pub error_rate: f64,
    pub p99_latency_ms: f64,
}

/// Rollback target specification.
#[derive(Debug, Clone, Copy)]
pub enum RollbackTarget {
    Canary,
    BlueGreen,
}

/// Rollback options.
#[derive(Debug, Clone)]
pub struct RollbackOptions {
    pub target: RollbackTarget,
    pub force: bool,
    pub timeout: Duration,
}

impl Default for RollbackOptions {
    fn default() -> Self {
        Self {
            target: RollbackTarget::Canary,
            force: false,
            timeout: Duration::from_secs(300),
        }
    }
}
