use memory_module::prelude::*;
use memory_module::storage::FileBackend;
use std::fs;

#[cfg(feature = "serde")]
#[test]
fn test_file_backend_roundtrip() {
    let profile = AgentProfile::default();
    let state = AgentState::default();
    let mut store = MemoryStore::new(profile, state);
    let memory = Memory::new(vec![0.1, 0.2], 0.0, 0.0, 1.0);
    let id = memory.id;
    store.add_memory(memory);

    let path = std::env::temp_dir().join(format!("mm_test_{}.json", uuid::Uuid::new_v4()));
    let backend = FileBackend::new(&path);

    store.save(&backend).expect("save");

    let loaded = MemoryStore::load(&backend).expect("load");
    assert!(loaded.get_memory(&id).is_some());

    fs::remove_file(&path).expect("cleanup");
}
