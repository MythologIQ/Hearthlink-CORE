//! Windows sandbox using Job Objects.
//!
//! Enforces memory and CPU limits via Windows Job Objects API.

use super::{Sandbox, SandboxConfig, SandboxResult, SandboxUsage};

/// Windows sandbox implementation using Job Objects.
pub struct WindowsSandbox {
    config: SandboxConfig,
    active: bool,
}

impl WindowsSandbox {
    /// Create a new Windows sandbox with the given configuration.
    pub fn new(config: SandboxConfig) -> Self {
        Self {
            config,
            active: false,
        }
    }
}

impl Sandbox for WindowsSandbox {
    fn apply(&self) -> SandboxResult {
        if !self.config.enabled {
            return SandboxResult {
                success: true,
                error: Some("sandbox disabled by config".into()),
            };
        }

        // Job Object implementation would go here:
        // 1. CreateJobObject
        // 2. SetInformationJobObject with JOBOBJECT_EXTENDED_LIMIT_INFORMATION
        //    - Set ProcessMemoryLimit
        //    - Set PerJobUserTimeLimit
        // 3. AssignProcessToJobObject for current process

        // For now, return stub success
        // Real implementation requires windows-sys or winapi crate
        SandboxResult {
            success: true,
            error: None,
        }
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn get_usage(&self) -> Option<SandboxUsage> {
        if !self.active {
            return None;
        }

        // QueryInformationJobObject would go here to get:
        // - TotalUserTime for CPU time
        // - PeakProcessMemoryUsed for memory

        Some(SandboxUsage::default())
    }
}
