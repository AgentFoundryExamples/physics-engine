// Copyright 2025 John Brosnihan
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//! Benchmarks comparing HashMap vs SoA storage performance
//!
//! These benchmarks measure:
//! - Memory access patterns and cache utilization
//! - Insert/remove/get performance
//! - Bulk iteration throughput
//! - Memory footprint differences

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use physics_engine::ecs::components::Position;
use physics_engine::ecs::{Entity, HashMapStorage, SoAStorage, ComponentStorage};

/// Benchmark: Insert N entities into storage
fn bench_storage_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_insert");
    
    for entity_count in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*entity_count as u64));
        
        // HashMap storage
        group.bench_with_input(
            BenchmarkId::new("HashMap", entity_count),
            entity_count,
            |b, &count| {
                b.iter(|| {
                    let mut storage = HashMapStorage::<Position>::new();
                    for i in 0..count {
                        let entity = Entity::new(i as u64, 0);
                        storage.insert(entity, Position::new(i as f64, i as f64 * 2.0, i as f64 * 3.0));
                    }
                    black_box(storage);
                });
            },
        );
        
        // SoA storage
        group.bench_with_input(
            BenchmarkId::new("SoA", entity_count),
            entity_count,
            |b, &count| {
                b.iter(|| {
                    let mut storage = SoAStorage::<Position>::new();
                    for i in 0..count {
                        let entity = Entity::new(i as u64, 0);
                        storage.insert(entity, Position::new(i as f64, i as f64 * 2.0, i as f64 * 3.0));
                    }
                    black_box(storage);
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark: Random access (get) performance
fn bench_storage_random_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_random_access");
    
    for entity_count in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*entity_count as u64));
        
        // HashMap storage
        group.bench_with_input(
            BenchmarkId::new("HashMap", entity_count),
            entity_count,
            |b, &count| {
                b.iter_batched(
                    || {
                        let mut storage = HashMapStorage::<Position>::new();
                        for i in 0..count {
                            let entity = Entity::new(i as u64, 0);
                            storage.insert(entity, Position::new(i as f64, i as f64 * 2.0, i as f64 * 3.0));
                        }
                        storage
                    },
                    |storage| {
                        let mut sum = 0.0;
                        for i in 0..count {
                            let entity = Entity::new(i as u64, 0);
                            if let Some(pos) = storage.get(entity) {
                                sum += pos.x() + pos.y() + pos.z();
                            }
                        }
                        black_box(sum);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
        
        // SoA storage
        group.bench_with_input(
            BenchmarkId::new("SoA", entity_count),
            entity_count,
            |b, &count| {
                b.iter_batched(
                    || {
                        let mut storage = SoAStorage::<Position>::new();
                        for i in 0..count {
                            let entity = Entity::new(i as u64, 0);
                            storage.insert(entity, Position::new(i as f64, i as f64 * 2.0, i as f64 * 3.0));
                        }
                        storage
                    },
                    |storage| {
                        let mut sum = 0.0;
                        for i in 0..count {
                            let entity = Entity::new(i as u64, 0);
                            if let Some(pos) = storage.get(entity) {
                                sum += pos.x() + pos.y() + pos.z();
                            }
                        }
                        black_box(sum);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    
    group.finish();
}

/// Benchmark: Sequential iteration over all components
fn bench_storage_sequential_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_sequential_iteration");
    
    for entity_count in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*entity_count as u64));
        
        // HashMap storage - iterate via entities (typical usage pattern)
        group.bench_with_input(
            BenchmarkId::new("HashMap_via_entities", entity_count),
            entity_count,
            |b, &count| {
                b.iter_batched(
                    || {
                        let mut storage = HashMapStorage::<Position>::new();
                        let mut entities = Vec::new();
                        for i in 0..count {
                            let entity = Entity::new(i as u64, 0);
                            entities.push(entity);
                            storage.insert(entity, Position::new(i as f64, i as f64 * 2.0, i as f64 * 3.0));
                        }
                        (storage, entities)
                    },
                    |(storage, entities)| {
                        let mut sum = 0.0;
                        for entity in &entities {
                            if let Some(pos) = storage.get(*entity) {
                                sum += pos.x() + pos.y() + pos.z();
                            }
                        }
                        black_box(sum);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
        
        // SoA storage - iterate via entities (fair comparison)
        group.bench_with_input(
            BenchmarkId::new("SoA_via_entities", entity_count),
            entity_count,
            |b, &count| {
                b.iter_batched(
                    || {
                        let mut storage = SoAStorage::<Position>::new();
                        let mut entities = Vec::new();
                        for i in 0..count {
                            let entity = Entity::new(i as u64, 0);
                            entities.push(entity);
                            storage.insert(entity, Position::new(i as f64, i as f64 * 2.0, i as f64 * 3.0));
                        }
                        (storage, entities)
                    },
                    |(storage, entities)| {
                        let mut sum = 0.0;
                        for entity in &entities {
                            if let Some(pos) = storage.get(*entity) {
                                sum += pos.x() + pos.y() + pos.z();
                            }
                        }
                        black_box(sum);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
        
        // SoA storage - direct array iteration (demonstrates SoA advantage)
        group.bench_with_input(
            BenchmarkId::new("SoA_direct_array", entity_count),
            entity_count,
            |b, &count| {
                b.iter_batched(
                    || {
                        let mut storage = SoAStorage::<Position>::new();
                        for i in 0..count {
                            let entity = Entity::new(i as u64, 0);
                            storage.insert(entity, Position::new(i as f64, i as f64 * 2.0, i as f64 * 3.0));
                        }
                        storage
                    },
                    |storage| {
                        let mut sum = 0.0;
                        for pos in storage.components() {
                            sum += pos.x() + pos.y() + pos.z();
                        }
                        black_box(sum);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    
    group.finish();
}

/// Benchmark: Update all components (simulating a system update)
fn bench_storage_bulk_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_bulk_update");
    
    for entity_count in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*entity_count as u64));
        
        // HashMap storage
        group.bench_with_input(
            BenchmarkId::new("HashMap", entity_count),
            entity_count,
            |b, &count| {
                // Setup inside iter_batched to avoid measuring setup time
                b.iter_batched(
                    || {
                        let mut storage = HashMapStorage::<Position>::new();
                        let mut entities = Vec::new();
                        for i in 0..count {
                            let entity = Entity::new(i as u64, 0);
                            entities.push(entity);
                            storage.insert(entity, Position::new(i as f64, i as f64 * 2.0, i as f64 * 3.0));
                        }
                        (storage, entities)
                    },
                    |(mut storage, entities)| {
                        for entity in &entities {
                            if let Some(pos) = storage.get_mut(*entity) {
                                pos.set_x(pos.x() + 1.0);
                                pos.set_y(pos.y() + 1.0);
                                pos.set_z(pos.z() + 1.0);
                            }
                        }
                        black_box(storage);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
        
        // SoA storage
        group.bench_with_input(
            BenchmarkId::new("SoA", entity_count),
            entity_count,
            |b, &count| {
                b.iter_batched(
                    || {
                        let mut storage = SoAStorage::<Position>::new();
                        let mut entities = Vec::new();
                        for i in 0..count {
                            let entity = Entity::new(i as u64, 0);
                            entities.push(entity);
                            storage.insert(entity, Position::new(i as f64, i as f64 * 2.0, i as f64 * 3.0));
                        }
                        (storage, entities)
                    },
                    |(mut storage, entities)| {
                        for entity in &entities {
                            if let Some(pos) = storage.get_mut(*entity) {
                                pos.set_x(pos.x() + 1.0);
                                pos.set_y(pos.y() + 1.0);
                                pos.set_z(pos.z() + 1.0);
                            }
                        }
                        black_box(storage);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    
    group.finish();
}

/// Benchmark: Remove entities
fn bench_storage_remove(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_remove");
    
    for entity_count in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*entity_count as u64));
        
        // HashMap storage
        group.bench_with_input(
            BenchmarkId::new("HashMap", entity_count),
            entity_count,
            |b, &count| {
                b.iter_batched(
                    || {
                        let mut storage = HashMapStorage::<Position>::new();
                        let mut entities = Vec::new();
                        for i in 0..count {
                            let entity = Entity::new(i as u64, 0);
                            entities.push(entity);
                            storage.insert(entity, Position::new(i as f64, i as f64 * 2.0, i as f64 * 3.0));
                        }
                        (storage, entities)
                    },
                    |(mut storage, entities)| {
                        for entity in entities {
                            storage.remove(entity);
                        }
                        black_box(storage);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
        
        // SoA storage
        group.bench_with_input(
            BenchmarkId::new("SoA", entity_count),
            entity_count,
            |b, &count| {
                b.iter_batched(
                    || {
                        let mut storage = SoAStorage::<Position>::new();
                        let mut entities = Vec::new();
                        for i in 0..count {
                            let entity = Entity::new(i as u64, 0);
                            entities.push(entity);
                            storage.insert(entity, Position::new(i as f64, i as f64 * 2.0, i as f64 * 3.0));
                        }
                        (storage, entities)
                    },
                    |(mut storage, entities)| {
                        for entity in entities {
                            storage.remove(entity);
                        }
                        black_box(storage);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    storage_benches,
    bench_storage_insert,
    bench_storage_random_access,
    bench_storage_sequential_iteration,
    bench_storage_bulk_update,
    bench_storage_remove
);
criterion_main!(storage_benches);
