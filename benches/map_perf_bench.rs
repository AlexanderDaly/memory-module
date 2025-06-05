use criterion::{criterion_group, criterion_main, Criterion};
use uuid::Uuid;
use std::collections::HashMap;

fn bench_hashmap_insert(c: &mut Criterion) {
    c.bench_function("hashmap_insert", |b| {
        b.iter(|| {
            let mut map: HashMap<Uuid, u32> = HashMap::new();
            for _ in 0..1000 {
                map.insert(Uuid::new_v4(), 42);
            }
        });
    });
}

fn bench_hashmap_lookup(c: &mut Criterion) {
    c.bench_function("hashmap_lookup", |b| {
        b.iter(|| {
            let mut map: HashMap<Uuid, u32> = HashMap::new();
            let keys: Vec<_> = (0..1000).map(|_| Uuid::new_v4()).collect();
            for k in &keys {
                map.insert(*k, 1);
            }
            for k in &keys {
                let _ = map.get(k);
            }
        });
    });
}

#[cfg(feature = "concurrent")]
use dashmap::DashMap;

#[cfg(feature = "concurrent")]
fn bench_dashmap_insert(c: &mut Criterion) {
    c.bench_function("dashmap_insert", |b| {
        b.iter(|| {
            let map: DashMap<Uuid, u32> = DashMap::new();
            for _ in 0..1000 {
                map.insert(Uuid::new_v4(), 42);
            }
        });
    });
}

#[cfg(feature = "concurrent")]
fn bench_dashmap_lookup(c: &mut Criterion) {
    c.bench_function("dashmap_lookup", |b| {
        b.iter(|| {
            let map: DashMap<Uuid, u32> = DashMap::new();
            let keys: Vec<_> = (0..1000).map(|_| Uuid::new_v4()).collect();
            for k in &keys {
                map.insert(*k, 1);
            }
            for k in &keys {
                let _ = map.get(k);
            }
        });
    });
}

criterion_group!(hashmap_benches, bench_hashmap_insert, bench_hashmap_lookup);
#[cfg(feature = "concurrent")]
criterion_group!(dashmap_benches, bench_dashmap_insert, bench_dashmap_lookup);

#[cfg(feature = "concurrent")]
criterion_main!(hashmap_benches, dashmap_benches);
#[cfg(not(feature = "concurrent"))]
criterion_main!(hashmap_benches);
