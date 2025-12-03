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
//! Plugin system for extensible physics engine
//!
//! This module provides a comprehensive plugin API that enables users to extend
//! the physics engine with custom objects, forces, and constraints without
//! modifying the core engine code.
//!
//! # Features
//!
//! - **Plugin Traits**: Define custom object factories, force providers, and constraint systems
//! - **Static Registration**: Register plugins at compile time for zero runtime overhead
//! - **Dependency Management**: Automatic dependency resolution with circular dependency detection
//! - **Version Checking**: API version compatibility validation between engine and plugins
//! - **Safe API**: Scoped access to ECS world and parallel execution without unsafe operations
//!
//! # Plugin Types
//!
//! The plugin system supports three main plugin types:
//!
//! ## Object Factories
//!
//! Create custom object types with pre-configured components:
//!
//! ```rust,ignore
//! use physics_engine::plugins::{Plugin, ObjectFactory};
//! use physics_engine::ecs::{World, Entity};
//!
//! struct PlanetFactory {
//!     default_mass: f64,
//! }
//!
//! impl Plugin for PlanetFactory {
//!     fn name(&self) -> &str { "planet_factory" }
//!     fn version(&self) -> &str { "1.0.0" }
//!     fn as_any(&self) -> &dyn std::any::Any { self }
//!     fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
//! }
//!
//! impl ObjectFactory for PlanetFactory {
//!     fn create_object(&self, world: &mut World) -> Result<Entity, String> {
//!         let entity = world.create_entity();
//!         // Add components...
//!         Ok(entity)
//!     }
//! }
//! ```
//!
//! ## Force Providers
//!
//! Implement custom force calculations:
//!
//! ```rust,ignore
//! use physics_engine::plugins::{Plugin, ForceProviderPlugin};
//! use physics_engine::ecs::systems::{Force, ForceRegistry, ForceProvider};
//! use physics_engine::ecs::Entity;
//!
//! struct GravityPlugin {
//!     gravity: f64,
//! }
//!
//! impl Plugin for GravityPlugin {
//!     fn name(&self) -> &str { "gravity" }
//!     fn version(&self) -> &str { "1.0.0" }
//!     fn as_any(&self) -> &dyn std::any::Any { self }
//!     fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
//! }
//!
//! impl ForceProvider for GravityPlugin {
//!     fn compute_force(&self, entity: Entity, registry: &ForceRegistry) -> Option<Force> {
//!         Some(Force::new(0.0, self.gravity, 0.0))
//!     }
//!     
//!     fn name(&self) -> &str { "gravity" }
//! }
//!
//! impl ForceProviderPlugin for GravityPlugin {
//!     fn as_force_provider(&self) -> &dyn ForceProvider {
//!         self
//!     }
//! }
//! ```
//!
//! ## Constraint Systems
//!
//! Enforce physical or geometric constraints:
//!
//! ```rust,ignore
//! use physics_engine::plugins::{Plugin, ConstraintSystem};
//! use physics_engine::ecs::components::{Position, Velocity, Mass};
//! use physics_engine::ecs::ComponentStorage;
//!
//! struct DistanceConstraint {
//!     distance: f64,
//! }
//!
//! impl Plugin for DistanceConstraint {
//!     fn name(&self) -> &str { "distance_constraint" }
//!     fn version(&self) -> &str { "1.0.0" }
//!     fn as_any(&self) -> &dyn std::any::Any { self }
//!     fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
//! }
//!
//! impl ConstraintSystem for DistanceConstraint {
//!     fn apply_constraint(
//!         &mut self,
//!         positions: &mut dyn ComponentStorage<Component = Position>,
//!         velocities: &mut dyn ComponentStorage<Component = Velocity>,
//!         masses: &dyn ComponentStorage<Component = Mass>,
//!     ) -> Result<(), String> {
//!         // Apply constraint logic...
//!         Ok(())
//!     }
//! }
//! ```
//!
//! # Plugin Registration
//!
//! ## Static Registration
//!
//! Register plugins directly in your code:
//!
//! ```rust,ignore
//! use physics_engine::plugins::PluginRegistry;
//!
//! let mut registry = PluginRegistry::new();
//! registry.register(Box::new(MyPlugin::new()))?;
//!
//! // Or use the macro
//! register_plugin!(registry, MyPlugin::new());
//! ```
//!
//! ## Dependency Management
//!
//! Plugins can declare dependencies on other plugins:
//!
//! ```rust,ignore
//! impl Plugin for MyPlugin {
//!     fn dependencies(&self) -> Vec<&str> {
//!         vec!["base_physics", "collision_detection"]
//!     }
//! }
//! ```
//!
//! The registry will:
//! - Verify all dependencies are registered
//! - Initialize plugins in dependency order
//! - Detect and reject circular dependencies
//!
//! # Environment Configuration
//!
//! Set the `PHYSICS_ENGINE_PLUGIN_PATH` environment variable to specify
//! plugin search paths (currently informational only):
//!
//! ```bash
//! export PHYSICS_ENGINE_PLUGIN_PATH=/usr/local/lib/physics-plugins:/home/user/plugins
//! ```
//!
//! See `.env.example` for configuration details.
//!
//! # Safety and Best Practices
//!
//! ## API Boundaries
//!
//! - Plugins receive immutable `PluginContext` references
//! - Component access is type-checked at compile time
//! - No raw pointers or unsafe operations exposed
//! - Thread pool is read-only to prevent thread creation
//!
//! ## Performance Considerations
//!
//! - Prefer static registration over dynamic loading
//! - Minimize allocations in hot paths (force computation, constraints)
//! - Use `#[inline]` for frequently called plugin methods
//! - Consider caching expensive calculations
//!
//! ## Error Handling
//!
//! - Return descriptive error messages from plugin methods
//! - Handle missing components gracefully
//! - Validate all inputs in plugin constructors
//! - Don't panic in production code
//!
//! # Version Compatibility
//!
//! The plugin API follows semantic versioning:
//!
//! - **Major version**: Breaking API changes
//! - **Minor version**: Backward-compatible additions
//! - **Patch version**: Bug fixes only
//!
//! Plugins must declare their API version:
//!
//! ```rust,ignore
//! impl Plugin for MyPlugin {
//!     fn api_version(&self) -> &str {
//!         physics_engine::plugins::PLUGIN_API_VERSION
//!     }
//! }
//! ```
//!
//! # Examples
//!
//! See the `docs/plugins.md` guide for detailed examples and best practices.

pub mod api;
pub mod registry;
pub mod gravity;

pub use api::{
    Plugin, PluginContext, ObjectFactory, ForceProviderPlugin,
    ConstraintSystem, PLUGIN_API_VERSION,
};
pub use registry::PluginRegistry;
pub use gravity::{GravityPlugin, GravitySystem, GRAVITATIONAL_CONSTANT};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify all expected types are exported
        let _version: &str = PLUGIN_API_VERSION;
    }
}
