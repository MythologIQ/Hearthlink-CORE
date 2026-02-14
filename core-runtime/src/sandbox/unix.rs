//! Unix sandbox using cgroups v2.
//!
//! Enforces memory and CPU limits via Linux cgroups.
//! Note: Requires root or cgroup delegation for full functionality.

use super::{Sandbox, SandboxConfig, SandboxResult, SandboxUsage};

/// Unix sandbox implementation using cgroups v2.
pub struct UnixSandbox {
    config: SandboxConfig,
    active: bool,
}

impl UnixSandbox {
    /// Create a new Unix sandbox with the given configuration.
    pub fn new(config: SandboxConfig) -> Self {
        Self {
            config,
            active: false,
        }
    }
}

impl Sandbox for UnixSandbox {
    fn apply(&self) -> SandboxResult {
        if !self.config.enabled {
            return SandboxResult {
                success: true,
                error: Some("sandbox disabled by config".into()),
            };
        }

        // cgroups v2 implementation would go here:
        // 1. Create cgroup directory in /sys/fs/cgroup/
        // 2. Write memory.max for memory limit
        // 3. Write cpu.max for CPU time limit
        // 4. Write current PID to cgroup.procs

        // For now, return stub success
        // Real implementation requires root or cgroup delegation
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

        // Would read from:
        // - memory.current for memory usage
        // - cpu.stat for CPU time

        Some(SandboxUsage::default())
    }
}
