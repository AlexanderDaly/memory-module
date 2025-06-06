//! Memory storage and retrieval utilities.
//!
//! This module defines [`MemoryStore`], an in-memory store that provides basic
//! operations for inserting, querying, and maintaining [`Memory`] items.

use crate::error::{MemoryError, Result};
use crate::model::{AgentProfile, AgentState, Memory};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use chrono::Utc;
use crate::simd_utils;
use std::collections::HashMap;
use uuid::Uuid;
#[cfg(feature = "faiss")]
use crate::faiss_index::FaissIndex;

#[cfg(feature = "serde")]
/// Current data format version for serialized stores.
pub const DATA_FORMAT_VERSION: u8 = 1;

#[cfg(feature = "serde")]
#[derive(Serialize, Deserialize)]
struct MemoryStoreData {
    #[serde(default)]
    version: u8,
    memories: HashMap<Uuid, Memory>,
    agent_profile: AgentProfile,
    agent_state: AgentState,
}

/// In-memory storage for memories with basic CRUD operations
pub struct MemoryStore {
    memories: HashMap<Uuid, Memory>,
    agent_profile: AgentProfile,
    agent_state: AgentState,
    #[cfg(feature = "faiss")]
    faiss_index: Option<FaissIndex>,
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new(AgentProfile::default(), AgentState::default())
    }
}

impl MemoryStore {
    /// Creates a new [`MemoryStore`] with the given [`AgentProfile`] and [`AgentState`].
    ///
    /// # Example
    ///
    /// ```
    /// use memory_module::prelude::*;
    ///
    /// let profile = AgentProfile::default();
    /// let state = AgentState {
    ///     current_age: 30.0,
    ///     sleep_debt: 0.0,
    ///     cortisol_level: 0.0,
    ///     fatigue: 0.0,
    ///     training_factor: 0.0,
    /// };
    /// let store = MemoryStore::new(profile, state);
    /// assert_eq!(store.agent_profile().k, 0.5);
    /// ```
    pub fn new(agent_profile: AgentProfile, agent_state: AgentState) -> Self {
        Self {
            memories: HashMap::new(),
            agent_profile,
            agent_state,
            #[cfg(feature = "faiss")]
            faiss_index: None,
        }
    }

    /// Adds a new memory to the store
    pub fn add_memory(&mut self, memory: Memory) -> Uuid {
        let id = memory.id;
        #[cfg(feature = "faiss")]
        {
            if let Some(index) = &mut self.faiss_index {
                let _ = index.add_vector(id, &memory.semantic_vector);
            } else if let Ok(mut idx) = FaissIndex::new(memory.semantic_vector.len()) {
                let _ = idx.add_vector(id, &memory.semantic_vector);
                self.faiss_index = Some(idx);
            }
        }
        self.memories.insert(id, memory);
        id
    }

    /// Retrieves a memory by ID
    pub fn get_memory(&self, id: &Uuid) -> Option<&Memory> {
        self.memories.get(id)
    }

    /// Retrieves a mutable reference to a memory by ID
    pub fn get_memory_mut(&mut self, id: &Uuid) -> Option<&mut Memory> {
        self.memories.get_mut(id)
    }

    /// Removes a memory by ID.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryError::NotFound`] if the requested memory does not exist.
    pub fn remove_memory(&mut self, id: &Uuid) -> Result<()> {
        self.memories
            .remove(id)
            .map(|_| ())
            .ok_or_else(|| MemoryError::not_found(id))
    }

    /// Finds memories matching a query vector, ordered by relevance.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryError::NotFound`] if no memories exist in the store.
    ///
    /// # Panics
    ///
    /// Panics if the provided `limit` is `0`.
    pub fn find_relevant(
        &mut self,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<Vec<(f32, Memory)>> {
        let now = Utc::now();

        #[cfg(feature = "faiss")]
        let mut scored: Vec<_> = if let Some(index) = &self.faiss_index {
            index
                .search(query_vector, limit)?
                .into_iter()
                .filter_map(|(dist, id)| {
                    self.memories.get(&id).map(|mem| {
                        let retention = mem.calculate_retention(now, &self.agent_state, &self.agent_profile);
                        (id, (1.0 / (1.0 + dist)) * retention)
                    })
                })
                .collect()
        } else {
            self
                .memories
                .iter()
                .map(|(id, mem)| {
                    let similarity = cosine_similarity(query_vector, &mem.semantic_vector);
                    let retention = mem.calculate_retention(now, &self.agent_state, &self.agent_profile);
                    (*id, similarity * retention)
                })
                .collect()
        };

        #[cfg(not(feature = "faiss"))]
        let mut scored: Vec<_> = self
            .memories
            .iter()
            .map(|(id, mem)| {
                let similarity = cosine_similarity(query_vector, &mem.semantic_vector);
                let retention = mem.calculate_retention(now, &self.agent_state, &self.agent_profile);
                (*id, similarity * retention)
            })
            .collect();

        // Sort by score in descending order
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top N and update their retrieval history
        let top_n = scored.into_iter().take(limit).collect::<Vec<_>>();
        
        // Update retrieval history for top memories
        for (id, _) in &top_n {
            if let Some(mem) = self.memories.get_mut(id) {
                mem.record_retrieval(self.agent_profile.rho);
            }
        }

        // Return copies of the top memories with their scores
        let result = top_n.into_iter()
            .filter_map(|(id, score)| {
                self.memories.get(&id).map(|mem| (score, mem.clone()))
            })
            .collect();
            
        Ok(result)
    }

    /// Finds relevant memories for multiple query vectors in a single call.
    ///
    /// This is a convenience wrapper that iterates over each query vector and
    /// returns a vector of results per query.
    pub fn find_relevant_batch(
        &mut self,
        query_vectors: &[Vec<f32>],
        limit: usize,
    ) -> Result<Vec<Vec<(f32, Memory)>>> {
        query_vectors
            .iter()
            .map(|q| self.find_relevant(q, limit))
            .collect()
    }

    /// Performs maintenance operations like pruning old memories.
    ///
    /// Returns the number of memories that were pruned.
    ///
    /// # Panics
    ///
    /// Panics if `retention_threshold` is not within `0.0..=1.0`.
    pub fn maintain(&mut self, retention_threshold: f32) -> usize {
        assert!(
            (0.0..=1.0).contains(&retention_threshold),
            "retention_threshold must be between 0.0 and 1.0"
        );
        let now = Utc::now();
        let before = self.memories.len();
        
        self.memories.retain(|_id, mem| {
            let retention = mem.calculate_retention(now, &self.agent_state, &self.agent_profile);
            retention >= retention_threshold
        });
        
        before - self.memories.len()
    }

    /// Updates the agent's state
    pub fn update_agent_state(&mut self, state: AgentState) {
        self.agent_state = state;
    }

    /// Gets the current agent profile
    pub fn agent_profile(&self) -> &AgentProfile {
        &self.agent_profile
    }

    /// Gets the current agent state
    pub fn agent_state(&self) -> &AgentState {
        &self.agent_state
    }
}

#[cfg(feature = "serde")]
impl Serialize for MemoryStore {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let data = MemoryStoreData {
            version: DATA_FORMAT_VERSION,
            memories: self.memories.clone(),
            agent_profile: self.agent_profile.clone(),
            agent_state: self.agent_state.clone(),
        };
        data.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for MemoryStore {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data = MemoryStoreData::deserialize(deserializer)?;
        if data.version != DATA_FORMAT_VERSION {
            return Err(serde::de::Error::custom(format!(
                "Incompatible data format version: expected {}, found {}",
                DATA_FORMAT_VERSION, data.version
            )));
        }
        Ok(Self {
            memories: data.memories,
            agent_profile: data.agent_profile,
            agent_state: data.agent_state,
            #[cfg(feature = "faiss")]
            faiss_index: None,
        })
    }
}

/// Calculates cosine similarity between two vectors.
///
/// Returns `0.0` if the vectors are empty or their lengths differ.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    simd_utils::cosine_similarity(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    #[cfg(feature = "serde")]
    use serde_json;

    fn create_test_memory(emotion: f32, days_old: i64) -> Memory {
        let mut memory = Memory::new(
            vec![0.1, 0.2, 0.3], 
            emotion, 
            25.0, 
            1.0
        );
        memory.timestamp = Utc::now() - Duration::days(days_old);
        memory
    }

    #[test]
    fn test_add_and_retrieve_memory() {
        let mut store = MemoryStore::new(AgentProfile::default(), AgentState {
            current_age: 30.0,
            sleep_debt: 0.0,
            cortisol_level: 0.0,
            fatigue: 0.0,
            training_factor: 0.0,
        });

        let memory = create_test_memory(0.5, 1);
        let id = memory.id;
        
        store.add_memory(memory);
        assert!(store.get_memory(&id).is_some());
        
        store.remove_memory(&id).unwrap();
        assert!(store.get_memory(&id).is_none());
    }

    #[test]
    fn test_find_relevant() {
        let mut store = MemoryStore::new(AgentProfile::default(), AgentState {
            current_age: 30.0,
            sleep_debt: 0.0,
            cortisol_level: 0.0,
            fatigue: 0.0,
            training_factor: 0.0,
        });

        // Add some test memories
        store.add_memory(create_test_memory(0.5, 1));
        store.add_memory(create_test_memory(-0.2, 2));
        store.add_memory(create_test_memory(0.8, 3));

        // Find relevant memories
        let results = store.find_relevant(&[0.1, 0.2, 0.3], 2).unwrap();
        assert_eq!(results.len(), 2);
        
        // Should be sorted by relevance
        assert!(results[0].0 >= results[1].0);
    }

    #[test]
    fn test_maintenance() {
        let mut store = MemoryStore::new(AgentProfile::default(), AgentState {
            current_age: 30.0,
            sleep_debt: 0.0,
            cortisol_level: 0.0,
            fatigue: 0.0,
            training_factor: 0.0,
        });

        // Add a memory that should be kept (recent)
        store.add_memory(create_test_memory(0.5, 1));
        
        // Add a memory that should be pruned (very old)
        let mut old_memory = create_test_memory(0.5, 1);
        old_memory.timestamp = Utc::now() - Duration::days(365);
        let old_id = old_memory.id;
        store.add_memory(old_memory);

        // Run maintenance with a threshold that should prune the old memory
        let pruned = store.maintain(0.1);
        
        assert!(pruned > 0);
        assert!(store.get_memory(&old_id).is_none());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serialization_roundtrip() {
        let mut store = MemoryStore::new(AgentProfile::default(), AgentState::default());
        let memory = Memory::new(vec![0.1, 0.2], 0.0, 0.0, 1.0);
        let id = memory.id;
        store.add_memory(memory);

        let json = serde_json::to_string(&store).expect("serialize");
        let deserialized: MemoryStore = serde_json::from_str(&json).expect("deserialize");

        assert!(deserialized.get_memory(&id).is_some());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serialization_includes_version() {
        let store = MemoryStore::new(AgentProfile::default(), AgentState::default());
        let json = serde_json::to_string(&store).expect("serialize");
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["version"], DATA_FORMAT_VERSION);
    }
}
