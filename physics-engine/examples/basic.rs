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
//! Basic example demonstrating the ECS structure
//!
//! This example shows how to create a world, spawn entities,
//! and interact with the basic ECS components.

use physics_engine::ecs::{World, Component, ComponentStorage, HashMapStorage, System, SystemExecutor};

#[derive(Debug)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

impl Component for Position {}

#[derive(Debug)]
struct Velocity {
    dx: f32,
    dy: f32,
    dz: f32,
}

impl Component for Velocity {}

// Simple physics system that applies velocity to position
struct PhysicsSystem;

impl System for PhysicsSystem {
    fn run(&mut self, _world: &mut World) {
        println!("  [PhysicsSystem] Running physics update...");
        // Note: In a real implementation, systems would query components from the world
        // This is a placeholder to demonstrate system execution
    }

    fn name(&self) -> &str {
        "PhysicsSystem"
    }
}

fn main() {
    println!("Physics Engine - Basic ECS Example");
    println!("===================================\n");

    // Create a new world
    let mut world = World::new();
    println!("Created new world");

    // Create some entities
    let entity1 = world.create_entity();
    let entity2 = world.create_entity();
    let entity3 = world.create_entity();
    
    println!("Created {} entities:", world.entity_count());
    println!("  - {}", entity1);
    println!("  - {}", entity2);
    println!("  - {}", entity3);

    // Create component storage
    let mut positions = HashMapStorage::<Position>::new();
    let mut velocities = HashMapStorage::<Velocity>::new();

    // Add components to entities
    positions.insert(entity1, Position { x: 0.0, y: 0.0, z: 0.0 });
    velocities.insert(entity1, Velocity { dx: 1.0, dy: 0.0, dz: 0.0 });

    positions.insert(entity2, Position { x: 5.0, y: 5.0, z: 0.0 });
    velocities.insert(entity2, Velocity { dx: -1.0, dy: 1.0, dz: 0.0 });

    positions.insert(entity3, Position { x: -3.0, y: 2.0, z: 1.0 });
    
    println!("\nComponent assignments:");
    println!("  Entity 1: Position + Velocity");
    println!("  Entity 2: Position + Velocity");
    println!("  Entity 3: Position only");

    // Query entities with position components
    println!("\nEntities with Position component:");
    for entity in world.entities() {
        if let Some(pos) = positions.get(*entity) {
            println!("  {} -> Position({:.1}, {:.1}, {:.1})", 
                     entity, pos.x, pos.y, pos.z);
        }
    }

    // Simulate a simple update loop
    println!("\nSimulating movement (entities with both Position and Velocity):");
    for entity in world.entities() {
        if let (Some(pos), Some(vel)) = (positions.get_mut(*entity), velocities.get(*entity)) {
            pos.x += vel.dx;
            pos.y += vel.dy;
            pos.z += vel.dz;
            println!("  {} moved to Position({:.1}, {:.1}, {:.1})", 
                     entity, pos.x, pos.y, pos.z);
        }
    }

    // Demonstrate system execution
    println!("\nDemonstrating system execution:");
    let mut executor = SystemExecutor::new();
    executor.add_system(PhysicsSystem);
    
    println!("  Registered {} system(s)", executor.system_count());
    
    #[cfg(feature = "parallel")]
    {
        println!("  Running systems with parallel support...");
        executor.run_parallel(&mut world);
    }
    
    #[cfg(not(feature = "parallel"))]
    {
        println!("  Running systems sequentially...");
        executor.run_sequential(&mut world);
    }

    // Clean up
    world.destroy_entity(entity2);
    println!("\nDestroyed {}", entity2);
    println!("Remaining entities: {}", world.entity_count());

    #[cfg(feature = "parallel")]
    println!("\n[Parallel execution support enabled via Rayon]");
    
    #[cfg(not(feature = "parallel"))]
    println!("\n[Running in sequential mode]");

    println!("\nExample completed successfully!");
}
