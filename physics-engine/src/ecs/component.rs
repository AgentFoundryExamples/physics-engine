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
/// This trait supports both traditional Array-of-Structures (AoS) access via `get()`/`get_mut()`
/// and Structure-of-Arrays (SoA) access via `field_arrays()` for SIMD-friendly bulk operations.
///
/// # Design Philosophy
///
/// The dual API approach allows:
/// - **Legacy compatibility**: Existing code using `get()`/`get_mut()` continues to work
/// - **SIMD optimization**: Systems can access contiguous field arrays for vectorization
/// - **Flexible storage**: Implementations can choose AoS (HashMap) or SoA (separate field vectors)
///
/// # SoA Access Pattern
///
/// For components with multiple fields (e.g., Position with x, y, z), SoA storage
/// provides separate contiguous arrays per field:
///
/// ```text
/// Traditional AoS:  [Pos{x:1,y:2,z:3}, Pos{x:4,y:5,z:6}, ...]
/// SoA Layout:       x: [1, 4, ...], y: [2, 5, ...], z: [3, 6, ...]
/// ```
///
/// This enables SIMD operations to process multiple values per instruction and
/// improves cache utilization when only specific fields are needed.
///
/// # Implementation Notes
///
/// - `field_arrays()` returns `None` for storage implementations that don't support SoA
/// - The default implementation returns `None`, making SoA opt-in for new storage types
/// - For components with single fields (e.g., Mass), SoA and AoS are equivalent
pub trait ComponentStorage: Send + Sync {
    /// The component type this storage manages
    type Component: Component;

    /// Insert a component for the given entity
    fn insert(&mut self, entity: Entity, component: Self::Component);

    /// Remove a component for the given entity
    fn remove(&mut self, entity: Entity) -> Option<Self::Component>;

    /// Get a reference to a component for the given entity
    ///
    /// This provides traditional per-entity access. For bulk operations,
    /// prefer `field_arrays()` when available for better cache performance.
    ///
    /// # Note for SoA Storage Implementations
    ///
    /// True Structure-of-Arrays storage implementations (e.g., `PositionSoAStorage`)
    /// return `None` from this method because they store fields in separate arrays
    /// and cannot construct temporary component references. Systems working with
    /// SoA storage must use `field_arrays()` for bulk operations instead.
    fn get(&self, entity: Entity) -> Option<&Self::Component>;

    /// Get a mutable reference to a component for the given entity
    ///
    /// This provides traditional per-entity access. For bulk operations,
    /// prefer `field_arrays_mut()` when available for better cache performance.
    ///
    /// # Note for SoA Storage Implementations
    ///
    /// True Structure-of-Arrays storage implementations return `None` from this
    /// method. Use `field_arrays_mut()` for bulk field mutations instead.
    fn get_mut(&mut self, entity: Entity) -> Option<&mut Self::Component>;

    /// Check if an entity has this component
    fn contains(&self, entity: Entity) -> bool;

    /// Clear all components
    fn clear(&mut self);

    /// Get read-only access to field arrays for SoA-style iteration
    ///
    /// This method enables SIMD-friendly bulk operations by exposing separate
    /// contiguous arrays for each component field. Returns `None` for storage
    /// implementations that don't support SoA (e.g., HashMap-based storage).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Process all positions with SIMD operations
    /// if let Some(field_arrays) = storage.field_arrays() {
    ///     let (x_array, y_array, z_array) = field_arrays.as_position_arrays();
    ///     // SIMD operations on x_array, y_array, z_array
    /// }
    /// ```
    ///
    /// # Returns
    ///
    /// - `Some(FieldArrays)` for SoA-compatible storage
    /// - `None` for AoS-only storage (use `get()`/`get_mut()` instead)
    fn field_arrays(&self) -> Option<FieldArrays<'_, Self::Component>> {
        None
    }

    /// Get mutable access to field arrays for SoA-style iteration
    ///
    /// This method enables SIMD-friendly bulk mutations by exposing separate
    /// contiguous arrays for each component field. Returns `None` for storage
    /// implementations that don't support SoA.
    ///
    /// # Safety and Borrowing
    ///
    /// The returned `FieldArraysMut` holds exclusive mutable borrows of the
    /// underlying field arrays. While held, `get()`, `get_mut()`, `insert()`,
    /// and `remove()` operations will fail to prevent aliasing violations.
    fn field_arrays_mut(&mut self) -> Option<FieldArraysMut<'_, Self::Component>> {
        None
    }
}

/// Read-only access to component field arrays in Structure-of-Arrays layout
///
/// This type provides access to separate contiguous arrays for each field
/// of a component type, enabling SIMD-friendly bulk operations. The specific
/// arrays available depend on the component type.
///
/// # Type-Specific Access
///
/// Each component type provides accessor methods for its fields:
/// - `Position`: `as_position_arrays()` → `(&[f64], &[f64], &[f64])` for x, y, z
/// - `Velocity`: `as_velocity_arrays()` → `(&[f64], &[f64], &[f64])` for dx, dy, dz
/// - `Acceleration`: `as_acceleration_arrays()` → `(&[f64], &[f64], &[f64])` for ax, ay, az
/// - `Mass`: `as_mass_array()` → `&[f64]` for values
///
/// # Example
///
/// ```rust,ignore
/// use physics_engine::ecs::{SoAStorage, ComponentStorage};
/// use physics_engine::ecs::components::Position;
///
/// let storage = SoAStorage::<Position>::new();
/// if let Some(arrays) = storage.field_arrays() {
///     let (x, y, z) = arrays.as_position_arrays();
///     // Process x, y, z with SIMD operations
/// }
/// ```
pub enum FieldArrays<'a, T: Component> {
    /// Position component field arrays (x, y, z)
    Position(&'a [f64], &'a [f64], &'a [f64]),
    /// Velocity component field arrays (dx, dy, dz)
    Velocity(&'a [f64], &'a [f64], &'a [f64]),
    /// Acceleration component field arrays (ax, ay, az)
    Acceleration(&'a [f64], &'a [f64], &'a [f64]),
    /// Mass component field array (value)
    Mass(&'a [f64]),
    /// Marker to use the generic type parameter
    _Phantom(std::marker::PhantomData<T>),
}

impl<'a, T: Component> FieldArrays<'a, T> {
    /// Access Position field arrays (x, y, z)
    ///
    /// # Panics
    ///
    /// Panics if this is not a Position field array
    pub fn as_position_arrays(&self) -> (&'a [f64], &'a [f64], &'a [f64]) {
        match self {
            FieldArrays::Position(x, y, z) => (*x, *y, *z),
            _ => panic!("Expected Position field arrays"),
        }
    }

    /// Access Velocity field arrays (dx, dy, dz)
    ///
    /// # Panics
    ///
    /// Panics if this is not a Velocity field array
    pub fn as_velocity_arrays(&self) -> (&'a [f64], &'a [f64], &'a [f64]) {
        match self {
            FieldArrays::Velocity(dx, dy, dz) => (*dx, *dy, *dz),
            _ => panic!("Expected Velocity field arrays"),
        }
    }

    /// Access Acceleration field arrays (ax, ay, az)
    ///
    /// # Panics
    ///
    /// Panics if this is not an Acceleration field array
    pub fn as_acceleration_arrays(&self) -> (&'a [f64], &'a [f64], &'a [f64]) {
        match self {
            FieldArrays::Acceleration(ax, ay, az) => (*ax, *ay, *az),
            _ => panic!("Expected Acceleration field arrays"),
        }
    }

    /// Access Mass field array (value)
    ///
    /// # Panics
    ///
    /// Panics if this is not a Mass field array
    pub fn as_mass_array(&self) -> &'a [f64] {
        match self {
            FieldArrays::Mass(values) => *values,
            _ => panic!("Expected Mass field array"),
        }
    }
}

/// Mutable access to component field arrays in Structure-of-Arrays layout
///
/// This type provides mutable access to separate contiguous arrays for each field
/// of a component type, enabling SIMD-friendly bulk mutations.
///
/// # Safety and Borrowing
///
/// While this object exists, it holds exclusive mutable borrows of the underlying
/// field arrays. Other storage operations (get, insert, remove) must not be called
/// until this is dropped.
pub enum FieldArraysMut<'a, T: Component> {
    /// Position component field arrays (x, y, z)
    Position(&'a mut [f64], &'a mut [f64], &'a mut [f64]),
    /// Velocity component field arrays (dx, dy, dz)
    Velocity(&'a mut [f64], &'a mut [f64], &'a mut [f64]),
    /// Acceleration component field arrays (ax, ay, az)
    Acceleration(&'a mut [f64], &'a mut [f64], &'a mut [f64]),
    /// Mass component field array (value)
    Mass(&'a mut [f64]),
    /// Marker to use the generic type parameter
    _Phantom(std::marker::PhantomData<T>),
}

impl<'a, T: Component> FieldArraysMut<'a, T> {
    /// Access Position field arrays mutably (x, y, z)
    ///
    /// # Panics
    ///
    /// Panics if this is not a Position field array
    pub fn as_position_arrays_mut(&mut self) -> (&mut [f64], &mut [f64], &mut [f64]) {
        match self {
            FieldArraysMut::Position(x, y, z) => (x, y, z),
            _ => panic!("Expected Position field arrays"),
        }
    }

    /// Access Velocity field arrays mutably (dx, dy, dz)
    ///
    /// # Panics
    ///
    /// Panics if this is not a Velocity field array
    pub fn as_velocity_arrays_mut(&mut self) -> (&mut [f64], &mut [f64], &mut [f64]) {
        match self {
            FieldArraysMut::Velocity(dx, dy, dz) => (dx, dy, dz),
            _ => panic!("Expected Velocity field arrays"),
        }
    }

    /// Access Acceleration field arrays mutably (ax, ay, az)
    ///
    /// # Panics
    ///
    /// Panics if this is not an Acceleration field array
    pub fn as_acceleration_arrays_mut(&mut self) -> (&mut [f64], &mut [f64], &mut [f64]) {
        match self {
            FieldArraysMut::Acceleration(ax, ay, az) => (ax, ay, az),
            _ => panic!("Expected Acceleration field arrays"),
        }
    }

    /// Access Mass field array mutably (value)
    ///
    /// # Panics
    ///
    /// Panics if this is not a Mass field array
    pub fn as_mass_array_mut(&mut self) -> &mut [f64] {
        match self {
            FieldArraysMut::Mass(values) => *values,
            _ => panic!("Expected Mass field array"),
        }
    }
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

/// Dense array component storage with cache-friendly layout
///
/// **Important**: Despite the name `SoAStorage`, this is a **dense Array-of-Structures (AoS)**
/// implementation, NOT a true Structure-of-Arrays layout. The name is preserved for API
/// compatibility, but the implementation stores complete component structures in a contiguous
/// `Vec<T>`.
///
/// This storage provides cache-friendly access through:
/// - **Dense packing**: All components stored contiguously in a single `Vec<T>`
/// - **Sequential access**: Iteration accesses memory sequentially, maximizing cache line usage
/// - **No pointer chasing**: Direct array indexing instead of HashMap pointer indirection
///
/// The storage maintains a sparse mapping from Entity to array index, supporting
/// efficient entity creation/destruction without leaving gaps in the dense arrays.
///
/// # Memory Layout
///
/// Current implementation (Dense AoS):
/// ```text
/// components: [Component{x,y,z}, Component{x,y,z}, Component{x,y,z}, ...]
/// ```
/// All components are stored contiguously in a single vector. This provides good
/// cache locality for iteration but loads entire component structures even if
/// only one field is needed.
///
/// A true SoA layout would be:
/// ```text
/// x_values: [x0, x1, x2, x3, ...]
/// y_values: [y0, y1, y2, y3, ...]
/// z_values: [z0, z1, z2, z3, ...]
/// ```
/// This is not implemented because the `ComponentStorage` trait requires returning
/// references to complete components (`&T`), which is incompatible with split field arrays.
///
/// # Copy Requirement
///
/// Components must implement `Copy` to support the `ComponentStorage` trait's
/// `get()` and `remove()` methods. The Copy bound also enables efficient returns
/// of component values.
///
/// For components that cannot implement `Copy` (e.g., types with heap allocations),
/// use `HashMapStorage` instead, or consider refactoring the component to use
/// value types (e.g., indices into separate data structures rather than owned data).
///
/// All physics components (Position, Velocity, Acceleration, Mass) are small,
/// stack-allocated types that implement `Copy` naturally.
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

    /// Check internal invariants for testing and debugging
    ///
    /// This method validates that the storage's internal state is consistent:
    /// - All three data structures have the same length
    /// - Entity-to-index mappings are bidirectional
    /// - No entity appears twice
    ///
    /// Returns `Ok(())` if all invariants hold, or `Err(String)` with a
    /// description of the violated invariant.
    #[cfg(test)]
    pub fn check_invariants(&self) -> Result<(), String> {
        // Check lengths match
        if self.entity_to_index.len() != self.index_to_entity.len() {
            return Err(format!(
                "Length mismatch: entity_to_index={}, index_to_entity={}",
                self.entity_to_index.len(),
                self.index_to_entity.len()
            ));
        }
        if self.entity_to_index.len() != self.components.len() {
            return Err(format!(
                "Length mismatch: entity_to_index={}, components={}",
                self.entity_to_index.len(),
                self.components.len()
            ));
        }

        // Check bidirectional mapping
        for (entity, &index) in &self.entity_to_index {
            if index >= self.index_to_entity.len() {
                return Err(format!(
                    "Entity {:?} maps to out-of-bounds index {}",
                    entity, index
                ));
            }
            if self.index_to_entity[index] != *entity {
                return Err(format!(
                    "Mapping inconsistency: entity {:?} -> index {}, but index {} -> entity {:?}",
                    entity, index, index, self.index_to_entity[index]
                ));
            }
        }

        // Check no duplicate entities in index_to_entity (O(n) with HashSet)
        let mut seen = std::collections::HashSet::new();
        for (i, entity) in self.index_to_entity.iter().enumerate() {
            if !seen.insert(*entity) {
                return Err(format!(
                    "Duplicate entity {:?} at index {}",
                    entity, i
                ));
            }
        }

        Ok(())
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
                // This must succeed - if it doesn't, our internal state is corrupted
                let idx = self.entity_to_index.get_mut(&swapped_entity)
                    .expect("Internal invariant violated: entity in index_to_entity but not in entity_to_index");
                *idx = index;
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

/// True Structure-of-Arrays storage for Position components
///
/// This storage implementation uses separate contiguous arrays for x, y, and z coordinates,
/// enabling SIMD-friendly bulk operations and optimal cache utilization.
///
/// # Memory Layout
///
/// ```text
/// x_values: [x0, x1, x2, x3, ...]
/// y_values: [y0, y1, y2, y3, ...]
/// z_values: [z0, z1, z2, z3, ...]
/// ```
///
/// # Example
///
/// ```rust
/// use physics_engine::ecs::{Entity, ComponentStorage, PositionSoAStorage};
/// use physics_engine::ecs::components::Position;
///
/// let mut storage = PositionSoAStorage::new();
/// let entity = Entity::new(1, 0);
///
/// storage.insert(entity, Position::new(1.0, 2.0, 3.0));
///
/// // Access via traditional API
/// assert_eq!(storage.get(entity).unwrap().x(), 1.0);
///
/// // Or access field arrays directly for SIMD operations
/// if let Some(arrays) = storage.field_arrays() {
///     let (x, y, z) = arrays.as_position_arrays();
///     // Process x, y, z with SIMD
/// }
/// ```
pub struct PositionSoAStorage {
    entity_to_index: HashMap<Entity, usize>,
    index_to_entity: Vec<Entity>,
    x_values: Vec<f64>,
    y_values: Vec<f64>,
    z_values: Vec<f64>,
}

impl PositionSoAStorage {
    /// Create a new empty Position SoA storage
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Create a new Position SoA storage with the given capacity
    pub fn with_capacity(capacity: usize) -> Self {
        PositionSoAStorage {
            entity_to_index: HashMap::with_capacity(capacity),
            index_to_entity: Vec::with_capacity(capacity),
            x_values: Vec::with_capacity(capacity),
            y_values: Vec::with_capacity(capacity),
            z_values: Vec::with_capacity(capacity),
        }
    }

    /// Get the number of components stored
    pub fn len(&self) -> usize {
        self.x_values.len()
    }

    /// Check if the storage is empty
    pub fn is_empty(&self) -> bool {
        self.x_values.is_empty()
    }
}

impl Default for PositionSoAStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentStorage for PositionSoAStorage {
    type Component = crate::ecs::components::Position;

    fn insert(&mut self, entity: Entity, component: Self::Component) {
        if let Some(&index) = self.entity_to_index.get(&entity) {
            // Entity already exists, update in place
            self.x_values[index] = component.x();
            self.y_values[index] = component.y();
            self.z_values[index] = component.z();
        } else {
            // New entity, append to end
            let new_index = self.x_values.len();
            self.x_values.push(component.x());
            self.y_values.push(component.y());
            self.z_values.push(component.z());
            self.entity_to_index.insert(entity, new_index);
            self.index_to_entity.push(entity);
        }
    }

    fn remove(&mut self, entity: Entity) -> Option<Self::Component> {
        if let Some(index) = self.entity_to_index.remove(&entity) {
            let x = self.x_values[index];
            let y = self.y_values[index];
            let z = self.z_values[index];

            // Swap with last element to avoid shifting
            let last_index = self.x_values.len() - 1;
            if index != last_index {
                self.x_values.swap(index, last_index);
                self.y_values.swap(index, last_index);
                self.z_values.swap(index, last_index);
                
                // Update the entity that was swapped
                let swapped_entity = self.index_to_entity[last_index];
                *self.entity_to_index.get_mut(&swapped_entity)
                    .expect("Internal invariant violated") = index;
                self.index_to_entity.swap(index, last_index);
            }
            
            self.x_values.pop();
            self.y_values.pop();
            self.z_values.pop();
            self.index_to_entity.pop();

            Some(Self::Component::new(x, y, z))
        } else {
            None
        }
    }

    fn get(&self, entity: Entity) -> Option<&Self::Component> {
        // True SoA storage cannot return references to individual components
        // because fields are stored in separate arrays. Systems should use
        // field_arrays() instead for SIMD-friendly bulk operations.
        let _ = entity; // Suppress unused warning
        None
    }

    fn get_mut(&mut self, entity: Entity) -> Option<&mut Self::Component> {
        // True SoA storage cannot return mutable references to individual components.
        // Use field_arrays_mut() for bulk field mutations.
        let _ = entity; // Suppress unused warning
        None
    }

    fn contains(&self, entity: Entity) -> bool {
        self.entity_to_index.contains_key(&entity)
    }

    fn clear(&mut self) {
        self.entity_to_index.clear();
        self.index_to_entity.clear();
        self.x_values.clear();
        self.y_values.clear();
        self.z_values.clear();
    }

    fn field_arrays(&self) -> Option<FieldArrays<'_, Self::Component>> {
        Some(FieldArrays::Position(
            &self.x_values,
            &self.y_values,
            &self.z_values,
        ))
    }

    fn field_arrays_mut(&mut self) -> Option<FieldArraysMut<'_, Self::Component>> {
        Some(FieldArraysMut::Position(
            &mut self.x_values,
            &mut self.y_values,
            &mut self.z_values,
        ))
    }
}

/// True Structure-of-Arrays storage for Velocity components
///
/// Similar to `PositionSoAStorage` but for velocity components (dx, dy, dz).
pub struct VelocitySoAStorage {
    entity_to_index: HashMap<Entity, usize>,
    index_to_entity: Vec<Entity>,
    dx_values: Vec<f64>,
    dy_values: Vec<f64>,
    dz_values: Vec<f64>,
}

impl VelocitySoAStorage {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        VelocitySoAStorage {
            entity_to_index: HashMap::with_capacity(capacity),
            index_to_entity: Vec::with_capacity(capacity),
            dx_values: Vec::with_capacity(capacity),
            dy_values: Vec::with_capacity(capacity),
            dz_values: Vec::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.dx_values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.dx_values.is_empty()
    }
}

impl Default for VelocitySoAStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentStorage for VelocitySoAStorage {
    type Component = crate::ecs::components::Velocity;

    fn insert(&mut self, entity: Entity, component: Self::Component) {
        if let Some(&index) = self.entity_to_index.get(&entity) {
            self.dx_values[index] = component.dx();
            self.dy_values[index] = component.dy();
            self.dz_values[index] = component.dz();
        } else {
            let new_index = self.dx_values.len();
            self.dx_values.push(component.dx());
            self.dy_values.push(component.dy());
            self.dz_values.push(component.dz());
            self.entity_to_index.insert(entity, new_index);
            self.index_to_entity.push(entity);
        }
    }

    fn remove(&mut self, entity: Entity) -> Option<Self::Component> {
        if let Some(index) = self.entity_to_index.remove(&entity) {
            let dx = self.dx_values[index];
            let dy = self.dy_values[index];
            let dz = self.dz_values[index];

            let last_index = self.dx_values.len() - 1;
            if index != last_index {
                self.dx_values.swap(index, last_index);
                self.dy_values.swap(index, last_index);
                self.dz_values.swap(index, last_index);
                
                let swapped_entity = self.index_to_entity[last_index];
                *self.entity_to_index.get_mut(&swapped_entity)
                    .expect("Internal invariant violated") = index;
                self.index_to_entity.swap(index, last_index);
            }
            
            self.dx_values.pop();
            self.dy_values.pop();
            self.dz_values.pop();
            self.index_to_entity.pop();

            Some(Self::Component::new(dx, dy, dz))
        } else {
            None
        }
    }

    fn get(&self, _entity: Entity) -> Option<&Self::Component> {
        None // Use field_arrays() for SoA storage
    }

    fn get_mut(&mut self, _entity: Entity) -> Option<&mut Self::Component> {
        None // Use field_arrays_mut() for SoA storage
    }

    fn contains(&self, entity: Entity) -> bool {
        self.entity_to_index.contains_key(&entity)
    }

    fn clear(&mut self) {
        self.entity_to_index.clear();
        self.index_to_entity.clear();
        self.dx_values.clear();
        self.dy_values.clear();
        self.dz_values.clear();
    }

    fn field_arrays(&self) -> Option<FieldArrays<'_, Self::Component>> {
        Some(FieldArrays::Velocity(
            &self.dx_values,
            &self.dy_values,
            &self.dz_values,
        ))
    }

    fn field_arrays_mut(&mut self) -> Option<FieldArraysMut<'_, Self::Component>> {
        Some(FieldArraysMut::Velocity(
            &mut self.dx_values,
            &mut self.dy_values,
            &mut self.dz_values,
        ))
    }
}

/// True Structure-of-Arrays storage for Acceleration components
pub struct AccelerationSoAStorage {
    entity_to_index: HashMap<Entity, usize>,
    index_to_entity: Vec<Entity>,
    ax_values: Vec<f64>,
    ay_values: Vec<f64>,
    az_values: Vec<f64>,
}

impl AccelerationSoAStorage {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        AccelerationSoAStorage {
            entity_to_index: HashMap::with_capacity(capacity),
            index_to_entity: Vec::with_capacity(capacity),
            ax_values: Vec::with_capacity(capacity),
            ay_values: Vec::with_capacity(capacity),
            az_values: Vec::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.ax_values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ax_values.is_empty()
    }
}

impl Default for AccelerationSoAStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentStorage for AccelerationSoAStorage {
    type Component = crate::ecs::components::Acceleration;

    fn insert(&mut self, entity: Entity, component: Self::Component) {
        if let Some(&index) = self.entity_to_index.get(&entity) {
            self.ax_values[index] = component.ax();
            self.ay_values[index] = component.ay();
            self.az_values[index] = component.az();
        } else {
            let new_index = self.ax_values.len();
            self.ax_values.push(component.ax());
            self.ay_values.push(component.ay());
            self.az_values.push(component.az());
            self.entity_to_index.insert(entity, new_index);
            self.index_to_entity.push(entity);
        }
    }

    fn remove(&mut self, entity: Entity) -> Option<Self::Component> {
        if let Some(index) = self.entity_to_index.remove(&entity) {
            let ax = self.ax_values[index];
            let ay = self.ay_values[index];
            let az = self.az_values[index];

            let last_index = self.ax_values.len() - 1;
            if index != last_index {
                self.ax_values.swap(index, last_index);
                self.ay_values.swap(index, last_index);
                self.az_values.swap(index, last_index);
                
                let swapped_entity = self.index_to_entity[last_index];
                *self.entity_to_index.get_mut(&swapped_entity)
                    .expect("Internal invariant violated") = index;
                self.index_to_entity.swap(index, last_index);
            }
            
            self.ax_values.pop();
            self.ay_values.pop();
            self.az_values.pop();
            self.index_to_entity.pop();

            Some(Self::Component::new(ax, ay, az))
        } else {
            None
        }
    }

    fn get(&self, _entity: Entity) -> Option<&Self::Component> {
        None // Use field_arrays() for SoA storage
    }

    fn get_mut(&mut self, _entity: Entity) -> Option<&mut Self::Component> {
        None // Use field_arrays_mut() for SoA storage
    }

    fn contains(&self, entity: Entity) -> bool {
        self.entity_to_index.contains_key(&entity)
    }

    fn clear(&mut self) {
        self.entity_to_index.clear();
        self.index_to_entity.clear();
        self.ax_values.clear();
        self.ay_values.clear();
        self.az_values.clear();
    }

    fn field_arrays(&self) -> Option<FieldArrays<'_, Self::Component>> {
        Some(FieldArrays::Acceleration(
            &self.ax_values,
            &self.ay_values,
            &self.az_values,
        ))
    }

    fn field_arrays_mut(&mut self) -> Option<FieldArraysMut<'_, Self::Component>> {
        Some(FieldArraysMut::Acceleration(
            &mut self.ax_values,
            &mut self.ay_values,
            &mut self.az_values,
        ))
    }
}

/// True Structure-of-Arrays storage for Mass components
pub struct MassSoAStorage {
    entity_to_index: HashMap<Entity, usize>,
    index_to_entity: Vec<Entity>,
    values: Vec<f64>,
}

impl MassSoAStorage {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        MassSoAStorage {
            entity_to_index: HashMap::with_capacity(capacity),
            index_to_entity: Vec::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl Default for MassSoAStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentStorage for MassSoAStorage {
    type Component = crate::ecs::components::Mass;

    fn insert(&mut self, entity: Entity, component: Self::Component) {
        if let Some(&index) = self.entity_to_index.get(&entity) {
            self.values[index] = component.value();
        } else {
            let new_index = self.values.len();
            self.values.push(component.value());
            self.entity_to_index.insert(entity, new_index);
            self.index_to_entity.push(entity);
        }
    }

    fn remove(&mut self, entity: Entity) -> Option<Self::Component> {
        if let Some(index) = self.entity_to_index.remove(&entity) {
            let value = self.values[index];

            let last_index = self.values.len() - 1;
            if index != last_index {
                self.values.swap(index, last_index);
                
                let swapped_entity = self.index_to_entity[last_index];
                *self.entity_to_index.get_mut(&swapped_entity)
                    .expect("Internal invariant violated") = index;
                self.index_to_entity.swap(index, last_index);
            }
            
            self.values.pop();
            self.index_to_entity.pop();

            Some(Self::Component::new(value))
        } else {
            None
        }
    }

    fn get(&self, _entity: Entity) -> Option<&Self::Component> {
        None // Use field_arrays() for SoA storage
    }

    fn get_mut(&mut self, _entity: Entity) -> Option<&mut Self::Component> {
        None // Use field_arrays_mut() for SoA storage
    }

    fn contains(&self, entity: Entity) -> bool {
        self.entity_to_index.contains_key(&entity)
    }

    fn clear(&mut self) {
        self.entity_to_index.clear();
        self.index_to_entity.clear();
        self.values.clear();
    }

    fn field_arrays(&self) -> Option<FieldArrays<'_, Self::Component>> {
        Some(FieldArrays::Mass(&self.values))
    }

    fn field_arrays_mut(&mut self) -> Option<FieldArraysMut<'_, Self::Component>> {
        Some(FieldArraysMut::Mass(&mut self.values))
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
    use crate::ecs::components::{Position, Velocity, Acceleration, Mass};

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

    #[test]
    fn test_soa_storage_invariants() {
        let mut storage = SoAStorage::<TestComponent>::new();
        
        // Initially empty, invariants should hold
        assert!(storage.check_invariants().is_ok());
        
        // Add some entities
        for i in 0..10 {
            let entity = Entity::new(i, 0);
            storage.insert(entity, TestComponent { x: i as f32, y: i as f32 * 2.0 });
            assert!(storage.check_invariants().is_ok(), 
                "Invariants violated after inserting entity {}", i);
        }
        
        // Remove some entities
        for i in (0..10).step_by(2) {
            let entity = Entity::new(i, 0);
            storage.remove(entity);
            assert!(storage.check_invariants().is_ok(), 
                "Invariants violated after removing entity {}", i);
        }
        
        // Update some entities
        for i in (1..10).step_by(2) {
            let entity = Entity::new(i, 0);
            storage.insert(entity, TestComponent { x: 100.0, y: 200.0 });
            assert!(storage.check_invariants().is_ok(), 
                "Invariants violated after updating entity {}", i);
        }
        
        // Clear and check
        storage.clear();
        assert!(storage.check_invariants().is_ok());
        assert_eq!(storage.len(), 0);
    }

    // Tests for true SoA storage implementations

    #[test]
    fn test_position_soa_storage_basic() {
        let mut storage = PositionSoAStorage::new();
        let entity = Entity::new(1, 0);
        
        let pos = Position::new(1.0, 2.0, 3.0);
        storage.insert(entity, pos);
        
        assert!(storage.contains(entity));
        assert_eq!(storage.len(), 1);
        
        // Access via field arrays
        let arrays = storage.field_arrays().unwrap();
        let (x, y, z) = arrays.as_position_arrays();
        assert_eq!(x[0], 1.0);
        assert_eq!(y[0], 2.0);
        assert_eq!(z[0], 3.0);
        
        // Remove and verify
        let removed = storage.remove(entity).unwrap();
        assert_eq!(removed.x(), 1.0);
        assert!(!storage.contains(entity));
        assert_eq!(storage.len(), 0);
    }

    #[test]
    fn test_position_soa_storage_field_arrays_mut() {
        let mut storage = PositionSoAStorage::new();
        let e1 = Entity::new(1, 0);
        let e2 = Entity::new(2, 0);
        
        storage.insert(e1, Position::new(1.0, 2.0, 3.0));
        storage.insert(e2, Position::new(4.0, 5.0, 6.0));
        
        // Mutate via field arrays
        {
            let mut arrays = storage.field_arrays_mut().unwrap();
            let (x, y, z) = arrays.as_position_arrays_mut();
            x[0] *= 2.0;
            y[0] *= 2.0;
            z[0] *= 2.0;
        }
        
        // Verify mutations
        let arrays = storage.field_arrays().unwrap();
        let (x, y, z) = arrays.as_position_arrays();
        assert_eq!(x[0], 2.0);
        assert_eq!(y[0], 4.0);
        assert_eq!(z[0], 6.0);
    }

    #[test]
    fn test_position_soa_storage_multiple_entities() {
        let mut storage = PositionSoAStorage::with_capacity(100);
        
        // Insert many entities
        for i in 0..100 {
            let entity = Entity::new(i, 0);
            storage.insert(entity, Position::new(i as f64, i as f64 * 2.0, i as f64 * 3.0));
        }
        
        assert_eq!(storage.len(), 100);
        
        // Verify field arrays have correct data
        let arrays = storage.field_arrays().unwrap();
        let (x, y, z) = arrays.as_position_arrays();
        assert_eq!(x.len(), 100);
        assert_eq!(x[50], 50.0);
        assert_eq!(y[50], 100.0);
        assert_eq!(z[50], 150.0);
        
        // Remove some entities
        for i in 0..50 {
            let entity = Entity::new(i, 0);
            storage.remove(entity);
        }
        
        assert_eq!(storage.len(), 50);
    }

    #[test]
    fn test_velocity_soa_storage_basic() {
        let mut storage = VelocitySoAStorage::new();
        let entity = Entity::new(1, 0);
        
        storage.insert(entity, Velocity::new(10.0, 20.0, 30.0));
        assert!(storage.contains(entity));
        
        let arrays = storage.field_arrays().unwrap();
        let (dx, dy, dz) = arrays.as_velocity_arrays();
        assert_eq!(dx[0], 10.0);
        assert_eq!(dy[0], 20.0);
        assert_eq!(dz[0], 30.0);
    }

    #[test]
    fn test_acceleration_soa_storage_basic() {
        let mut storage = AccelerationSoAStorage::new();
        let entity = Entity::new(1, 0);
        
        storage.insert(entity, Acceleration::new(0.0, -9.81, 0.0));
        assert!(storage.contains(entity));
        
        let arrays = storage.field_arrays().unwrap();
        let (ax, ay, az) = arrays.as_acceleration_arrays();
        assert_eq!(ax[0], 0.0);
        assert_eq!(ay[0], -9.81);
        assert_eq!(az[0], 0.0);
    }

    #[test]
    fn test_mass_soa_storage_basic() {
        let mut storage = MassSoAStorage::new();
        let e1 = Entity::new(1, 0);
        let e2 = Entity::new(2, 0);
        
        storage.insert(e1, Mass::new(10.0));
        storage.insert(e2, Mass::immovable());
        
        assert_eq!(storage.len(), 2);
        
        let arrays = storage.field_arrays().unwrap();
        let values = arrays.as_mass_array();
        assert_eq!(values[0], 10.0);
        assert_eq!(values[1], 0.0);
    }

    #[test]
    fn test_mass_soa_storage_field_arrays_mut() {
        let mut storage = MassSoAStorage::new();
        let entity = Entity::new(1, 0);
        
        storage.insert(entity, Mass::new(5.0));
        
        // Mutate via field arrays
        {
            let mut arrays = storage.field_arrays_mut().unwrap();
            let values = arrays.as_mass_array_mut();
            values[0] = 10.0;
        }
        
        // Verify mutation
        let arrays = storage.field_arrays().unwrap();
        let values = arrays.as_mass_array();
        assert_eq!(values[0], 10.0);
    }

    #[test]
    fn test_soa_storage_swap_remove() {
        let mut storage = PositionSoAStorage::new();
        
        let e1 = Entity::new(1, 0);
        let e2 = Entity::new(2, 0);
        let e3 = Entity::new(3, 0);
        
        storage.insert(e1, Position::new(1.0, 2.0, 3.0));
        storage.insert(e2, Position::new(4.0, 5.0, 6.0));
        storage.insert(e3, Position::new(7.0, 8.0, 9.0));
        
        // Remove middle element (should swap with last)
        storage.remove(e2);
        
        assert_eq!(storage.len(), 2);
        assert!(storage.contains(e1));
        assert!(!storage.contains(e2));
        assert!(storage.contains(e3));
        
        // Verify data integrity after swap
        let arrays = storage.field_arrays().unwrap();
        let (x, _y, _z) = arrays.as_position_arrays();
        assert_eq!(x.len(), 2);
    }

    #[test]
    fn test_soa_storage_update_existing() {
        let mut storage = PositionSoAStorage::new();
        let entity = Entity::new(1, 0);
        
        storage.insert(entity, Position::new(1.0, 2.0, 3.0));
        storage.insert(entity, Position::new(10.0, 20.0, 30.0));
        
        // Should update in place, not add new entry
        assert_eq!(storage.len(), 1);
        
        let arrays = storage.field_arrays().unwrap();
        let (x, y, z) = arrays.as_position_arrays();
        assert_eq!(x[0], 10.0);
        assert_eq!(y[0], 20.0);
        assert_eq!(z[0], 30.0);
    }

    #[test]
    fn test_new_soa_storage_clear() {
        let mut storage = VelocitySoAStorage::new();
        
        for i in 0..10 {
            storage.insert(Entity::new(i, 0), Velocity::new(i as f64, 0.0, 0.0));
        }
        
        assert_eq!(storage.len(), 10);
        storage.clear();
        assert_eq!(storage.len(), 0);
        assert!(storage.is_empty());
    }

    #[test]
    fn test_new_soa_storage_entity_generations() {
        let mut storage = PositionSoAStorage::new();
        let e1_gen0 = Entity::new(1, 0);
        let e1_gen1 = Entity::new(1, 1);
        
        storage.insert(e1_gen0, Position::new(1.0, 2.0, 3.0));
        assert!(storage.contains(e1_gen0));
        assert!(!storage.contains(e1_gen1)); // Different generation
        
        storage.remove(e1_gen0);
        assert!(!storage.contains(e1_gen0));
        
        storage.insert(e1_gen1, Position::new(10.0, 20.0, 30.0));
        assert!(!storage.contains(e1_gen0)); // Old generation not present
        assert!(storage.contains(e1_gen1));
    }

    #[test]
    fn test_soa_storage_large_scale() {
        let mut storage = PositionSoAStorage::with_capacity(1000);
        
        // Insert 1000 entities
        for i in 0..1000 {
            storage.insert(Entity::new(i, 0), Position::new(i as f64, 0.0, 0.0));
        }
        
        assert_eq!(storage.len(), 1000);
        
        // Verify contiguous field arrays
        let arrays = storage.field_arrays().unwrap();
        let (x, _y, _z) = arrays.as_position_arrays();
        assert_eq!(x.len(), 1000);
        
        // Remove half
        for i in (0..1000).step_by(2) {
            storage.remove(Entity::new(i, 0));
        }
        
        assert_eq!(storage.len(), 500);
    }

    #[test]
    fn test_soa_vs_hashmap_apis() {
        // Verify that both storage types implement ComponentStorage
        fn test_storage<S: ComponentStorage<Component = Position>>(mut storage: S) {
            let entity = Entity::new(1, 0);
            storage.insert(entity, Position::new(1.0, 2.0, 3.0));
            assert!(storage.contains(entity));
            storage.clear();
        }
        
        test_storage(HashMapStorage::<Position>::new());
        test_storage(PositionSoAStorage::new());
    }

    #[test]
    fn test_field_arrays_immutable_borrow() {
        let mut storage = PositionSoAStorage::new();
        storage.insert(Entity::new(1, 0), Position::new(1.0, 2.0, 3.0));
        storage.insert(Entity::new(2, 0), Position::new(4.0, 5.0, 6.0));
        
        // Multiple immutable borrows are OK
        let arrays1 = storage.field_arrays().unwrap();
        let arrays2 = storage.field_arrays().unwrap();
        
        let (x1, _, _) = arrays1.as_position_arrays();
        let (x2, _, _) = arrays2.as_position_arrays();
        
        assert_eq!(x1.len(), 2);
        assert_eq!(x2.len(), 2);
    }
}

