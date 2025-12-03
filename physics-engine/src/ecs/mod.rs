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
//! Entity Component System (ECS) core implementation
//!
//! This module provides the foundational ECS architecture including:
//! - Entity management
//! - Component storage with cache-friendly data layouts
//! - System execution framework
//! - Newtonian physics components and systems
//! - System scheduler with parallel execution support
//! - Optional parallel execution support via Rayon

mod entity;
mod component;
mod system;
mod world;

/// Newtonian physics components
pub mod components;
/// Newtonian physics systems
pub mod systems;
/// System scheduler
pub mod scheduler;

pub use entity::{Entity, EntityId};
pub use component::{Component, ComponentStorage, HashMapStorage, SoAStorage};
pub use system::{System, SystemExecutor};
pub use world::World;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_creation() {
        let world = World::new();
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_entity_creation() {
        let mut world = World::new();
        let entity = world.create_entity();
        assert_eq!(world.entity_count(), 1);
        assert!(world.is_entity_alive(entity));
    }
}
