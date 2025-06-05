//! A biologically inspired memory module for AI agents.
//! 
//! This crate provides a flexible framework for modeling memory formation, retention, and retrieval
//! based on cognitive science principles. It's designed to be integrated into AI systems that
//! require realistic memory behavior, such as NPCs in games or virtual assistants.
//!
//! # Features
//! - Biologically plausible memory formation and decay
//! - Emotional modulation of memory strength
//! - Interference-based forgetting
//! - Capacity-limited memory storage
//! - Configurable agent profiles for different memory behaviors
//!
//! # Example
//! ```no_run
//! use memory_module::{
//!     AgentProfile, AgentState, Memory, MemoryStore,
//!     chrono::Utc,
//! };
//!
//! // Create an agent profile with default parameters
//! let profile = AgentProfile::default();
//! 
//! // Create an initial agent state
//! let state = AgentState {
//!     current_age: 30.0,
//!     sleep_debt: 0.2,
//!     cortisol_level: 0.1,
//!     fatigue: 0.3,
//!     training_factor: 0.4,
//! };
//!
//! // Create a new memory store
//! let mut store = MemoryStore::new(profile, state);
//!
//! // Create a new memory
//! let memory = Memory::new(
//!     vec![0.1, 0.2, 0.3], // Semantic vector
//!     0.5,                // Emotion (-1.0 to 1.0)
//!     25.0,               // Age at formation
//!     1.0,                // Capacity weight
//! );
//!
//! // Add the memory to the store
//! let memory_id = store.add_memory(memory);
//!
//! // Later, retrieve relevant memories
//! let query = vec![0.15, 0.25, 0.35];
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![doc(html_root_url = "https://docs.rs/memory-module/0.1.0")]
#![doc(html_logo_url = "https://example.com/logo.png")]
#![doc(html_favicon_url = "https://example.com/favicon.ico")]

pub mod error;
pub mod model;
pub mod store;
pub mod storage;
pub mod simd_utils;
#[cfg(feature = "concurrent")]
pub mod concurrent_store;
#[cfg(feature = "concurrent")]
pub mod sharded_store;
#[cfg(any(feature = "faiss"))]
pub mod faiss_index;
pub mod persistence;
#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
pub mod migration;

// Re-exports
pub use chrono;
pub use model::{AgentProfile, AgentState, Memory};
pub use store::MemoryStore;
#[cfg(feature = "serde")]
pub use storage::{FileBackend, StoredData};
#[cfg(all(feature = "serde", feature = "sqlite"))]
pub use storage::SqliteBackend;
pub use storage::StorageBackend;
#[cfg(feature = "concurrent")]
pub use concurrent_store::ConcurrentMemoryStore;
#[cfg(feature = "concurrent")]
pub use sharded_store::ShardedMemoryStore;
pub use persistence::{Load, Save};
pub use uuid;
#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
pub use migration::run_migrations;

/// Prelude for convenient importing
///
/// This module provides a prelude that can be imported to bring commonly used types
/// and traits into scope with a single `use` statement.
///
/// # Example
/// ```
/// use memory_module::prelude::*;
/// ```
pub mod prelude {
    pub use crate::error::{MemoryError, Result};
    pub use crate::model::{AgentProfile, AgentState, Memory};
    pub use crate::store::MemoryStore;
    pub use crate::persistence::{Load, Save};
    pub use crate::StorageBackend;
    #[cfg(feature = "serde")]
    pub use crate::FileBackend;
    #[cfg(all(feature = "serde", feature = "sqlite"))]
    pub use crate::SqliteBackend;
    #[cfg(feature = "serde")]
    pub use crate::StoredData;
    #[cfg(feature = "concurrent")]
    pub use crate::concurrent_store::ConcurrentMemoryStore;
    #[cfg(feature = "concurrent")]
    pub use crate::sharded_store::ShardedMemoryStore;
    #[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
    pub use crate::run_migrations;
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    #[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
    use sqlx::AnyPool;

    #[test]
    fn test_prelude_reexports() {
        // Verify that all expected items are re-exported
        let _: MemoryError = MemoryError::NotFound("test".to_string());
        let _: Result<()> = Ok(());
        let _ = AgentProfile::default();
        let _ = AgentState::default();
        let _ = Memory::new(vec![], 0.0, 0.0, 0.0);
        let _ = MemoryStore::default();
        // Ensure Save/Load traits are in scope
        fn assert_save_load<T: Save + Load>() {}
        let _ = assert_save_load::<MemoryStore>;
        #[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
        {
            let _fn: fn(&sqlx::AnyPool) -> Result<()> = run_migrations;
            let _ = _fn;
        }
    }
}
