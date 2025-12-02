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
//! Entity management
//!
//! Entities are unique identifiers in the ECS that represent game objects.
//! They are lightweight handles that tie together components.

use std::fmt;

/// Unique identifier for an entity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(u64);

impl EntityId {
    /// Create a new EntityId from a raw u64 value
    pub fn new(id: u64) -> Self {
        EntityId(id)
    }

    /// Get the raw u64 value
    pub fn raw(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entity({})", self.0)
    }
}

/// Entity handle with generational index support for safe references
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    id: EntityId,
    generation: u32,
}

impl Entity {
    /// Create a new entity with the given ID and generation
    pub fn new(id: u64, generation: u32) -> Self {
        Entity {
            id: EntityId::new(id),
            generation,
        }
    }

    /// Get the entity ID
    pub fn id(&self) -> EntityId {
        self.id
    }

    /// Get the generation number
    pub fn generation(&self) -> u32 {
        self.generation
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entity({}, gen: {})", self.id.0, self.generation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        let entity = Entity::new(42, 1);
        assert_eq!(entity.id().raw(), 42);
        assert_eq!(entity.generation(), 1);
    }

    #[test]
    fn test_entity_equality() {
        let e1 = Entity::new(1, 0);
        let e2 = Entity::new(1, 0);
        let e3 = Entity::new(1, 1);
        assert_eq!(e1, e2);
        assert_ne!(e1, e3);
    }
}
