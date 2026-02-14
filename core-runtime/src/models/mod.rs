//! Model management module for CORE Runtime.
//!
//! Handles model loading, registry tracking, manifest parsing, and hot-swap operations.

pub mod manifest;

mod drain;
mod loader;
mod preload;
mod registry;
mod router;
mod swap;

pub use drain::{DrainError, FlightGuard, FlightTracker};
pub use loader::{LoadError, MappedModel, ModelLoader, ModelMetadata, ModelPath};
pub use manifest::{ModelArchitecture, ModelCapability, ModelManifest};
pub use preload::{ModelPreloader, PreloadError, PreloadedModel};
pub use registry::{ModelHandle, ModelRegistry};
pub use router::{ModelRouter, RouterError};
pub use swap::{SwapError, SwapManager, SwapResult};
