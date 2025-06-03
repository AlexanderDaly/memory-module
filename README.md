# Memory Module

A Rust library implementing a biologically inspired memory model for AI agents, based on cognitive science principles of memory formation, retention, and retrieval.

## Features

- **Biologically Plausible**: Models memory based on established psychological and neuroscientific principles
- **Configurable**: Tune parameters to model different memory profiles
- **Efficient**: Designed for real-time use in games and interactive applications
- **Extensible**: Easy to integrate with different storage backends and AI systems
- **Persistent**: Includes a file-based backend and trait for custom storage implementations

## Core Concepts

### Memory Formation
- **Phase-Dependent Plasticity**: Memory formation varies with the agent's age
- **Emotional Modulation**: Emotional events form stronger memories
- **Capacity Limits**: Working memory constraints affect memory formation

### Memory Retention
- **Time-Based Decay**: Memories fade over time according to a power law
- **Interference**: Similar memories compete with and weaken each other
- **Emotional Bias**: Emotional events are remembered better

### Memory Retrieval
- **Content-Addressable**: Memories are retrieved based on similarity to current context
- **Strength-Dependent**: Stronger memories are more likely to be retrieved
- **Reconsolidation**: Retrieved memories are strengthened and updated

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
memory-module = { path = "./memory-module", features = ["serde"] }
```

### Example

```rust
use memory_module::prelude::*;

// Create an agent profile and state
let profile = AgentProfile::default();
let state = AgentState {
    current_age: 25.0,
    sleep_debt: 0.2,
    cortisol_level: 0.1,
    fatigue: 0.3,
    training_factor: 0.4,
};

// Create a memory store
let mut store = MemoryStore::new(profile, state);

// Add a memory
let memory = Memory::new(
    vec![0.1, 0.2, 0.3], // Semantic vector
    0.5,                // Emotion (-1.0 to 1.0)
    25.0,              // Age at formation
    1.0,               // Capacity weight
);
store.add_memory(memory);

// Retrieve relevant memories
let query = vec![0.15, 0.25, 0.35];
let relevant = store.find_relevant(&query, 5).unwrap();
```

## Running Examples

```bash
# Run the basic usage example
cargo run --example basic_usage --features="serde"
```

## Documentation

Detailed documentation is available via `cargo doc --open`.
For a discussion of approximate indexing options, see
[docs/indexing_comparison.md](docs/indexing_comparison.md).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
