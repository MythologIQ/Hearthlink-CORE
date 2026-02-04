//! Model management module for CORE Runtime.
//!
//! Handles model loading, registry tracking, and hot-swap operations.

mod loader;
mod registry;
mod swap;

pub use loader::{LoadError, ModelLoader, ModelPath};
pub use registry::{ModelHandle, ModelRegistry};
pub use swap::{SwapError, SwapManager};
