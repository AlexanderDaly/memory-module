//! Core data models for the memory module.
//!
//! This module defines the fundamental data structures and their associated
//! behaviors that make up the memory system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

use crate::error::Result;

/// Represents the current cognitive and emotional state of an agent.
///
/// This state can influence how memories are formed, retained, and retrieved.
/// All values are normalized to the range [0.0, 1.0].
///
/// # Examples
///
/// ```
/// use memory_module::model::AgentState;
///
/// // Create a stressed and fatigued agent state
/// let state = AgentState {
///     stress: 0.8,    // High stress
///     fatigue: 0.7,   // High fatigue
///     focus: 0.3,     // Low focus
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentState {
    /// Current stress level (0.0 = none, 1.0 = maximum stress)
    ///
    /// Higher stress levels can negatively impact memory formation and retrieval.
    pub stress: f32,
    
    /// Current fatigue level (0.0 = fully rested, 1.0 = completely fatigued)
    ///
    /// Fatigue affects the agent's ability to form and retrieve memories.
    pub fatigue: f32,
    
    /// Current focus level (0.0 = completely distracted, 1.0 = fully focused)
    ///
    /// Higher focus improves memory formation and recall accuracy.
    pub focus: f32,
}

impl Default for AgentState {
    /// Creates a default `AgentState` with neutral values.
    ///
    /// ```
    /// use memory_module::model::AgentState;
    ///
    /// let state = AgentState::default();
    /// assert_eq!(state.stress, 0.0);
    /// assert_eq!(state.fatigue, 0.0);
    /// assert_eq!(state.focus, 1.0);
    /// ```
    fn default() -> Self {
        Self {
            stress: 0.0,
            fatigue: 0.0,
            focus: 1.0,
        }
    }
}

/// Configuration parameters that define an agent's memory characteristics.
///
/// These parameters control how memories are formed, retained, and forgotten.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentProfile {
    /// Base decay rate for memories (higher = faster decay)
    ///
    /// This controls how quickly memories naturally decay over time.
    /// Typical range: 0.01 (slow decay) to 0.5 (fast decay)
    pub decay_rate: f32,
    
    /// Emotional bias factor (how much emotion affects memory strength)
    ///
    /// Higher values make emotional memories more resistant to forgetting.
    /// Typical range: 0.0 (no effect) to 2.0 (strong effect)
    pub emotional_bias: f32,
    
    /// Capacity factor (affects how quickly memories interfere with each other)
    ///
    /// Higher values mean the agent has more limited memory capacity.
    /// Typical range: 0.1 (large capacity) to 2.0 (limited capacity)
    pub capacity_factor: f32,
    
    /// Interference factor (how much similar memories affect each other)
    ///
    /// Higher values mean more interference between similar memories.
    /// Typical range: 0.0 (no interference) to 1.0 (strong interference)
    pub interference_factor: f32,
}

impl Default for AgentProfile {
    /// Creates a default `AgentProfile` with balanced parameters.
    ///
    /// ```
    /// use memory_module::model::AgentProfile;
    ///
    /// let profile = AgentProfile::default();
    /// assert_eq!(profile.decay_rate, 0.1);
    /// assert_eq!(profile.emotional_bias, 0.5);
    /// assert_eq!(profile.capacity_factor, 1.0);
    /// assert_eq!(profile.interference_factor, 0.3);
    /// ```
    fn default() -> Self {
        Self {
            decay_rate: 0.1,
            emotional_bias: 0.5,
            capacity_factor: 1.0,
            interference_factor: 0.3,
        }
    }
}

/// Represents a single memory with associated metadata and retrieval history.
///
/// Each memory contains:
/// - A semantic vector embedding of the memory content
/// - Emotional context and formation time
/// - Retrieval history and metadata
///
/// # Examples
///
/// ```
/// use memory_module::model::Memory;
///
/// // Create a new memory
/// let memory = Memory::new(
///     vec![0.1, 0.2, 0.3], // Semantic vector
///     0.5,                 // Emotion (-1.0 to 1.0)
///     25.0,               // Age at formation
///     0.8,                // Capacity weight (0.0-1.0)
/// );
///
/// // Add metadata
/// let memory = memory.with_metadata("source", "conversation");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Memory {
    /// Unique identifier for the memory
    pub id: Uuid,
    
    /// Semantic vector representation of the memory
    ///
    /// This vector should encode the semantic meaning of the memory in a way that
    /// similar memories have similar vectors (high cosine similarity).
    pub semantic_vector: Vec<f32>,
    
    /// Emotional valence (-1.0 to 1.0)
    ///
    /// - Negative values: Negative emotions (fear, sadness)
    /// - 0.0: Neutral
    /// - Positive values: Positive emotions (happiness, excitement)
    pub emotion: f32,
    
    /// Age at formation (in arbitrary units, typically years)
    ///
    /// This can be the agent's age when the memory was formed, or another
    /// time scale relevant to the application.
    pub age_at_formation: f64,
    
    /// Capacity weight (0.0 to 1.0)
    ///
    /// How much this memory contributes to capacity limitations.
    /// Lower values mean the memory takes up less "space" in memory.
    pub capacity_weight: f32,
    
    /// When the memory was formed
    pub timestamp: DateTime<Utc>,
    
    /// When the memory was last retrieved
    pub last_retrieved: DateTime<Utc>,
    
    /// Number of times the memory has been retrieved
    pub retrieval_count: u32,
    
    /// Additional metadata as key-value pairs
    ///
    /// This can be used to store application-specific information about the memory.
    pub metadata: serde_json::Value,
    
    /// History of when this memory was retrieved
    pub recall_history: VecDeque<DateTime<Utc>>,
    
    /// Current memory strength (λ in the retention equation)
    pub memory_strength: f32,
    
    /// Decay parameters
    pub decay_params: DecayParams,
}

/// Parameters that control memory decay
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DecayParams {
    /// Base decay rate (α)
    pub alpha: f32,
    
    /// Initial time scale (β₀)
    pub beta_0: f32,
}

impl Default for DecayParams {
    fn default() -> Self {
        Self {
            alpha: 0.8,
            beta_0: 0.01,
        }
    }
}

impl Memory {
    /// Creates a new memory with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `semantic_vector` - Semantic vector representation of the memory
    /// * `emotion` - Emotional valence (-1.0 to 1.0)
    /// * `age_at_formation` - Age at formation (in arbitrary units)
    /// * `capacity_weight` - How much capacity this memory uses (0.0-1.0)
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_module::model::Memory;
    ///
    /// let memory = Memory::new(
    ///     vec![0.1, 0.2, 0.3],
    ///     0.5,   // Slightly positive emotion
    ///     25.0,  // Formed at age 25
    ///     0.8,   // High capacity weight
    /// );
    /// ```
    pub fn new(semantic_vector: Vec<f32>, emotion: f32, age_at_formation: f64, capacity_weight: f32) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            semantic_vector,
            emotion: emotion.clamp(-1.0, 1.0),
            age_at_formation,
            capacity_weight: capacity_weight.clamp(0.0, 1.0),
            timestamp: now,
            last_retrieved: now,
            retrieval_count: 0,
            metadata: serde_json::json!({}),
            recall_history: VecDeque::new(),
            memory_strength: 1.0,
            decay_params: DecayParams::default(),
        }
    }

    /// Calculates the current retention strength of the memory.
    ///
    /// The retention strength is a value between 0.0 (completely forgotten)
    /// and 1.0 (perfectly retained) that represents how well the memory is
    /// currently retained.
    ///
    /// # Arguments
    ///
    /// * `now` - Current timestamp
    /// * `agent_state` - Current state of the agent
    /// * `profile` - Agent's memory profile
    ///
    /// # Returns
    ///
    /// A value between 0.0 and 1.0 representing the current retention strength.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_module::model::{Memory, AgentState, AgentProfile};
    /// use chrono::Utc;
    ///
    /// let memory = Memory::new(vec![0.1, 0.2], 0.5, 25.0, 0.8);
    /// let agent_state = AgentState::default();
    /// let profile = AgentProfile::default();
    /// let retention = memory.calculate_retention(Utc::now(), &agent_state, &profile);
    ///
    /// assert!(retention > 0.0 && retention <= 1.0);
    /// ```
    pub fn calculate_retention(&self, now: DateTime<Utc>, agent_state: &AgentState, profile: &AgentProfile) -> f32 {
        // Time since formation in days
        let t_days = (now - self.timestamp).num_seconds() as f32 / 86_400.0;
        
        // Phase(a)
        let phase = 1.0 / (1.0 + 
            (profile.capacity_factor * (self.age_at_formation - profile.capacity_factor) as f64).exp() as f32
        ) + profile.interference_factor;
        
        // Decay(t)
        let beta = self.decay_params.beta_0 * 
            (1.0 + agent_state.stress + agent_state.fatigue);
        let decay = (1.0 + beta * t_days).powf(-self.decay_params.alpha);
        
        // Emotional bias
        let emo_bias = if self.emotion.abs() > profile.emotional_bias {
            1.0 + profile.emotional_bias * self.emotion.abs()
        } else {
            1.0 + profile.emotional_bias * self.emotion
        };
        
        // Capacity competition
        let c_max = agent_profile.c_base * 
            (1.0 - agent_state.fatigue + agent_state.training_factor);
        let cap_comp = (self.capacity_weight.min(c_max) / agent_profile.c_base).max(0.0);
        
        // Interference (simplified - would use ANN in production)
        // For now, we'll use a placeholder value
        let interference = 1.0;  // Would be exp(-kappa * s * t) in full implementation
        
        // Retention calculation
        let retention = phase * decay * emo_bias * cap_comp * interference * self.memory_strength;
        retention.max(0.0).min(1.0)
    }
}

/// Represents the current state of the agent
#[derive(Debug, Clone)]
pub struct AgentState {
    /// Current age of the agent in years
    pub current_age: f64,
    
    /// Current sleep debt (normalized 0.0-1.0)
    pub sleep_debt: f32,
    
    /// Current stress/cortisol level (normalized 0.0-1.0)
    pub cortisol_level: f32,
    
    /// Current fatigue level (normalized 0.0-1.0)
    pub fatigue: f32,
    
    /// Training/experience factor (normalized 0.0-1.0)
    pub training_factor: f32,
}

/// Agent-specific parameters that control memory formation and retention
#[derive(Debug, Clone)]
pub struct AgentProfile {
    /// Phase steepness (k)
    pub k: f64,
    
    /// Age for half-max plasticity (a_mid)
    pub a_mid: f64,
    
    /// Minimum phase (ε)
    pub epsilon: f32,
    
    /// Shock threshold (θ_shock)
    pub theta_shock: f32,
    
    /// Trauma boost (γ)
    pub gamma: f32,
    
    /// Normal emotional slope (η)
    pub eta: f32,
    
    /// Base capacity (C_base)
    pub c_base: f32,
    
    /// Retrieval-based strengthening factor (ρ)
    pub rho: f32,
    
    /// Interference constant (κ)
    pub kappa: f32,
}

impl Default for AgentProfile {
    fn default() -> Self {
        Self {
            k: 0.5,
            a_mid: 22.0,
            epsilon: 0.2,
            theta_shock: 0.7,
            gamma: 1.5,
            eta: 0.3,
            c_base: 100.0,
            rho: 0.1,
            kappa: 0.05,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use chrono::Duration;
    
    #[test]
    fn test_memory_creation() {
        let vector = vec![0.1, 0.2, 0.3];
        let memory = Memory::new(vector.clone(), 0.5, 25.0, 1.0);
        
        assert_eq!(memory.emotion, 0.5);
        assert_eq!(memory.semantic_vector, vector);
        assert_eq!(memory.memory_strength, 1.0);
        assert!(memory.recall_history.is_empty());
    }
    
    #[test]
    fn test_retrieval_recording() {
        let mut memory = Memory::new(vec![], 0.0, 25.0, 1.0);
        let rho = 0.1;
        
        memory.record_retrieval(rho);
        
        assert_relative_eq!(memory.memory_strength, 1.0 / 1.1, epsilon = 1e-6);
        assert_eq!(memory.recall_history.len(), 1);
    }
    
    #[test]
    fn test_retention_calculation() {
        let now = Utc::now();
        let mut memory = Memory::new(vec![0.1, 0.2, 0.3], 0.5, 25.0, 1.0);
        
        // Set a fixed timestamp for testing
        memory.timestamp = now - Duration::days(1);
        
        let agent_state = AgentState {
            current_age: 30.0,
            sleep_debt: 0.2,
            cortisol_level: 0.1,
            fatigue: 0.3,
            training_factor: 0.4,
        };
        
        let agent_profile = AgentProfile::default();
        
        let retention = memory.calculate_retention(now, &agent_state, &agent_profile);
        
        // Just verify it's in the expected range
        assert!(retention > 0.0 && retention <= 1.0);
    }
}
