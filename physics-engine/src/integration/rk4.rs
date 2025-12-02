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
//! Runge-Kutta 4th order (RK4) integrator implementation
//!
//! The RK4 method is a classical explicit integrator that provides fourth-order
//! accuracy for smooth ordinary differential equations. It is widely used for
//! high-precision simulations where forces vary smoothly.
//!
//! # Algorithm
//!
//! The RK4 method computes four intermediate derivatives per timestep:
//!
//! ```text
//! k1 = f(t, y)
//! k2 = f(t + dt/2, y + k1*dt/2)
//! k3 = f(t + dt/2, y + k2*dt/2)
//! k4 = f(t + dt, y + k3*dt)
//! y(t + dt) = y(t) + (k1 + 2*k2 + 2*k3 + k4)*dt/6
//! ```
//!
//! For our second-order system (position and velocity):
//! ```text
//! k1_v = a(x, v)
//! k1_x = v
//! k2_v = a(x + k1_x*dt/2, v + k1_v*dt/2)
//! k2_x = v + k1_v*dt/2
//! ... and so on
//! ```
//!
//! # Properties
//!
//! - **Fourth-order accurate**: Local error O(dt⁵), global error O(dt⁴)
//! - **Explicit method**: Easy to implement, no implicit solve needed
//! - **Not symplectic**: Energy may drift over long simulations
//! - **Four evaluations per step**: More expensive than Verlet
//!
//! # References
//!
//! - Butcher, J. C. (2016). Numerical Methods for Ordinary Differential Equations
//!   (3rd ed.). Wiley. Chapter 3.
//! - Press, W. H., Teukolsky, S. A., Vetterling, W. T., & Flannery, B. P. (2007).
//!   Numerical Recipes: The Art of Scientific Computing (3rd ed.). Cambridge
//!   University Press. Section 17.1.
//! - Kutta, W. (1901). Beitrag zur näherungsweisen Integration totaler
//!   Differentialgleichungen. Zeitschrift für Mathematik und Physik, 46, 435-453.

use crate::ecs::{Entity, ComponentStorage};
use crate::ecs::components::{Position, Velocity, Acceleration, Mass};
use crate::ecs::systems::ForceRegistry;
use super::Integrator;
use std::collections::HashMap;

/// Runge-Kutta 4th order integrator for physics simulation
///
/// This integrator provides high accuracy for smooth dynamics at the cost
/// of 4x force evaluations per timestep compared to simpler methods.
/// Best suited for systems where high precision is needed and forces
/// vary smoothly with position.
///
/// # Example
///
/// ```
/// use physics_engine::integration::{RK4Integrator, Integrator};
///
/// let mut integrator = RK4Integrator::new(1.0 / 60.0); // 60 FPS
/// assert_eq!(integrator.timestep(), 1.0 / 60.0);
/// ```
pub struct RK4Integrator {
    timestep: f64,
    // Reusable buffers to avoid allocation on each integration step
    k1_positions: HashMap<Entity, Position>,
    k1_velocities: HashMap<Entity, Velocity>,
    k2_positions: HashMap<Entity, Position>,
    k2_velocities: HashMap<Entity, Velocity>,
    k3_positions: HashMap<Entity, Position>,
    k3_velocities: HashMap<Entity, Velocity>,
    k4_positions: HashMap<Entity, Position>,
    k4_velocities: HashMap<Entity, Velocity>,
    temp_accelerations: HashMap<Entity, Acceleration>,
}

impl RK4Integrator {
    /// Create a new RK4 integrator with the given timestep
    ///
    /// # Panics
    ///
    /// Panics if timestep is non-positive, NaN, or infinite
    pub fn new(timestep: f64) -> Self {
        assert!(
            timestep > 0.0 && timestep.is_finite(),
            "Timestep must be positive and finite"
        );
        RK4Integrator {
            timestep,
            k1_positions: HashMap::new(),
            k1_velocities: HashMap::new(),
            k2_positions: HashMap::new(),
            k2_velocities: HashMap::new(),
            k3_positions: HashMap::new(),
            k3_velocities: HashMap::new(),
            k4_positions: HashMap::new(),
            k4_velocities: HashMap::new(),
            temp_accelerations: HashMap::new(),
        }
    }

    /// Clear internal buffers (automatically done at start of each integration)
    fn clear_buffers(&mut self) {
        self.k1_positions.clear();
        self.k1_velocities.clear();
        self.k2_positions.clear();
        self.k2_velocities.clear();
        self.k3_positions.clear();
        self.k3_velocities.clear();
        self.k4_positions.clear();
        self.k4_velocities.clear();
        self.temp_accelerations.clear();
    }

    /// Compute derivative (velocity, acceleration) at current state
    fn compute_derivative(
        &mut self,
        entity: Entity,
        position: &Position,
        velocity: &Velocity,
        _dt_factor: f64,
        use_buffer: Option<(Position, Velocity)>,
        positions: &mut impl ComponentStorage<Component = Position>,
        masses: &impl ComponentStorage<Component = Mass>,
        force_registry: &mut ForceRegistry,
    ) -> Option<(Velocity, Acceleration)> {
        // For RK4, we need to temporarily update position to compute forces
        let (eval_pos, eval_vel) = if let Some((buf_pos, buf_vel)) = use_buffer {
            (buf_pos, buf_vel)
        } else {
            (*position, *velocity)
        };

        // Temporarily set position to evaluation point
        if let Some(pos) = positions.get_mut(entity) {
            *pos = eval_pos;
        }

        // Compute forces at this position
        force_registry.clear_forces();
        force_registry.accumulate_for_entity(entity);

        // Compute acceleration from forces
        let mass = masses.get(entity)?;
        if mass.is_immovable() {
            return None;
        }

        // If no force, acceleration is zero (free motion)
        let acceleration = if let Some(force) = force_registry.get_force(entity) {
            let inv_mass = mass.inverse();
            Acceleration::new(
                force.fx * inv_mass,
                force.fy * inv_mass,
                force.fz * inv_mass,
            )
        } else {
            Acceleration::zero()
        };

        if !acceleration.is_valid() {
            return None;
        }

        // Derivative is (velocity, acceleration)
        Some((eval_vel, acceleration))
    }
}

impl Integrator for RK4Integrator {
    fn name(&self) -> &str {
        "Runge-Kutta 4"
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
        _accelerations: &impl ComponentStorage<Component = Acceleration>,
        masses: &impl ComponentStorage<Component = Mass>,
        force_registry: &mut ForceRegistry,
        warn_on_missing: bool,
    ) -> usize
    where
        I: Iterator<Item = &'a Entity>,
    {
        let dt = self.timestep;
        let dt_2 = dt * 0.5;
        let dt_6 = dt / 6.0;

        self.clear_buffers();

        let entities_vec: Vec<Entity> = entities.copied().collect();
        let mut updated_count = 0;

        // Store initial state for all entities
        let mut initial_positions = HashMap::new();
        let mut initial_velocities = HashMap::new();

        for entity in &entities_vec {
            if let (Some(pos), Some(vel)) = (positions.get(*entity), velocities.get(*entity)) {
                initial_positions.insert(*entity, *pos);
                initial_velocities.insert(*entity, *vel);
            }
        }

        // Compute k1 for all entities
        for entity in &entities_vec {
            let pos = match initial_positions.get(entity) {
                Some(p) => p,
                None => continue,
            };
            let vel = match initial_velocities.get(entity) {
                Some(v) => v,
                None => continue,
            };

            if let Some((_k1_vel, k1_acc)) = self.compute_derivative(
                *entity,
                pos,
                vel,
                0.0,
                None,
                positions,
                masses,
                force_registry,
            ) {
                // k1 for position is just velocity
                self.k1_positions.insert(*entity, Position::new(
                    vel.dx(), vel.dy(), vel.dz()
                ));
                // k1 for velocity is acceleration
                self.k1_velocities.insert(*entity, Velocity::new(
                    k1_acc.ax(), k1_acc.ay(), k1_acc.az()
                ));
            }
        }

        // Compute k2 for all entities
        for entity in &entities_vec {
            let pos = match initial_positions.get(entity) {
                Some(p) => p,
                None => continue,
            };
            let vel = match initial_velocities.get(entity) {
                Some(v) => v,
                None => continue,
            };
            let k1_pos = match self.k1_positions.get(entity) {
                Some(k) => k,
                None => continue,
            };
            let k1_vel = match self.k1_velocities.get(entity) {
                Some(k) => k,
                None => continue,
            };

            // Evaluate at y + k1*dt/2
            let eval_pos = Position::new(
                pos.x() + k1_pos.x() * dt_2,
                pos.y() + k1_pos.y() * dt_2,
                pos.z() + k1_pos.z() * dt_2,
            );
            let eval_vel = Velocity::new(
                vel.dx() + k1_vel.dx() * dt_2,
                vel.dy() + k1_vel.dy() * dt_2,
                vel.dz() + k1_vel.dz() * dt_2,
            );

            if let Some((k2_vel, k2_acc)) = self.compute_derivative(
                *entity,
                pos,
                vel,
                dt_2,
                Some((eval_pos, eval_vel)),
                positions,
                masses,
                force_registry,
            ) {
                self.k2_positions.insert(*entity, Position::new(
                    k2_vel.dx(), k2_vel.dy(), k2_vel.dz()
                ));
                self.k2_velocities.insert(*entity, Velocity::new(
                    k2_acc.ax(), k2_acc.ay(), k2_acc.az()
                ));
            }
        }

        // Compute k3 for all entities
        for entity in &entities_vec {
            let pos = match initial_positions.get(entity) {
                Some(p) => p,
                None => continue,
            };
            let vel = match initial_velocities.get(entity) {
                Some(v) => v,
                None => continue,
            };
            let k2_pos = match self.k2_positions.get(entity) {
                Some(k) => k,
                None => continue,
            };
            let k2_vel = match self.k2_velocities.get(entity) {
                Some(k) => k,
                None => continue,
            };

            let eval_pos = Position::new(
                pos.x() + k2_pos.x() * dt_2,
                pos.y() + k2_pos.y() * dt_2,
                pos.z() + k2_pos.z() * dt_2,
            );
            let eval_vel = Velocity::new(
                vel.dx() + k2_vel.dx() * dt_2,
                vel.dy() + k2_vel.dy() * dt_2,
                vel.dz() + k2_vel.dz() * dt_2,
            );

            if let Some((k3_vel, k3_acc)) = self.compute_derivative(
                *entity,
                pos,
                vel,
                dt_2,
                Some((eval_pos, eval_vel)),
                positions,
                masses,
                force_registry,
            ) {
                self.k3_positions.insert(*entity, Position::new(
                    k3_vel.dx(), k3_vel.dy(), k3_vel.dz()
                ));
                self.k3_velocities.insert(*entity, Velocity::new(
                    k3_acc.ax(), k3_acc.ay(), k3_acc.az()
                ));
            }
        }

        // Compute k4 for all entities
        for entity in &entities_vec {
            let pos = match initial_positions.get(entity) {
                Some(p) => p,
                None => continue,
            };
            let vel = match initial_velocities.get(entity) {
                Some(v) => v,
                None => continue,
            };
            let k3_pos = match self.k3_positions.get(entity) {
                Some(k) => k,
                None => continue,
            };
            let k3_vel = match self.k3_velocities.get(entity) {
                Some(k) => k,
                None => continue,
            };

            let eval_pos = Position::new(
                pos.x() + k3_pos.x() * dt,
                pos.y() + k3_pos.y() * dt,
                pos.z() + k3_pos.z() * dt,
            );
            let eval_vel = Velocity::new(
                vel.dx() + k3_vel.dx() * dt,
                vel.dy() + k3_vel.dy() * dt,
                vel.dz() + k3_vel.dz() * dt,
            );

            if let Some((k4_vel, k4_acc)) = self.compute_derivative(
                *entity,
                pos,
                vel,
                dt,
                Some((eval_pos, eval_vel)),
                positions,
                masses,
                force_registry,
            ) {
                self.k4_positions.insert(*entity, Position::new(
                    k4_vel.dx(), k4_vel.dy(), k4_vel.dz()
                ));
                self.k4_velocities.insert(*entity, Velocity::new(
                    k4_acc.ax(), k4_acc.ay(), k4_acc.az()
                ));
            }
        }

        // Final update: y(t+dt) = y(t) + (k1 + 2*k2 + 2*k3 + k4)*dt/6
        for entity in &entities_vec {
            let pos = match initial_positions.get(entity) {
                Some(p) => p,
                None => continue,
            };
            let vel = match initial_velocities.get(entity) {
                Some(v) => v,
                None => continue,
            };

            let (k1_pos, k2_pos, k3_pos, k4_pos) = match (
                self.k1_positions.get(entity),
                self.k2_positions.get(entity),
                self.k3_positions.get(entity),
                self.k4_positions.get(entity),
            ) {
                (Some(k1), Some(k2), Some(k3), Some(k4)) => (k1, k2, k3, k4),
                _ => continue,
            };

            let (k1_vel, k2_vel, k3_vel, k4_vel) = match (
                self.k1_velocities.get(entity),
                self.k2_velocities.get(entity),
                self.k3_velocities.get(entity),
                self.k4_velocities.get(entity),
            ) {
                (Some(k1), Some(k2), Some(k3), Some(k4)) => (k1, k2, k3, k4),
                _ => continue,
            };

            // Update position
            let new_pos = Position::new(
                pos.x() + (k1_pos.x() + 2.0 * k2_pos.x() + 2.0 * k3_pos.x() + k4_pos.x()) * dt_6,
                pos.y() + (k1_pos.y() + 2.0 * k2_pos.y() + 2.0 * k3_pos.y() + k4_pos.y()) * dt_6,
                pos.z() + (k1_pos.z() + 2.0 * k2_pos.z() + 2.0 * k3_pos.z() + k4_pos.z()) * dt_6,
            );

            // Update velocity
            let new_vel = Velocity::new(
                vel.dx() + (k1_vel.dx() + 2.0 * k2_vel.dx() + 2.0 * k3_vel.dx() + k4_vel.dx()) * dt_6,
                vel.dy() + (k1_vel.dy() + 2.0 * k2_vel.dy() + 2.0 * k3_vel.dy() + k4_vel.dy()) * dt_6,
                vel.dz() + (k1_vel.dz() + 2.0 * k2_vel.dz() + 2.0 * k3_vel.dz() + k4_vel.dz()) * dt_6,
            );

            if !new_pos.is_valid() || !new_vel.is_valid() {
                if warn_on_missing {
                    eprintln!("Warning: Invalid state after RK4 update for {:?}", entity);
                }
                continue;
            }

            // Commit final state
            if let Some(p) = positions.get_mut(*entity) {
                *p = new_pos;
            }
            if let Some(v) = velocities.get_mut(*entity) {
                *v = new_vel;
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

    #[test]
    fn test_rk4_creation() {
        let integrator = RK4Integrator::new(0.01);
        assert_eq!(integrator.timestep(), 0.01);
        assert_eq!(integrator.name(), "Runge-Kutta 4");
    }

    #[test]
    #[should_panic(expected = "Timestep must be positive and finite")]
    fn test_rk4_invalid_timestep() {
        RK4Integrator::new(0.0);
    }

    #[test]
    fn test_rk4_set_timestep() {
        let mut integrator = RK4Integrator::new(0.01);
        integrator.set_timestep(0.02);
        assert_eq!(integrator.timestep(), 0.02);
    }

    #[test]
    fn test_rk4_free_motion() {
        // Test free motion (no forces) - velocity should remain constant
        let mut integrator = RK4Integrator::new(0.1);
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
        // With no forces, position should update by velocity * dt
        assert!((pos.x() - 0.1).abs() < 1e-10);
        assert!((pos.y() - 0.2).abs() < 1e-10);
        assert!((pos.z() - 0.3).abs() < 1e-10);
    }

    #[test]
    fn test_rk4_buffer_reuse() {
        // Test that buffers are properly reused across multiple integrations
        let mut integrator = RK4Integrator::new(0.01);
        let entity = Entity::new(1, 0);

        let mut positions = HashMapStorage::<Position>::new();
        positions.insert(entity, Position::new(0.0, 0.0, 0.0));

        let mut velocities = HashMapStorage::<Velocity>::new();
        velocities.insert(entity, Velocity::new(1.0, 0.0, 0.0));

        let accelerations = HashMapStorage::<Acceleration>::new();
        let mut masses = HashMapStorage::<Mass>::new();
        masses.insert(entity, Mass::new(1.0));

        let mut force_registry = ForceRegistry::new();

        // Run multiple integrations
        for _ in 0..5 {
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
        }

        // Should complete without errors
        let pos = positions.get(entity).unwrap();
        assert!(pos.is_valid());
    }
}
