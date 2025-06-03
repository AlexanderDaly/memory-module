//! Error handling for the memory module.
//!
//! This module provides the error type and associated utilities for handling
//! errors that can occur during memory operations.

use std::fmt;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The error type for memory module operations.
///
/// This enum represents the various kinds of errors that can occur when working
/// with the memory module. Each variant contains additional context about the error.
///
/// # Examples
///
/// ```
/// use memory_module::error::MemoryError;
///
/// // Creating different types of errors
/// let not_found = MemoryError::not_found("123e4567-e89b-12d3-a456-426614174000");
/// let invalid_param = MemoryError::invalid_param("retention_threshold", -1.0);
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum MemoryError {
    /// The requested memory was not found.
    ///
    /// This typically occurs when trying to access a memory that doesn't exist
    /// or has been pruned.
    NotFound(String),
    
    /// An error occurred during serialization or deserialization.
    ///
    /// This can happen when loading or saving memories to persistent storage.
    Serialization(String),
    
    /// An error occurred during a storage operation.
    ///
    /// This is used for low-level storage errors, such as I/O errors.
    Storage(String),
    
    /// A provided parameter was invalid.
    ///
    /// This is used when a function receives an invalid parameter value.
    InvalidParameter(String),
    
    /// The requested operation is not supported.
    ///
    /// This is used when an operation is not implemented or not applicable
    /// in the current context.
    NotSupported(String),
    
    #[cfg(feature = "faiss")]
    #[error("FAISS error: {0}")]
    FaissError(#[from] faiss::error::Error),
}

impl std::error::Error for MemoryError {}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryError::NotFound(msg) => write!(f, "Memory not found: {}", msg),
            MemoryError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            MemoryError::Storage(msg) => write!(f, "Storage error: {}", msg),
            MemoryError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            MemoryError::NotSupported(msg) => write!(f, "Operation not supported: {}", msg),
            #[cfg(feature = "faiss")]
            MemoryError::FaissError(err) => write!(f, "FAISS error: {}", err),
        }
    }
}

impl MemoryError {
    /// Creates a new `NotFound` error for a memory with the given ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the memory that was not found
    ///
    /// # Example
    ///
    /// ```
    /// use memory_module::error::MemoryError;
    ///
    /// let error = MemoryError::not_found("123e4567-e89b-12d3-a456-426614174000");
    /// assert_eq!(
    ///     error.to_string(),
    ///     "Memory not found: Memory with id 123e4567-e89b-12d3-a456-426614174000 not found"
    /// );
    /// ```
    pub fn not_found<T: fmt::Display>(id: T) -> Self {
        MemoryError::NotFound(format!("Memory with id {} not found", id))
    }

    /// Creates a new `InvalidParameter` error for the given parameter and value.
    ///
    /// # Arguments
    ///
    /// * `param` - The name of the invalid parameter
    /// * `value` - The invalid value that was provided
    ///
    /// # Example
    ///
    /// ```
    /// use memory_module::error::MemoryError;
    ///
    /// let error = MemoryError::invalid_param("retention_threshold", -1.0);
    /// assert_eq!(
    ///     error.to_string(),
    ///     "Invalid parameter: Invalid value '-1' for parameter 'retention_threshold'"
    /// );
    /// ```
    pub fn invalid_param<T: fmt::Display>(param: &str, value: T) -> Self {
        MemoryError::InvalidParameter(format!(
            "Invalid value '{}' for parameter '{}'", 
            value, param
        ))
    }

    /// Returns `true` if this error is a `NotFound` error.
    ///
    /// # Example
    ///
    /// ```
    /// use memory_module::error::MemoryError;
    ///
    /// let error = MemoryError::not_found("123");
    /// assert!(error.is_not_found());
    /// assert!(!error.is_invalid_parameter());
    /// ```
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound(_))
    }

    /// Returns `true` if this error is an `InvalidParameter` error.
    ///
    /// # Example
    ///
    /// ```
    /// use memory_module::error::MemoryError;
    ///
    /// let error = MemoryError::invalid_param("threshold", -1.0);
    /// assert!(error.is_invalid_parameter());
    /// assert!(!error.is_not_found());
    /// ```
    pub fn is_invalid_parameter(&self) -> bool {
        matches!(self, Self::InvalidParameter(_))
    }
}

/// A specialized `Result` type for memory module operations.
///
/// This is a convenience type that defaults to using `MemoryError` as the error type.
pub type Result<T> = std::result::Result<T, MemoryError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_error_display() {
        let not_found = MemoryError::not_found("123");
        assert_eq!(
            not_found.to_string(),
            "Memory not found: Memory with id 123 not found"
        );

        let invalid_param = MemoryError::invalid_param("threshold", -1.0);
        assert_eq!(
            invalid_param.to_string(),
            "Invalid parameter: Invalid value '-1' for parameter 'threshold'"
        );
    }

    #[test]
    fn test_error_source() {
        let error = MemoryError::not_found("123");
        assert!(error.source().is_none());
    }
}
