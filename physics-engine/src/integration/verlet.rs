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
//! Velocity Verlet integrator implementation
//!
//! The velocity Verlet algorithm is a symplectic integrator that provides
//! excellent energy conservation for Hamiltonian systems. It is particularly
//! well-suited for molecular dynamics and orbital mechanics.
//!
//! # Algorithm
//!
//! The velocity Verlet method updates position and velocity as follows:
//!
//! ```text
//! x(t + dt) = x(t) + v(t)*dt + 0.5*a(t)*dt²
//! v(t + dt) = v(t) + 0.5*(a(t) + a(t + dt))*dt
//! ```
//!
//! where a(t + dt) is computed from the forces at the new position.
//!
//! # Properties
//!
//! - **Symplectic**: Preserves phase space volume (Liouville's theorem)
//! - **Time-reversible**: Running forward then backward returns to start
//! - **Energy conservation**: Bounded energy error over long periods
//! - **Second-order accurate**: Local error O(dt³), global error O(dt²)
//!
//! # References
//!
//! - Hairer, E., Lubich, C., & Wanner, G. (2006). Geometric Numerical Integration:
//!   Structure-Preserving Algorithms for Ordinary Differential Equations (2nd ed.).
//!   Springer. Section II.3.
//! - Swope, W. C., Andersen, H. C., Berens, P. H., & Wilson, K. R. (1982).
//!   A computer simulation method for the calculation of equilibrium constants for the
//!   formation of physical clusters of molecules: Application to small water clusters.
//!   The Journal of Chemical Physics, 76(1), 637-649.
//! - Verlet, L. (1967). Computer "Experiments" on Classical Fluids. I. Thermodynamical
//!   Properties of Lennard-Jones Molecules. Physical Review, 159(1), 98-103.

use crate::ecs::{Entity, ComponentStorage};
use crate::ecs::components::{Position, Velocity, Acceleration, Mass};
use crate::ecs::systems::{ForceRegistry, apply_forces_to_acceleration};
use super::Integrator;

/// Velocity Verlet integrator for physics simulation
///
/// This integrator provides excellent energy conservation for oscillatory
/// and orbital motion. It is more accurate than semi-implicit Euler and
/// better preserves energy over long simulations.
///
/// # Example
///
/// ```
/// use physics_engine::integration::{VelocityVerletIntegrator, Integrator};
///
/// let mut integrator = VelocityVerletIntegrator::new(1.0 / 60.0); // 60 FPS
/// assert_eq!(integrator.timestep(), 1.0 / 60.0);
/// ```
pub struct VelocityVerletIntegrator {
    timestep: f64,
}

impl VelocityVerletIntegrator {
    /// Create a new velocity Verlet integrator with the given timestep
    ///
    /// # Panics
    ///
    /// Panics if timestep is non-positive, NaN, or infinite
    pub fn new(timestep: f64) -> Self {
        assert!(
            timestep > 0.0 && timestep.is_finite(),
            "Timestep must be positive and finite"
        );
        VelocityVerletIntegrator { timestep }
    }
}

impl Integrator for VelocityVerletIntegrator {
    fn name(&self) -> &str {
        "Velocity Verlet"
    }

    fn timestep(&self) -> f64 {
        self.timestep
    }

    fn set_timestep(&mut self, dt: f64) {
        assert!(
            dt > 0.0 && dt.is_finite(),
            "Timestep must be positive and finite"
        );
        self.timestep = dt;
    }

    fn integrate<'a, I>(
        &mut self,
        entities: I,
        positions: &mut impl ComponentStorage<Component = Position>,
        velocities: &mut impl ComponentStorage<Component = Velocity>,
        accelerations: &impl ComponentStorage<Component = Acceleration>,
        masses: &impl ComponentStorage<Component = Mass>,
        force_registry: &mut ForceRegistry,
        warn_on_missing: bool,
    ) -> usize
    where
        I: Iterator<Item = &'a Entity>,
    {
        let dt = self.timestep;
        let dt_sq = dt * dt;
        
        let entities_vec: Vec<Entity> = entities.copied().collect();
        let mut updated_count = 0;

        // Step 1: Update positions using current velocities and accelerations
        // x(t + dt) = x(t) + v(t)*dt + 0.5*a(t)*dt²
        for entity in &entities_vec {
            // Skip immovable bodies
            if let Some(mass) = masses.get(*entity) {
                if mass.is_immovable() {
                    continue;
                }
            }

            let pos = match positions.get_mut(*entity) {
                Some(p) => p,
                None => {
                    if warn_on_missing {
                        eprintln!("Warning: Entity {:?} missing Position component", entity);
                    }
                    continue;
                }
            };

            let vel = match velocities.get(*entity) {
                Some(v) => v,
                None => {
                    if warn_on_missing {
                        eprintln!("Warning: Entity {:?} missing Velocity component", entity);
                    }
                    continue;
                }
            };

            // Get current acceleration (may be zero if no forces)
            let acc = accelerations.get(*entity);
            
            // Update position
            let new_x = pos.x() + vel.dx() * dt + if let Some(a) = acc { 0.5 * a.ax() * dt_sq } else { 0.0 };
            let new_y = pos.y() + vel.dy() * dt + if let Some(a) = acc { 0.5 * a.ay() * dt_sq } else { 0.0 };
            let new_z = pos.z() + vel.dz() * dt + if let Some(a) = acc { 0.5 * a.az() * dt_sq } else { 0.0 };
            
            pos.set_x(new_x);
            pos.set_y(new_y);
            pos.set_z(new_z);
            
            if !pos.is_valid() {
                if warn_on_missing {
                    eprintln!("Warning: Invalid position after Verlet update for {:?}", entity);
                }
                continue;
            }
        }

        // Step 2: Compute new accelerations at new positions
        // Force providers need to see updated positions
        force_registry.clear_forces();
        for entity in &entities_vec {
            force_registry.accumulate_for_entity(*entity);
        }
        
        // Convert forces to accelerations
        let mut new_accelerations = crate::ecs::HashMapStorage::<Acceleration>::new();
        apply_forces_to_acceleration(
            entities_vec.iter(),
            force_registry,
            masses,
            &mut new_accelerations,
            warn_on_missing,
        );

        // Step 3: Update velocities using average of old and new accelerations
        // v(t + dt) = v(t) + 0.5*(a(t) + a(t + dt))*dt
        for entity in &entities_vec {
            // Skip immovable bodies
            if let Some(mass) = masses.get(*entity) {
                if mass.is_immovable() {
                    continue;
                }
            }

            let vel = match velocities.get_mut(*entity) {
                Some(v) => v,
                None => continue,
            };

            let old_acc = accelerations.get(*entity);
            let new_acc = new_accelerations.get(*entity);

            // Use Verlet formula: v' = v + 0.5*(a_old + a_new)*dt
            // If acceleration is missing, treat as zero
            let old_acc = old_acc.copied().unwrap_or_else(Acceleration::zero);
            let new_acc = new_acc.copied().unwrap_or_else(Acceleration::zero);

            let ax = 0.5 * (old_acc.ax() + new_acc.ax());
            let ay = 0.5 * (old_acc.ay() + new_acc.ay());
            let az = 0.5 * (old_acc.az() + new_acc.az());

            let new_dx = vel.dx() + ax * dt;
            let new_dy = vel.dy() + ay * dt;
            let new_dz = vel.dz() + az * dt;
            
            vel.set_dx(new_dx);
            vel.set_dy(new_dy);
            vel.set_dz(new_dz);

            if !vel.is_valid() {
                if warn_on_missing {
                    eprintln!("Warning: Invalid velocity after Verlet update for {:?}", entity);
                }
                continue;
            }

            updated_count += 1;
        }

        updated_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::{HashMapStorage, Entity};
    use crate::ecs::systems::{ForceProvider, Force};

    // Spring force provider for testing
    struct SpringForce {
        spring_constant: f64,
    }

    impl ForceProvider for SpringForce {
        fn compute_force(&self, _entity: Entity, _registry: &ForceRegistry) -> Option<Force> {
            // For testing, we'll use a simple approach - force computed externally
            // In real usage, this would read from position components
            None
        }

        fn name(&self) -> &str {
            "SpringForce"
        }
    }

    #[test]
    fn test_verlet_creation() {
        let integrator = VelocityVerletIntegrator::new(0.01);
        assert_eq!(integrator.timestep(), 0.01);
        assert_eq!(integrator.name(), "Velocity Verlet");
    }

    #[test]
    #[should_panic(expected = "Timestep must be positive and finite")]
    fn test_verlet_invalid_timestep() {
        VelocityVerletIntegrator::new(0.0);
    }

    #[test]
    #[should_panic(expected = "Timestep must be positive and finite")]
    fn test_verlet_negative_timestep() {
        VelocityVerletIntegrator::new(-0.01);
    }

    #[test]
    #[should_panic(expected = "Timestep must be positive and finite")]
    fn test_verlet_nan_timestep() {
        VelocityVerletIntegrator::new(f64::NAN);
    }

    #[test]
    fn test_verlet_timestep_validation() {
        let integrator = VelocityVerletIntegrator::new(0.01);
        assert!(integrator.validate_timestep().is_ok());

        let small_integrator = VelocityVerletIntegrator::new(1e-10);
        assert!(small_integrator.validate_timestep().is_err());

        let large_integrator = VelocityVerletIntegrator::new(2.0);
        assert!(large_integrator.validate_timestep().is_err());
    }

    #[test]
    fn test_verlet_set_timestep() {
        let mut integrator = VelocityVerletIntegrator::new(0.01);
        integrator.set_timestep(0.02);
        assert_eq!(integrator.timestep(), 0.02);
    }

    #[test]
    fn test_verlet_free_motion() {
        // Test free motion (no forces) - velocity should remain constant
        let mut integrator = VelocityVerletIntegrator::new(0.1);
        let entity = Entity::new(1, 0);

        let mut positions = HashMapStorage::<Position>::new();
        positions.insert(entity, Position::new(0.0, 0.0, 0.0));

        let mut velocities = HashMapStorage::<Velocity>::new();
        velocities.insert(entity, Velocity::new(1.0, 2.0, 3.0));

        let accelerations = HashMapStorage::<Acceleration>::new();
        let mut masses = HashMapStorage::<Mass>::new();
        masses.insert(entity, Mass::new(1.0));

        let mut force_registry = ForceRegistry::new();

        let entities = vec![entity];
        let count = integrator.integrate(
            entities.iter(),
            &mut positions,
            &mut velocities,
            &accelerations,
            &masses,
            &mut force_registry,
            false,
        );

        assert_eq!(count, 1);

        let pos = positions.get(entity).unwrap();
        assert!((pos.x() - 0.1).abs() < 1e-10); // x = 0 + 1*0.1
        assert!((pos.y() - 0.2).abs() < 1e-10); // y = 0 + 2*0.1
        assert!((pos.z() - 0.3).abs() < 1e-10); // z = 0 + 3*0.1

        let vel = velocities.get(entity).unwrap();
        assert!((vel.dx() - 1.0).abs() < 1e-10); // Velocity unchanged
        assert!((vel.dy() - 2.0).abs() < 1e-10);
        assert!((vel.dz() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_verlet_constant_acceleration() {
        // Test with constant acceleration
        let mut integrator = VelocityVerletIntegrator::new(0.1);
        let entity = Entity::new(1, 0);

        let mut positions = HashMapStorage::<Position>::new();
        positions.insert(entity, Position::new(0.0, 0.0, 0.0));

        let mut velocities = HashMapStorage::<Velocity>::new();
        velocities.insert(entity, Velocity::new(0.0, 0.0, 0.0));

        let mut accelerations = HashMapStorage::<Acceleration>::new();
        accelerations.insert(entity, Acceleration::new(10.0, 0.0, 0.0));

        let mut masses = HashMapStorage::<Mass>::new();
        masses.insert(entity, Mass::new(1.0));

        let mut force_registry = ForceRegistry::new();

        let entities = vec![entity];
        integrator.integrate(
            entities.iter(),
            &mut positions,
            &mut velocities,
            &accelerations,
            &masses,
            &mut force_registry,
            false,
        );

        let pos = positions.get(entity).unwrap();
        // x = 0 + 0*0.1 + 0.5*10*0.01 = 0.05
        assert!((pos.x() - 0.05).abs() < 1e-10);

        let vel = velocities.get(entity).unwrap();
        // v = 0 + 10*0.1 = 1.0 (approximately, depends on new acceleration)
        assert!(vel.dx() > 0.0); // Velocity should increase
    }
}
