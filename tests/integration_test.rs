use memory_module::prelude::*;
use memory_module::chrono::{Duration, Utc};

#[test]
fn test_memory_lifecycle() {
    // Create an agent profile with default parameters
    let profile = AgentProfile::default();
    
    // Create an initial agent state
    let state = AgentState {
        current_age: 25.0,
        sleep_debt: 0.0,
        cortisol_level: 0.0,
        fatigue: 0.0,
        training_factor: 0.0,
    };
    
    // Create a new memory store
    let mut store = MemoryStore::new(profile, state);
    
    // Create a test memory with current timestamp
    let mut memory = Memory::new(
        vec![0.1, 0.2, 0.3],
        0.5,
        25.0,
        1.0,
    );
    memory.timestamp = Utc::now();
    
    // Add the memory to the store
    let memory_id = memory.id;
    store.add_memory(memory);
    
    // Verify the memory was added
    assert!(store.get_memory(&memory_id).is_some());
    
    // Search for the memory
    let results = store.find_relevant(&[0.1, 0.2, 0.3], 1).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1.id, memory_id);
    
    // Verify retrieval updated the memory
    let memory = store.get_memory(&memory_id).unwrap();
    assert_eq!(memory.recall_history.len(), 1);
    assert!(memory.memory_strength < 1.0);
    
    // Run maintenance with a very low threshold to avoid pruning
    // The retention score is very small due to the decay calculation
    let threshold = 0.0001;
    let pruned = store.maintain(threshold);
    
    // The memory should not be pruned with the low threshold
    assert!(store.get_memory(&memory_id).is_some(), "Memory was pruned but shouldn't have been!");
    assert_eq!(pruned, 0, "Expected no memories to be pruned with threshold {}", threshold);
    
    // Remove the memory
    store.remove_memory(&memory_id).unwrap();
    assert!(store.get_memory(&memory_id).is_none());
}

#[test]
fn test_retention_calculation() {
    let now = Utc::now();
    let mut memory = Memory::new(
        vec![0.1, 0.2, 0.3],
        0.5,
        25.0,
        1.0,
    );
    
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
    
    // Calculate retention
    let retention = memory.calculate_retention(now, &agent_state, &agent_profile);
    
    // Should be in the expected range
    assert!(retention > 0.0 && retention <= 1.0);
    
    // Emotional memories should have higher retention
    let mut emotional_memory = memory.clone();
    emotional_memory.emotion = 0.9; // Strong positive emotion
    let emotional_retention = emotional_memory.calculate_retention(now, &agent_state, &agent_profile);
    assert!(emotional_retention > retention);
}

#[test]
fn test_find_relevant_batch() {
    let profile = AgentProfile::default();
    let state = AgentState {
        current_age: 25.0,
        sleep_debt: 0.0,
        cortisol_level: 0.0,
        fatigue: 0.0,
        training_factor: 0.0,
    };

    let mut store = MemoryStore::new(profile, state);
    store.add_memory(Memory::new(vec![0.1, 0.2, 0.3], 0.0, 25.0, 1.0));
    store.add_memory(Memory::new(vec![0.2, 0.3, 0.4], 0.0, 25.0, 1.0));

    let queries = vec![vec![0.1, 0.2, 0.3], vec![0.2, 0.3, 0.4]];
    let results = store.find_relevant_batch(&queries, 1).unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].len(), 1);
    assert_eq!(results[1].len(), 1);
}

#[cfg(feature = "concurrent")]
#[test]
fn test_concurrent_store_basic() {
    let profile = AgentProfile::default();
    let state = AgentState {
        current_age: 25.0,
        sleep_debt: 0.0,
        cortisol_level: 0.0,
        fatigue: 0.0,
        training_factor: 0.0,
    };

    let store = ConcurrentMemoryStore::new(profile, state);
    let memory = Memory::new(vec![0.0, 1.0], 0.0, 25.0, 1.0);
    let id = memory.id;
    store.add_memory(memory);
    let retrieved = store.get_memory(&id).expect("memory missing");
    assert_eq!(retrieved.id, id);
}

#[cfg(feature = "concurrent")]
#[test]
fn test_sharded_store_basic() {
    let profile = AgentProfile::default();
    let state = AgentState {
        current_age: 25.0,
        sleep_debt: 0.0,
        cortisol_level: 0.0,
        fatigue: 0.0,
        training_factor: 0.0,
    };

    let store = ShardedMemoryStore::new(profile, state, 4);
    let memory = Memory::new(vec![1.0, 0.0], 0.0, 25.0, 1.0);
    let id = memory.id;
    store.add_memory(memory);
    let retrieved = store.get_memory(&id).expect("missing");
    assert_eq!(retrieved.id, id);
}
