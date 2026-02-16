// Copyright 2024-2026 Veritas SDR Contributors
// Licensed under the Apache License, Version 2.0

//! GPU Backend Support
//!
//! Provides GPU acceleration for inference using CUDA (NVIDIA) or Metal (Apple Silicon).
//! This module implements the GPU abstraction layer for Veritas SDR.

use std::fmt;
use std::sync::Arc;
use thiserror::Error;

/// GPU Backend Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuBackend {
    /// NVIDIA CUDA backend
    Cuda,
    /// Apple Metal backend (macOS only)
    Metal,
    /// CPU fallback (no GPU)
    Cpu,
}

impl Default for GpuBackend {
    fn default() -> Self {
        Self::Cpu
    }
}

impl fmt::Display for GpuBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpuBackend::Cuda => write!(f, "CUDA"),
            GpuBackend::Metal => write!(f, "Metal"),
            GpuBackend::Cpu => write!(f, "CPU"),
        }
    }
}

/// GPU Device Information
#[derive(Debug, Clone)]
pub struct GpuDevice {
    /// Backend type
    pub backend: GpuBackend,
    /// Device index (for multi-GPU systems)
    pub index: usize,
    /// Device name
    pub name: String,
    /// Total memory in bytes
    pub total_memory: u64,
    /// Available memory in bytes
    pub available_memory: u64,
    /// Compute capability (CUDA only)
    pub compute_capability: Option<(u32, u32)>,
}

impl GpuDevice {
    /// Create a CPU device
    pub fn cpu() -> Self {
        Self {
            backend: GpuBackend::Cpu,
            index: 0,
            name: "CPU".to_string(),
            total_memory: 0,
            available_memory: 0,
            compute_capability: None,
        }
    }

    /// Check if device has enough memory
    pub fn has_memory(&self, required: u64) -> bool {
        self.backend == GpuBackend::Cpu || self.available_memory >= required
    }

    /// Get memory utilization percentage
    pub fn memory_utilization(&self) -> f32 {
        if self.total_memory == 0 {
            return 0.0;
        }
        ((self.total_memory - self.available_memory) as f64 / self.total_memory as f64) as f32
    }
}

/// GPU Configuration
#[derive(Debug, Clone)]
pub struct GpuConfig {
    /// Preferred backend
    pub backend: GpuBackend,
    /// Device index to use
    pub device_index: usize,
    /// Memory fraction to use (0.0 - 1.0)
    pub memory_fraction: f32,
    /// Enable flash attention
    pub flash_attention: bool,
    /// Number of GPU layers to offload
    pub gpu_layers: u32,
    /// Split model across multiple GPUs
    pub multi_gpu: bool,
    /// Main GPU for multi-GPU setups
    pub main_gpu: usize,
}

impl Default for GpuConfig {
    fn default() -> Self {
        Self {
            backend: GpuBackend::Cpu,
            device_index: 0,
            memory_fraction: 0.9,
            flash_attention: true,
            gpu_layers: 0,
            multi_gpu: false,
            main_gpu: 0,
        }
    }
}

impl GpuConfig {
    /// Create a CPU-only configuration
    pub fn cpu() -> Self {
        Self {
            backend: GpuBackend::Cpu,
            gpu_layers: 0,
            ..Default::default()
        }
    }

    /// Create a CUDA configuration with all layers on GPU
    pub fn cuda_all_layers() -> Self {
        Self {
            backend: GpuBackend::Cuda,
            gpu_layers: u32::MAX,
            ..Default::default()
        }
    }

    /// Create a Metal configuration (macOS)
    #[cfg(target_os = "macos")]
    pub fn metal() -> Self {
        Self {
            backend: GpuBackend::Metal,
            gpu_layers: u32::MAX,
            ..Default::default()
        }
    }
}

/// GPU Error Types
#[derive(Debug, Error)]
pub enum GpuError {
    #[error("No GPU devices available")]
    NoDevicesAvailable,
    
    #[error("CUDA not available: {0}")]
    CudaNotAvailable(String),
    
    #[error("Metal not available: {0}")]
    MetalNotAvailable(String),
    
    #[error("Device not found: {0}")]
    DeviceNotFound(usize),
    
    #[error("Out of GPU memory: required {required} bytes, available {available} bytes")]
    OutOfMemory {
        required: u64,
        available: u64,
    },
    
    #[error("GPU operation failed: {0}")]
    OperationFailed(String),
    
    #[error("Memory allocation failed: {0}")]
    AllocationFailed(String),
    
    #[error("Kernel launch failed: {0}")]
    KernelLaunchFailed(String),
}

/// GPU Manager - Handles device detection and memory management
pub struct GpuManager {
    /// Available devices
    devices: Vec<GpuDevice>,
    /// Current configuration
    config: GpuConfig,
    /// Active device
    active_device: Option<Arc<GpuDevice>>,
}

impl GpuManager {
    /// Create a new GPU manager
    pub fn new(config: GpuConfig) -> Result<Self, GpuError> {
        let mut manager = Self {
            devices: Vec::new(),
            config,
            active_device: None,
        };
        
        manager.detect_devices()?;
        manager.select_device()?;
        
        Ok(manager)
    }

    /// Detect available GPU devices
    pub fn detect_devices(&mut self) -> Result<(), GpuError> {
        self.devices.clear();
        
        // Always add CPU as fallback
        self.devices.push(GpuDevice::cpu());
        
        // Detect CUDA devices
        #[cfg(feature = "cuda")]
        {
            if let Ok(cuda_devices) = self.detect_cuda_devices() {
                self.devices.extend(cuda_devices);
            }
        }
        
        // Detect Metal devices (macOS only)
        #[cfg(all(feature = "metal", target_os = "macos"))]
        {
            if let Ok(metal_devices) = self.detect_metal_devices() {
                self.devices.extend(metal_devices);
            }
        }
        
        if self.devices.len() == 1 && self.config.backend != GpuBackend::Cpu {
            return Err(GpuError::NoDevicesAvailable);
        }
        
        Ok(())
    }

    /// Select the active device based on configuration
    pub fn select_device(&mut self) -> Result<(), GpuError> {
        let device = self.devices
            .iter()
            .find(|d| d.backend == self.config.backend && d.index == self.config.device_index)
            .cloned();
        
        match device {
            Some(d) => {
                self.active_device = Some(Arc::new(d));
                Ok(())
            }
            None => {
                // Fall back to CPU if requested backend not available
                if self.config.backend != GpuBackend::Cpu {
                    self.active_device = Some(Arc::new(GpuDevice::cpu()));
                    Ok(())
                } else {
                    Err(GpuError::DeviceNotFound(self.config.device_index))
                }
            }
        }
    }

    /// Get the active device
    pub fn active_device(&self) -> Option<&GpuDevice> {
        self.active_device.as_deref()
    }

    /// Get all available devices
    pub fn available_devices(&self) -> &[GpuDevice] {
        &self.devices
    }

    /// Check if GPU is available
    pub fn is_gpu_available(&self) -> bool {
        self.devices.iter().any(|d| d.backend != GpuBackend::Cpu)
    }

    /// Get available GPU backends
    pub fn available_backends(&self) -> Vec<GpuBackend> {
        self.devices
            .iter()
            .map(|d| d.backend)
            .filter(|b| *b != GpuBackend::Cpu)
            .collect()
    }

    /// Allocate GPU memory
    pub fn allocate_memory(&self, size: u64) -> Result<GpuMemory, GpuError> {
        let device = self.active_device.as_ref().ok_or(GpuError::NoDevicesAvailable)?;
        
        if !device.has_memory(size) {
            return Err(GpuError::OutOfMemory {
                required: size,
                available: device.available_memory,
            });
        }
        
        // Actual allocation would happen here with CUDA/Metal bindings
        Ok(GpuMemory {
            size,
            device: device.clone(),
            ptr: std::ptr::null_mut(),
        })
    }

    /// Detect CUDA devices (stub - requires cuda crate)
    #[cfg(feature = "cuda")]
    fn detect_cuda_devices(&self) -> Result<Vec<GpuDevice>, GpuError> {
        // This would use the cuda crate to detect devices
        // For now, return empty vec
        Ok(Vec::new())
    }

    /// Detect Metal devices (stub - requires metal crate)
    #[cfg(all(feature = "metal", target_os = "macos"))]
    fn detect_metal_devices(&self) -> Result<Vec<GpuDevice>, GpuError> {
        // This would use the metal crate to detect devices
        // For now, return empty vec
        Ok(Vec::new())
    }
}

/// GPU Memory Handle
pub struct GpuMemory {
    /// Size in bytes
    pub size: u64,
    /// Device the memory is allocated on
    pub device: Arc<GpuDevice>,
    /// Pointer to GPU memory (opaque)
    pub ptr: *mut std::ffi::c_void,
}

impl Drop for GpuMemory {
    fn drop(&mut self) {
        // Actual deallocation would happen here
        // Safety: ptr is valid and points to GPU memory
    }
}

// Safety: GpuMemory can be sent between threads
unsafe impl Send for GpuMemory {}
unsafe impl Sync for GpuMemory {}

/// GPU Memory Pool for efficient allocation
pub struct GpuMemoryPool {
    /// Device for this pool
    device: Arc<GpuDevice>,
    /// Allocated blocks
    blocks: Vec<GpuMemory>,
    /// Total allocated size
    total_allocated: u64,
    /// Maximum pool size
    max_size: u64,
}

impl GpuMemoryPool {
    /// Create a new memory pool
    pub fn new(device: Arc<GpuDevice>, max_size: u64) -> Self {
        Self {
            device,
            blocks: Vec::new(),
            total_allocated: 0,
            max_size,
        }
    }

    /// Allocate from pool
    pub fn allocate(&mut self, size: u64) -> Result<&GpuMemory, GpuError> {
        if self.total_allocated + size > self.max_size {
            return Err(GpuError::OutOfMemory {
                required: size,
                available: self.max_size - self.total_allocated,
            });
        }
        
        let memory = GpuMemory {
            size,
            device: self.device.clone(),
            ptr: std::ptr::null_mut(),
        };
        
        self.blocks.push(memory);
        self.total_allocated += size;
        
        Ok(self.blocks.last().unwrap())
    }

    /// Get pool utilization
    pub fn utilization(&self) -> f32 {
        if self.max_size == 0 {
            return 0.0;
        }
        self.total_allocated as f32 / self.max_size as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_backend_display() {
        assert_eq!(format!("{}", GpuBackend::Cuda), "CUDA");
        assert_eq!(format!("{}", GpuBackend::Metal), "Metal");
        assert_eq!(format!("{}", GpuBackend::Cpu), "CPU");
    }

    #[test]
    fn test_gpu_device_cpu() {
        let device = GpuDevice::cpu();
        assert_eq!(device.backend, GpuBackend::Cpu);
        assert!(device.has_memory(0));
        assert_eq!(device.memory_utilization(), 0.0);
    }

    #[test]
    fn test_gpu_config_default() {
        let config = GpuConfig::default();
        assert_eq!(config.backend, GpuBackend::Cpu);
        assert_eq!(config.gpu_layers, 0);
    }

    #[test]
    fn test_gpu_config_cpu() {
        let config = GpuConfig::cpu();
        assert_eq!(config.backend, GpuBackend::Cpu);
        assert_eq!(config.gpu_layers, 0);
    }

    #[test]
    fn test_gpu_config_cuda_all_layers() {
        let config = GpuConfig::cuda_all_layers();
        assert_eq!(config.backend, GpuBackend::Cuda);
        assert_eq!(config.gpu_layers, u32::MAX);
    }

    #[test]
    fn test_gpu_manager_cpu_only() {
        let config = GpuConfig::cpu();
        let manager = GpuManager::new(config).unwrap();
        
        assert!(manager.active_device().is_some());
        assert_eq!(manager.active_device().unwrap().backend, GpuBackend::Cpu);
    }

    #[test]
    fn test_gpu_memory_pool() {
        let device = Arc::new(GpuDevice::cpu());
        let mut pool = GpuMemoryPool::new(device, 1024);
        
        let mem = pool.allocate(512).unwrap();
        assert_eq!(mem.size, 512);
        assert_eq!(pool.utilization(), 0.5);
    }

    #[test]
    fn test_gpu_memory_pool_out_of_memory() {
        let device = Arc::new(GpuDevice::cpu());
        let mut pool = GpuMemoryPool::new(device, 1024);
        
        pool.allocate(512).unwrap();
        let result = pool.allocate(1024);
        
        assert!(matches!(result, Err(GpuError::OutOfMemory { .. })));
    }
}
