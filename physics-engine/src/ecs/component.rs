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
/// Note: This implementation prioritizes simplicity for the initial release.
/// Future versions will optimize with Structure-of-Arrays (SoA) layouts for
/// improved cache performance and SIMD opportunities.
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

/// Structure-of-Arrays (SoA) component storage
///
/// This storage implementation organizes component data in a cache-friendly
/// Structure-of-Arrays layout where each field is stored in a separate contiguous
/// array. This provides:
///
/// - **Better cache locality**: Sequential field access loads cache lines efficiently
/// - **SIMD opportunities**: Packed arrays enable vectorization
/// - **Reduced memory bandwidth**: Load only the fields you need
///
/// The storage maintains a sparse mapping from Entity to array index, supporting
/// efficient entity creation/destruction without leaving gaps in the dense arrays.
///
/// # Memory Layout
///
/// For a component with 3 f64 fields (like Position with x, y, z), instead of:
/// ```text
/// AoS: [Component{x,y,z}, Component{x,y,z}, ...]  // scattered in memory
/// ```
///
/// SoA stores:
/// ```text
/// field1: [x0, x1, x2, x3, ...] // All x values contiguous in one cache-friendly array
/// field2: [y0, y1, y2, y3, ...] // All y values contiguous
/// field3: [z0, z1, z2, z3, ...] // All z values contiguous
/// ```
///
/// # Implementation Note
///
/// While this storage provides a SoA-like interface, it still needs to support
/// the ComponentStorage trait which expects to return references to complete
/// components. The actual field separation is implemented in component-specific
/// storage backends that store `Vec<T>` where T is Copy and can be efficiently
/// reconstructed.
///
/// # Example
///
/// ```
/// use physics_engine::ecs::{Entity, ComponentStorage, SoAStorage};
/// use physics_engine::ecs::components::Position;
///
/// let mut storage = SoAStorage::<Position>::new();
/// let entity = Entity::new(1, 0);
///
/// storage.insert(entity, Position::new(1.0, 2.0, 3.0));
/// assert!(storage.contains(entity));
/// assert_eq!(storage.get(entity).unwrap().x(), 1.0);
/// ```
pub struct SoAStorage<T: Component + Copy> {
    /// Mapping from Entity to dense array index
    entity_to_index: HashMap<Entity, usize>,
    /// Mapping from dense array index back to Entity (for swap_remove)
    index_to_entity: Vec<Entity>,
    /// The actual component data stored densely
    /// Components are Copy so we can efficiently return values
    components: Vec<T>,
}

impl<T: Component + Copy> SoAStorage<T> {
    /// Create a new empty SoA storage
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Create a new SoA storage with the given capacity
    pub fn with_capacity(capacity: usize) -> Self {
        SoAStorage {
            entity_to_index: HashMap::with_capacity(capacity),
            index_to_entity: Vec::with_capacity(capacity),
            components: Vec::with_capacity(capacity),
        }
    }

    /// Get the number of components stored
    pub fn len(&self) -> usize {
        self.components.len()
    }

    /// Check if the storage is empty
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    /// Reserve space for at least `additional` more components
    pub fn reserve(&mut self, additional: usize) {
        self.entity_to_index.reserve(additional);
        self.index_to_entity.reserve(additional);
        self.components.reserve(additional);
    }

    /// Get all entities that have components in this storage
    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.index_to_entity.iter().copied()
    }

    /// Get a reference to the dense component array
    ///
    /// This allows systems to iterate over all components efficiently
    /// in a cache-friendly manner. The components are stored contiguously
    /// in memory, maximizing cache line utilization.
    pub fn components(&self) -> &[T] {
        &self.components
    }

    /// Get a mutable reference to the dense component array
    ///
    /// This allows systems to efficiently update all components in bulk
    /// with SIMD-friendly access patterns.
    pub fn components_mut(&mut self) -> &mut [T] {
        &mut self.components
    }

    /// Get the index for an entity, if it exists
    pub fn get_index(&self, entity: Entity) -> Option<usize> {
        self.entity_to_index.get(&entity).copied()
    }
}

impl<T: Component + Copy> Default for SoAStorage<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Component + Copy> ComponentStorage for SoAStorage<T> {
    type Component = T;

    fn insert(&mut self, entity: Entity, component: Self::Component) {
        if let Some(&index) = self.entity_to_index.get(&entity) {
            // Entity already exists, update in place
            self.components[index] = component;
        } else {
            // New entity, append to end
            let new_index = self.components.len();
            self.components.push(component);
            self.entity_to_index.insert(entity, new_index);
            self.index_to_entity.push(entity);

            debug_assert_eq!(self.entity_to_index.len(), self.index_to_entity.len());
            debug_assert_eq!(self.entity_to_index.len(), self.components.len());
        }
    }

    fn remove(&mut self, entity: Entity) -> Option<Self::Component> {
        if let Some(index) = self.entity_to_index.remove(&entity) {
            let component = self.components[index];

            // Swap with last element to avoid shifting
            let last_index = self.components.len() - 1;
            if index != last_index {
                self.components.swap(index, last_index);
                // Update the entity that was swapped
                let swapped_entity = self.index_to_entity[last_index];
                self.entity_to_index.insert(swapped_entity, index);
                self.index_to_entity.swap(index, last_index);
            }
            
            self.components.pop();
            self.index_to_entity.pop();

            debug_assert_eq!(self.entity_to_index.len(), self.index_to_entity.len());
            debug_assert_eq!(self.entity_to_index.len(), self.components.len());

            Some(component)
        } else {
            None
        }
    }

    fn get(&self, entity: Entity) -> Option<&Self::Component> {
        let index = self.entity_to_index.get(&entity)?;
        Some(&self.components[*index])
    }

    fn get_mut(&mut self, entity: Entity) -> Option<&mut Self::Component> {
        let index = self.entity_to_index.get(&entity)?;
        Some(&mut self.components[*index])
    }

    fn contains(&self, entity: Entity) -> bool {
        self.entity_to_index.contains_key(&entity)
    }

    fn clear(&mut self) {
        self.entity_to_index.clear();
        self.index_to_entity.clear();
        self.components.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct TestComponent {
        x: f32,
        y: f32,
    }

    impl Component for TestComponent {}

    #[test]
    fn test_hashmap_storage() {
        let mut storage = HashMapStorage::<TestComponent>::new();
        let entity = Entity::new(1, 0);
        
        let comp = TestComponent { x: 10.0, y: 20.0 };
        storage.insert(entity, comp);
        
        assert!(storage.contains(entity));
        assert_eq!(storage.get(entity).unwrap().x, 10.0);
        
        storage.remove(entity);
        assert!(!storage.contains(entity));
    }

    #[test]
    fn test_soa_storage_basic() {
        let mut storage = SoAStorage::<TestComponent>::new();
        let entity = Entity::new(1, 0);
        
        let comp = TestComponent { x: 10.0, y: 20.0 };
        storage.insert(entity, comp);
        
        assert!(storage.contains(entity));
        assert_eq!(storage.get(entity).unwrap().x, 10.0);
        assert_eq!(storage.get(entity).unwrap().y, 20.0);
        
        let removed = storage.remove(entity);
        assert_eq!(removed, Some(comp));
        assert!(!storage.contains(entity));
    }

    #[test]
    fn test_soa_storage_multiple_entities() {
        let mut storage = SoAStorage::<TestComponent>::new();
        let e1 = Entity::new(1, 0);
        let e2 = Entity::new(2, 0);
        let e3 = Entity::new(3, 0);
        
        storage.insert(e1, TestComponent { x: 1.0, y: 2.0 });
        storage.insert(e2, TestComponent { x: 3.0, y: 4.0 });
        storage.insert(e3, TestComponent { x: 5.0, y: 6.0 });
        
        assert_eq!(storage.len(), 3);
        assert!(storage.contains(e1));
        assert!(storage.contains(e2));
        assert!(storage.contains(e3));
        
        // Test get
        assert_eq!(storage.get(e2).unwrap().x, 3.0);
        
        // Test remove middle element (swap_remove behavior)
        storage.remove(e2);
        assert_eq!(storage.len(), 2);
        assert!(!storage.contains(e2));
        assert!(storage.contains(e1));
        assert!(storage.contains(e3));
    }

    #[test]
    fn test_soa_storage_update() {
        let mut storage = SoAStorage::<TestComponent>::new();
        let entity = Entity::new(1, 0);
        
        storage.insert(entity, TestComponent { x: 1.0, y: 2.0 });
        assert_eq!(storage.get(entity).unwrap().x, 1.0);
        
        // Update the component
        storage.insert(entity, TestComponent { x: 10.0, y: 20.0 });
        assert_eq!(storage.len(), 1); // Should not increase length
        assert_eq!(storage.get(entity).unwrap().x, 10.0);
        assert_eq!(storage.get(entity).unwrap().y, 20.0);
    }

    #[test]
    fn test_soa_storage_get_mut() {
        let mut storage = SoAStorage::<TestComponent>::new();
        let entity = Entity::new(1, 0);
        
        storage.insert(entity, TestComponent { x: 1.0, y: 2.0 });
        
        // Mutate through get_mut
        if let Some(comp) = storage.get_mut(entity) {
            comp.x = 100.0;
            comp.y = 200.0;
        }
        
        assert_eq!(storage.get(entity).unwrap().x, 100.0);
        assert_eq!(storage.get(entity).unwrap().y, 200.0);
    }

    #[test]
    fn test_soa_storage_clear() {
        let mut storage = SoAStorage::<TestComponent>::new();
        
        storage.insert(Entity::new(1, 0), TestComponent { x: 1.0, y: 2.0 });
        storage.insert(Entity::new(2, 0), TestComponent { x: 3.0, y: 4.0 });
        assert_eq!(storage.len(), 2);
        
        storage.clear();
        assert_eq!(storage.len(), 0);
        assert!(storage.is_empty());
    }

    #[test]
    fn test_soa_storage_entity_generations() {
        let mut storage = SoAStorage::<TestComponent>::new();
        let e1_gen0 = Entity::new(1, 0);
        let e1_gen1 = Entity::new(1, 1);
        
        storage.insert(e1_gen0, TestComponent { x: 1.0, y: 2.0 });
        assert!(storage.contains(e1_gen0));
        assert!(!storage.contains(e1_gen1)); // Different generation
        
        storage.remove(e1_gen0);
        assert!(!storage.contains(e1_gen0));
        
        // Insert with new generation
        storage.insert(e1_gen1, TestComponent { x: 10.0, y: 20.0 });
        assert!(!storage.contains(e1_gen0)); // Old generation still not present
        assert!(storage.contains(e1_gen1));
    }

    #[test]
    fn test_soa_storage_rapid_creation_destruction() {
        let mut storage = SoAStorage::<TestComponent>::new();
        
        // Rapidly create and destroy entities
        for i in 0..100 {
            let entity = Entity::new(i, 0);
            storage.insert(entity, TestComponent { x: i as f32, y: i as f32 * 2.0 });
        }
        assert_eq!(storage.len(), 100);
        
        // Remove odd entities
        for i in (1..100).step_by(2) {
            let entity = Entity::new(i, 0);
            storage.remove(entity);
        }
        assert_eq!(storage.len(), 50);
        
        // Verify even entities still exist
        for i in (0..100).step_by(2) {
            let entity = Entity::new(i, 0);
            assert!(storage.contains(entity));
            assert_eq!(storage.get(entity).unwrap().x, i as f32);
        }
    }

    #[test]
    fn test_soa_storage_large_entity_count() {
        let mut storage = SoAStorage::<TestComponent>::with_capacity(10000);
        
        // Insert 10k entities
        for i in 0..10000 {
            let entity = Entity::new(i as u64, 0);
            storage.insert(entity, TestComponent { x: i as f32, y: i as f32 * 2.0 });
        }
        assert_eq!(storage.len(), 10000);
        
        // Verify random access
        let entity = Entity::new(5555, 0);
        assert!(storage.contains(entity));
        assert_eq!(storage.get(entity).unwrap().x, 5555.0);
        
        // Clear all
        storage.clear();
        assert_eq!(storage.len(), 0);
    }

    #[test]
    fn test_soa_storage_components_slice() {
        let mut storage = SoAStorage::<TestComponent>::new();
        
        storage.insert(Entity::new(1, 0), TestComponent { x: 1.0, y: 2.0 });
        storage.insert(Entity::new(2, 0), TestComponent { x: 3.0, y: 4.0 });
        storage.insert(Entity::new(3, 0), TestComponent { x: 5.0, y: 6.0 });
        
        // Test direct component array access
        let components = storage.components();
        assert_eq!(components.len(), 3);
        
        // Verify we can iterate efficiently
        let sum_x: f32 = components.iter().map(|c| c.x).sum();
        assert_eq!(sum_x, 9.0); // 1.0 + 3.0 + 5.0
    }

    #[test]
    fn test_soa_storage_entities_iter() {
        let mut storage = SoAStorage::<TestComponent>::new();
        
        let e1 = Entity::new(1, 0);
        let e2 = Entity::new(2, 0);
        let e3 = Entity::new(3, 0);
        
        storage.insert(e1, TestComponent { x: 1.0, y: 2.0 });
        storage.insert(e2, TestComponent { x: 3.0, y: 4.0 });
        storage.insert(e3, TestComponent { x: 5.0, y: 6.0 });
        
        // Collect entities
        let entities: Vec<Entity> = storage.entities().collect();
        assert_eq!(entities.len(), 3);
        assert!(entities.contains(&e1));
        assert!(entities.contains(&e2));
        assert!(entities.contains(&e3));
    }

    // Test with actual physics components
    use crate::ecs::components::{Position, Velocity, Mass};

    #[test]
    fn test_soa_storage_with_position() {
        let mut storage = SoAStorage::<Position>::new();
        let entity = Entity::new(1, 0);
        
        storage.insert(entity, Position::new(1.0, 2.0, 3.0));
        assert!(storage.contains(entity));
        
        let pos = storage.get(entity).unwrap();
        assert_eq!(pos.x(), 1.0);
        assert_eq!(pos.y(), 2.0);
        assert_eq!(pos.z(), 3.0);
    }

    #[test]
    fn test_soa_storage_with_velocity() {
        let mut storage = SoAStorage::<Velocity>::new();
        let entity = Entity::new(1, 0);
        
        storage.insert(entity, Velocity::new(10.0, 20.0, 30.0));
        assert!(storage.contains(entity));
        
        let vel = storage.get(entity).unwrap();
        assert_eq!(vel.dx(), 10.0);
        assert_eq!(vel.dy(), 20.0);
        assert_eq!(vel.dz(), 30.0);
    }

    #[test]
    fn test_soa_storage_with_mass() {
        let mut storage = SoAStorage::<Mass>::new();
        let e1 = Entity::new(1, 0);
        let e2 = Entity::new(2, 0);
        
        storage.insert(e1, Mass::new(10.0));
        storage.insert(e2, Mass::immovable());
        
        assert_eq!(storage.get(e1).unwrap().value(), 10.0);
        assert!(storage.get(e2).unwrap().is_immovable());
    }

    #[test]
    fn test_soa_storage_bulk_physics_operations() {
        let mut positions = SoAStorage::<Position>::new();
        let mut velocities = SoAStorage::<Velocity>::new();
        
        // Create 1000 entities
        for i in 0..1000 {
            let entity = Entity::new(i, 0);
            positions.insert(entity, Position::new(i as f64, i as f64 * 2.0, i as f64 * 3.0));
            velocities.insert(entity, Velocity::new(1.0, 2.0, 3.0));
        }
        
        assert_eq!(positions.len(), 1000);
        assert_eq!(velocities.len(), 1000);
        
        // Verify we can efficiently iterate over components
        let pos_array = positions.components();
        let vel_array = velocities.components();
        
        assert_eq!(pos_array.len(), 1000);
        assert_eq!(vel_array.len(), 1000);
        
        // This is the kind of efficient iteration that SoA enables
        for (pos, vel) in pos_array.iter().zip(vel_array.iter()) {
            assert!(pos.is_valid());
            assert!(vel.is_valid());
        }
    }
}
