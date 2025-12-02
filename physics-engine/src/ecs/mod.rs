//! Entity Component System (ECS) core implementation
//!
//! This module provides the foundational ECS architecture including:
//! - Entity management
//! - Component storage with cache-friendly data layouts
//! - System execution framework
//! - Optional parallel execution support via Rayon

mod entity;
mod component;
mod system;
mod world;

pub use entity::{Entity, EntityId};
pub use component::{Component, ComponentStorage, HashMapStorage};
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
