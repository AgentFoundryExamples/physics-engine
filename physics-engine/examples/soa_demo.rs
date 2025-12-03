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

//! Structure-of-Arrays (SoA) Storage Demonstration
//!
//! This example demonstrates the difference between traditional Array-of-Structures (AoS)
//! and true Structure-of-Arrays (SoA) storage for physics components.
//!
//! # Key Concepts
//!
//! - **AoS (SoAStorage)**: Stores complete components contiguously: [Pos{x,y,z}, Pos{x,y,z}, ...]
//! - **True SoA (PositionSoAStorage)**: Stores fields separately: x:[x0,x1,...], y:[y0,y1,...], z:[z0,z1,...]
//!
//! # Performance Benefits
//!
//! - **Cache efficiency**: Load only the fields you need (e.g., only x-coordinates)
//! - **SIMD-friendly**: Contiguous field arrays enable vectorization
//! - **Memory bandwidth**: 2-3× reduction when updating single fields
//!
//! # Usage
//!
//! ```bash
//! cargo run --example soa_demo --release
//! ```

use physics_engine::ecs::{
    ComponentStorage, Entity, HashMapStorage, PositionSoAStorage, SoAStorage, VelocitySoAStorage,
    World,
};
use physics_engine::ecs::components::{Position, Velocity};
use std::time::Instant;

fn main() {
    println!("=== Structure-of-Arrays (SoA) Storage Demonstration ===\n");

    demonstrate_storage_types();
    println!();
    demonstrate_field_arrays();
    println!();
    demonstrate_bulk_operations();
    println!();
    performance_comparison();
}

/// Demonstrate the three storage types
fn demonstrate_storage_types() {
    println!("1. Storage Type Comparison\n");

    let mut world = World::new();
    let e1 = world.create_entity();
    let e2 = world.create_entity();

    // HashMap Storage (traditional)
    println!("  a) HashMapStorage (flexible, per-entity access):");
    let mut hashmap_storage = HashMapStorage::<Position>::new();
    hashmap_storage.insert(e1, Position::new(1.0, 2.0, 3.0));
    hashmap_storage.insert(e2, Position::new(4.0, 5.0, 6.0));

    if let Some(pos) = hashmap_storage.get(e1) {
        println!("     - Entity {:?} position: ({}, {}, {})", e1, pos.x(), pos.y(), pos.z());
    }
    println!("     - Supports: per-entity get()/get_mut()");
    println!("     - field_arrays(): {:?}", hashmap_storage.field_arrays().is_some());

    // SoAStorage (dense AoS)
    println!("\n  b) SoAStorage (dense AoS, good cache):");
    let mut aos_storage = SoAStorage::<Position>::new();
    aos_storage.insert(e1, Position::new(1.0, 2.0, 3.0));
    aos_storage.insert(e2, Position::new(4.0, 5.0, 6.0));

    if let Some(pos) = aos_storage.get(e1) {
        println!("     - Entity {:?} position: ({}, {}, {})", e1, pos.x(), pos.y(), pos.z());
    }
    println!("     - Supports: per-entity get()/get_mut() + bulk components()");
    println!("     - field_arrays(): {:?}", aos_storage.field_arrays().is_some());

    // True SoA Storage
    println!("\n  c) PositionSoAStorage (true SoA, optimal SIMD):");
    let mut soa_storage = PositionSoAStorage::new();
    soa_storage.insert(e1, Position::new(1.0, 2.0, 3.0));
    soa_storage.insert(e2, Position::new(4.0, 5.0, 6.0));

    println!("     - Entity {:?} get(): {:?}", e1, soa_storage.get(e1).is_some());
    println!("     - Supports: field_arrays() only (no per-entity get)");
    println!("     - field_arrays(): {:?}", soa_storage.field_arrays().is_some());

    if let Some(arrays) = soa_storage.field_arrays() {
        let (x, y, z) = arrays.as_position_arrays();
        println!("     - Field arrays: x={:?}, y={:?}, z={:?}", x, y, z);
    }
}

/// Demonstrate field array access patterns
fn demonstrate_field_arrays() {
    println!("2. Field Array Access Patterns\n");

    let mut world = World::new();
    let mut storage = PositionSoAStorage::new();

    // Create some entities
    for i in 0..5 {
        let entity = world.create_entity();
        storage.insert(
            entity,
            Position::new(i as f64, i as f64 * 2.0, i as f64 * 3.0),
        );
    }

    println!("  a) Read-only field arrays:");
    if let Some(arrays) = storage.field_arrays() {
        let (x, y, z) = arrays.as_position_arrays();
        println!("     - x values: {:?}", x);
        println!("     - y values: {:?}", y);
        println!("     - z values: {:?}", z);
    }

    println!("\n  b) Mutable field arrays (updating only x-coordinates):");
    if let Some(mut arrays) = storage.field_arrays_mut() {
        let (x, _y, _z) = arrays.as_position_arrays_mut();
        // Only modify x values - y and z are not touched
        for val in x.iter_mut() {
            *val += 100.0;
        }
        println!("     - Updated all x values by +100.0");
    }

    println!("\n  c) Verify changes:");
    if let Some(arrays) = storage.field_arrays() {
        let (x, y, z) = arrays.as_position_arrays();
        println!("     - x values: {:?}", x);
        println!("     - y values: {:?} (unchanged)", y);
        println!("     - z values: {:?} (unchanged)", z);
    }
}

/// Demonstrate bulk operations
fn demonstrate_bulk_operations() {
    println!("3. Bulk Operations (typical physics update)\n");

    let mut world = World::with_capacity(1000);
    let mut positions = PositionSoAStorage::with_capacity(1000);
    let mut velocities = VelocitySoAStorage::with_capacity(1000);

    // Create 1000 entities with positions and velocities
    for i in 0..1000 {
        let entity = world.create_entity();
        positions.insert(entity, Position::new(i as f64, 0.0, 0.0));
        velocities.insert(entity, Velocity::new(1.0, 0.0, 0.0));
    }

    println!("  Created 1000 entities with positions and velocities");

    // Simulate a time step: position += velocity * dt
    let dt = 0.01;

    if let (Some(mut pos_arrays), Some(vel_arrays)) =
        (positions.field_arrays_mut(), velocities.field_arrays())
    {
        let (px, py, pz) = pos_arrays.as_position_arrays_mut();
        let (vx, vy, vz) = vel_arrays.as_velocity_arrays();

        // Update all positions in bulk
        for i in 0..px.len() {
            px[i] += vx[i] * dt;
            py[i] += vy[i] * dt;
            pz[i] += vz[i] * dt;
        }

        println!("  Updated all positions: p' = p + v * dt");
        println!("  First 5 x positions: {:?}", &px[0..5]);
    }
}

/// Performance comparison between storage types
fn performance_comparison() {
    println!("4. Performance Comparison (1M updates)\n");

    const ENTITY_COUNT: usize = 10000;
    const ITERATIONS: usize = 100;

    let mut world = World::with_capacity(ENTITY_COUNT);

    // Setup AoS storage
    let mut aos_storage = SoAStorage::<Position>::with_capacity(ENTITY_COUNT);
    for i in 0..ENTITY_COUNT {
        let entity = world.create_entity();
        aos_storage.insert(entity, Position::new(i as f64, 0.0, 0.0));
    }

    // Setup true SoA storage
    let mut soa_storage = PositionSoAStorage::with_capacity(ENTITY_COUNT);
    world.clear();
    for i in 0..ENTITY_COUNT {
        let entity = world.create_entity();
        soa_storage.insert(entity, Position::new(i as f64, 0.0, 0.0));
    }

    // Benchmark AoS (via components())
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let components = aos_storage.components_mut();
        for pos in components.iter_mut() {
            pos.set_x(pos.x() + 1.0);
        }
    }
    let aos_duration = start.elapsed();

    // Benchmark true SoA (via field_arrays_mut())
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        if let Some(mut arrays) = soa_storage.field_arrays_mut() {
            let (x, _y, _z) = arrays.as_position_arrays_mut();
            for val in x.iter_mut() {
                *val += 1.0;
            }
        }
    }
    let soa_duration = start.elapsed();

    println!("  AoS (SoAStorage):          {:?}", aos_duration);
    println!("  True SoA (field arrays):   {:?}", soa_duration);
    println!("\n  Speedup: {:.2}×", aos_duration.as_secs_f64() / soa_duration.as_secs_f64());

    println!("\n  Benefits of true SoA:");
    println!("  - Only loads x-coordinates (saves ~66% memory bandwidth)");
    println!("  - Better cache locality (sequential field access)");
    println!("  - SIMD-friendly (contiguous f64 arrays)");
    println!("  - Expected: 2-3× speedup for bulk field updates");
}
