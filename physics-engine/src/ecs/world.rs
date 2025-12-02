//! World management
//!
//! The World is the central container for all ECS data,
//! managing entities, components, and providing query interfaces.

use crate::ecs::Entity;
use std::collections::HashSet;

/// The main ECS world container
///
/// World manages entity lifecycles and serves as the central
/// access point for all ECS operations.
pub struct World {
    next_entity_id: u64,
    entity_generations: Vec<u32>,
    alive_entities: HashSet<Entity>,
}

impl World {
    /// Create a new empty world
    pub fn new() -> Self {
        World {
            next_entity_id: 0,
            entity_generations: Vec::new(),
            alive_entities: HashSet::new(),
        }
    }

    /// Create a new entity
    pub fn create_entity(&mut self) -> Entity {
        let id = self.next_entity_id;
        self.next_entity_id += 1;

        // Extend generations vector if needed
        if id as usize >= self.entity_generations.len() {
            self.entity_generations.resize(id as usize + 1, 0);
        }

        let generation = self.entity_generations[id as usize];
        let entity = Entity::new(id, generation);
        self.alive_entities.insert(entity);
        
        entity
    }

    /// Destroy an entity
    ///
    /// This increments the generation counter to invalidate old references
    pub fn destroy_entity(&mut self, entity: Entity) -> bool {
        if self.alive_entities.remove(&entity) {
            // Increment generation for this entity ID
            let id = entity.id().raw() as usize;
            if id < self.entity_generations.len() {
                self.entity_generations[id] = self.entity_generations[id].wrapping_add(1);
            }
            true
        } else {
            false
        }
    }

    /// Check if an entity is alive
    pub fn is_entity_alive(&self, entity: Entity) -> bool {
        self.alive_entities.contains(&entity)
    }

    /// Get the number of alive entities
    pub fn entity_count(&self) -> usize {
        self.alive_entities.len()
    }

    /// Clear all entities
    pub fn clear(&mut self) {
        self.alive_entities.clear();
        self.entity_generations.clear();
        self.next_entity_id = 0;
    }

    /// Get an iterator over all alive entities
    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.alive_entities.iter()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_entity_lifecycle() {
        let mut world = World::new();
        
        let e1 = world.create_entity();
        let e2 = world.create_entity();
        
        assert_eq!(world.entity_count(), 2);
        assert!(world.is_entity_alive(e1));
        assert!(world.is_entity_alive(e2));
        
        world.destroy_entity(e1);
        assert_eq!(world.entity_count(), 1);
        assert!(!world.is_entity_alive(e1));
        assert!(world.is_entity_alive(e2));
    }

    #[test]
    fn test_entity_generation() {
        let mut world = World::new();
        
        let e1 = world.create_entity();
        let id = e1.id();
        let gen1 = e1.generation();
        
        world.destroy_entity(e1);
        let e2 = world.create_entity();
        
        // New entity should have different generation if reusing same ID
        if e2.id() == id {
            assert_ne!(e2.generation(), gen1);
        }
    }

    #[test]
    fn test_world_clear() {
        let mut world = World::new();
        world.create_entity();
        world.create_entity();
        
        assert_eq!(world.entity_count(), 2);
        world.clear();
        assert_eq!(world.entity_count(), 0);
    }
}
