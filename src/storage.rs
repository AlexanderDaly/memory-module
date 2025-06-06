use crate::error::{MemoryError, Result};
use crate::model::{AgentProfile, AgentState, Memory};
use crate::store::MemoryStore;
use std::collections::HashMap;
use std::fs::{File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use uuid::Uuid;

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

/// Data container used for serialization of [`MemoryStore`] state.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StoredData {
    /// All memories indexed by id.
    pub memories: HashMap<Uuid, Memory>,
    /// The agent profile associated with the store.
    pub agent_profile: AgentProfile,
    /// The agent state associated with the store.
    pub agent_state: AgentState,
}

/// Trait describing a persistence backend for [`MemoryStore`].
pub trait StorageBackend {
    /// Load stored data from the backend.
    fn load(&self) -> Result<StoredData>;
    /// Save data to the backend.
    fn save(&self, data: &StoredData) -> Result<()>;
}

/// Simple JSON file-based storage backend.
#[cfg(feature = "serde")]
pub struct FileBackend {
    path: PathBuf,
}

#[cfg(feature = "serde")]
impl FileBackend {
    /// Create a new [`FileBackend`] with the given path.
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self { path: path.into() }
    }
}

#[cfg(feature = "serde")]
impl StorageBackend for FileBackend {
    fn load(&self) -> Result<StoredData> {
        if !self.path.exists() {
            return Ok(StoredData {
                memories: HashMap::new(),
                agent_profile: AgentProfile::default(),
                agent_state: AgentState::default(),
            });
        }
        let file = File::open(&self.path).map_err(|e| MemoryError::Storage(e.to_string()))?;
        let reader = BufReader::new(file);
        let data: StoredData = serde_json::from_reader(reader)
            .map_err(|e| MemoryError::Serialization(e.to_string()))?;
        Ok(data)
    }

    fn save(&self, data: &StoredData) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| MemoryError::Storage(e.to_string()))?;
        }
        let file = File::create(&self.path).map_err(|e| MemoryError::Storage(e.to_string()))?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, data)
            .map_err(|e| MemoryError::Serialization(e.to_string()))?;
        Ok(())
    }
}

/// SQLite-based storage backend using `sqlx`.
#[cfg(all(feature = "serde", feature = "sqlite"))]
pub struct SqliteBackend {
    url: String,
}

#[cfg(all(feature = "serde", feature = "sqlite"))]
impl SqliteBackend {
    /// Create a new [`SqliteBackend`] with the given connection URL.
    pub fn new<U: Into<String>>(url: U) -> Self {
        Self { url: url.into() }
    }

    fn block_on<F: std::future::Future>(&self, fut: F) -> F::Output {
        tokio::runtime::Runtime::new()
            .expect("create runtime")
            .block_on(fut)
    }
}

#[cfg(all(feature = "serde", feature = "sqlite"))]
impl StorageBackend for SqliteBackend {
    fn load(&self) -> Result<StoredData> {
        use sqlx::sqlite::SqlitePoolOptions;

        let url = self.url.clone();
        self.block_on(async move {
            let pool = SqlitePoolOptions::new()
                .connect(&url)
                .await
                .map_err(|e| MemoryError::Storage(e.to_string()))?;

            sqlx::query(
                "CREATE TABLE IF NOT EXISTS memory_store (id INTEGER PRIMARY KEY, data TEXT NOT NULL)",
            )
            .execute(&pool)
            .await
            .map_err(|e| MemoryError::Storage(e.to_string()))?;

            if let Some(row) = sqlx::query_as::<_, (String,)>(
                "SELECT data FROM memory_store WHERE id = 1",
            )
            .fetch_optional(&pool)
            .await
            .map_err(|e| MemoryError::Storage(e.to_string()))?
            {
                let data: StoredData = serde_json::from_str(&row.0)
                    .map_err(|e| MemoryError::Serialization(e.to_string()))?;
                Ok(data)
            } else {
                Ok(StoredData {
                    memories: HashMap::new(),
                    agent_profile: AgentProfile::default(),
                    agent_state: AgentState::default(),
                })
            }
        })
    }

    fn save(&self, data: &StoredData) -> Result<()> {
        use sqlx::sqlite::SqlitePoolOptions;

        let json = serde_json::to_string(data)
            .map_err(|e| MemoryError::Serialization(e.to_string()))?;
        let url = self.url.clone();
        self.block_on(async move {
            let pool = SqlitePoolOptions::new()
                .connect(&url)
                .await
                .map_err(|e| MemoryError::Storage(e.to_string()))?;

            sqlx::query(
                "CREATE TABLE IF NOT EXISTS memory_store (id INTEGER PRIMARY KEY, data TEXT NOT NULL)",
            )
            .execute(&pool)
            .await
            .map_err(|e| MemoryError::Storage(e.to_string()))?;

            sqlx::query(
                "INSERT INTO memory_store (id, data) VALUES (1, ?1) \
                 ON CONFLICT(id) DO UPDATE SET data=excluded.data",
            )
            .bind(json)
            .execute(&pool)
            .await
            .map_err(|e| MemoryError::Storage(e.to_string()))?;

            Ok(())
        })
    }
}

#[cfg(feature = "serde")]
impl MemoryStore {
    /// Persist the store to the given backend.
    pub fn save<B: StorageBackend>(&self, backend: &B) -> Result<()> {
        let data = StoredData {
            memories: self.memories.clone(),
            agent_profile: self.agent_profile.clone(),
            agent_state: self.agent_state.clone(),
        };
        backend.save(&data)
    }

    /// Load a [`MemoryStore`] from the given backend.
    pub fn load<B: StorageBackend>(backend: &B) -> Result<Self> {
        let data = backend.load()?;
        Ok(Self {
            memories: data.memories,
            agent_profile: data.agent_profile,
            agent_state: data.agent_state,
            #[cfg(feature = "faiss")]
            faiss_index: None,
        })
    }
}

