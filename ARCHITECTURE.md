# Memory Module Architecture

This document provides a high level overview of the main components in the
`memory-module` crate and how they interact with each other. The goal is to help
new contributors quickly understand the core design.

## Components

### AgentProfile
Configuration parameters that define how an agent forms and retains memories.
Fields such as `decay_rate`, `emotional_bias` and `capacity_factor` determine
how quickly memories fade and how emotions influence retention.

### AgentState
Represents the agent's current cognitive and physical state. Values like
`stress`, `fatigue` and `focus` influence the retention algorithm at runtime.

### Memory
The fundamental unit stored in the system. Each `Memory` contains a semantic
vector, emotional valence and metadata such as timestamps and recall history.
It exposes methods to calculate the current retention strength and to record
retrieval events.

### MemoryStore
The central structure that manages all memories. Internally it holds a `HashMap`
of memories indexed by `Uuid`. The store provides operations to add, query and
remove memories. Retrieval uses cosine similarity combined with each memory's
retention score to rank results. Maintenance operations prune memories whose
retention falls below a configurable threshold.

## Data Flow
1. **Insertion** – A `Memory` is created and added to `MemoryStore` via
   `add_memory`. The store assigns it an identifier and stores it for later
   retrieval.
2. **Querying** – `find_relevant` computes relevance scores for each memory using
   cosine similarity and the current retention strength. Top results are
   returned and their retrieval history is updated.
3. **Maintenance** – `maintain` periodically evaluates all memories and removes
   those whose retention strength is below the provided threshold.

The `AgentProfile` and `AgentState` influence the retention calculations during
both querying and maintenance, allowing the store to simulate realistic memory
behaviour.

## Decision Log
- _2024-05-13_: Chose simple HashMap-based store for initial implementation to optimize for clarity over performance.
