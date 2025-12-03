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
//! Numerical integration methods for physics simulation
//!
//! This module provides different numerical integration schemes for updating
//! position and velocity based on forces. Each integrator has different
//! accuracy, stability, and performance characteristics.
//!
//! # Integrators
//!
//! - **Velocity Verlet**: Symplectic integrator with good energy conservation
//! - **RK4 (Runge-Kutta 4th order)**: Higher accuracy for smooth dynamics
//!
//! # Choosing an Integrator
//!
//! - **Velocity Verlet**: Best for long-running simulations with oscillatory motion
//!   (springs, orbital mechanics). Better energy conservation but less accurate
//!   for highly nonlinear forces.
//!
//! - **RK4**: Best for simulations requiring high accuracy with smooth forces.
//!   More computationally expensive (4x force evaluations per step) but handles
//!   nonlinear dynamics better.
//!
//! # Timestep Guidelines
//!
//! - Too small: Numerical precision issues and wasted computation
//! - Too large: Instability and inaccuracy
//! - Recommended: Start with dt = 1/60 (60 FPS) and adjust based on simulation needs
//! - For stiff systems: Smaller timesteps or implicit integrators may be needed

use crate::ecs::{Entity, ComponentStorage};
use crate::ecs::components::{Position, Velocity, Acceleration, Mass};
use crate::ecs::systems::ForceRegistry;

mod verlet;
mod rk4;
mod simd_helpers;

pub use verlet::VelocityVerletIntegrator;
pub use rk4::RK4Integrator;
pub use simd_helpers::*;

/// Calculate kinetic energy for a single entity
///
/// KE = 0.5 * m * v²
pub fn calculate_kinetic_energy(
    velocity: &Velocity,
    mass: &Mass,
) -> f64 {
    if mass.is_immovable() {
        return 0.0;
    }
    let v_sq = velocity.dx() * velocity.dx() 
             + velocity.dy() * velocity.dy() 
             + velocity.dz() * velocity.dz();
    0.5 * mass.value() * v_sq
}

/// Calculate total kinetic energy for multiple entities
pub fn calculate_total_kinetic_energy<'a, I>(
    entities: I,
    velocities: &impl ComponentStorage<Component = Velocity>,
    masses: &impl ComponentStorage<Component = Mass>,
) -> f64
where
    I: Iterator<Item = &'a Entity>,
{
    let mut total = 0.0;
    for entity in entities {
        if let (Some(vel), Some(mass)) = (velocities.get(*entity), masses.get(*entity)) {
            total += calculate_kinetic_energy(vel, mass);
        }
    }
    total
}

/// Trait for numerical integration methods
///
/// Integrators update position and velocity components based on forces
/// acting on entities. Different integrators trade off between accuracy,
/// stability, and computational cost.
pub trait Integrator: Send + Sync {
    /// Get the name of this integrator
    fn name(&self) -> &str;

    /// Get the timestep used by this integrator
    fn timestep(&self) -> f64;

    /// Set the timestep for this integrator
    ///
    /// # Panics
    ///
    /// Panics if timestep is non-positive, NaN, or infinite
    fn set_timestep(&mut self, dt: f64);

    /// Validate the timestep for stability
    ///
    /// Returns warnings if the timestep might cause numerical issues.
    /// Extremely small timesteps may lead to precision loss, while large
    /// timesteps may cause instability.
    fn validate_timestep(&self) -> Result<(), String> {
        let dt = self.timestep();
        
        if dt <= 0.0 || !dt.is_finite() {
            return Err(format!("Invalid timestep: {}. Must be positive and finite.", dt));
        }
        
        // Warn about very small timesteps (potential precision issues)
        if dt < 1e-9 {
            return Err(format!(
                "Warning: Timestep {} is extremely small and may cause precision loss with f64. \
                Consider using larger timestep or higher precision types.",
                dt
            ));
        }
        
        // Warn about very large timesteps (potential stability issues)
        if dt > 1.0 {
            return Err(format!(
                "Warning: Timestep {} is large and may cause instability. \
                Consider using smaller timesteps for better accuracy.",
                dt
            ));
        }
        
        Ok(())
    }

    /// Integrate motion for a collection of entities
    ///
    /// Updates position and velocity components based on forces and the
    /// integrator's numerical method. Returns the number of entities updated.
    ///
    /// # Arguments
    ///
    /// * `entities` - Iterator over entities to integrate
    /// * `positions` - Mutable storage for position components
    /// * `velocities` - Mutable storage for velocity components
    /// * `accelerations` - Storage for acceleration components (read-only)
    /// * `masses` - Storage for mass components (read-only)
    /// * `force_registry` - Registry for computing forces (used by some integrators)
    /// * `warn_on_missing` - Whether to log warnings for entities missing components
    ///
    /// # Returns
    ///
    /// Number of entities successfully updated
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
        I: Iterator<Item = &'a Entity>;
}

#[cfg(test)]
mod tests {
    // Simple harmonic oscillator test fixture
    // Mass-spring system: F = -kx, analytical solution: x(t) = A*cos(ωt + φ)
    struct HarmonicOscillator {
        spring_constant: f64,
        mass: f64,
        initial_position: f64,
        initial_velocity: f64,
    }

    impl HarmonicOscillator {
        fn new(k: f64, m: f64, x0: f64, v0: f64) -> Self {
            HarmonicOscillator {
                spring_constant: k,
                mass: m,
                initial_position: x0,
                initial_velocity: v0,
            }
        }

        fn omega(&self) -> f64 {
            (self.spring_constant / self.mass).sqrt()
        }

        fn amplitude(&self) -> f64 {
            let omega = self.omega();
            (self.initial_position.powi(2) + (self.initial_velocity / omega).powi(2)).sqrt()
        }

        fn phase(&self) -> f64 {
            let omega = self.omega();
            (-self.initial_velocity / (omega * self.initial_position)).atan()
        }

        fn position_at(&self, t: f64) -> f64 {
            let omega = self.omega();
            let amplitude = self.amplitude();
            let phase = self.phase();
            amplitude * (omega * t + phase).cos()
        }

        fn energy(&self, x: f64, v: f64) -> f64 {
            0.5 * self.mass * v * v + 0.5 * self.spring_constant * x * x
        }
    }

    #[test]
    fn test_harmonic_oscillator_physics() {
        let sho = HarmonicOscillator::new(100.0, 1.0, 1.0, 0.0);
        
        // Verify omega calculation
        let omega = (100.0_f64).sqrt();
        assert!((sho.omega() - omega).abs() < 1e-10);
        
        // At t=0, position should be initial position
        assert!((sho.position_at(0.0) - 1.0).abs() < 1e-10);
        
        // Energy should be constant
        let e0 = sho.energy(1.0, 0.0);
        let e1 = sho.energy(0.0, 10.0);
        assert!((e0 - e1).abs() < 1e-6); // Energy approximately conserved
    }
}
