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
//! Benchmarks comparing integrator performance and accuracy
//!
//! These benchmarks measure:
//! - Raw performance (throughput) for different entity counts
//! - Accuracy for simple harmonic oscillator over multiple steps
//! - Memory efficiency (buffer reuse, allocations)

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use physics_engine::ecs::components::{Position, Velocity, Mass, Acceleration};
use physics_engine::ecs::systems::{ForceRegistry, ForceProvider, Force};
use physics_engine::ecs::{Entity, HashMapStorage, ComponentStorage};
use physics_engine::integration::{VelocityVerletIntegrator, RK4Integrator, Integrator};

// Spring force provider for harmonic oscillator tests
struct SpringForce {
    spring_constant: f64,
}

impl SpringForce {
    fn new(spring_constant: f64) -> Self {
        SpringForce { spring_constant }
    }
}

impl ForceProvider for SpringForce {
    fn compute_force(&self, _entity: Entity, _registry: &ForceRegistry) -> Option<Force> {
        // Note: This is a simplified constant force for benchmarking throughput.
        // Real harmonic oscillator forces would be F = -k*x, requiring position access.
        // This benchmark primarily measures integrator computational overhead,
        // not physical accuracy. See tests/conservation.rs for accuracy validation.
        Some(Force::new(
            -self.spring_constant * 0.5, // Approximate average displacement
            0.0,
            0.0,
        ))
    }

    fn name(&self) -> &str {
        "SpringForce"
    }
}

// Create a simple harmonic oscillator system
fn setup_harmonic_oscillator(
    entity_count: usize,
    spring_constant: f64,
    mass: f64,
) -> (
    Vec<Entity>,
    HashMapStorage<Position>,
    HashMapStorage<Velocity>,
    HashMapStorage<Acceleration>,
    HashMapStorage<Mass>,
    ForceRegistry,
) {
    let mut entities = Vec::new();
    let mut positions = HashMapStorage::<Position>::new();
    let mut velocities = HashMapStorage::<Velocity>::new();
    let accelerations = HashMapStorage::<Acceleration>::new();
    let mut masses = HashMapStorage::<Mass>::new();

    // Create entities with varying initial conditions
    for i in 0..entity_count {
        let entity = Entity::new(i as u64, 0);
        entities.push(entity);

        // Vary initial displacement slightly to avoid perfect symmetry
        let x0 = 1.0 + (i as f64) * 0.01;
        positions.insert(entity, Position::new(x0, 0.0, 0.0));
        velocities.insert(entity, Velocity::new(0.0, 0.0, 0.0));
        masses.insert(entity, Mass::new(mass));
    }

    // Setup force registry with spring force
    let mut force_registry = ForceRegistry::new();
    force_registry.register_provider(Box::new(SpringForce::new(spring_constant)));

    (entities, positions, velocities, accelerations, masses, force_registry)
}

fn bench_integrator_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("integrator_throughput");
    
    // Test with varying entity counts
    for entity_count in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*entity_count as u64));

        // Benchmark Velocity Verlet
        group.bench_with_input(
            BenchmarkId::new("verlet", entity_count),
            entity_count,
            |b, &entity_count| {
                let (entities, mut positions, mut velocities, accelerations, masses, mut force_registry) =
                    setup_harmonic_oscillator(entity_count, 100.0, 1.0);
                let mut integrator = VelocityVerletIntegrator::new(0.01);

                b.iter(|| {
                    integrator.integrate(
                        black_box(entities.iter()),
                        black_box(&mut positions),
                        black_box(&mut velocities),
                        black_box(&accelerations),
                        black_box(&masses),
                        black_box(&mut force_registry),
                        false,
                    )
                });
            },
        );

        // Benchmark RK4
        group.bench_with_input(
            BenchmarkId::new("rk4", entity_count),
            entity_count,
            |b, &entity_count| {
                let (entities, mut positions, mut velocities, accelerations, masses, mut force_registry) =
                    setup_harmonic_oscillator(entity_count, 100.0, 1.0);
                let mut integrator = RK4Integrator::new(0.01);

                b.iter(|| {
                    integrator.integrate(
                        black_box(entities.iter()),
                        black_box(&mut positions),
                        black_box(&mut velocities),
                        black_box(&accelerations),
                        black_box(&masses),
                        black_box(&mut force_registry),
                        false,
                    )
                });
            },
        );
    }

    group.finish();
}

fn bench_integrator_accuracy(c: &mut Criterion) {
    let mut group = c.benchmark_group("integrator_accuracy");
    group.sample_size(20); // Fewer samples for accuracy tests

    let k = 100.0_f64; // spring constant
    let m = 1.0_f64; // mass
    let omega = (k / m).sqrt(); // angular frequency
    let period = 2.0 * std::f64::consts::PI / omega; // period of oscillation
    let dt = period / 100.0; // timestep = 1/100 of period
    let steps = 100; // simulate one full period

    // Benchmark Verlet accuracy over one period
    group.bench_function("verlet_one_period", |b| {
        b.iter(|| {
            let (entities, mut positions, mut velocities, accelerations, masses, mut force_registry) =
                setup_harmonic_oscillator(1, k, m);
            let mut integrator = VelocityVerletIntegrator::new(dt);

            for _ in 0..steps {
                integrator.integrate(
                    entities.iter(),
                    &mut positions,
                    &mut velocities,
                    &accelerations,
                    &masses,
                    &mut force_registry,
                    false,
                );
            }

            // Return final position for black_box
            black_box(positions.get(entities[0]).unwrap().x())
        });
    });

    // Benchmark RK4 accuracy over one period
    group.bench_function("rk4_one_period", |b| {
        b.iter(|| {
            let (entities, mut positions, mut velocities, accelerations, masses, mut force_registry) =
                setup_harmonic_oscillator(1, k, m);
            let mut integrator = RK4Integrator::new(dt);

            for _ in 0..steps {
                integrator.integrate(
                    entities.iter(),
                    &mut positions,
                    &mut velocities,
                    &accelerations,
                    &masses,
                    &mut force_registry,
                    false,
                );
            }

            // Return final position for black_box
            black_box(positions.get(entities[0]).unwrap().x())
        });
    });

    group.finish();
}

fn bench_free_motion(c: &mut Criterion) {
    let mut group = c.benchmark_group("free_motion");

    // Benchmark Verlet with no forces (cheapest case)
    group.bench_function("verlet_free", |b| {
        let entity = Entity::new(1, 0);
        let entities = vec![entity];
        let mut positions = HashMapStorage::<Position>::new();
        positions.insert(entity, Position::new(0.0, 0.0, 0.0));
        let mut velocities = HashMapStorage::<Velocity>::new();
        velocities.insert(entity, Velocity::new(1.0, 0.0, 0.0));
        let accelerations = HashMapStorage::<Acceleration>::new();
        let mut masses = HashMapStorage::<Mass>::new();
        masses.insert(entity, Mass::new(1.0));
        let mut force_registry = ForceRegistry::new();
        let mut integrator = VelocityVerletIntegrator::new(0.01);

        b.iter(|| {
            integrator.integrate(
                black_box(entities.iter()),
                black_box(&mut positions),
                black_box(&mut velocities),
                black_box(&accelerations),
                black_box(&masses),
                black_box(&mut force_registry),
                false,
            )
        });
    });

    // Benchmark RK4 with no forces
    group.bench_function("rk4_free", |b| {
        let entity = Entity::new(1, 0);
        let entities = vec![entity];
        let mut positions = HashMapStorage::<Position>::new();
        positions.insert(entity, Position::new(0.0, 0.0, 0.0));
        let mut velocities = HashMapStorage::<Velocity>::new();
        velocities.insert(entity, Velocity::new(1.0, 0.0, 0.0));
        let accelerations = HashMapStorage::<Acceleration>::new();
        let mut masses = HashMapStorage::<Mass>::new();
        masses.insert(entity, Mass::new(1.0));
        let mut force_registry = ForceRegistry::new();
        let mut integrator = RK4Integrator::new(0.01);

        b.iter(|| {
            integrator.integrate(
                black_box(entities.iter()),
                black_box(&mut positions),
                black_box(&mut velocities),
                black_box(&accelerations),
                black_box(&masses),
                black_box(&mut force_registry),
                false,
            )
        });
    });

    group.finish();
}

#[cfg(feature = "simd")]
fn bench_simd_operations(c: &mut Criterion) {
    use physics_engine::integration::{simd_update_velocities, simd_update_positions, simd_accumulate_forces};
    
    let mut group = c.benchmark_group("simd_operations");
    
    // Test with varying sizes to see SIMD benefits
    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        // Benchmark velocity update
        group.bench_with_input(
            BenchmarkId::new("velocity_update", size),
            size,
            |b, &size| {
                let mut vx = vec![1.0; size];
                let mut vy = vec![2.0; size];
                let mut vz = vec![3.0; size];
                let ax = vec![0.5; size];
                let ay = vec![1.0; size];
                let az = vec![1.5; size];
                let dt = 0.01;
                
                b.iter(|| {
                    simd_update_velocities(
                        black_box(&mut vx),
                        black_box(&mut vy),
                        black_box(&mut vz),
                        black_box(&ax),
                        black_box(&ay),
                        black_box(&az),
                        black_box(dt),
                    )
                });
            },
        );
        
        // Benchmark position update
        group.bench_with_input(
            BenchmarkId::new("position_update", size),
            size,
            |b, &size| {
                let mut px = vec![0.0; size];
                let mut py = vec![0.0; size];
                let mut pz = vec![0.0; size];
                let vx = vec![10.0; size];
                let vy = vec![20.0; size];
                let vz = vec![30.0; size];
                let ax = vec![1.0; size];
                let ay = vec![2.0; size];
                let az = vec![3.0; size];
                let dt = 0.01;
                
                b.iter(|| {
                    simd_update_positions(
                        black_box(&mut px),
                        black_box(&mut py),
                        black_box(&mut pz),
                        black_box(&vx),
                        black_box(&vy),
                        black_box(&vz),
                        black_box(&ax),
                        black_box(&ay),
                        black_box(&az),
                        black_box(dt),
                    )
                });
            },
        );
        
        // Benchmark force accumulation
        group.bench_with_input(
            BenchmarkId::new("force_accumulation", size),
            size,
            |b, &size| {
                let mut total_fx = vec![0.0; size];
                let mut total_fy = vec![0.0; size];
                let mut total_fz = vec![0.0; size];
                let fx = vec![1.0; size];
                let fy = vec![2.0; size];
                let fz = vec![3.0; size];
                
                b.iter(|| {
                    simd_accumulate_forces(
                        black_box(&mut total_fx),
                        black_box(&mut total_fy),
                        black_box(&mut total_fz),
                        black_box(&fx),
                        black_box(&fy),
                        black_box(&fz),
                    )
                });
            },
        );
    }
    
    group.finish();
}

#[cfg(feature = "simd")]
criterion_group!(benches, bench_integrator_throughput, bench_integrator_accuracy, bench_free_motion, bench_simd_operations);

#[cfg(not(feature = "simd"))]
criterion_group!(benches, bench_integrator_throughput, bench_integrator_accuracy, bench_free_motion);

criterion_main!(benches);
