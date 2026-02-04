//! Model loading and validation.

use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("Model path not allowed: {0}")]
    PathNotAllowed(PathBuf),

    #[error("Model file not found: {0}")]
    NotFound(PathBuf),

    #[error("Invalid model format: {0}")]
    InvalidFormat(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Validated model path within allowed directories.
#[derive(Debug, Clone)]
pub struct ModelPath {
    path: PathBuf,
}

impl ModelPath {
    pub fn as_path(&self) -> &Path {
        &self.path
    }
}

/// Allowed directories for model loading.
const ALLOWED_DIRS: &[&str] = &["models", "tokenizers"];

/// Loads and validates models from allowed directories.
pub struct ModelLoader {
    base_path: PathBuf,
}

impl ModelLoader {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Validate and create a ModelPath if within allowed directories.
    pub fn validate_path(&self, relative_path: &str) -> Result<ModelPath, LoadError> {
        let full_path = self.base_path.join(relative_path);
        let canonical = full_path.canonicalize().map_err(|_| {
            LoadError::NotFound(full_path.clone())
        })?;

        let is_allowed = ALLOWED_DIRS.iter().any(|dir| {
            let allowed = self.base_path.join(dir);
            canonical.starts_with(&allowed)
        });

        if !is_allowed {
            return Err(LoadError::PathNotAllowed(canonical));
        }

        Ok(ModelPath { path: canonical })
    }

    /// Load model metadata from validated path.
    pub fn load_metadata(&self, model_path: &ModelPath) -> Result<ModelMetadata, LoadError> {
        let path = model_path.as_path();

        if !path.exists() {
            return Err(LoadError::NotFound(path.to_path_buf()));
        }

        let size = std::fs::metadata(path)?.len();
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(ModelMetadata { name, size_bytes: size })
    }
}

/// Basic model metadata.
#[derive(Debug, Clone)]
pub struct ModelMetadata {
    pub name: String,
    pub size_bytes: u64,
}
