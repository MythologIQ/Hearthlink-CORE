//! Hearthlink CORE Runtime
//!
//! A sandboxed, offline inference engine that performs model execution only.
//! No authority over data, tools, or system actions.
//!
//! # Design Principles (C.O.R.E.)
//!
//! - **Contained**: Sandbox with no ambient privileges
//! - **Offline**: Zero network access (inbound/outbound blocked)
//! - **Restricted**: IPC-only communication with authenticated callers
//! - **Execution**: Pure compute, no business logic or decision authority
//!
//! # Security Boundaries
//!
//! - Process: Separate OS process, restricted user
//! - Filesystem: Read `models/`, `tokenizers/`. Write `temp/`, `cache/`.
//! - Network: Blocked (deny all)
//! - IPC: Named pipes/Unix sockets only. No HTTP/REST/WebSocket.

pub mod engine;
pub mod ipc;
pub mod memory;
pub mod models;
pub mod scheduler;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use engine::InferenceEngine;
use ipc::{IpcHandler, IpcHandlerConfig, SessionAuth};
use memory::{ContextCache, ContextCacheConfig, GpuMemory, GpuMemoryConfig, MemoryPool, MemoryPoolConfig};
use models::{ModelLoader, ModelRegistry};
use scheduler::{BatchConfig, BatchProcessor, RequestQueue, RequestQueueConfig};

/// Runtime configuration.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub base_path: PathBuf,
    pub auth_token: String,
    pub session_timeout: Duration,
    pub max_context_length: usize,
    pub memory_pool: MemoryPoolConfig,
    pub gpu_memory: GpuMemoryConfig,
    pub context_cache: ContextCacheConfig,
    pub request_queue: RequestQueueConfig,
    pub batch: BatchConfig,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            base_path: PathBuf::from("."),
            auth_token: String::new(),
            session_timeout: Duration::from_secs(3600),
            max_context_length: 4096,
            memory_pool: MemoryPoolConfig::default(),
            gpu_memory: GpuMemoryConfig::default(),
            context_cache: ContextCacheConfig::default(),
            request_queue: RequestQueueConfig::default(),
            batch: BatchConfig::default(),
        }
    }
}

/// The CORE Runtime instance.
pub struct Runtime {
    pub memory_pool: MemoryPool,
    pub gpu_memory: GpuMemory,
    pub context_cache: ContextCache,
    pub model_loader: ModelLoader,
    pub model_registry: Arc<ModelRegistry>,
    pub inference_engine: InferenceEngine,
    pub request_queue: Arc<RequestQueue>,
    pub batch_processor: BatchProcessor,
    pub ipc_handler: IpcHandler,
}

impl Runtime {
    /// Create a new runtime instance with the given configuration.
    pub fn new(config: RuntimeConfig) -> Self {
        let memory_pool = MemoryPool::new(config.memory_pool);
        let gpu_memory = GpuMemory::new(config.gpu_memory);
        let context_cache = ContextCache::new(config.context_cache);
        let model_loader = ModelLoader::new(config.base_path);
        let model_registry = Arc::new(ModelRegistry::new());
        let inference_engine = InferenceEngine::new(config.max_context_length);
        let request_queue = Arc::new(RequestQueue::new(config.request_queue));
        let batch_processor = BatchProcessor::new(config.batch);

        let session_auth = Arc::new(SessionAuth::new(&config.auth_token, config.session_timeout));
        let ipc_handler = IpcHandler::new(
            session_auth,
            request_queue.clone(),
            IpcHandlerConfig::default(),
        );

        Self {
            memory_pool,
            gpu_memory,
            context_cache,
            model_loader,
            model_registry,
            inference_engine,
            request_queue,
            batch_processor,
            ipc_handler,
        }
    }
}
