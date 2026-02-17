// Copyright 2024-2026 Veritas SDR Contributors
// Licensed under the Apache License, Version 2.0

//! Multi-GPU Support
//!
//! Provides model parallelism and pipeline parallelism across multiple GPUs.
//! Supports both NVIDIA CUDA and Apple Metal multi-GPU configurations.

use std::sync::Arc;
use thiserror::Error;

use super::{GpuBackend, GpuDevice};

/// Multi-GPU Error Types
#[derive(Debug, Error)]
pub enum MultiGpuError {
    #[error("No multi-GPU configuration available")]
    NoMultiGpuConfig,

    #[error("Insufficient GPUs: requested {required}, available {available}")]
    InsufficientGpus { required: usize, available: usize },

    #[error("GPU {index} not available: {reason}")]
    GpuNotAvailable { index: usize, reason: String },

    #[error("Model partitioning failed: {0}")]
    PartitioningFailed(String),

    #[error("Cross-GPU communication failed: {0}")]
    CommunicationFailed(String),

    #[error("Load balancing failed: {0}")]
    LoadBalancingFailed(String),

    #[error("Synchronization failed: {0}")]
    SynchronizationFailed(String),
}

/// Strategy for distributing model across GPUs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultiGpuStrategy {
    /// Split layers evenly across all GPUs
    LayerParallelism,
    /// Split tensors across GPUs (tensor parallelism)
    TensorParallelism,
    /// Pipeline stages across GPUs
    PipelineParallelism,
    /// Expert parallelism for MoE models
    ExpertParallelism,
    /// Automatic selection based on model and hardware
    Auto,
}

impl Default for MultiGpuStrategy {
    fn default() -> Self {
        Self::Auto
    }
}

/// GPU partition configuration
#[derive(Debug, Clone)]
pub struct GpuPartition {
    /// GPU index
    pub gpu_index: usize,
    /// Layers assigned to this GPU (for layer parallelism)
    pub layers: std::ops::Range<usize>,
    /// Memory budget for this partition in bytes
    pub memory_budget: u64,
    /// Percentage of model parameters on this GPU
    pub parameter_fraction: f32,
}

/// Multi-GPU Configuration
#[derive(Debug, Clone)]
pub struct MultiGpuConfig {
    /// Strategy for multi-GPU distribution
    pub strategy: MultiGpuStrategy,
    /// Number of GPUs to use (0 = all available)
    pub num_gpus: usize,
    /// Main GPU for coordination
    pub main_gpu: usize,
    /// Enable peer-to-peer communication
    pub enable_p2p: bool,
    /// Maximum memory per GPU (0 = auto)
    pub max_memory_per_gpu: u64,
    /// Load balancing threshold (0.0 - 1.0)
    pub load_balance_threshold: f32,
    /// Enable gradient checkpointing for memory efficiency
    pub gradient_checkpointing: bool,
}

impl Default for MultiGpuConfig {
    fn default() -> Self {
        Self {
            strategy: MultiGpuStrategy::Auto,
            num_gpus: 0,
            main_gpu: 0,
            enable_p2p: true,
            max_memory_per_gpu: 0,
            load_balance_threshold: 0.1,
            gradient_checkpointing: false,
        }
    }
}

/// Multi-GPU Manager - handles coordination across multiple GPUs
pub struct MultiGpuManager {
    /// Available GPU devices
    devices: Vec<Arc<GpuDevice>>,
    /// Configuration
    config: MultiGpuConfig,
    /// Partition assignments
    partitions: Vec<GpuPartition>,
    /// Active strategy
    active_strategy: MultiGpuStrategy,
}

impl MultiGpuManager {
    /// Create a new multi-GPU manager
    pub fn new(
        devices: Vec<Arc<GpuDevice>>,
        config: MultiGpuConfig,
    ) -> Result<Self, MultiGpuError> {
        if devices.len() < 2 {
            return Err(MultiGpuError::InsufficientGpus {
                required: 2,
                available: devices.len(),
            });
        }

        // Filter to only GPU devices (not CPU)
        let gpu_devices: Vec<Arc<GpuDevice>> = devices
            .into_iter()
            .filter(|d| d.backend != GpuBackend::Cpu)
            .collect();

        if gpu_devices.len() < 2 {
            return Err(MultiGpuError::InsufficientGpus {
                required: 2,
                available: gpu_devices.len(),
            });
        }

        let num_gpus = if config.num_gpus == 0 {
            gpu_devices.len()
        } else {
            config.num_gpus.min(gpu_devices.len())
        };

        let devices_to_use: Vec<Arc<GpuDevice>> = gpu_devices.into_iter().take(num_gpus).collect();

        let manager = Self {
            devices: devices_to_use,
            config,
            partitions: Vec::new(),
            active_strategy: MultiGpuStrategy::Auto,
        };

        Ok(manager)
    }

    /// Get available GPUs
    pub fn devices(&self) -> &[Arc<GpuDevice>] {
        &self.devices
    }

    /// Get number of GPUs in use
    pub fn num_gpus(&self) -> usize {
        self.devices.len()
    }

    /// Partition a model across GPUs
    pub fn partition_model(
        &mut self,
        num_layers: usize,
        model_size_bytes: u64,
    ) -> Result<&[GpuPartition], MultiGpuError> {
        // Determine strategy
        self.active_strategy = self.determine_strategy(num_layers, model_size_bytes);

        // Create partitions based on strategy
        self.partitions = match self.active_strategy {
            MultiGpuStrategy::LayerParallelism => {
                self.partition_by_layers(num_layers, model_size_bytes)?
            }
            MultiGpuStrategy::TensorParallelism => self.partition_by_tensors(model_size_bytes)?,
            MultiGpuStrategy::PipelineParallelism => {
                self.partition_by_pipeline(num_layers, model_size_bytes)?
            }
            MultiGpuStrategy::ExpertParallelism => self.partition_by_experts(model_size_bytes)?,
            MultiGpuStrategy::Auto => {
                // Auto already determined, use layer parallelism as default
                self.partition_by_layers(num_layers, model_size_bytes)?
            }
        };

        Ok(&self.partitions)
    }

    /// Determine optimal strategy based on model and hardware
    fn determine_strategy(&self, num_layers: usize, _model_size: u64) -> MultiGpuStrategy {
        if self.config.strategy != MultiGpuStrategy::Auto {
            return self.config.strategy;
        }

        // Heuristics for strategy selection
        let num_gpus = self.devices.len();

        // Check if all GPUs have similar memory (homogeneous)
        let memory_variance = self.compute_memory_variance();
        let is_homogeneous = memory_variance < 0.2;

        // Check for P2P support
        let has_p2p = self.config.enable_p2p && self.check_p2p_support();

        // Large models with many layers benefit from pipeline parallelism
        if num_layers > 48 && num_gpus >= 4 && has_p2p {
            return MultiGpuStrategy::PipelineParallelism;
        }

        // Homogeneous systems benefit from tensor parallelism
        if is_homogeneous && has_p2p && num_gpus <= 8 {
            return MultiGpuStrategy::TensorParallelism;
        }

        // Default to layer parallelism
        MultiGpuStrategy::LayerParallelism
    }

    /// Compute variance in GPU memory sizes
    fn compute_memory_variance(&self) -> f32 {
        if self.devices.is_empty() {
            return 0.0;
        }

        let memories: Vec<u64> = self.devices.iter().map(|d| d.total_memory).collect();
        let mean = memories.iter().sum::<u64>() as f32 / memories.len() as f32;

        if mean == 0.0 {
            return 0.0;
        }

        let variance: f32 = memories
            .iter()
            .map(|&m| {
                let diff = m as f32 - mean;
                diff * diff
            })
            .sum::<f32>()
            / memories.len() as f32;

        (variance.sqrt() / mean).abs()
    }

    /// Check if peer-to-peer communication is supported
    fn check_p2p_support(&self) -> bool {
        // P2P requires same backend (all CUDA or all Metal)
        let backends: std::collections::HashSet<GpuBackend> =
            self.devices.iter().map(|d| d.backend).collect();

        if backends.len() != 1 {
            return false;
        }

        // CUDA P2P check would go here
        #[cfg(feature = "cuda")]
        {
            if backends.contains(&GpuBackend::Cuda) {
                // Would check CUDA P2P access using cudarc
                return true; // Assume supported for now
            }
        }

        // Metal doesn't support P2P in the traditional sense
        // but unified memory on Apple Silicon provides similar benefits
        false
    }

    /// Partition model by layers
    fn partition_by_layers(
        &self,
        num_layers: usize,
        model_size: u64,
    ) -> Result<Vec<GpuPartition>, MultiGpuError> {
        let num_gpus = self.devices.len();
        let layers_per_gpu = num_layers / num_gpus;
        let extra_layers = num_layers % num_gpus;

        let mut partitions = Vec::with_capacity(num_gpus);
        let mut current_layer = 0;

        // Compute memory per GPU based on available memory
        let total_available: u64 = self.devices.iter().map(|d| d.available_memory).sum();

        for (i, device) in self.devices.iter().enumerate() {
            // Assign layers, distributing extras to first GPUs
            let gpu_layers = layers_per_gpu + if i < extra_layers { 1 } else { 0 };
            let layer_start = current_layer;
            let layer_end = current_layer + gpu_layers;

            // Compute memory budget proportional to available memory
            let memory_fraction = device.available_memory as f64 / total_available as f64;
            let memory_budget = (model_size as f64 * memory_fraction) as u64;

            let parameter_fraction = gpu_layers as f32 / num_layers as f32;

            partitions.push(GpuPartition {
                gpu_index: device.index,
                layers: layer_start..layer_end,
                memory_budget,
                parameter_fraction,
            });

            current_layer = layer_end;
        }

        Ok(partitions)
    }

    /// Partition model by tensors (tensor parallelism)
    fn partition_by_tensors(&self, model_size: u64) -> Result<Vec<GpuPartition>, MultiGpuError> {
        let num_gpus = self.devices.len();

        // Tensor parallelism splits each tensor across GPUs
        // Each GPU gets 1/num_gpus of each tensor
        let size_per_gpu = model_size / num_gpus as u64;

        let partitions: Vec<GpuPartition> = self
            .devices
            .iter()
            .enumerate()
            .map(|(_i, device)| GpuPartition {
                gpu_index: device.index,
                layers: 0..usize::MAX, // All layers, but partial tensors
                memory_budget: size_per_gpu,
                parameter_fraction: 1.0 / num_gpus as f32,
            })
            .collect();

        Ok(partitions)
    }

    /// Partition model by pipeline stages
    fn partition_by_pipeline(
        &self,
        num_layers: usize,
        model_size: u64,
    ) -> Result<Vec<GpuPartition>, MultiGpuError> {
        // Pipeline parallelism is similar to layer parallelism
        // but optimized for sequential processing
        self.partition_by_layers(num_layers, model_size)
    }

    /// Partition model for MoE expert parallelism
    fn partition_by_experts(&self, model_size: u64) -> Result<Vec<GpuPartition>, MultiGpuError> {
        // Expert parallelism distributes experts across GPUs
        // Each GPU gets a subset of experts
        let num_gpus = self.devices.len();
        let size_per_gpu = model_size / num_gpus as u64;

        let partitions: Vec<GpuPartition> = self
            .devices
            .iter()
            .enumerate()
            .map(|(_i, device)| GpuPartition {
                gpu_index: device.index,
                layers: 0..usize::MAX, // All layers, but partial experts
                memory_budget: size_per_gpu,
                parameter_fraction: 1.0 / num_gpus as f32,
            })
            .collect();

        Ok(partitions)
    }

    /// Get current partitions
    pub fn partitions(&self) -> &[GpuPartition] {
        &self.partitions
    }

    /// Get active strategy
    pub fn active_strategy(&self) -> MultiGpuStrategy {
        self.active_strategy
    }

    /// Check if load is balanced across GPUs
    pub fn is_load_balanced(&self) -> bool {
        if self.partitions.is_empty() {
            return true;
        }

        let fractions: Vec<f32> = self
            .partitions
            .iter()
            .map(|p| p.parameter_fraction)
            .collect();

        let mean = fractions.iter().sum::<f32>() / fractions.len() as f32;

        fractions
            .iter()
            .all(|&f| (f - mean).abs() <= self.config.load_balance_threshold)
    }

    /// Get total memory across all GPUs
    pub fn total_memory(&self) -> u64 {
        self.devices.iter().map(|d| d.total_memory).sum()
    }

    /// Get total available memory across all GPUs
    pub fn total_available_memory(&self) -> u64 {
        self.devices.iter().map(|d| d.available_memory).sum()
    }

    /// Get memory utilization across all GPUs
    pub fn memory_utilization(&self) -> f32 {
        let total = self.total_memory();
        if total == 0 {
            return 0.0;
        }
        let available = self.total_available_memory();
        ((total - available) as f64 / total as f64) as f32
    }
}

/// Cross-GPU Communication Manager
pub struct CrossGpuCommunication {
    /// Source GPU
    _source: usize,
    /// Destination GPU
    _destination: usize,
    /// Whether P2P is enabled
    p2p_enabled: bool,
}

impl CrossGpuCommunication {
    /// Create a new cross-GPU communication channel
    pub fn new(source: usize, destination: usize, p2p_enabled: bool) -> Self {
        Self {
            _source: source,
            _destination: destination,
            p2p_enabled,
        }
    }

    /// Check if direct P2P transfer is possible
    pub fn can_direct_transfer(&self) -> bool {
        self.p2p_enabled
    }

    /// Get transfer method description
    pub fn transfer_method(&self) -> &'static str {
        if self.p2p_enabled {
            "P2P Direct"
        } else {
            "Host Staging"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_devices() -> Vec<Arc<GpuDevice>> {
        vec![
            Arc::new(GpuDevice {
                backend: GpuBackend::Cuda,
                index: 0,
                name: "GPU 0".to_string(),
                total_memory: 24_000_000_000,
                available_memory: 20_000_000_000,
                compute_capability: Some((8, 6)),
            }),
            Arc::new(GpuDevice {
                backend: GpuBackend::Cuda,
                index: 1,
                name: "GPU 1".to_string(),
                total_memory: 24_000_000_000,
                available_memory: 22_000_000_000,
                compute_capability: Some((8, 6)),
            }),
            Arc::new(GpuDevice {
                backend: GpuBackend::Cuda,
                index: 2,
                name: "GPU 2".to_string(),
                total_memory: 24_000_000_000,
                available_memory: 18_000_000_000,
                compute_capability: Some((8, 6)),
            }),
        ]
    }

    #[test]
    fn test_multi_gpu_manager_creation() {
        let devices = create_test_devices();
        let config = MultiGpuConfig::default();

        let manager = MultiGpuManager::new(devices, config);
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        assert_eq!(manager.num_gpus(), 3);
    }

    #[test]
    fn test_multi_gpu_insufficient_gpus() {
        let devices = vec![Arc::new(GpuDevice::cpu())];
        let config = MultiGpuConfig::default();

        let result = MultiGpuManager::new(devices, config);
        assert!(matches!(
            result,
            Err(MultiGpuError::InsufficientGpus { .. })
        ));
    }

    #[test]
    fn test_partition_by_layers() {
        let devices = create_test_devices();
        let config = MultiGpuConfig {
            strategy: MultiGpuStrategy::LayerParallelism,
            ..Default::default()
        };

        let mut manager = MultiGpuManager::new(devices, config).unwrap();
        let partitions = manager.partition_model(96, 40_000_000_000).unwrap();

        assert_eq!(partitions.len(), 3);

        // Each GPU should have ~32 layers
        let total_layers: usize = partitions.iter().map(|p| p.layers.len()).sum();
        assert_eq!(total_layers, 96);
    }

    #[test]
    fn test_memory_variance() {
        let devices = create_test_devices();
        let config = MultiGpuConfig::default();

        let manager = MultiGpuManager::new(devices, config).unwrap();
        let variance = manager.compute_memory_variance();

        // Same total memory, different available - variance should be low
        assert!(variance < 0.2);
    }

    #[test]
    fn test_total_memory() {
        let devices = create_test_devices();
        let config = MultiGpuConfig::default();

        let manager = MultiGpuManager::new(devices, config).unwrap();
        let total = manager.total_memory();

        assert_eq!(total, 72_000_000_000); // 3 * 24GB
    }

    #[test]
    fn test_cross_gpu_communication() {
        let comm = CrossGpuCommunication::new(0, 1, true);
        assert!(comm.can_direct_transfer());
        assert_eq!(comm.transfer_method(), "P2P Direct");

        let comm = CrossGpuCommunication::new(0, 1, false);
        assert!(!comm.can_direct_transfer());
        assert_eq!(comm.transfer_method(), "Host Staging");
    }

    #[test]
    fn test_multi_gpu_strategy_default() {
        assert_eq!(MultiGpuStrategy::default(), MultiGpuStrategy::Auto);
    }
}
