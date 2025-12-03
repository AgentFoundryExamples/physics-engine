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
//! Newtonian physics systems
//!
//! This module provides systems for force accumulation and Newtonian mechanics.
//! Systems are designed to be generic and configurable via plugins rather than
//! hardcoding specific simulation constants.

use crate::ecs::{Entity, ComponentStorage};
use crate::ecs::components::{Acceleration, Mass, Velocity};
use std::collections::HashMap;

/// Represents a 3D force vector
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Force {
    /// X component of the force in Newtons
    pub fx: f64,
    /// Y component of the force in Newtons
    pub fy: f64,
    /// Z component of the force in Newtons
    pub fz: f64,
}

impl Force {
    /// Create a new force vector
    pub fn new(fx: f64, fy: f64, fz: f64) -> Self {
        Force { fx, fy, fz }
    }

    /// Create a zero force
    pub fn zero() -> Self {
        Force::new(0.0, 0.0, 0.0)
    }

    /// Check if the force is valid (all components finite)
    pub fn is_valid(&self) -> bool {
        self.fx.is_finite() && self.fy.is_finite() && self.fz.is_finite()
    }

    /// Add another force to this one
    pub fn add(&mut self, other: &Force) {
        self.fx += other.fx;
        self.fy += other.fy;
        self.fz += other.fz;
    }

    /// Get the magnitude of the force
    pub fn magnitude(&self) -> f64 {
        (self.fx * self.fx + self.fy * self.fy + self.fz * self.fz).sqrt()
    }
}

/// Trait for force providers that can be registered with the force registry
///
/// Force providers compute forces based on entity state and can represent
/// gravity, springs, drag, user input, or any other force-generating mechanism.
pub trait ForceProvider: Send + Sync {
    /// Compute the force to apply to a specific entity
    ///
    /// Returns None if this provider doesn't apply to the entity or if
    /// required components are missing.
    fn compute_force(&self, entity: Entity, registry: &ForceRegistry) -> Option<Force>;

    /// Get a descriptive name for this force provider
    fn name(&self) -> &str;
}

/// Registry for managing force providers and accumulating forces per entity
///
/// The force registry allows plugins to register arbitrary force providers
/// that will be applied to entities during physics updates. Forces are
/// accumulated per entity and can be used to compute accelerations.
///
/// # Logging
///
/// Currently uses `eprintln!` for warnings. Future versions will integrate with
/// the `log` crate to allow configurable logging handlers.
pub struct ForceRegistry {
    providers: Vec<Box<dyn ForceProvider>>,
    accumulated_forces: HashMap<Entity, Force>,
    /// Configuration for overflow/NaN detection
    pub max_force_magnitude: f64,
    /// Whether to log warnings for skipped entities
    pub warn_on_missing_components: bool,
}

impl ForceRegistry {
    /// Create a new force registry
    pub fn new() -> Self {
        ForceRegistry {
            providers: Vec::new(),
            accumulated_forces: HashMap::new(),
            max_force_magnitude: 1e10, // 10 billion Newtons default limit
            warn_on_missing_components: true,
        }
    }

    /// Register a force provider
    pub fn register_provider(&mut self, provider: Box<dyn ForceProvider>) {
        self.providers.push(provider);
    }

    /// Clear all accumulated forces
    pub fn clear_forces(&mut self) {
        self.accumulated_forces.clear();
    }

    /// Clear all providers and accumulated forces
    ///
    /// This is useful for resetting the registry between simulation steps
    /// when force providers need to be re-registered with updated force values.
    pub fn clear(&mut self) {
        self.providers.clear();
        self.accumulated_forces.clear();
    }

    /// Accumulate forces for a specific entity from all providers
    ///
    /// Returns true if forces were accumulated, false if entity was skipped
    pub fn accumulate_for_entity(&mut self, entity: Entity) -> bool {
        let mut total_force = Force::zero();
        let mut has_forces = false;

        for provider in &self.providers {
            if let Some(force) = provider.compute_force(entity, self) {
                if !force.is_valid() {
                    if self.warn_on_missing_components {
                        // Use Debug formatting to prevent injection attacks
                        eprintln!("Warning: Force provider produced invalid force (NaN/Inf) for {:?}", entity);
                    }
                    continue;
                }

                total_force.add(&force);
                has_forces = true;
            }
        }

        // Check for overflow
        if has_forces && total_force.magnitude() > self.max_force_magnitude {
            if self.warn_on_missing_components {
                let mag = total_force.magnitude();
                // Sanitize numeric output
                eprintln!("Warning: Total force magnitude {:.2e} exceeds limit {:.2e} for {:?}", 
                          mag, self.max_force_magnitude, entity);
            }
            // Clamp to max magnitude
            let mag = total_force.magnitude();
            let scale = self.max_force_magnitude / mag;
            total_force.fx *= scale;
            total_force.fy *= scale;
            total_force.fz *= scale;
        }

        if has_forces {
            self.accumulated_forces.insert(entity, total_force);
        }

        has_forces
    }

    /// Get the accumulated force for an entity
    pub fn get_force(&self, entity: Entity) -> Option<Force> {
        self.accumulated_forces.get(&entity).copied()
    }

    /// Get the number of registered providers
    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }
}

impl Default for ForceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Apply accumulated forces to compute accelerations (F = ma)
///
/// This function takes accumulated forces and mass components to compute
/// accelerations according to Newton's second law. Entities without mass
/// or with immovable mass are skipped with optional warnings.
///
/// # Arguments
///
/// * `entities` - Iterator over entities to process
/// * `force_registry` - Registry containing accumulated forces
/// * `masses` - Storage for mass components
/// * `accelerations` - Storage for acceleration components (output)
/// * `warn_on_missing` - Whether to log warnings for entities without required components
///
/// # Returns
///
/// Number of entities that had their acceleration updated
pub fn apply_forces_to_acceleration<'a, I>(
    entities: I,
    force_registry: &ForceRegistry,
    masses: &impl ComponentStorage<Component = Mass>,
    accelerations: &mut impl ComponentStorage<Component = Acceleration>,
    warn_on_missing: bool,
) -> usize
where
    I: Iterator<Item = &'a Entity>,
{
    let mut updated_count = 0;

    for entity in entities {
        // Skip if no force accumulated
        let force = match force_registry.get_force(*entity) {
            Some(f) => f,
            None => continue,
        };

        // Skip if no mass component
        let mass = match masses.get(*entity) {
            Some(m) => m,
            None => {
                if warn_on_missing {
                    eprintln!("Warning: Entity {:?} has force but no Mass component, skipping", entity);
                }
                continue;
            }
        };

        // Skip immovable bodies
        if mass.is_immovable() {
            continue;
        }

        // Compute acceleration: a = F/m
        let inv_mass = mass.inverse();
        let acceleration = Acceleration::new(
            force.fx * inv_mass,
            force.fy * inv_mass,
            force.fz * inv_mass,
        );

        // Validate acceleration
        if !acceleration.is_valid() {
            if warn_on_missing {
                eprintln!("Warning: Computed invalid acceleration for entity {:?}, skipping", entity);
            }
            continue;
        }

        // Update or insert acceleration component
        if accelerations.contains(*entity) {
            if let Some(acc) = accelerations.get_mut(*entity) {
                *acc = acceleration;
            }
        } else {
            accelerations.insert(*entity, acceleration);
        }

        updated_count += 1;
    }

    updated_count
}

/// Integration system that updates velocity and position based on acceleration
///
/// Performs semi-implicit (symplectic) Euler integration:
/// - v' = v + a*dt
/// - p' = p + v'*dt
/// 
/// This method is more stable than explicit Euler for physics simulations.
/// More sophisticated integrators (Verlet, RK4) can be added as alternative systems.
///
/// Immovable bodies (zero or near-zero mass) are skipped entirely to prevent
/// numerical drift.
///
/// # Arguments
///
/// * `entities` - Iterator over entities to process
/// * `dt` - Time step in seconds
/// * `positions` - Storage for position components
/// * `velocities` - Storage for velocity components
/// * `accelerations` - Storage for acceleration components
/// * `masses` - Storage for mass components (to check for immovable bodies)
/// * `warn_on_missing` - Whether to log warnings for entities without required components
///
/// # Returns
///
/// Number of entities that were updated
pub fn integrate_motion<'a, I>(
    entities: I,
    dt: f64,
    positions: &mut impl ComponentStorage<Component = crate::ecs::components::Position>,
    velocities: &mut impl ComponentStorage<Component = Velocity>,
    accelerations: &impl ComponentStorage<Component = Acceleration>,
    masses: &impl ComponentStorage<Component = Mass>,
    warn_on_missing: bool,
) -> usize
where
    I: Iterator<Item = &'a Entity>,
{
    let mut updated_count = 0;

    for entity in entities {
        // Skip immovable bodies or entities without a mass component
        if masses.get(*entity).map_or(true, |m| m.is_immovable()) {
            continue;
        }

        // Get acceleration (may not exist if no forces applied)
        let acc = accelerations.get(*entity);

        // Get velocity (required)
        let vel = match velocities.get_mut(*entity) {
            Some(v) => v,
            None => {
                if warn_on_missing && acc.is_some() {
                    eprintln!("Warning: Entity {:?} has acceleration but no Velocity component, skipping", entity);
                }
                continue;
            }
        };

        // Get position (required)
        let pos = match positions.get_mut(*entity) {
            Some(p) => p,
            None => {
                if warn_on_missing {
                    eprintln!("Warning: Entity {:?} has velocity but no Position component, skipping", entity);
                }
                continue;
            }
        };

        // Update velocity if acceleration exists: v' = v + a*dt
        if let Some(a) = acc {
            vel.set_dx(vel.dx() + a.ax() * dt);
            vel.set_dy(vel.dy() + a.ay() * dt);
            vel.set_dz(vel.dz() + a.az() * dt);
        }

        // Update position: p' = p + v*dt
        pos.set_x(pos.x() + vel.dx() * dt);
        pos.set_y(pos.y() + vel.dy() * dt);
        pos.set_z(pos.z() + vel.dz() * dt);

        // Validate results
        if !pos.is_valid() || !vel.is_valid() {
            if warn_on_missing {
                eprintln!("Warning: Integration produced invalid state for entity {:?}", entity);
            }
            continue;
        }

        updated_count += 1;
    }

    updated_count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::{HashMapStorage, Entity};
    use crate::ecs::components::Position;

    #[test]
    fn test_force_creation() {
        let force = Force::new(10.0, 20.0, 30.0);
        assert_eq!(force.fx, 10.0);
        assert_eq!(force.fy, 20.0);
        assert_eq!(force.fz, 30.0);
    }

    #[test]
    fn test_force_validation() {
        let valid = Force::new(1.0, 2.0, 3.0);
        assert!(valid.is_valid());

        let invalid = Force::new(f64::NAN, 2.0, 3.0);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_force_add() {
        let mut f1 = Force::new(1.0, 2.0, 3.0);
        let f2 = Force::new(4.0, 5.0, 6.0);
        f1.add(&f2);
        assert_eq!(f1.fx, 5.0);
        assert_eq!(f1.fy, 7.0);
        assert_eq!(f1.fz, 9.0);
    }

    #[test]
    fn test_force_magnitude() {
        let force = Force::new(3.0, 4.0, 0.0);
        assert_eq!(force.magnitude(), 5.0);
    }

    struct TestForceProvider {
        force: Force,
    }

    impl ForceProvider for TestForceProvider {
        fn compute_force(&self, _entity: Entity, _registry: &ForceRegistry) -> Option<Force> {
            Some(self.force)
        }

        fn name(&self) -> &str {
            "TestForceProvider"
        }
    }

    #[test]
    fn test_force_registry() {
        let mut registry = ForceRegistry::new();
        assert_eq!(registry.provider_count(), 0);

        let provider = Box::new(TestForceProvider {
            force: Force::new(10.0, 0.0, 0.0),
        });
        registry.register_provider(provider);
        assert_eq!(registry.provider_count(), 1);
    }

    #[test]
    fn test_force_accumulation() {
        let mut registry = ForceRegistry::new();
        
        // Register two force providers
        registry.register_provider(Box::new(TestForceProvider {
            force: Force::new(10.0, 0.0, 0.0),
        }));
        registry.register_provider(Box::new(TestForceProvider {
            force: Force::new(0.0, 20.0, 0.0),
        }));

        let entity = Entity::new(1, 0);
        assert!(registry.accumulate_for_entity(entity));

        let force = registry.get_force(entity).unwrap();
        assert_eq!(force.fx, 10.0);
        assert_eq!(force.fy, 20.0);
        assert_eq!(force.fz, 0.0);
    }

    #[test]
    fn test_force_overflow_detection() {
        let mut registry = ForceRegistry::new();
        registry.max_force_magnitude = 100.0;
        registry.warn_on_missing_components = false;

        registry.register_provider(Box::new(TestForceProvider {
            force: Force::new(1000.0, 0.0, 0.0), // Exceeds limit
        }));

        let entity = Entity::new(1, 0);
        registry.accumulate_for_entity(entity);

        let force = registry.get_force(entity).unwrap();
        // Should be clamped to max magnitude
        assert!(force.magnitude() <= 100.0 + 1e-6);
    }

    #[test]
    fn test_apply_forces_to_acceleration() {
        let mut registry = ForceRegistry::new();
        registry.register_provider(Box::new(TestForceProvider {
            force: Force::new(20.0, 0.0, 0.0),
        }));

        let entity = Entity::new(1, 0);
        registry.accumulate_for_entity(entity);

        let mut masses = HashMapStorage::<Mass>::new();
        masses.insert(entity, Mass::new(10.0)); // 10 kg

        let mut accelerations = HashMapStorage::<Acceleration>::new();

        let entities = vec![entity];
        let count = apply_forces_to_acceleration(
            entities.iter(),
            &registry,
            &masses,
            &mut accelerations,
            false,
        );

        assert_eq!(count, 1);
        let acc = accelerations.get(entity).unwrap();
        assert_eq!(acc.ax(), 2.0); // F/m = 20/10 = 2
        assert_eq!(acc.ay(), 0.0);
        assert_eq!(acc.az(), 0.0);
    }

    #[test]
    fn test_apply_forces_skips_immovable() {
        let mut registry = ForceRegistry::new();
        registry.register_provider(Box::new(TestForceProvider {
            force: Force::new(100.0, 0.0, 0.0),
        }));

        let entity = Entity::new(1, 0);
        registry.accumulate_for_entity(entity);

        let mut masses = HashMapStorage::<Mass>::new();
        masses.insert(entity, Mass::immovable());

        let mut accelerations = HashMapStorage::<Acceleration>::new();

        let entities = vec![entity];
        let count = apply_forces_to_acceleration(
            entities.iter(),
            &registry,
            &masses,
            &mut accelerations,
            false,
        );

        // Should skip immovable entity
        assert_eq!(count, 0);
        assert!(!accelerations.contains(entity));
    }

    #[test]
    fn test_integrate_motion() {
        let entity = Entity::new(1, 0);

        let mut positions = HashMapStorage::<Position>::new();
        positions.insert(entity, Position::new(0.0, 0.0, 0.0));

        let mut velocities = HashMapStorage::<Velocity>::new();
        velocities.insert(entity, Velocity::new(10.0, 0.0, 0.0));

        let mut accelerations = HashMapStorage::<Acceleration>::new();
        accelerations.insert(entity, Acceleration::new(2.0, 0.0, 0.0));

        let mut masses = HashMapStorage::<Mass>::new();
        masses.insert(entity, Mass::new(1.0));

        let dt = 0.1; // 0.1 seconds
        let entities = vec![entity];
        let count = integrate_motion(
            entities.iter(),
            dt,
            &mut positions,
            &mut velocities,
            &accelerations,
            &masses,
            false,
        );

        assert_eq!(count, 1);

        // Check velocity: v' = v + a*dt = 10 + 2*0.1 = 10.2
        let vel = velocities.get(entity).unwrap();
        assert!((vel.dx() - 10.2).abs() < 1e-10);

        // Check position: p' = p + v'*dt = 0 + 10.2*0.1 = 1.02 (using updated velocity)
        let pos = positions.get(entity).unwrap();
        assert!((pos.x() - 1.02).abs() < 1e-10);
    }

    #[test]
    fn test_integrate_motion_without_acceleration() {
        let entity = Entity::new(1, 0);

        let mut positions = HashMapStorage::<Position>::new();
        positions.insert(entity, Position::new(0.0, 0.0, 0.0));

        let mut velocities = HashMapStorage::<Velocity>::new();
        velocities.insert(entity, Velocity::new(5.0, 0.0, 0.0));

        let accelerations = HashMapStorage::<Acceleration>::new(); // No acceleration

        let mut masses = HashMapStorage::<Mass>::new();
        masses.insert(entity, Mass::new(1.0));

        let dt = 0.1;
        let entities = vec![entity];
        let count = integrate_motion(
            entities.iter(),
            dt,
            &mut positions,
            &mut velocities,
            &accelerations,
            &masses,
            false,
        );

        assert_eq!(count, 1);

        // Velocity should not change without acceleration
        let vel = velocities.get(entity).unwrap();
        assert_eq!(vel.dx(), 5.0);

        // Position should update: p' = p + v*dt = 0 + 5*0.1 = 0.5
        let pos = positions.get(entity).unwrap();
        assert!((pos.x() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_integrate_motion_skips_immovable() {
        let entity = Entity::new(1, 0);

        let mut positions = HashMapStorage::<Position>::new();
        positions.insert(entity, Position::new(0.0, 0.0, 0.0));

        let mut velocities = HashMapStorage::<Velocity>::new();
        velocities.insert(entity, Velocity::new(5.0, 0.0, 0.0));

        let mut accelerations = HashMapStorage::<Acceleration>::new();
        accelerations.insert(entity, Acceleration::new(10.0, 0.0, 0.0));

        let mut masses = HashMapStorage::<Mass>::new();
        masses.insert(entity, Mass::immovable());

        let dt = 0.1;
        let entities = vec![entity];
        let count = integrate_motion(
            entities.iter(),
            dt,
            &mut positions,
            &mut velocities,
            &accelerations,
            &masses,
            false,
        );

        // Should skip immovable entity
        assert_eq!(count, 0);

        // Position and velocity should not change
        let vel = velocities.get(entity).unwrap();
        assert_eq!(vel.dx(), 5.0);
        let pos = positions.get(entity).unwrap();
        assert_eq!(pos.x(), 0.0);
    }

    #[test]
    fn test_missing_components_handling() {
        let mut registry = ForceRegistry::new();
        registry.warn_on_missing_components = false;
        registry.register_provider(Box::new(TestForceProvider {
            force: Force::new(10.0, 0.0, 0.0),
        }));

        let entity = Entity::new(1, 0);
        registry.accumulate_for_entity(entity);

        let masses = HashMapStorage::<Mass>::new(); // No mass
        let mut accelerations = HashMapStorage::<Acceleration>::new();

        let entities = vec![entity];
        let count = apply_forces_to_acceleration(
            entities.iter(),
            &registry,
            &masses,
            &mut accelerations,
            false,
        );

        // Should skip entity without mass
        assert_eq!(count, 0);
    }
}
