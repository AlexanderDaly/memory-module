//! Memory storage and retrieval utilities.
//!
//! This module defines [`MemoryStore`], an in-memory store that provides basic
//! operations for inserting, querying, and maintaining [`Memory`] items.

use crate::error::{MemoryError, Result};
use crate::model::{AgentProfile, AgentState, Memory};
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

/// In-memory storage for memories with basic CRUD operations
pub struct MemoryStore {
    memories: HashMap<Uuid, Memory>,
    agent_profile: AgentProfile,
    agent_state: AgentState,
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
        }
    }

    /// Adds a new memory to the store
    pub fn add_memory(&mut self, memory: Memory) -> Uuid {
        let id = memory.id;
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
        
        // First pass: score all memories
        let mut scored: Vec<_> = self
            .memories
            .iter()
            .map(|(id, mem)| {
                // Calculate relevance score (cosine similarity * retention)
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

/// Calculates cosine similarity between two vectors.
///
/// Returns `0.0` if the vectors are empty or their lengths differ.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    crate::simd::cosine_similarity(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

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
}
