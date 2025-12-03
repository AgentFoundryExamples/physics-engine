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
//! Benchmarks for memory pooling performance
//!
//! Measures the impact of memory pooling on allocation churn and frame times.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use physics_engine::ecs::{Entity, HashMapStorage, ComponentStorage, World};
use physics_engine::ecs::components::{Position, Velocity, Acceleration, Mass};
use physics_engine::ecs::systems::{ForceRegistry, ForceProvider, Force};
use physics_engine::integration::{RK4Integrator, Integrator};
use physics_engine::pool::PoolConfig;

// Simple constant force for benchmarking
struct ConstantForce {
    force: Force,
}

impl ForceProvider for ConstantForce {
    fn compute_force(&self, _entity: Entity, _registry: &ForceRegistry) -> Option<Force> {
        Some(self.force)
    }

    fn name(&self) -> &str {
        "ConstantForce"
    }
}

fn setup_simulation(n_entities: usize) -> (
    Vec<Entity>,
    HashMapStorage<Position>,
    HashMapStorage<Velocity>,
    HashMapStorage<Acceleration>,
    HashMapStorage<Mass>,
    ForceRegistry,
) {
    let mut world = World::with_capacity(n_entities);
    let mut positions = HashMapStorage::new();
    let mut velocities = HashMapStorage::new();
    let mut accelerations = HashMapStorage::new();
    let mut masses = HashMapStorage::new();

    let entities: Vec<Entity> = (0..n_entities)
        .map(|i| {
            let entity = world.create_entity();
            positions.insert(entity, Position::new(i as f64, 0.0, 0.0));
            velocities.insert(entity, Velocity::new(1.0, 0.0, 0.0));
            accelerations.insert(entity, Acceleration::new(0.1, 0.0, 0.0));
            masses.insert(entity, Mass::new(1.0));
            entity
        })
        .collect();

    let mut force_registry = ForceRegistry::new();
    force_registry.register_provider(Box::new(ConstantForce {
        force: Force::new(1.0, 0.0, 0.0),
    }));

    (entities, positions, velocities, accelerations, masses, force_registry)
}

fn bench_rk4_default_pools(c: &mut Criterion) {
    let mut group = c.benchmark_group("rk4_pooling");
    
    for n_entities in [10, 100, 1000].iter() {
        let (entities, mut positions, mut velocities, accelerations, masses, mut force_registry) = 
            setup_simulation(*n_entities);
        
        group.bench_with_input(
            BenchmarkId::new("default_config", n_entities),
            n_entities,
            |b, _| {
                let mut integrator = RK4Integrator::new(0.01);
                b.iter(|| {
                    integrator.integrate(
                        black_box(entities.iter()),
                        &mut positions,
                        &mut velocities,
                        &accelerations,
                        &masses,
                        &mut force_registry,
                        false,
                    )
                });
            },
        );
    }
    
    group.finish();
}

fn bench_rk4_custom_pools(c: &mut Criterion) {
    let mut group = c.benchmark_group("rk4_custom_pools");
    
    for n_entities in [10, 100, 1000].iter() {
        let (entities, mut positions, mut velocities, accelerations, masses, mut force_registry) = 
            setup_simulation(*n_entities);
        
        group.bench_with_input(
            BenchmarkId::new("large_capacity", n_entities),
            n_entities,
            |b, _| {
                // Larger initial capacity to avoid early resizes
                let pool_config = PoolConfig::new(256, 16);
                let mut integrator = RK4Integrator::with_pool_config(0.01, pool_config);
                b.iter(|| {
                    integrator.integrate(
                        black_box(entities.iter()),
                        &mut positions,
                        &mut velocities,
                        &accelerations,
                        &masses,
                        &mut force_registry,
                        false,
                    )
                });
            },
        );
    }
    
    group.finish();
}

fn bench_world_preallocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("world_preallocation");
    
    for n_entities in [100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("no_prealloc", n_entities),
            n_entities,
            |b, &n| {
                b.iter(|| {
                    let mut world = World::new();
                    for _ in 0..n {
                        black_box(world.create_entity());
                    }
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("with_prealloc", n_entities),
            n_entities,
            |b, &n| {
                b.iter(|| {
                    let mut world = World::with_capacity(n);
                    for _ in 0..n {
                        black_box(world.create_entity());
                    }
                });
            },
        );
    }
    
    group.finish();
}

fn bench_pool_stats_overhead(c: &mut Criterion) {
    let (entities, mut positions, mut velocities, accelerations, masses, mut force_registry) = 
        setup_simulation(100);
    
    c.bench_function("rk4_with_stats_check", |b| {
        let mut integrator = RK4Integrator::new(0.01);
        b.iter(|| {
            integrator.integrate(
                entities.iter(),
                &mut positions,
                &mut velocities,
                &accelerations,
                &masses,
                &mut force_registry,
                false,
            );
            // Check stats after integration
            black_box(integrator.pool_stats());
        });
    });
}

criterion_group!(
    benches,
    bench_rk4_default_pools,
    bench_rk4_custom_pools,
    bench_world_preallocation,
    bench_pool_stats_overhead,
);
criterion_main!(benches);
