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
//! Gravitational force plugin implementing Newton's law of universal gravitation
//!
//! This plugin computes pairwise gravitational forces between all entities
//! with mass and position components using parallel computation.
//!
//! # Physics Background
//!
//! Newton's law of universal gravitation states that every point mass attracts
//! every other point mass with a force proportional to the product of their
//! masses and inversely proportional to the square of the distance between them:
//!
//! **F = G * (m₁ * m₂) / r²**
//!
//! Where:
//! - F is the magnitude of the gravitational force
//! - G is the gravitational constant (6.674 × 10⁻¹¹ N⋅m²/kg²)
//! - m₁ and m₂ are the masses of the two objects
//! - r is the distance between the centers of mass
//!
//! # References
//!
//! - Newton, I. (1687). "Philosophiæ Naturalis Principia Mathematica"
//! - [CODATA 2018 value for G](https://physics.nist.gov/cgi-bin/cuu/Value?bg)
//! - Goldstein, H., Poole, C., & Safko, J. (2002). "Classical Mechanics" (3rd ed.)
//!
//! # Implementation Details
//!
//! ## Softening Factor
//!
//! To prevent singularities when particles are very close or occupy the same
//! position, a softening factor ε (epsilon) is added to the distance:
//!
//! **F = G * (m₁ * m₂) / (r² + ε²)**
//!
//! This is a standard technique in N-body simulations. See:
//! - Dehnen, W. (2001). "Towards optimal softening in three-dimensional N-body codes"
//! - Aarseth, S. J. (2003). "Gravitational N-Body Simulations"
//!
//! ## Parallel Computation
//!
//! For N bodies, we need to compute N*(N-1)/2 pairwise interactions. This
//! plugin uses Rayon to parallelize force computations across entities,
//! splitting work into chunks for efficient parallel processing.
//!
//! ## Numerical Stability
//!
//! - Zero-length vectors are detected and result in zero force
//! - Negative masses are rejected during configuration
//! - Force magnitudes are validated to be finite
//! - Softening prevents division by extremely small numbers

use crate::ecs::{Entity, ComponentStorage};
use crate::ecs::components::{Position, Mass};
use crate::ecs::systems::{Force, ForceRegistry, ForceProvider};
use crate::plugins::{Plugin, ForceProviderPlugin, PluginContext};
use std::any::Any;
use std::sync::Arc;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Standard gravitational constant in SI units (m³/(kg⋅s²))
///
/// CODATA 2018 recommended value: 6.67430(15) × 10⁻¹¹ m³/(kg⋅s²)
/// Source: https://physics.nist.gov/cgi-bin/cuu/Value?bg
pub const GRAVITATIONAL_CONSTANT: f64 = 6.67430e-11;

/// Default softening factor to prevent singularities (meters)
///
/// This value is chosen to be small enough not to affect typical planetary
/// simulations while preventing numerical issues when particles get very close.
pub const DEFAULT_SOFTENING: f64 = 1e3; // 1 km

/// Gravitational force plugin configuration
///
/// Implements Newton's law of universal gravitation with configurable
/// parameters for gravitational constant, softening, and performance tuning.
///
/// # Example
///
/// ```rust,no_run
/// use physics_engine::plugins::gravity::{GravityPlugin, GRAVITATIONAL_CONSTANT};
///
/// // For solar system simulation with realistic physics
/// let mut gravity = GravityPlugin::new(GRAVITATIONAL_CONSTANT);
/// gravity.set_softening(1e3); // 1 km softening
///
/// // For scaled simulation (e.g., demonstration)
/// let scaled = GravityPlugin::with_scaled_g(1e-3);
/// ```
#[derive(Clone)]
pub struct GravityPlugin {
    /// Gravitational constant (default: GRAVITATIONAL_CONSTANT)
    g_constant: f64,
    /// Softening factor to prevent singularities (default: DEFAULT_SOFTENING)
    softening: f64,
    /// Chunk size for parallel processing (0 = auto)
    chunk_size: usize,
    /// Whether to warn about invalid calculations
    warn_on_invalid: bool,
    /// Maximum expected force magnitude (for high-force warning suppression)
    max_expected_force: f64,
    /// Whether to warn about high forces exceeding max_expected_force
    warn_on_high_forces: bool,
}

impl GravityPlugin {
    /// Create a new gravity plugin with the specified gravitational constant
    ///
    /// # Arguments
    ///
    /// * `g_constant` - Gravitational constant in SI units (m³/(kg⋅s²))
    ///   Use `GRAVITATIONAL_CONSTANT` for realistic physics simulations.
    ///
    /// # Panics
    ///
    /// Panics if `g_constant` is negative or not finite.
    pub fn new(g_constant: f64) -> Self {
        assert!(
            g_constant >= 0.0 && g_constant.is_finite(),
            "Gravitational constant must be non-negative and finite"
        );

        GravityPlugin {
            g_constant,
            softening: DEFAULT_SOFTENING,
            chunk_size: 0, // Auto-determine based on thread count
            warn_on_invalid: true,
            max_expected_force: 1e10, // 10 billion Newtons default
            warn_on_high_forces: true,
        }
    }

    /// Create a gravity plugin with a scaled gravitational constant
    ///
    /// Useful for demonstration simulations where realistic G is too small.
    pub fn with_scaled_g(scale_factor: f64) -> Self {
        Self::new(GRAVITATIONAL_CONSTANT * scale_factor)
    }

    /// Create a gravity plugin with default settings (standard G)
    pub fn default_settings() -> Self {
        Self::new(GRAVITATIONAL_CONSTANT)
    }

    /// Set the softening factor
    ///
    /// The softening factor prevents singularities when particles are very
    /// close together. Typical values range from 0 (no softening) to 1e6 m.
    ///
    /// # Arguments
    ///
    /// * `softening` - Softening distance in meters
    ///
    /// # Panics
    ///
    /// Panics if `softening` is negative or not finite.
    pub fn set_softening(&mut self, softening: f64) {
        assert!(
            softening >= 0.0 && softening.is_finite(),
            "Softening factor must be non-negative and finite"
        );
        self.softening = softening;
    }

    /// Get the current softening factor
    pub fn softening(&self) -> f64 {
        self.softening
    }

    /// Set the chunk size for parallel processing
    ///
    /// Set to 0 for automatic determination based on thread count.
    /// Larger chunks reduce scheduling overhead but may cause load imbalance.
    pub fn set_chunk_size(&mut self, size: usize) {
        self.chunk_size = size;
    }

    /// Set whether to warn about invalid force calculations
    pub fn set_warn_on_invalid(&mut self, warn: bool) {
        self.warn_on_invalid = warn;
    }

    /// Set the maximum expected force magnitude
    ///
    /// Forces exceeding this value will trigger warnings if `warn_on_high_forces` is enabled.
    /// This is useful for detecting unexpected simulation behavior while suppressing
    /// warnings for expected high-force scenarios (e.g., close planetary encounters).
    ///
    /// # Arguments
    ///
    /// * `max_force` - Maximum expected force magnitude in Newtons
    ///
    /// # Panics
    ///
    /// Panics if `max_force` is negative or not finite.
    pub fn set_max_expected_force(&mut self, max_force: f64) {
        assert!(
            max_force >= 0.0 && max_force.is_finite(),
            "Maximum expected force must be non-negative and finite"
        );
        self.max_expected_force = max_force;
    }

    /// Get the current maximum expected force magnitude
    pub fn max_expected_force(&self) -> f64 {
        self.max_expected_force
    }

    /// Set whether to warn about high forces exceeding max_expected_force
    pub fn set_warn_on_high_forces(&mut self, warn: bool) {
        self.warn_on_high_forces = warn;
    }

    /// Check if high force warnings are enabled
    pub fn warn_on_high_forces(&self) -> bool {
        self.warn_on_high_forces
    }

    /// Compute gravitational force between two entities
    ///
    /// Returns None if either entity is missing required components or if
    /// the force calculation fails validation.
    fn compute_pairwise_force(
        &self,
        entity1: Entity,
        entity2: Entity,
        positions: &impl ComponentStorage<Component = Position>,
        masses: &impl ComponentStorage<Component = Mass>,
    ) -> Option<Force> {
        // Get components for both entities
        let pos1 = positions.get(entity1)?;
        let pos2 = positions.get(entity2)?;
        let mass1 = masses.get(entity1)?;
        let mass2 = masses.get(entity2)?;

        // Skip immovable bodies (they don't experience forces)
        if mass1.is_immovable() {
            return None;
        }

        // Calculate displacement vector from entity1 to entity2
        let dx = pos2.x() - pos1.x();
        let dy = pos2.y() - pos1.y();
        let dz = pos2.z() - pos1.z();

        // Calculate distance squared with softening
        let r_squared = dx * dx + dy * dy + dz * dz;
        let softened_r_squared = r_squared + self.softening * self.softening;

        // Avoid division by exactly zero (though softening should prevent this)
        if softened_r_squared == 0.0 {
            if self.warn_on_invalid {
                eprintln!("Warning: Zero distance between {:?} and {:?}", entity1, entity2);
            }
            return None;
        }

        // Calculate force magnitude: F = G * m1 * m2 / (r² + ε²)
        let force_magnitude = self.g_constant * mass1.value() * mass2.value() / softened_r_squared;

        // Validate force magnitude
        if !force_magnitude.is_finite() {
            if self.warn_on_invalid {
                eprintln!(
                    "Warning: Invalid force magnitude between {:?} and {:?}",
                    entity1, entity2
                );
            }
            return None;
        }

        // Check for unexpectedly high forces
        if self.warn_on_high_forces && force_magnitude > self.max_expected_force {
            eprintln!(
                "Warning: High force magnitude {:.2e} N exceeds expected maximum {:.2e} N between {:?} and {:?}",
                force_magnitude, self.max_expected_force, entity1, entity2
            );
        }

        // Calculate force direction (unit vector * magnitude / distance)
        // F_vec = F_mag * (r_vec / |r|) = F_mag * r_vec / |r|
        // Since F_mag = G*m1*m2/(r²+ε²), we need the unit vector: r_vec/|r|
        // Where |r| = sqrt(r²+ε²) when using softening
        // So: F_vec = [G*m1*m2/(r²+ε²)] * r_vec / sqrt(r²+ε²)
        //           = G*m1*m2 * r_vec / (r²+ε²)^(3/2)
        let r = softened_r_squared.sqrt();
        let force_scale = force_magnitude / r;

        let fx = force_scale * dx;
        let fy = force_scale * dy;
        let fz = force_scale * dz;

        // Final validation
        if !fx.is_finite() || !fy.is_finite() || !fz.is_finite() {
            if self.warn_on_invalid {
                eprintln!(
                    "Warning: Invalid force components between {:?} and {:?}",
                    entity1, entity2
                );
            }
            return None;
        }

        Some(Force::new(fx, fy, fz))
    }

    /// Compute total gravitational force on an entity from all other entities
    ///
    /// This is called by the force registry to accumulate forces for each entity.
    fn compute_force_for_entity(
        &self,
        entity: Entity,
        positions: &impl ComponentStorage<Component = Position>,
        masses: &impl ComponentStorage<Component = Mass>,
        all_entities: &[Entity],
    ) -> Option<Force> {
        let mut total_force = Force::zero();
        let mut has_force = false;

        // Compute pairwise forces with all other entities
        for &other_entity in all_entities {
            // Skip self-interaction
            if other_entity == entity {
                continue;
            }

            if let Some(force) = self.compute_pairwise_force(entity, other_entity, positions, masses) {
                total_force.add(&force);
                has_force = true;
            }
        }

        if has_force {
            Some(total_force)
        } else {
            None
        }
    }
}

impl Plugin for GravityPlugin {
    fn name(&self) -> &str {
        "gravity"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn initialize(&mut self, context: &PluginContext) -> Result<(), String> {
        // Auto-configure chunk size based on thread count
        if self.chunk_size == 0 {
            let threads = context.thread_count();
            // Rule of thumb: aim for at least 4 chunks per thread for load balancing
            self.chunk_size = (threads * 4).max(1);
        }

        Ok(())
    }
}

impl ForceProvider for GravityPlugin {
    fn compute_force(&self, _entity: Entity, _registry: &ForceRegistry) -> Option<Force> {
        // NOTE: This implementation returns None because gravitational forces require
        // knowledge of ALL entities in the system (N-body problem). The generic
        // ForceProvider interface only provides access to a single entity at a time.
        //
        // Instead, use GravitySystem::compute_forces() which efficiently computes
        // all pairwise gravitational interactions in a single pass.
        //
        // This trait implementation is provided for API compatibility but is not
        // intended to be used directly. Attempting to register this plugin with
        // a ForceRegistry will not produce gravitational forces.
        None
    }

    fn name(&self) -> &str {
        "gravity"
    }
}

impl ForceProviderPlugin for GravityPlugin {
    fn as_force_provider(&self) -> &dyn ForceProvider {
        self
    }
}

impl crate::plugins::api::WorldAwareForceProvider for GravityPlugin {
    fn compute_forces_for_world(
        &self,
        _entities: &[Entity],
        _world: &crate::ecs::World,
        _force_registry: &mut ForceRegistry,
    ) -> Result<usize, String> {
        // Create a GravitySystem with this plugin and use it to compute forces
        let _system = GravitySystem::new_from_plugin(Arc::new(self.clone()));
        
        // We need to get component storage from the world
        // For now, we'll use the existing compute_forces method
        // In a real implementation, we'd access world's component storage directly
        
        // This is a placeholder - the actual implementation would access
        // Position and Mass components from the world
        Ok(0)
    }
}

/// Specialized system for computing gravitational forces efficiently
///
/// This provides a more efficient implementation than the generic ForceProvider
/// interface by computing all pairwise forces in a single pass.
pub struct GravitySystem {
    plugin: Arc<GravityPlugin>,
}

impl GravitySystem {
    /// Create a new gravity system with the given plugin configuration
    pub fn new(plugin: GravityPlugin) -> Self {
        GravitySystem {
            plugin: Arc::new(plugin),
        }
    }

    /// Create a new gravity system from an Arc-wrapped plugin
    ///
    /// Internal helper for WorldAwareForceProvider implementation
    fn new_from_plugin(plugin: Arc<GravityPlugin>) -> Self {
        GravitySystem { plugin }
    }

    /// Compute gravitational forces for all entities and accumulate in registry
    ///
    /// This efficiently computes N-body gravitational interactions using
    /// parallel processing when available.
    ///
    /// # Arguments
    ///
    /// * `entities` - Slice of all entities to consider
    /// * `positions` - Position component storage
    /// * `masses` - Mass component storage
    /// * `force_registry` - Registry to accumulate forces
    ///
    /// # Returns
    ///
    /// Number of entities that had gravitational forces computed
    pub fn compute_forces(
        &self,
        entities: &[Entity],
        positions: &impl ComponentStorage<Component = Position>,
        masses: &impl ComponentStorage<Component = Mass>,
        force_registry: &mut ForceRegistry,
    ) -> usize {
        #[cfg(feature = "parallel")]
        {
            self.compute_forces_parallel(entities, positions, masses, force_registry)
        }

        #[cfg(not(feature = "parallel"))]
        {
            self.compute_forces_sequential(entities, positions, masses, force_registry)
        }
    }

    #[cfg(feature = "parallel")]
    fn compute_forces_parallel(
        &self,
        entities: &[Entity],
        positions: &impl ComponentStorage<Component = Position>,
        masses: &impl ComponentStorage<Component = Mass>,
        force_registry: &mut ForceRegistry,
    ) -> usize {
        use std::collections::HashMap;

        let plugin = &self.plugin;

        // Compute forces in parallel chunks
        let chunk_size = if plugin.chunk_size > 0 {
            plugin.chunk_size
        } else {
            (entities.len() / 4).max(1)
        };

        // Use Rayon's reduce pattern to eliminate mutex contention.
        // Each thread computes forces for its chunk into a local HashMap,
        // then we reduce all HashMaps into a single one without global locking.
        let forces = entities
            .par_chunks(chunk_size)
            .map(|chunk| {
                let mut local_forces = HashMap::new();
                for &entity in chunk {
                    if let Some(force) = plugin.compute_force_for_entity(entity, positions, masses, entities) {
                        local_forces.insert(entity, force);
                    }
                }
                local_forces
            })
            .reduce(
                HashMap::new,
                |mut acc, local_forces| {
                    acc.extend(local_forces);
                    acc
                },
            );

        // Accumulate forces in registry
        let count = forces.len();
        
        for (entity, force) in forces {
            force_registry.register_provider(Box::new(SimpleForceProvider::new(entity, force)));
        }

        count
    }

    #[cfg(not(feature = "parallel"))]
    fn compute_forces_sequential(
        &self,
        entities: &[Entity],
        positions: &impl ComponentStorage<Component = Position>,
        masses: &impl ComponentStorage<Component = Mass>,
        force_registry: &mut ForceRegistry,
    ) -> usize {
        let plugin = &self.plugin;
        let mut count = 0;

        for &entity in entities {
            if let Some(force) = plugin.compute_force_for_entity(entity, positions, masses, entities) {
                force_registry.register_provider(Box::new(SimpleForceProvider::new(entity, force)));
                count += 1;
            }
        }

        count
    }
}

/// Simple force provider that returns a pre-computed force for a specific entity
struct SimpleForceProvider {
    target_entity: Entity,
    force: Force,
}

impl SimpleForceProvider {
    fn new(entity: Entity, force: Force) -> Self {
        SimpleForceProvider {
            target_entity: entity,
            force,
        }
    }
}

impl ForceProvider for SimpleForceProvider {
    fn compute_force(&self, entity: Entity, _registry: &ForceRegistry) -> Option<Force> {
        if entity == self.target_entity {
            Some(self.force)
        } else {
            None
        }
    }

    fn name(&self) -> &str {
        "simple_force"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::{World, HashMapStorage};

    #[test]
    fn test_gravitational_constant() {
        // Verify the constant is in the right ballpark
        assert!(GRAVITATIONAL_CONSTANT > 6.6e-11);
        assert!(GRAVITATIONAL_CONSTANT < 6.7e-11);
    }

    #[test]
    fn test_plugin_creation() {
        use crate::plugins::Plugin;
        
        let plugin = GravityPlugin::new(GRAVITATIONAL_CONSTANT);
        assert_eq!(Plugin::name(&plugin), "gravity");
        assert_eq!(plugin.version(), "1.0.0");
        assert_eq!(plugin.g_constant, GRAVITATIONAL_CONSTANT);
        assert_eq!(plugin.softening(), DEFAULT_SOFTENING);
    }

    #[test]
    fn test_plugin_with_scaled_g() {
        let plugin = GravityPlugin::with_scaled_g(1e10);
        assert!(plugin.g_constant > GRAVITATIONAL_CONSTANT);
    }

    #[test]
    #[should_panic(expected = "Gravitational constant must be non-negative and finite")]
    fn test_negative_g_panics() {
        GravityPlugin::new(-1.0);
    }

    #[test]
    #[should_panic(expected = "Softening factor must be non-negative and finite")]
    fn test_negative_softening_panics() {
        let mut plugin = GravityPlugin::new(GRAVITATIONAL_CONSTANT);
        plugin.set_softening(-1.0);
    }

    #[test]
    fn test_pairwise_force_calculation() {
        let plugin = GravityPlugin::new(GRAVITATIONAL_CONSTANT);
        let mut world = World::new();
        
        let entity1 = world.create_entity();
        let entity2 = world.create_entity();

        let mut positions = HashMapStorage::<Position>::new();
        let mut masses = HashMapStorage::<Mass>::new();

        // Place two 1000 kg masses 1 km apart along x-axis
        positions.insert(entity1, Position::new(0.0, 0.0, 0.0));
        positions.insert(entity2, Position::new(1000.0, 0.0, 0.0));
        masses.insert(entity1, Mass::new(1000.0));
        masses.insert(entity2, Mass::new(1000.0));

        let force = plugin.compute_pairwise_force(entity1, entity2, &positions, &masses);
        assert!(force.is_some());

        let f = force.unwrap();
        // Force should be positive x direction (toward entity2)
        assert!(f.fx > 0.0);
        assert_eq!(f.fy, 0.0);
        assert_eq!(f.fz, 0.0);
        
        // Verify magnitude is reasonable (exact value depends on softening)
        // The force should be on the order of G * m^2 / r^2
        assert!(f.magnitude() > 0.0);
        assert!(f.magnitude() < 1.0); // Should be very small for these parameters
    }

    #[test]
    fn test_zero_distance_handling() {
        let mut plugin = GravityPlugin::new(GRAVITATIONAL_CONSTANT);
        plugin.set_softening(0.0); // No softening
        plugin.set_warn_on_invalid(false); // Suppress warnings in test
        
        let mut world = World::new();
        let entity1 = world.create_entity();
        let entity2 = world.create_entity();

        let mut positions = HashMapStorage::<Position>::new();
        let mut masses = HashMapStorage::<Mass>::new();

        // Place both entities at the same position
        positions.insert(entity1, Position::new(0.0, 0.0, 0.0));
        positions.insert(entity2, Position::new(0.0, 0.0, 0.0));
        masses.insert(entity1, Mass::new(1000.0));
        masses.insert(entity2, Mass::new(1000.0));

        let force = plugin.compute_pairwise_force(entity1, entity2, &positions, &masses);
        // Should return None due to zero distance
        assert!(force.is_none());
    }

    #[test]
    fn test_softening_prevents_singularity() {
        let mut plugin = GravityPlugin::new(GRAVITATIONAL_CONSTANT);
        plugin.set_softening(100.0); // 100 m softening
        
        let mut world = World::new();
        let entity1 = world.create_entity();
        let entity2 = world.create_entity();

        let mut positions = HashMapStorage::<Position>::new();
        let mut masses = HashMapStorage::<Mass>::new();

        // Place entities 1 meter apart (much less than softening)
        positions.insert(entity1, Position::new(0.0, 0.0, 0.0));
        positions.insert(entity2, Position::new(1.0, 0.0, 0.0));
        masses.insert(entity1, Mass::new(1000.0));
        masses.insert(entity2, Mass::new(1000.0));

        let force = plugin.compute_pairwise_force(entity1, entity2, &positions, &masses);
        // With softening, should get a finite force
        assert!(force.is_some());
        
        let f = force.unwrap();
        // Force should be finite and reasonable
        assert!(f.is_valid());
        assert!(f.magnitude() > 0.0);
        // With 100m softening and 1m separation, force is dominated by softening
        // F ≈ G * m^2 / ε^2 = 6.67e-11 * 1e6 / 1e4 = 6.67e-9 N
        assert!(f.magnitude() < 1e-6); // Should be very small
    }

    #[test]
    fn test_immovable_bodies_ignored() {
        let plugin = GravityPlugin::new(GRAVITATIONAL_CONSTANT);
        let mut world = World::new();
        
        let entity1 = world.create_entity();
        let entity2 = world.create_entity();

        let mut positions = HashMapStorage::<Position>::new();
        let mut masses = HashMapStorage::<Mass>::new();

        positions.insert(entity1, Position::new(0.0, 0.0, 0.0));
        positions.insert(entity2, Position::new(1000.0, 0.0, 0.0));
        masses.insert(entity1, Mass::immovable()); // Immovable
        masses.insert(entity2, Mass::new(1000.0));

        let force = plugin.compute_pairwise_force(entity1, entity2, &positions, &masses);
        // Should return None for immovable body
        assert!(force.is_none());
    }

    #[test]
    fn test_warning_suppression_config() {
        let mut plugin = GravityPlugin::new(GRAVITATIONAL_CONSTANT);
        
        // Test default values
        assert!(plugin.warn_on_high_forces());
        assert_eq!(plugin.max_expected_force(), 1e10);
        
        // Test setting max expected force
        plugin.set_max_expected_force(1e20);
        assert_eq!(plugin.max_expected_force(), 1e20);
        
        // Test disabling warnings
        plugin.set_warn_on_high_forces(false);
        assert!(!plugin.warn_on_high_forces());
    }

    #[test]
    #[should_panic(expected = "Maximum expected force must be non-negative and finite")]
    fn test_negative_max_force_panics() {
        let mut plugin = GravityPlugin::new(GRAVITATIONAL_CONSTANT);
        plugin.set_max_expected_force(-1.0);
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn test_parallel_gravity_correctness() {
        use crate::ecs::systems::ForceRegistry;
        
        let plugin = GravityPlugin::new(GRAVITATIONAL_CONSTANT);
        let gravity_system = GravitySystem::new(plugin);
        let mut world = World::new();
        
        // Create three bodies in a line
        let e1 = world.create_entity();
        let e2 = world.create_entity();
        let e3 = world.create_entity();
        
        let mut positions = HashMapStorage::<Position>::new();
        let mut masses = HashMapStorage::<Mass>::new();
        let mut force_registry = ForceRegistry::new();
        
        // Bodies at 1000 km spacing
        positions.insert(e1, Position::new(0.0, 0.0, 0.0));
        positions.insert(e2, Position::new(1e6, 0.0, 0.0));
        positions.insert(e3, Position::new(2e6, 0.0, 0.0));
        
        masses.insert(e1, Mass::new(1e10));
        masses.insert(e2, Mass::new(1e10));
        masses.insert(e3, Mass::new(1e10));
        
        let entities = vec![e1, e2, e3];
        
        // Compute forces - this registers SimpleForceProviders
        let count = gravity_system.compute_forces(&entities, &positions, &masses, &mut force_registry);
        
        // All 3 entities should have forces computed
        assert_eq!(count, 3);
        
        // Need to accumulate forces from registered providers
        for entity in &entities {
            force_registry.accumulate_for_entity(*entity);
        }
        
        // Check that forces were accumulated
        assert!(force_registry.get_force(e1).is_some());
        assert!(force_registry.get_force(e2).is_some());
        assert!(force_registry.get_force(e3).is_some());
        
        // Middle body (e2) should have forces pulling in both directions
        let f2 = force_registry.get_force(e2).unwrap();
        // Force from e1 is negative x (pulling left)
        // Force from e3 is positive x (pulling right)
        // They should roughly cancel out due to symmetry
        assert!(f2.fx.abs() < 1e-6); // Nearly zero net force due to symmetry
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn test_parallel_large_entity_count() {
        use crate::ecs::systems::ForceRegistry;
        
        let mut plugin = GravityPlugin::new(GRAVITATIONAL_CONSTANT);
        plugin.set_warn_on_invalid(false); // Suppress warnings for test
        plugin.set_warn_on_high_forces(false);
        
        let gravity_system = GravitySystem::new(plugin);
        let mut world = World::new();
        
        // Create 100 entities to test scalability
        let mut entities = Vec::new();
        let mut positions = HashMapStorage::<Position>::new();
        let mut masses = HashMapStorage::<Mass>::new();
        
        for i in 0..100 {
            let entity = world.create_entity();
            entities.push(entity);
            
            // Spread entities in a grid pattern
            let x = (i % 10) as f64 * 1e6;
            let y = (i / 10) as f64 * 1e6;
            positions.insert(entity, Position::new(x, y, 0.0));
            masses.insert(entity, Mass::new(1e10));
        }
        
        let mut force_registry = ForceRegistry::new();
        force_registry.max_force_magnitude = 1e20; // Increase limit for test
        
        // Compute forces - this should not exhaust memory
        let count = gravity_system.compute_forces(&entities, &positions, &masses, &mut force_registry);
        
        // Should compute forces for all entities
        assert_eq!(count, 100);
    }
}
