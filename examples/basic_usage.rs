//! Basic example of using the memory module

use memory_module::prelude::*;
use memory_module::chrono::{Duration, Utc};
use memory_module::error::Result;

fn main() -> Result<()> {
    // Set up logging
    env_logger::init();
    
    println!("=== Memory Module Example ===\n");
    
    // Create an agent profile with default parameters
    let profile = AgentProfile::default();
    
    // Create an initial agent state
    let state = AgentState {
        current_age: 25.0,
        sleep_debt: 0.2,
        cortisol_level: 0.1,
        fatigue: 0.3,
        training_factor: 0.4,
    };
    
    // Create a new memory store
    let mut store = MemoryStore::new(profile, state);
    
    // Create some memories with different emotional valences
    let memories = vec![
        ("neutral_event", vec![0.1, 0.2, 0.3], 0.1),
        ("happy_event", vec![0.4, 0.5, 0.6], 0.8),
        ("sad_event", vec![0.7, 0.8, 0.9], -0.6),
        ("traumatic_event", vec![0.9, 0.1, 0.2], -0.9),
    ];
    
    // Add memories to the store
    for (i, (description, vector, emotion)) in memories.into_iter().enumerate() {
        let mut memory = Memory::new(
            vector.clone(),
            emotion as f32,
            25.0 - (i as f64 * 0.5), // Vary age at formation
            1.0,
        );
        
        // Add some metadata
        memory.metadata = serde_json::json!({
            "description": description,
            "source": "example"
        });
        
        // Make the traumatic event older to test retention
        if description == "traumatic_event" {
            memory.timestamp = Utc::now() - Duration::days(30);
        }
        
        store.add_memory(memory);
    }
    
    // Simulate some time passing
    println!("Simulating time passing...\n");
    
    // Update agent state (e.g., more tired, more experienced)
    let new_state = AgentState {
        current_age: 25.1, // 1.2 months later
        sleep_debt: 0.3,
        cortisol_level: 0.2,
        fatigue: 0.4,
        training_factor: 0.5,
    };
    
    store.update_agent_state(new_state);
    
    // Search for relevant memories
    let query = vec![0.5, 0.6, 0.7];
    println!("Searching for memories similar to {:?}", query);
    
    let relevant = store.find_relevant(&query, 3)?;
    
    println!("\nTop relevant memories:");
    for (score, memory) in relevant {
        let description = memory.metadata.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("<no description>");
            
        let retention = memory.calculate_retention(
            Utc::now(),
            store.agent_state(),
            store.agent_profile(),
        );
        
        println!(
            "- {} (score: {:.3}, retention: {:.3})",
            description, score, retention
        );
    }
    
    // Run maintenance to prune weak memories
    println!("\nRunning maintenance...");
    let pruned = store.maintain(0.1);
    println!("Pruned {} weak memories", pruned);
    
    Ok(())
}
