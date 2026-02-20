// Copyright 2024-2026 GG-CORE Contributors
// SPDX-License-Identifier: Apache-2.0

//! Kubernetes integration types.
//!
//! Defines Rust types matching the GgRuntime and GgModel CRDs.

pub mod types;

// K8s CRD types - names match the actual CRD kind for compatibility
pub use types::{VeritasModel, VeritasModelSpec, VeritasRuntime, VeritasRuntimeSpec};
