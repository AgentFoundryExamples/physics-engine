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
//! Plugin API traits and context for custom extensions
//!
//! This module defines the plugin system API that allows users to create
//! custom objects, forces, and constraints without modifying the core engine.
//!
//! # Safety Contracts
//!
//! Plugins must:
//! - Not access or modify forbidden components without proper permissions
//! - Ensure thread-safety when implementing Send + Sync
//! - Validate all inputs and handle errors gracefully
//! - Not create circular dependencies with other plugins

use crate::ecs::{Entity, ComponentStorage, World};
use crate::ecs::components::{Position, Velocity, Mass};
use std::any::Any;

#[cfg(feature = "parallel")]
use rayon::ThreadPool;

/// Version of the plugin API
///
/// This version must match between the engine and plugins to ensure compatibility.
/// Format: MAJOR.MINOR.PATCH following semantic versioning.
pub const PLUGIN_API_VERSION: &str = "0.1.0";

/// Context provided to plugins with scoped access to engine internals
///
/// The plugin context exposes safe, controlled access to the ECS world,
/// integrator configuration, and parallel execution utilities without
/// allowing unsafe or unrestricted modifications.
///
/// # Safety Guarantees
///
/// - Immutable world access prevents data races
/// - Component access is type-checked at compile time
/// - Thread pool handle is read-only, preventing thread creation
/// - No unsafe pointers or raw memory access exposed
pub struct PluginContext<'a> {
    /// Immutable reference to the ECS world
    world: &'a World,
    /// Name of the active integrator
    integrator_name: &'a str,
    /// Current simulation timestep
    timestep: f64,
    #[cfg(feature = "parallel")]
    /// Handle to the Rayon thread pool (if parallel feature enabled)
    thread_pool: Option<&'a ThreadPool>,
}

impl<'a> PluginContext<'a> {
    /// Create a new plugin context
    ///
    /// This is only callable by the engine, not by plugins.
    pub(crate) fn new(
        world: &'a World,
        integrator_name: &'a str,
        timestep: f64,
        #[cfg(feature = "parallel")] thread_pool: Option<&'a ThreadPool>,
    ) -> Self {
        PluginContext {
            world,
            integrator_name,
            timestep,
            #[cfg(feature = "parallel")]
            thread_pool,
        }
    }

    /// Get immutable access to the ECS world
    pub fn world(&self) -> &World {
        self.world
    }

    /// Get the name of the currently active integrator
    pub fn integrator_name(&self) -> &str {
        self.integrator_name
    }

    /// Get the current simulation timestep
    pub fn timestep(&self) -> f64 {
        self.timestep
    }

    /// Get the number of threads available for parallel execution
    ///
    /// Returns 1 if parallel feature is disabled or no thread pool is configured.
    pub fn thread_count(&self) -> usize {
        #[cfg(feature = "parallel")]
        {
            self.thread_pool
                .map(|pool| pool.current_num_threads())
                .unwrap_or(1)
        }
        #[cfg(not(feature = "parallel"))]
        {
            1
        }
    }

    /// Check if parallel execution is enabled
    pub fn is_parallel_enabled(&self) -> bool {
        #[cfg(feature = "parallel")]
        {
            self.thread_pool.is_some()
        }
        #[cfg(not(feature = "parallel"))]
        {
            false
        }
    }

    /// Get a snapshot of all entities in the world
    ///
    /// Returns a vector containing all entity IDs currently in the world.
    /// This is useful for N-body simulations where forces depend on all entities.
    ///
    /// # Performance Note
    ///
    /// This creates a snapshot allocation. For performance-critical code,
    /// consider caching the entity list if it doesn't change frequently.
    pub fn get_entities(&self) -> Vec<Entity> {
        self.world.entities().copied().collect()
    }
}

/// Lifecycle hooks for plugins
///
/// Plugins implement these methods to initialize, update, and clean up
/// their state during engine execution.
pub trait Plugin: Send + Sync {
    /// Get the name of this plugin
    ///
    /// Must be unique across all registered plugins.
    fn name(&self) -> &str;

    /// Get the version of this plugin
    ///
    /// Should follow semantic versioning (MAJOR.MINOR.PATCH).
    fn version(&self) -> &str;

    /// Get the plugin API version this plugin was built against
    ///
    /// Used for compatibility checking. Should return PLUGIN_API_VERSION.
    fn api_version(&self) -> &str {
        PLUGIN_API_VERSION
    }

    /// Get the list of plugin names this plugin depends on
    ///
    /// The engine will ensure dependencies are loaded before this plugin.
    /// Circular dependencies will be detected and result in an error.
    fn dependencies(&self) -> Vec<&str> {
        Vec::new()
    }

    /// Initialize the plugin
    ///
    /// Called once when the plugin is registered, before any update calls.
    /// Can be used to set up internal state or validate configuration.
    ///
    /// # Errors
    ///
    /// Returns an error message if initialization fails.
    fn initialize(&mut self, _context: &PluginContext) -> Result<(), String> {
        Ok(())
    }

    /// Update the plugin state
    ///
    /// Called each simulation frame to allow the plugin to update its state.
    /// This is called after initialization and before shutdown.
    fn update(&mut self, _context: &PluginContext) -> Result<(), String> {
        Ok(())
    }

    /// Shutdown the plugin
    ///
    /// Called once when the plugin is unregistered or the engine shuts down.
    /// Can be used to clean up resources or persist state.
    fn shutdown(&mut self) -> Result<(), String> {
        Ok(())
    }

    /// Allow downcasting to concrete plugin types
    ///
    /// This enables type-safe access to plugin-specific functionality.
    fn as_any(&self) -> &dyn Any;

    /// Allow mutable downcasting to concrete plugin types
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Factory for creating custom objects with components
///
/// ObjectFactory plugins can spawn entities with pre-configured components,
/// enabling users to create custom object types (e.g., planets, springs, ragdolls).
///
/// # Example
///
/// ```rust,ignore
/// struct PlanetFactory {
///     default_mass: f64,
/// }
///
/// impl ObjectFactory for PlanetFactory {
///     fn create_object(&self, world: &mut World) -> Result<Entity, String> {
///         let entity = world.create_entity();
///         // Add components for a planet...
///         Ok(entity)
///     }
/// }
/// ```
pub trait ObjectFactory: Plugin {
    /// Create a new object in the world
    ///
    /// Spawns an entity and attaches the necessary components for this object type.
    ///
    /// # Arguments
    ///
    /// * `world` - Mutable access to the ECS world for entity creation
    ///
    /// # Returns
    ///
    /// The created entity ID on success, or an error message on failure.
    ///
    /// # Safety
    ///
    /// Must not store references to the world or its components.
    fn create_object(&self, world: &mut World) -> Result<Entity, String>;
}

/// Provider for custom force implementations
///
/// ForceProvider plugins can compute arbitrary forces based on entity state,
/// enabling gravity, springs, drag, electromagnetic forces, and more.
///
/// Note: The existing ForceProvider trait from systems.rs is used directly.
/// This trait extends it with plugin lifecycle management.
///
/// # Example
///
/// ```rust,ignore
/// struct GravityPlugin {
///     gravity: f64,
/// }
///
/// impl Plugin for GravityPlugin { /* ... */ }
///
/// impl ForceProvider for GravityPlugin {
///     fn compute_force(&self, entity: Entity, registry: &ForceRegistry) -> Option<Force> {
///         // Compute gravitational force...
///         Some(Force::new(0.0, -9.81 * mass, 0.0))
///     }
///     
///     fn name(&self) -> &str {
///         "gravity"
///     }
/// }
/// ```
pub trait ForceProviderPlugin: Plugin + crate::ecs::systems::ForceProvider {
    /// Get a reference to self as a ForceProvider trait object
    ///
    /// This allows the plugin to be registered with the ForceRegistry.
    fn as_force_provider(&self) -> &dyn crate::ecs::systems::ForceProvider;
}

/// Provider for forces that depend on all entities in the world
///
/// WorldAwareForceProvider extends ForceProvider for cases where force computation
/// requires knowledge of all entities, such as N-body gravitational simulations.
/// This trait provides a specialized interface that can be more efficient than
/// the per-entity ForceProvider interface.
///
/// # Example
///
/// ```rust,ignore
/// struct NBodyGravityPlugin {
///     g_constant: f64,
/// }
///
/// impl WorldAwareForceProvider for NBodyGravityPlugin {
///     fn compute_forces_for_world(
///         &self,
///         entities: &[Entity],
///         world: &World,
///         force_registry: &mut ForceRegistry,
///     ) -> Result<usize, String> {
///         // Compute all pairwise gravitational forces efficiently
///         // Register computed forces with the registry
///         Ok(entities.len())
///     }
/// }
/// ```
pub trait WorldAwareForceProvider: Plugin {
    /// Compute forces for all entities in the world
    ///
    /// This method is called to compute forces that depend on the global state
    /// of all entities. The implementation should compute forces and register
    /// them with the provided force registry.
    ///
    /// # Arguments
    ///
    /// * `entities` - Slice of all entities to consider
    /// * `world` - Immutable reference to the world for component access
    /// * `force_registry` - Mutable registry to accumulate computed forces
    ///
    /// # Returns
    ///
    /// Number of entities that had forces computed, or error message on failure
    ///
    /// # Performance
    ///
    /// Implementations should use parallel computation when possible via
    /// `PluginContext::thread_count()` and `is_parallel_enabled()`.
    fn compute_forces_for_world(
        &self,
        entities: &[Entity],
        world: &World,
        force_registry: &mut crate::ecs::systems::ForceRegistry,
    ) -> Result<usize, String>;
}

/// System for applying custom constraints
///
/// ConstraintSystem plugins can enforce geometric or physical constraints,
/// such as joints, distance limits, collision response, and contact resolution.
///
/// # Safety Contracts
///
/// - Must not create infinite loops or deadlocks
/// - Must validate all entity references before access
/// - Must handle missing or invalid components gracefully
/// - Must be deterministic for reproducible simulations
///
/// # Example
///
/// ```rust,ignore
/// struct DistanceConstraint {
///     entity_a: Entity,
///     entity_b: Entity,
///     distance: f64,
/// }
///
/// impl ConstraintSystem for DistanceConstraint {
///     fn apply_constraint(&mut self, positions: &mut ComponentStorage<Position>) -> Result<(), String> {
///         // Enforce distance constraint between entities...
///         Ok(())
///     }
/// }
/// ```
pub trait ConstraintSystem: Plugin {
    /// Apply the constraint to entities
    ///
    /// Modifies component state to satisfy the constraint. Called during the
    /// constraint resolution stage of the simulation pipeline.
    ///
    /// # Arguments
    ///
    /// * `positions` - Mutable access to position components
    /// * `velocities` - Mutable access to velocity components
    /// * `masses` - Immutable access to mass components
    ///
    /// # Returns
    ///
    /// Ok(()) if constraint was applied successfully, Err with message on failure.
    ///
    /// # Safety
    ///
    /// Must not store references to component storage beyond this call.
    fn apply_constraint(
        &mut self,
        positions: &mut dyn ComponentStorage<Component = Position>,
        velocities: &mut dyn ComponentStorage<Component = Velocity>,
        masses: &dyn ComponentStorage<Component = Mass>,
    ) -> Result<(), String>;

    /// Get the priority of this constraint
    ///
    /// Constraints are applied in ascending priority order. Lower values run first.
    /// Default priority is 100.
    fn priority(&self) -> i32 {
        100
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin {
        name: String,
        initialized: bool,
    }

    impl TestPlugin {
        fn new(name: &str) -> Self {
            TestPlugin {
                name: name.to_string(),
                initialized: false,
            }
        }
    }

    impl Plugin for TestPlugin {
        fn name(&self) -> &str {
            &self.name
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        fn initialize(&mut self, _context: &PluginContext) -> Result<(), String> {
            self.initialized = true;
            Ok(())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    #[test]
    fn test_plugin_api_version() {
        assert_eq!(PLUGIN_API_VERSION, "0.1.0");
    }

    #[test]
    fn test_plugin_creation() {
        let plugin = TestPlugin::new("test");
        assert_eq!(plugin.name(), "test");
        assert_eq!(plugin.version(), "1.0.0");
        assert_eq!(plugin.api_version(), PLUGIN_API_VERSION);
        assert!(!plugin.initialized);
    }

    #[test]
    fn test_plugin_dependencies() {
        let plugin = TestPlugin::new("test");
        assert_eq!(plugin.dependencies().len(), 0);
    }

    #[test]
    fn test_plugin_context() {
        let world = World::new();
        let integrator_name = "verlet";
        let timestep = 0.016;

        #[cfg(feature = "parallel")]
        let context = PluginContext::new(&world, integrator_name, timestep, None);
        #[cfg(not(feature = "parallel"))]
        let context = PluginContext::new(&world, integrator_name, timestep);

        assert_eq!(context.integrator_name(), "verlet");
        assert_eq!(context.timestep(), 0.016);
        assert!(context.thread_count() >= 1);
    }

    #[test]
    fn test_plugin_initialization() {
        let world = World::new();
        let integrator_name = "verlet";
        let timestep = 0.016;

        #[cfg(feature = "parallel")]
        let context = PluginContext::new(&world, integrator_name, timestep, None);
        #[cfg(not(feature = "parallel"))]
        let context = PluginContext::new(&world, integrator_name, timestep);

        let mut plugin = TestPlugin::new("test");
        assert!(!plugin.initialized);

        plugin.initialize(&context).unwrap();
        assert!(plugin.initialized);
    }

    #[test]
    fn test_plugin_downcasting() {
        let mut plugin = TestPlugin::new("test");
        
        // Test immutable downcast
        let any = plugin.as_any();
        let downcasted = any.downcast_ref::<TestPlugin>();
        assert!(downcasted.is_some());
        assert_eq!(downcasted.unwrap().name(), "test");

        // Test mutable downcast
        let any_mut = plugin.as_any_mut();
        let downcasted_mut = any_mut.downcast_mut::<TestPlugin>();
        assert!(downcasted_mut.is_some());
    }

    #[test]
    fn test_plugin_context_get_entities() {
        let mut world = World::new();
        let e1 = world.create_entity();
        let e2 = world.create_entity();
        let e3 = world.create_entity();
        
        let integrator_name = "verlet";
        let timestep = 0.016;

        #[cfg(feature = "parallel")]
        let context = PluginContext::new(&world, integrator_name, timestep, None);
        #[cfg(not(feature = "parallel"))]
        let context = PluginContext::new(&world, integrator_name, timestep);

        let entities = context.get_entities();
        assert_eq!(entities.len(), 3);
        assert!(entities.contains(&e1));
        assert!(entities.contains(&e2));
        assert!(entities.contains(&e3));
    }
}
