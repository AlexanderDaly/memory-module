use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use memory_module::prelude::*;

fn bench_memory_store_insert(c: &mut Criterion) {
    let profile = AgentProfile::default();
    let state = AgentState::default();
    c.bench_function("memory_store_insert", |b| {
        b.iter_batched(
            || MemoryStore::new(profile.clone(), state.clone()),
            |mut store| {
                for _ in 0..1000 {
                    let mem = Memory::new(vec![0.1, 0.2, 0.3], 0.0, 0.0, 1.0);
                    store.add_memory(mem);
                }
            },
            BatchSize::SmallInput,
        )
    });
}

fn bench_memory_store_query(c: &mut Criterion) {
    let profile = AgentProfile::default();
    let state = AgentState::default();
    c.bench_function("memory_store_query", |b| {
        b.iter_batched(
            || {
                let mut store = MemoryStore::new(profile.clone(), state.clone());
                for _ in 0..1000 {
                    let mem = Memory::new(vec![0.1, 0.2, 0.3], 0.0, 0.0, 1.0);
                    store.add_memory(mem);
                }
                store
            },
            |mut store| {
                let _ = store.find_relevant(&[0.1, 0.2, 0.3], 10).unwrap();
            },
            BatchSize::SmallInput,
        )
    });
}

#[cfg(feature = "concurrent")]
fn bench_concurrent_store_insert(c: &mut Criterion) {
    let profile = AgentProfile::default();
    let state = AgentState::default();
    c.bench_function("concurrent_store_insert", |b| {
        b.iter_batched(
            || ConcurrentMemoryStore::new(profile.clone(), state.clone()),
            |store| {
                for _ in 0..1000 {
                    let mem = Memory::new(vec![0.1, 0.2, 0.3], 0.0, 0.0, 1.0);
                    store.add_memory(mem);
                }
            },
            BatchSize::SmallInput,
        )
    });
}

#[cfg(feature = "concurrent")]
fn bench_concurrent_store_query(c: &mut Criterion) {
    let profile = AgentProfile::default();
    let state = AgentState::default();
    c.bench_function("concurrent_store_query", |b| {
        b.iter_batched(
            || {
                let store = ConcurrentMemoryStore::new(profile.clone(), state.clone());
                for _ in 0..1000 {
                    let mem = Memory::new(vec![0.1, 0.2, 0.3], 0.0, 0.0, 1.0);
                    store.add_memory(mem);
                }
                store
            },
            |store| {
                let _ = store.find_relevant(&[0.1, 0.2, 0.3], 10).unwrap();
            },
            BatchSize::SmallInput,
        )
    });
}

#[cfg(feature = "concurrent")]
fn bench_sharded_store_insert(c: &mut Criterion) {
    let profile = AgentProfile::default();
    let state = AgentState::default();
    c.bench_function("sharded_store_insert", |b| {
        b.iter_batched(
            || ShardedMemoryStore::new(profile.clone(), state.clone(), 4),
            |store| {
                for _ in 0..1000 {
                    let mem = Memory::new(vec![0.1, 0.2, 0.3], 0.0, 0.0, 1.0);
                    store.add_memory(mem);
                }
            },
            BatchSize::SmallInput,
        )
    });
}

#[cfg(feature = "concurrent")]
fn bench_sharded_store_query(c: &mut Criterion) {
    let profile = AgentProfile::default();
    let state = AgentState::default();
    c.bench_function("sharded_store_query", |b| {
        b.iter_batched(
            || {
                let store = ShardedMemoryStore::new(profile.clone(), state.clone(), 4);
                for _ in 0..1000 {
                    let mem = Memory::new(vec![0.1, 0.2, 0.3], 0.0, 0.0, 1.0);
                    store.add_memory(mem);
                }
                store
            },
            |store| {
                let _ = store.find_relevant(&[0.1, 0.2, 0.3], 10).unwrap();
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(basic_benches, bench_memory_store_insert, bench_memory_store_query);
#[cfg(feature = "concurrent")]
criterion_group!(concurrent_benches, bench_concurrent_store_insert, bench_concurrent_store_query, bench_sharded_store_insert, bench_sharded_store_query);

#[cfg(feature = "concurrent")]
criterion_main!(basic_benches, concurrent_benches);
#[cfg(not(feature = "concurrent"))]
criterion_main!(basic_benches);
