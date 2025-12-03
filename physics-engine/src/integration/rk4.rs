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
use crate::pool::{HashMapPool, PoolConfig};
use super::Integrator;
use std::collections::HashMap;

/// Runge-Kutta 4th order integrator for physics simulation
///
/// This integrator provides high accuracy for smooth dynamics at the cost
/// of 4x force evaluations per timestep compared to simpler methods.
/// Best suited for systems where high precision is needed and forces
/// vary smoothly with position.
///
/// # Memory Pooling
///
/// RK4 uses memory pools for intermediate k1-k4 buffers to reduce
/// allocation churn. Pools are configured at creation time and reused
/// across integration steps.
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
    // Memory pools for reusable buffers to reduce allocation churn
    position_pool: HashMapPool<Entity, Position>,
    velocity_pool: HashMapPool<Entity, Velocity>,
    acceleration_pool: HashMapPool<Entity, Acceleration>,
}

impl RK4Integrator {
    /// Create a new RK4 integrator with the given timestep
    ///
    /// Uses default pool configuration (64 initial capacity, 8 max pool size).
    ///
    /// # Panics
    ///
    /// Panics if timestep is non-positive, NaN, or infinite
    pub fn new(timestep: f64) -> Self {
        Self::with_pool_config(timestep, PoolConfig::default())
    }

    /// Create a new RK4 integrator with custom pool configuration
    ///
    /// # Arguments
    ///
    /// * `timestep` - Integration timestep in seconds
    /// * `pool_config` - Configuration for buffer pools
    ///
    /// # Panics
    ///
    /// Panics if timestep is non-positive, NaN, or infinite
    pub fn with_pool_config(timestep: f64, pool_config: PoolConfig) -> Self {
        assert!(
            timestep > 0.0 && timestep.is_finite(),
            "Timestep must be positive and finite"
        );
        RK4Integrator {
            timestep,
            position_pool: HashMapPool::with_config(pool_config.clone()),
            velocity_pool: HashMapPool::with_config(pool_config.clone()),
            acceleration_pool: HashMapPool::with_config(pool_config),
        }
    }

    /// Get pool statistics for monitoring
    pub fn pool_stats(&self) -> (crate::pool::PoolStats, crate::pool::PoolStats, crate::pool::PoolStats) {
        (
            self.position_pool.stats(),
            self.velocity_pool.stats(),
            self.acceleration_pool.stats(),
        )
    }

    /// Clear all buffer pools (useful for shutdown or reset)
    pub fn clear_pools(&self) {
        self.position_pool.clear();
        self.velocity_pool.clear();
        self.acceleration_pool.clear();
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

        let entities_vec: Vec<Entity> = entities.copied().collect();
        let mut updated_count = 0;

        // Acquire buffers from pools (automatically returned on scope exit)
        let mut k1_positions = self.position_pool.acquire();
        let mut k1_velocities = self.velocity_pool.acquire();
        let mut k2_positions = self.position_pool.acquire();
        let mut k2_velocities = self.velocity_pool.acquire();
        let mut k3_positions = self.position_pool.acquire();
        let mut k3_velocities = self.velocity_pool.acquire();
        let mut k4_positions = self.position_pool.acquire();
        let mut k4_velocities = self.velocity_pool.acquire();

        // Store initial state for all entities
        let mut initial_positions = HashMap::new();
        let mut initial_velocities = HashMap::new();

        for entity in &entities_vec {
            if let (Some(pos), Some(vel)) = (positions.get(*entity), velocities.get(*entity)) {
                // Skip immovable bodies
                if masses.get(*entity).map_or(true, |m| m.is_immovable()) {
                    continue;
                }
                initial_positions.insert(*entity, *pos);
                initial_velocities.insert(*entity, *vel);
            }
        }

        // ==================== STAGE 1: Compute k1 ====================
        // Compute k1 at initial state (t, y0)
        // All entities remain at their initial positions during this stage
        
        force_registry.clear_forces();
        for entity in &entities_vec {
            force_registry.accumulate_for_entity(*entity);
        }

        for entity in &entities_vec {
            let pos = match initial_positions.get(entity) {
                Some(p) => p,
                None => continue,
            };
            let vel = match initial_velocities.get(entity) {
                Some(v) => v,
                None => continue,
            };

            let mass = match masses.get(*entity) {
                Some(m) => m,
                None => continue,
            };

            // k1 for position derivative is current velocity (dx/dt = v)
            k1_positions.insert(*entity, Position::new(
                vel.dx(), vel.dy(), vel.dz()
            ));

            // k1 for velocity derivative is acceleration at current state (dv/dt = a)
            let acceleration = if let Some(force) = force_registry.get_force(*entity) {
                let inv_mass = mass.inverse();
                Acceleration::new(
                    force.fx * inv_mass,
                    force.fy * inv_mass,
                    force.fz * inv_mass,
                )
            } else {
                Acceleration::zero()
            };

            if acceleration.is_valid() {
                k1_velocities.insert(*entity, Velocity::new(
                    acceleration.ax(), acceleration.ay(), acceleration.az()
                ));
            }
        }

        // ==================== STAGE 2: Compute k2 ====================
        // Compute k2 at intermediate state (t + dt/2, y0 + k1*dt/2)
        // Update ALL entities' positions to their k2 evaluation points
        
        for entity in &entities_vec {
            let pos = match initial_positions.get(entity) {
                Some(p) => p,
                None => continue,
            };
            let k1_pos = match k1_positions.get(entity) {
                Some(k) => k,
                None => continue,
            };

            // Move entity to intermediate position: pos + k1*dt/2
            let intermediate_pos = Position::new(
                pos.x() + k1_pos.x() * dt_2,
                pos.y() + k1_pos.y() * dt_2,
                pos.z() + k1_pos.z() * dt_2,
            );

            if let Some(p) = positions.get_mut(*entity) {
                *p = intermediate_pos;
            }
        }

        // Now compute forces with ALL entities at their intermediate positions
        force_registry.clear_forces();
        for entity in &entities_vec {
            force_registry.accumulate_for_entity(*entity);
        }

        for entity in &entities_vec {
            let vel = match initial_velocities.get(entity) {
                Some(v) => v,
                None => continue,
            };
            let k1_vel = match k1_velocities.get(entity) {
                Some(k) => k,
                None => continue,
            };

            let mass = match masses.get(*entity) {
                Some(m) => m,
                None => continue,
            };

            // k2 for position derivative is velocity at intermediate state
            let intermediate_vel = Velocity::new(
                vel.dx() + k1_vel.dx() * dt_2,
                vel.dy() + k1_vel.dy() * dt_2,
                vel.dz() + k1_vel.dz() * dt_2,
            );
            k2_positions.insert(*entity, Position::new(
                intermediate_vel.dx(), intermediate_vel.dy(), intermediate_vel.dz()
            ));

            // k2 for velocity derivative is acceleration at intermediate state
            let acceleration = if let Some(force) = force_registry.get_force(*entity) {
                let inv_mass = mass.inverse();
                Acceleration::new(
                    force.fx * inv_mass,
                    force.fy * inv_mass,
                    force.fz * inv_mass,
                )
            } else {
                Acceleration::zero()
            };

            if acceleration.is_valid() {
                k2_velocities.insert(*entity, Velocity::new(
                    acceleration.ax(), acceleration.ay(), acceleration.az()
                ));
            }
        }

        // ==================== STAGE 3: Compute k3 ====================
        // Compute k3 at intermediate state (t + dt/2, y0 + k2*dt/2)
        // Update ALL entities' positions to their k3 evaluation points
        
        for entity in &entities_vec {
            let pos = match initial_positions.get(entity) {
                Some(p) => p,
                None => continue,
            };
            let k2_pos = match k2_positions.get(entity) {
                Some(k) => k,
                None => continue,
            };

            // Move entity to intermediate position: pos + k2*dt/2
            let intermediate_pos = Position::new(
                pos.x() + k2_pos.x() * dt_2,
                pos.y() + k2_pos.y() * dt_2,
                pos.z() + k2_pos.z() * dt_2,
            );

            if let Some(p) = positions.get_mut(*entity) {
                *p = intermediate_pos;
            }
        }

        // Compute forces with ALL entities at their k3 intermediate positions
        force_registry.clear_forces();
        for entity in &entities_vec {
            force_registry.accumulate_for_entity(*entity);
        }

        for entity in &entities_vec {
            let vel = match initial_velocities.get(entity) {
                Some(v) => v,
                None => continue,
            };
            let k2_vel = match k2_velocities.get(entity) {
                Some(k) => k,
                None => continue,
            };

            let mass = match masses.get(*entity) {
                Some(m) => m,
                None => continue,
            };

            // k3 for position derivative is velocity at intermediate state
            let intermediate_vel = Velocity::new(
                vel.dx() + k2_vel.dx() * dt_2,
                vel.dy() + k2_vel.dy() * dt_2,
                vel.dz() + k2_vel.dz() * dt_2,
            );
            k3_positions.insert(*entity, Position::new(
                intermediate_vel.dx(), intermediate_vel.dy(), intermediate_vel.dz()
            ));

            // k3 for velocity derivative is acceleration at intermediate state
            let acceleration = if let Some(force) = force_registry.get_force(*entity) {
                let inv_mass = mass.inverse();
                Acceleration::new(
                    force.fx * inv_mass,
                    force.fy * inv_mass,
                    force.fz * inv_mass,
                )
            } else {
                Acceleration::zero()
            };

            if acceleration.is_valid() {
                k3_velocities.insert(*entity, Velocity::new(
                    acceleration.ax(), acceleration.ay(), acceleration.az()
                ));
            }
        }

        // ==================== STAGE 4: Compute k4 ====================
        // Compute k4 at end state (t + dt, y0 + k3*dt)
        // Update ALL entities' positions to their k4 evaluation points
        
        for entity in &entities_vec {
            let pos = match initial_positions.get(entity) {
                Some(p) => p,
                None => continue,
            };
            let k3_pos = match k3_positions.get(entity) {
                Some(k) => k,
                None => continue,
            };

            // Move entity to end position: pos + k3*dt
            let end_pos = Position::new(
                pos.x() + k3_pos.x() * dt,
                pos.y() + k3_pos.y() * dt,
                pos.z() + k3_pos.z() * dt,
            );

            if let Some(p) = positions.get_mut(*entity) {
                *p = end_pos;
            }
        }

        // Compute forces with ALL entities at their k4 end positions
        force_registry.clear_forces();
        for entity in &entities_vec {
            force_registry.accumulate_for_entity(*entity);
        }

        for entity in &entities_vec {
            let vel = match initial_velocities.get(entity) {
                Some(v) => v,
                None => continue,
            };
            let k3_vel = match k3_velocities.get(entity) {
                Some(k) => k,
                None => continue,
            };

            let mass = match masses.get(*entity) {
                Some(m) => m,
                None => continue,
            };

            // k4 for position derivative is velocity at end state
            let end_vel = Velocity::new(
                vel.dx() + k3_vel.dx() * dt,
                vel.dy() + k3_vel.dy() * dt,
                vel.dz() + k3_vel.dz() * dt,
            );
            k4_positions.insert(*entity, Position::new(
                end_vel.dx(), end_vel.dy(), end_vel.dz()
            ));

            // k4 for velocity derivative is acceleration at end state
            let acceleration = if let Some(force) = force_registry.get_force(*entity) {
                let inv_mass = mass.inverse();
                Acceleration::new(
                    force.fx * inv_mass,
                    force.fy * inv_mass,
                    force.fz * inv_mass,
                )
            } else {
                Acceleration::zero()
            };

            if acceleration.is_valid() {
                k4_velocities.insert(*entity, Velocity::new(
                    acceleration.ax(), acceleration.ay(), acceleration.az()
                ));
            }
        }

        // ==================== FINAL UPDATE ====================
        // Restore all entities to their original positions before applying the final update
        // This ensures the positions storage is in a clean state for the final update
        for entity in &entities_vec {
            if let Some(initial_pos) = initial_positions.get(entity) {
                if let Some(p) = positions.get_mut(*entity) {
                    *p = *initial_pos;
                }
            }
        }
        
        // Apply the RK4 weighted average: y(t+dt) = y(t) + (k1 + 2*k2 + 2*k3 + k4)*dt/6
        for entity in &entities_vec {
            // Re-check immovability in case it changed during integration
            if masses.get(*entity).map_or(true, |m| m.is_immovable()) {
                continue;
            }
            
            let pos = match initial_positions.get(entity) {
                Some(p) => p,
                None => continue,
            };
            let vel = match initial_velocities.get(entity) {
                Some(v) => v,
                None => continue,
            };

            let (k1_pos, k2_pos, k3_pos, k4_pos) = match (
                k1_positions.get(entity),
                k2_positions.get(entity),
                k3_positions.get(entity),
                k4_positions.get(entity),
            ) {
                (Some(k1), Some(k2), Some(k3), Some(k4)) => (k1, k2, k3, k4),
                _ => continue,
            };

            let (k1_vel, k2_vel, k3_vel, k4_vel) = match (
                k1_velocities.get(entity),
                k2_velocities.get(entity),
                k3_velocities.get(entity),
                k4_velocities.get(entity),
            ) {
                (Some(k1), Some(k2), Some(k3), Some(k4)) => (k1, k2, k3, k4),
                _ => continue,
            };

            // Update position with RK4 formula
            let new_pos = Position::new(
                pos.x() + (k1_pos.x() + 2.0 * k2_pos.x() + 2.0 * k3_pos.x() + k4_pos.x()) * dt_6,
                pos.y() + (k1_pos.y() + 2.0 * k2_pos.y() + 2.0 * k3_pos.y() + k4_pos.y()) * dt_6,
                pos.z() + (k1_pos.z() + 2.0 * k2_pos.z() + 2.0 * k3_pos.z() + k4_pos.z()) * dt_6,
            );

            // Update velocity with RK4 formula
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
