#![cfg(feature = "concurrent")]

use crate::error::{MemoryError, Result};
use crate::model::{AgentProfile, AgentState, Memory};
use chrono::Utc;
use dashmap::DashMap;
use uuid::Uuid;

/// Thread-safe memory store using `DashMap` for concurrent access.
pub struct ConcurrentMemoryStore {
    memories: DashMap<Uuid, Memory>,
    agent_profile: AgentProfile,
    agent_state: AgentState,
}

impl ConcurrentMemoryStore {
    /// Creates a new [`ConcurrentMemoryStore`].
    pub fn new(agent_profile: AgentProfile, agent_state: AgentState) -> Self {
        Self {
            memories: DashMap::new(),
            agent_profile,
            agent_state,
        }
    }

    /// Adds a new memory to the store.
    pub fn add_memory(&self, memory: Memory) -> Uuid {
        let id = memory.id;
        self.memories.insert(id, memory);
        id
    }

    /// Retrieves a memory by ID, returning a cloned value.
    pub fn get_memory(&self, id: &Uuid) -> Option<Memory> {
        self.memories.get(id).map(|m| m.clone())
    }

    /// Removes a memory by ID.
    pub fn remove_memory(&self, id: &Uuid) -> Result<()> {
        self.memories
            .remove(id)
            .map(|_| ())
            .ok_or_else(|| MemoryError::not_found(id))
    }

    /// Finds memories matching a query vector, ordered by relevance.
    pub fn find_relevant(
        &self,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<Vec<(f32, Memory)>> {
        let now = Utc::now();

        // First pass: score all memories
        let mut scored: Vec<_> = self
            .memories
            .iter()
            .map(|entry| {
                let id = *entry.key();
                let mem = entry.value();
                let similarity = cosine_similarity(query_vector, &mem.semantic_vector);
                let retention = mem.calculate_retention(now, &self.agent_state, &self.agent_profile);
                (id, similarity * retention)
            })
            .collect();

        // Sort by score in descending order
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let top_n: Vec<_> = scored.into_iter().take(limit).collect();

        // Update retrieval history for top memories
        for (id, _) in &top_n {
            if let Some(mut mem) = self.memories.get_mut(id) {
                mem.record_retrieval(self.agent_profile.rho);
            }
        }

        let result = top_n
            .into_iter()
            .filter_map(|(id, score)| self.memories.get(&id).map(|mem| (score, mem.clone())))
            .collect();

        Ok(result)
    }

    /// Finds relevant memories for multiple query vectors in a single call.
    pub fn find_relevant_batch(
        &self,
        query_vectors: &[Vec<f32>],
        limit: usize,
    ) -> Result<Vec<Vec<(f32, Memory)>>> {
        query_vectors
            .iter()
            .map(|q| self.find_relevant(q, limit))
            .collect()
    }

    /// Performs maintenance operations like pruning old memories.
    pub fn maintain(&self, retention_threshold: f32) -> usize {
        assert!((0.0..=1.0).contains(&retention_threshold));
        let now = Utc::now();
        let before = self.memories.len();
        self.memories.retain(|_id, mem| {
            let retention = mem.calculate_retention(now, &self.agent_state, &self.agent_profile);
            retention >= retention_threshold
        });
        before - self.memories.len()
    }

    /// Updates the agent's state.
    pub fn update_agent_state(&self, state: AgentState) {
        self.agent_state = state;
    }

    /// Gets the current agent profile.
    pub fn agent_profile(&self) -> &AgentProfile {
        &self.agent_profile
    }

    /// Gets the current agent state.
    pub fn agent_state(&self) -> &AgentState {
        &self.agent_state
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }
    let dot_product: f32 = a.iter().zip(b).map(|(&x, &y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|&x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|&x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

