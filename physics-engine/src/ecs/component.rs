//! Component storage and management
//!
//! Components are data containers that can be attached to entities.
//! This module provides traits and storage mechanisms optimized for
//! cache-friendly access patterns.

use crate::ecs::Entity;
use std::any::TypeId;
use std::collections::HashMap;

/// Trait that all components must implement
///
/// Components should be plain data structures without behavior.
/// Keep components small and focused for better cache performance.
pub trait Component: 'static + Send + Sync {
    /// Get the type ID of this component
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

/// Storage interface for components
///
/// Implementations should prioritize cache-friendly data layouts,
/// such as structure-of-arrays (SoA) for better SIMD and parallel access.
pub trait ComponentStorage: Send + Sync {
    /// The component type this storage manages
    type Component: Component;

    /// Insert a component for the given entity
    fn insert(&mut self, entity: Entity, component: Self::Component);

    /// Remove a component for the given entity
    fn remove(&mut self, entity: Entity) -> Option<Self::Component>;

    /// Get a reference to a component for the given entity
    fn get(&self, entity: Entity) -> Option<&Self::Component>;

    /// Get a mutable reference to a component for the given entity
    fn get_mut(&mut self, entity: Entity) -> Option<&mut Self::Component>;

    /// Check if an entity has this component
    fn contains(&self, entity: Entity) -> bool;

    /// Clear all components
    fn clear(&mut self);
}

/// Simple HashMap-based component storage
///
/// TODO: Replace with more cache-friendly SoA-based storage for production use
pub struct HashMapStorage<T: Component> {
    components: HashMap<Entity, T>,
}

impl<T: Component> HashMapStorage<T> {
    /// Create a new empty storage
    pub fn new() -> Self {
        HashMapStorage {
            components: HashMap::new(),
        }
    }
}

impl<T: Component> Default for HashMapStorage<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Component> ComponentStorage for HashMapStorage<T> {
    type Component = T;

    fn insert(&mut self, entity: Entity, component: Self::Component) {
        self.components.insert(entity, component);
    }

    fn remove(&mut self, entity: Entity) -> Option<Self::Component> {
        self.components.remove(&entity)
    }

    fn get(&self, entity: Entity) -> Option<&Self::Component> {
        self.components.get(&entity)
    }

    fn get_mut(&mut self, entity: Entity) -> Option<&mut Self::Component> {
        self.components.get_mut(&entity)
    }

    fn contains(&self, entity: Entity) -> bool {
        self.components.contains_key(&entity)
    }

    fn clear(&mut self) {
        self.components.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    impl Component for Position {}

    #[test]
    fn test_component_storage() {
        let mut storage = HashMapStorage::<Position>::new();
        let entity = Entity::new(1, 0);
        
        let pos = Position { x: 10.0, y: 20.0 };
        storage.insert(entity, pos);
        
        assert!(storage.contains(entity));
        assert_eq!(storage.get(entity).unwrap().x, 10.0);
        
        storage.remove(entity);
        assert!(!storage.contains(entity));
    }
}
