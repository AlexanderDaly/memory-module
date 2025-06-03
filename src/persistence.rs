//! Persistence utilities for saving and loading data structures.
//!
//! These traits provide simple file-based persistence using `serde`.

use crate::error::{MemoryError, Result};
use std::fs;
use std::path::Path;

#[cfg(feature = "serde")]
use serde::{de::DeserializeOwned, Serialize};

/// Trait for saving a value to persistent storage.
#[cfg(feature = "serde")]
pub trait Save {
    /// Save the value to the specified path.
    fn save<P: AsRef<Path>>(&self, path: P) -> Result<()>;
}

/// Trait for loading a value from persistent storage.
#[cfg(feature = "serde")]
pub trait Load: Sized {
    /// Load the value from the specified path.
    fn load<P: AsRef<Path>>(path: P) -> Result<Self>;
}

#[cfg(feature = "serde")]
impl<T> Save for T
where
    T: Serialize,
{
    fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let data = serde_json::to_vec(self)
            .map_err(|e| MemoryError::Serialization(e.to_string()))?;
        fs::write(path, data).map_err(|e| MemoryError::Storage(e.to_string()))
    }
}

#[cfg(feature = "serde")]
impl<T> Load for T
where
    T: DeserializeOwned,
{
    fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = fs::read(path).map_err(|e| MemoryError::Storage(e.to_string()))?;
        serde_json::from_slice(&data).map_err(|e| MemoryError::Serialization(e.to_string()))
    }
}
