#![cfg(feature = "concurrent")]

use crate::error::{MemoryError, Result};
use crate::model::{AgentProfile, AgentState, Memory};
use chrono::Utc;
use dashmap::DashMap;
use uuid::Uuid;

/// Memory store that partitions data across multiple shards for scalability.
pub struct ShardedMemoryStore {
    shards: Vec<DashMap<Uuid, Memory>>,
    agent_profile: AgentProfile,
    agent_state: AgentState,
}

impl ShardedMemoryStore {
    /// Creates a new [`ShardedMemoryStore`] with the specified number of shards.
    pub fn new(agent_profile: AgentProfile, agent_state: AgentState, num_shards: usize) -> Self {
        assert!(num_shards > 0, "num_shards must be greater than 0");
        let shards = (0..num_shards).map(|_| DashMap::new()).collect();
        Self {
            shards,
            agent_profile,
            agent_state,
        }
    }

    fn shard_index(&self, id: &Uuid) -> usize {
        (id.as_u128() as usize) % self.shards.len()
    }

    /// Adds a new memory to the appropriate shard.
    pub fn add_memory(&self, memory: Memory) -> Uuid {
        let id = memory.id;
        let idx = self.shard_index(&id);
        self.shards[idx].insert(id, memory);
        id
    }

    /// Retrieves a memory by ID, returning a cloned value.
    pub fn get_memory(&self, id: &Uuid) -> Option<Memory> {
        let idx = self.shard_index(id);
        self.shards[idx].get(id).map(|m| m.clone())
    }

    /// Removes a memory by ID.
    pub fn remove_memory(&self, id: &Uuid) -> Result<()> {
        let idx = self.shard_index(id);
        self.shards[idx]
            .remove(id)
            .map(|_| ())
            .ok_or_else(|| MemoryError::not_found(id))
    }

    /// Finds memories matching a query vector, ordered by relevance across all shards.
    pub fn find_relevant(&self, query_vector: &[f32], limit: usize) -> Result<Vec<(f32, Memory)>> {
        let now = Utc::now();
        let mut scored: Vec<_> = self
            .shards
            .iter()
            .flat_map(|shard| {
                shard.iter().map(|entry| {
                    let id = *entry.key();
                    let mem = entry.value();
                    let similarity = cosine_similarity(query_vector, &mem.semantic_vector);
                    let retention = mem.calculate_retention(now, &self.agent_state, &self.agent_profile);
                    (id, similarity * retention)
                })
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let top_n: Vec<_> = scored.into_iter().take(limit).collect();

        for (id, _) in &top_n {
            let idx = self.shard_index(id);
            if let Some(mut mem) = self.shards[idx].get_mut(id) {
                mem.record_retrieval(self.agent_profile.rho);
            }
        }

        let result = top_n
            .into_iter()
            .filter_map(|(id, score)| {
                let idx = self.shard_index(&id);
                self.shards[idx].get(&id).map(|mem| (score, mem.clone()))
            })
            .collect();

        Ok(result)
    }

    /// Performs maintenance operations like pruning old memories on all shards.
    pub fn maintain(&self, retention_threshold: f32) -> usize {
        assert!((0.0..=1.0).contains(&retention_threshold));
        let now = Utc::now();
        let mut total_pruned = 0;
        for shard in &self.shards {
            let before = shard.len();
            shard.retain(|_id, mem| {
                let retention = mem.calculate_retention(now, &self.agent_state, &self.agent_profile);
                retention >= retention_threshold
            });
            total_pruned += before - shard.len();
        }
        total_pruned
    }

    /// Updates the agent's state.
    pub fn update_agent_state(&self, state: AgentState) {
        self.agent_state = state;
    }

    /// Returns a reference to the agent profile.
    pub fn agent_profile(&self) -> &AgentProfile {
        &self.agent_profile
    }

    /// Returns a reference to the agent state.
    pub fn agent_state(&self) -> &AgentState {
        &self.agent_state
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    crate::simd::cosine_similarity(a, b)
}

