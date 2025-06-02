# Architecture Documentation Todo List (Improvement Task 1.2)

## 1. Create/Update `ARCHITECTURE.md`
- [x] **Explain Core Concepts:**
    - [x] Define what a "memory" is in the context of this module.
    - [x] Describe the lifecycle of a memory item (creation, storage, retrieval, potential eviction).
    - [x] Explain key terms and their significance (e.g., relevance, retention, similarity).
    - [x] Outline the overall goals and non-goals of the memory module.
- [x] **Detail Components:**
    - [x] Describe the `MemoryStore` and its primary responsibilities.
    - [x] Detail the structure of a `Memory` item and its fields.
    - [x] Explain the role of `AgentProfile` and `AgentState` in relation to memory operations.
    - [x] Document any other key structs or enums involved in the core architecture.
- [x] **Illustrate Interactions:**
    - [x] Provide a high-level overview of how different components interact (e.g., how an external system might use `MemoryStore`).
    - [x] Briefly explain the primary data flow within the module.

## 2. Document Memory Retention Algorithm
- [x] **Explain the Algorithm:**
    - [x] Clearly describe the current memory retention/pruning algorithm used in the `maintain` function.
    - [x] Explain the logic behind calculating a memory's relevance or importance for retention.
    - [x] Detail how `retention_threshold` and other parameters influence the algorithm.
- [x] **Discuss Trade-offs:**
    - [x] Analyze the pros and cons of the current algorithm (e.g., simplicity vs. accuracy, performance characteristics).
    - [x] Discuss potential scenarios where the current algorithm might be suboptimal.
    - [x] Mention any alternative algorithms considered and why the current one was chosen (if applicable).
- [x] **Provide Examples (Optional but Recommended):**
    - [x] Illustrate with a small example how memories would be pruned given a specific set of memories and parameters.

## 3. Add Data Flow Diagrams
- [x] **Identify Key Processes:**
    - [x] Confirm the key processes are: memory insertion, memory retrieval (querying), and memory eviction/maintenance.
- [x] **Create Diagrams:**
    - [x] For memory insertion:
        - [x] Show data flow from API call to storage, including any transformations or calculations (e.g., embedding generation if it were part of this module).
    - [x] For memory retrieval:
        - [x] Show data flow from query input to ranked results, including similarity calculations and filtering.
    - [x] For memory eviction:
        - [x] Show how the `maintain` process identifies and removes memories.
- [x] **Choose a Format/Tool:**
    - [x] Decide on a tool or format for diagrams (e.g., Mermaid.js for embedding in Markdown, draw.io/Lucidchart for external images).
    - [x] Ensure diagrams are clear, well-annotated, and easy to understand.
- [x] **Integrate into `ARCHITECTURE.md`:**
    - [x] Embed or link these diagrams within the relevant sections of `ARCHITECTURE.md`.

## 4. Add Sequence Diagrams for Common API Interactions
- [x] **Identify Common API Interactions:**
    - [x] `MemoryStore::new()`
    - [x] `MemoryStore::insert_memory()`
    - [x] `MemoryStore::get_memory()`
    - [x] `MemoryStore::remove_memory()`
    - [x] `MemoryStore::find_relevant()`
    - [x] `MemoryStore::maintain()`
- [x] **Create Diagrams:**
    - [x] For each identified interaction, create a sequence diagram.
    - [x] Show the sequence of calls between the user/client, the `MemoryStore`, and any internal components or data structures involved.
    - [x] Clearly label messages and lifelines.
- [x] **Choose a Format/Tool:** (Same as for data flow diagrams)
    - [x] Ensure consistency in style with data flow diagrams.
- [x] **Integrate into `ARCHITECTURE.md` or API Docs:**
    - [x] Embed or link these diagrams within `ARCHITECTURE.md` or potentially within the API documentation for the respective functions if more appropriate. 
