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
//! Integration tests capturing known integrator failures
//!
//! These tests document the current failure modes in both Velocity Verlet
//! and RK4 integrators before remediation. They are marked with #[ignore]
//! to prevent CI failures, but can be run explicitly for investigation:
//!
//! ```bash
//! cargo test --test integration_failures -- --ignored
//! ```
//!
//! Once the integrators are fixed, these tests should pass and the #[ignore]
//! attribute can be removed.

use physics_engine::ecs::components::{Position, Velocity, Mass, Acceleration};
use physics_engine::ecs::systems::{ForceRegistry, ForceProvider, Force, apply_forces_to_acceleration};
use physics_engine::ecs::{Entity, HashMapStorage, ComponentStorage};
use physics_engine::integration::{VelocityVerletIntegrator, RK4Integrator, Integrator};
use physics_engine::plugins::gravity::{GravityPlugin, GravitySystem, GRAVITATIONAL_CONSTANT, DEFAULT_SOFTENING};

/// Constant force provider for testing
struct ConstantForce {
    force: Force,
}

impl ForceProvider for ConstantForce {
    fn compute_force(&self, _entity: Entity, _registry: &ForceRegistry) -> Option<Force> {
        Some(self.force)
    }
    fn name(&self) -> &str {
        "ConstantForce"
    }
}

/// Calculate two-body gravitational potential energy with softening
fn calculate_potential_energy_two_body(
    m1: f64,
    m2: f64,
    pos1: &Position,
    pos2: &Position,
    g: f64,
    softening: f64,
) -> f64 {
    let dx = pos2.x() - pos1.x();
    let dy = pos2.y() - pos1.y();
    let dz = pos2.z() - pos1.z();
    let r_squared = dx * dx + dy * dy + dz * dz;
    let r_softened = (r_squared + softening * softening).sqrt();
    -g * m1 * m2 / r_softened
}

/// Test that demonstrates kinetic energy should change under constant force
/// 
/// CURRENT BEHAVIOR: Kinetic energy remains constant (BUG)
/// EXPECTED BEHAVIOR: Kinetic energy should increase as potential energy decreases
///
/// This test captures the failure mode observed in solar_system example where
/// kinetic energy remains frozen while orbital radius increases.
#[test]
#[ignore = "Known failure - kinetic energy does not change under constant force"]
fn test_verlet_kinetic_energy_changes_under_constant_force() {
    let entity = Entity::new(1, 0);

    let mut positions = HashMapStorage::<Position>::new();
    positions.insert(entity, Position::new(0.0, 0.0, 0.0));

    let mut velocities = HashMapStorage::<Velocity>::new();
    let v0 = 10.0; // Initial velocity in x direction
    velocities.insert(entity, Velocity::new(v0, 0.0, 0.0));

    let mut accelerations = HashMapStorage::<Acceleration>::new();
    let a = 10.0; // Constant acceleration
    accelerations.insert(entity, Acceleration::new(a, 0.0, 0.0));

    let mut masses = HashMapStorage::<Mass>::new();
    let m = 1.0;
    masses.insert(entity, Mass::new(m));

    // Create constant force provider
    let mut force_registry = ForceRegistry::new();
    force_registry.register_provider(Box::new(ConstantForce {
        force: Force::new(m * a, 0.0, 0.0), // F = ma
    }));

    let dt = 0.01;
    let steps = 100;

    let mut integrator = VelocityVerletIntegrator::new(dt);
    let entities = vec![entity];

    // Calculate initial kinetic energy
    let initial_ke = 0.5 * m * v0 * v0;

    // Run integration
    for _ in 0..steps {
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

    // Calculate final kinetic energy
    let vel = velocities.get(entity).unwrap();
    let v_sq = vel.dx() * vel.dx() + vel.dy() * vel.dy() + vel.dz() * vel.dz();
    let final_ke = 0.5 * m * v_sq;

    // With constant acceleration, velocity should be: v = v0 + a*t
    let t = dt * steps as f64;
    let expected_v = v0 + a * t;
    let expected_ke = 0.5 * m * expected_v * expected_v;

    // Kinetic energy should increase
    assert!(
        final_ke > initial_ke * 1.5,
        "Kinetic energy should increase significantly under constant acceleration. \
         Initial KE: {:.3e}, Final KE: {:.3e}, Expected KE: {:.3e}",
        initial_ke,
        final_ke,
        expected_ke
    );

    // Check velocity change
    let v_mag = v_sq.sqrt();
    let velocity_error = (v_mag - expected_v).abs() / expected_v;
    assert!(
        velocity_error < 0.01,
        "Velocity should change under constant force. Error: {:.3}%",
        velocity_error * 100.0
    );
}

/// Test RK4 with the same scenario
#[test]
#[ignore = "Known failure - kinetic energy does not change under constant force"]
fn test_rk4_kinetic_energy_changes_under_constant_force() {
    let entity = Entity::new(1, 0);

    let mut positions = HashMapStorage::<Position>::new();
    positions.insert(entity, Position::new(0.0, 0.0, 0.0));

    let mut velocities = HashMapStorage::<Velocity>::new();
    let v0 = 10.0;
    velocities.insert(entity, Velocity::new(v0, 0.0, 0.0));

    let accelerations = HashMapStorage::<Acceleration>::new();

    let mut masses = HashMapStorage::<Mass>::new();
    let m = 1.0;
    masses.insert(entity, Mass::new(m));

    let a = 10.0;
    let mut force_registry = ForceRegistry::new();
    force_registry.register_provider(Box::new(ConstantForce {
        force: Force::new(m * a, 0.0, 0.0),
    }));

    let dt = 0.01;
    let steps = 100;

    let mut integrator = RK4Integrator::new(dt);
    let entities = vec![entity];

    let initial_ke = 0.5 * m * v0 * v0;

    for _ in 0..steps {
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

    let vel = velocities.get(entity).unwrap();
    let v_sq = vel.dx() * vel.dx() + vel.dy() * vel.dy() + vel.dz() * vel.dz();
    let final_ke = 0.5 * m * v_sq;

    let t = dt * steps as f64;
    let expected_v = v0 + a * t;
    let expected_ke = 0.5 * m * expected_v * expected_v;

    assert!(
        final_ke > initial_ke * 1.5,
        "Kinetic energy should increase significantly under constant acceleration. \
         Initial KE: {:.3e}, Final KE: {:.3e}, Expected KE: {:.3e}",
        initial_ke,
        final_ke,
        expected_ke
    );

    let v_mag = v_sq.sqrt();
    let velocity_error = (v_mag - expected_v).abs() / expected_v;
    assert!(
        velocity_error < 0.01,
        "Velocity should change under constant force. Error: {:.3}%",
        velocity_error * 100.0
    );
}

/// Test that circular orbit should remain stable
///
/// CURRENT BEHAVIOR: Orbit expands dramatically (Earth goes from 1 AU to 6.4 AU)
/// EXPECTED BEHAVIOR: Orbital radius should remain within ~10% of initial value
///
/// NOTE: This is a simplified one-body problem with a fixed central force.
/// The "sun" entity has such a large mass that its motion is negligible.
#[test]
#[ignore = "Known failure - circular orbits become unstable and expand"]
fn test_verlet_circular_orbit_stability() {
    // Simplified two-body problem: Sun and Earth
    // Sun is much more massive, so we treat it as approximately fixed
    let sun = Entity::new(1, 0);
    let earth = Entity::new(2, 0);

    let mut positions = HashMapStorage::<Position>::new();
    let mut velocities = HashMapStorage::<Velocity>::new();
    let mut accelerations = HashMapStorage::<Acceleration>::new();
    let mut masses = HashMapStorage::<Mass>::new();

    // Sun at origin with huge mass (so it barely moves)
    let m_sun = 1.989e30; // kg
    positions.insert(sun, Position::new(0.0, 0.0, 0.0));
    velocities.insert(sun, Velocity::new(0.0, 0.0, 0.0));
    accelerations.insert(sun, Acceleration::zero());
    masses.insert(sun, Mass::new(m_sun));

    // Earth at 1 AU with circular orbit velocity
    let au = 1.495978707e11; // meters
    let m_earth = 5.972e24; // kg
    positions.insert(earth, Position::new(au, 0.0, 0.0));

    // Circular orbit velocity: v = sqrt(G*M/r)
    let v_circular = (GRAVITATIONAL_CONSTANT * m_sun / au).sqrt();
    velocities.insert(earth, Velocity::new(0.0, v_circular, 0.0));
    accelerations.insert(earth, Acceleration::zero());
    masses.insert(earth, Mass::new(m_earth));

    let entities = vec![sun, earth];

    // Setup gravity system
    let gravity_plugin = GravityPlugin::new(GRAVITATIONAL_CONSTANT);
    let gravity_system = GravitySystem::new(gravity_plugin);
    let mut force_registry = ForceRegistry::new();
    force_registry.max_force_magnitude = 1e30; // Allow large gravitational forces
    force_registry.warn_on_missing_components = false; // Reduce noise in tests

    // Integrate for 1 year with 1-day timestep
    let dt = 86400.0; // 1 day in seconds
    let year = 365.25 * dt;
    let steps = (year / dt) as usize;

    let mut integrator = VelocityVerletIntegrator::new(dt);

    let initial_r = au;

    for _ in 0..steps {
        // Compute forces
        gravity_system.compute_forces(&entities, &positions, &masses, &mut force_registry);

        // Apply forces to accelerations
        apply_forces_to_acceleration(
            entities.iter(),
            &force_registry,
            &masses,
            &mut accelerations,
            false,
        );

        // Integrate
        integrator.integrate(
            entities.iter(),
            &mut positions,
            &mut velocities,
            &accelerations,
            &masses,
            &mut force_registry,
            false,
        );

        force_registry.clear_forces();
    }

    // Check Earth's final orbital radius
    let earth_pos = positions.get(earth).unwrap();
    let final_r = (earth_pos.x() * earth_pos.x() + 
                   earth_pos.y() * earth_pos.y() + 
                   earth_pos.z() * earth_pos.z()).sqrt();

    let radius_change = ((final_r - initial_r) / initial_r).abs();

    assert!(
        radius_change < 0.1,
        "Orbital radius should remain stable (< 10% change). \
         Initial: {:.3e} m ({:.3} AU), Final: {:.3e} m ({:.3} AU), Change: {:.1}%",
        initial_r,
        initial_r / au,
        final_r,
        final_r / au,
        radius_change * 100.0
    );
}

/// Test energy conservation in gravitational system
///
/// CURRENT BEHAVIOR: Total energy drifts by 175%
/// EXPECTED BEHAVIOR: Total energy drift should be < 10% for reasonable timestep
#[test]
#[ignore = "Known failure - massive energy drift (175%)"]
fn test_verlet_energy_conservation_gravity() {
    let sun = Entity::new(1, 0);
    let earth = Entity::new(2, 0);

    let mut positions = HashMapStorage::<Position>::new();
    let mut velocities = HashMapStorage::<Velocity>::new();
    let mut accelerations = HashMapStorage::<Acceleration>::new();
    let mut masses = HashMapStorage::<Mass>::new();

    let m_sun = 1.989e30;
    let m_earth = 5.972e24;
    let au = 1.495978707e11;

    positions.insert(sun, Position::new(0.0, 0.0, 0.0));
    velocities.insert(sun, Velocity::new(0.0, 0.0, 0.0));
    accelerations.insert(sun, Acceleration::zero());
    masses.insert(sun, Mass::new(m_sun));

    positions.insert(earth, Position::new(au, 0.0, 0.0));
    let v_circular = (GRAVITATIONAL_CONSTANT * m_sun / au).sqrt();
    velocities.insert(earth, Velocity::new(0.0, v_circular, 0.0));
    accelerations.insert(earth, Acceleration::zero());
    masses.insert(earth, Mass::new(m_earth));

    let entities = vec![sun, earth];

    let gravity_plugin = GravityPlugin::new(GRAVITATIONAL_CONSTANT);
    let gravity_system = GravitySystem::new(gravity_plugin);
    let mut force_registry = ForceRegistry::new();
    force_registry.max_force_magnitude = 1e30; // Allow large gravitational forces
    force_registry.warn_on_missing_components = false; // Reduce noise in tests

    // Calculate initial energy
    let initial_ke = 0.5 * m_earth * v_circular * v_circular;
    let sun_pos = positions.get(sun).unwrap();
    let earth_pos = positions.get(earth).unwrap();
    let initial_pe = calculate_potential_energy_two_body(
        m_sun,
        m_earth,
        sun_pos,
        earth_pos,
        GRAVITATIONAL_CONSTANT,
        DEFAULT_SOFTENING,
    );
    let initial_energy = initial_ke + initial_pe;

    let dt = 86400.0;
    let year = 365.25 * dt;
    let steps = (year / dt) as usize;

    let mut integrator = VelocityVerletIntegrator::new(dt);

    for _ in 0..steps {
        gravity_system.compute_forces(&entities, &positions, &masses, &mut force_registry);
        apply_forces_to_acceleration(
            entities.iter(),
            &force_registry,
            &masses,
            &mut accelerations,
            false,
        );
        integrator.integrate(
            entities.iter(),
            &mut positions,
            &mut velocities,
            &accelerations,
            &masses,
            &mut force_registry,
            false,
        );
        force_registry.clear_forces();
    }

    // Calculate final energy
    let earth_vel = velocities.get(earth).unwrap();
    let v_mag = (earth_vel.dx() * earth_vel.dx() + 
                 earth_vel.dy() * earth_vel.dy() + 
                 earth_vel.dz() * earth_vel.dz()).sqrt();
    let final_ke = 0.5 * m_earth * v_mag * v_mag;

    let sun_pos = positions.get(sun).unwrap();
    let earth_pos = positions.get(earth).unwrap();
    let final_pe = calculate_potential_energy_two_body(
        m_sun,
        m_earth,
        sun_pos,
        earth_pos,
        GRAVITATIONAL_CONSTANT,
        DEFAULT_SOFTENING,
    );
    let final_energy = final_ke + final_pe;

    let energy_drift = ((final_energy - initial_energy) / initial_energy).abs();

    assert!(
        energy_drift < 0.10,
        "Energy drift should be < 10% for dt=1 day. \
         Initial: {:.6e} J, Final: {:.6e} J, Drift: {:.1}%",
        initial_energy,
        final_energy,
        energy_drift * 100.0
    );
}
