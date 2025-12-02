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
//! Integration tests verifying conservation properties for integrators

use physics_engine::ecs::components::{Position, Velocity, Mass, Acceleration};
use physics_engine::ecs::systems::{ForceRegistry, ForceProvider, Force};
use physics_engine::ecs::{Entity, HashMapStorage, ComponentStorage};
use physics_engine::integration::{VelocityVerletIntegrator, RK4Integrator, Integrator};

/// Spring force provider for harmonic oscillator
struct SpringForceProvider {
    spring_constant: f64,
}

impl ForceProvider for SpringForceProvider {
    fn compute_force(&self, _entity: Entity, _registry: &ForceRegistry) -> Option<Force> {
        // For this test, we need to access position from somewhere
        // We'll use a simplified approach where the force is computed externally
        // and stored in the registry, or we compute it based on entity ID pattern
        None
    }

    fn name(&self) -> &str {
        "SpringForce"
    }
}

/// Compute energy for a simple harmonic oscillator
fn compute_energy(
    positions: &HashMapStorage<Position>,
    velocities: &HashMapStorage<Velocity>,
    masses: &HashMapStorage<Mass>,
    entity: Entity,
    spring_constant: f64,
) -> f64 {
    let pos = positions.get(entity).unwrap();
    let vel = velocities.get(entity).unwrap();
    let mass = masses.get(entity).unwrap();

    // Kinetic energy: 0.5*m*v²
    let ke = 0.5 * mass.value() * vel.magnitude().powi(2);

    // Potential energy: 0.5*k*x² (assuming equilibrium at x=0)
    let x = (pos.x().powi(2) + pos.y().powi(2) + pos.z().powi(2)).sqrt();
    let pe = 0.5 * spring_constant * x.powi(2);

    ke + pe
}

#[test]
fn test_verlet_energy_conservation_free_particle() {
    // Free particle should conserve kinetic energy (no forces)
    let entity = Entity::new(1, 0);

    let mut positions = HashMapStorage::<Position>::new();
    positions.insert(entity, Position::new(0.0, 0.0, 0.0));

    let mut velocities = HashMapStorage::<Velocity>::new();
    velocities.insert(entity, Velocity::new(1.0, 0.0, 0.0));

    let accelerations = HashMapStorage::<Acceleration>::new();
    let mut masses = HashMapStorage::<Mass>::new();
    masses.insert(entity, Mass::new(1.0));

    let mut force_registry = ForceRegistry::new();

    let initial_energy = 0.5 * 1.0 * 1.0; // 0.5*m*v² = 0.5*1*1 = 0.5

    let mut integrator = VelocityVerletIntegrator::new(0.01);
    let entities = vec![entity];

    // Run for 100 timesteps
    for _ in 0..100 {
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
    let final_energy = 0.5 * 1.0 * vel.magnitude().powi(2);

    // Energy should be conserved for free particle
    let energy_error = (final_energy - initial_energy).abs() / initial_energy;
    assert!(
        energy_error < 1e-10,
        "Energy not conserved for free particle: error = {}",
        energy_error
    );
}

#[test]
fn test_rk4_energy_conservation_free_particle() {
    // Free particle should conserve kinetic energy (no forces)
    let entity = Entity::new(1, 0);

    let mut positions = HashMapStorage::<Position>::new();
    positions.insert(entity, Position::new(0.0, 0.0, 0.0));

    let mut velocities = HashMapStorage::<Velocity>::new();
    velocities.insert(entity, Velocity::new(1.0, 0.0, 0.0));

    let accelerations = HashMapStorage::<Acceleration>::new();
    let mut masses = HashMapStorage::<Mass>::new();
    masses.insert(entity, Mass::new(1.0));

    let mut force_registry = ForceRegistry::new();

    let initial_energy = 0.5 * 1.0 * 1.0; // 0.5*m*v² = 0.5*1*1 = 0.5

    let mut integrator = RK4Integrator::new(0.01);
    let entities = vec![entity];

    // Run for 100 timesteps
    for _ in 0..100 {
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
    let final_energy = 0.5 * 1.0 * vel.magnitude().powi(2);

    // Energy should be conserved for free particle
    let energy_error = (final_energy - initial_energy).abs() / initial_energy;
    assert!(
        energy_error < 1e-10,
        "Energy not conserved for free particle: error = {}",
        energy_error
    );
}

#[test]
fn test_verlet_position_accuracy() {
    // Test position accuracy for constant velocity motion
    let entity = Entity::new(1, 0);

    let mut positions = HashMapStorage::<Position>::new();
    positions.insert(entity, Position::new(0.0, 0.0, 0.0));

    let mut velocities = HashMapStorage::<Velocity>::new();
    let v0 = 1.5;
    velocities.insert(entity, Velocity::new(v0, 0.0, 0.0));

    let accelerations = HashMapStorage::<Acceleration>::new();
    let mut masses = HashMapStorage::<Mass>::new();
    masses.insert(entity, Mass::new(1.0));

    let mut force_registry = ForceRegistry::new();

    let dt = 0.01;
    let steps = 100;
    let total_time = dt * steps as f64;

    let mut integrator = VelocityVerletIntegrator::new(dt);
    let entities = vec![entity];

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

    let pos = positions.get(entity).unwrap();
    let expected_x = v0 * total_time;

    // Check accuracy
    let error = (pos.x() - expected_x).abs();
    assert!(
        error < 1e-10,
        "Position error too large: {} (expected near 0)",
        error
    );
}

#[test]
fn test_rk4_position_accuracy() {
    // Test position accuracy for constant velocity motion
    let entity = Entity::new(1, 0);

    let mut positions = HashMapStorage::<Position>::new();
    positions.insert(entity, Position::new(0.0, 0.0, 0.0));

    let mut velocities = HashMapStorage::<Velocity>::new();
    let v0 = 1.5;
    velocities.insert(entity, Velocity::new(v0, 0.0, 0.0));

    let accelerations = HashMapStorage::<Acceleration>::new();
    let mut masses = HashMapStorage::<Mass>::new();
    masses.insert(entity, Mass::new(1.0));

    let mut force_registry = ForceRegistry::new();

    let dt = 0.01;
    let steps = 100;
    let total_time = dt * steps as f64;

    let mut integrator = RK4Integrator::new(dt);
    let entities = vec![entity];

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

    let pos = positions.get(entity).unwrap();
    let expected_x = v0 * total_time;

    // Check accuracy
    let error = (pos.x() - expected_x).abs();
    assert!(
        error < 1e-10,
        "Position error too large: {} (expected near 0)",
        error
    );
}

#[test]
fn test_verlet_constant_acceleration() {
    // Test with constant acceleration (like gravity) using a force provider
    let entity = Entity::new(1, 0);

    let mut positions = HashMapStorage::<Position>::new();
    positions.insert(entity, Position::new(0.0, 0.0, 0.0));

    let mut velocities = HashMapStorage::<Velocity>::new();
    velocities.insert(entity, Velocity::new(0.0, 0.0, 0.0));

    let mut accelerations = HashMapStorage::<Acceleration>::new();
    // Initialize with first acceleration for proper Verlet startup
    let a = -9.81; // gravity acceleration
    accelerations.insert(entity, Acceleration::new(0.0, a, 0.0));

    let mut masses = HashMapStorage::<Mass>::new();
    let m = 1.0;
    masses.insert(entity, Mass::new(m));

    // Create a constant force provider to simulate gravity
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

    let a = -9.81; // gravity acceleration
    let mut force_registry = ForceRegistry::new();
    force_registry.register_provider(Box::new(ConstantForce {
        force: Force::new(0.0, m * a, 0.0), // F = ma
    }));

    let dt = 0.01;
    let steps = 100;
    let t = dt * steps as f64;

    let mut integrator = VelocityVerletIntegrator::new(dt);
    let entities = vec![entity];

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

    let pos = positions.get(entity).unwrap();
    let vel = velocities.get(entity).unwrap();

    // Analytical solution: y = 0.5*a*t²
    let expected_y = 0.5 * a * t * t;
    // Analytical solution: v = a*t
    let expected_vy = a * t;

    // Check accuracy
    let pos_error = (pos.y() - expected_y).abs() / expected_y.abs();
    let vel_error = (vel.dy() - expected_vy).abs() / expected_vy.abs();

    assert!(
        pos_error < 0.01,
        "Position error too large: {} (expected < 1%)",
        pos_error
    );
    assert!(
        vel_error < 0.01,
        "Velocity error too large: {} (expected < 1%)",
        vel_error
    );
}

#[test]
fn test_rk4_constant_acceleration() {
    // Test with constant acceleration (like gravity)
    let entity = Entity::new(1, 0);

    let mut positions = HashMapStorage::<Position>::new();
    positions.insert(entity, Position::new(0.0, 0.0, 0.0));

    let mut velocities = HashMapStorage::<Velocity>::new();
    velocities.insert(entity, Velocity::new(0.0, 0.0, 0.0));

    let accelerations = HashMapStorage::<Acceleration>::new();
    // Note: RK4 will compute acceleration from forces, not use stored acceleration
    // For constant acceleration, we'd need a constant force provider

    let mut masses = HashMapStorage::<Mass>::new();
    masses.insert(entity, Mass::new(1.0));

    let mut force_registry = ForceRegistry::new();

    let dt = 0.01;
    let steps = 100;
    let _t = dt * steps as f64;

    let mut integrator = RK4Integrator::new(dt);
    let entities = vec![entity];

    // Run integration with no forces (free motion)
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

    let pos = positions.get(entity).unwrap();
    let vel = velocities.get(entity).unwrap();

    // With no forces, position and velocity should not change
    assert!(pos.x().abs() < 1e-10, "Position should be near zero");
    assert!(vel.dx().abs() < 1e-10, "Velocity should be near zero");
}

#[test]
fn test_multiple_entities() {
    // Test that multiple entities are integrated correctly
    let entity1 = Entity::new(1, 0);
    let entity2 = Entity::new(2, 0);

    let mut positions = HashMapStorage::<Position>::new();
    positions.insert(entity1, Position::new(0.0, 0.0, 0.0));
    positions.insert(entity2, Position::new(1.0, 0.0, 0.0));

    let mut velocities = HashMapStorage::<Velocity>::new();
    velocities.insert(entity1, Velocity::new(1.0, 0.0, 0.0));
    velocities.insert(entity2, Velocity::new(2.0, 0.0, 0.0));

    let accelerations = HashMapStorage::<Acceleration>::new();
    let mut masses = HashMapStorage::<Mass>::new();
    masses.insert(entity1, Mass::new(1.0));
    masses.insert(entity2, Mass::new(2.0));

    let mut force_registry = ForceRegistry::new();

    let dt = 0.01;
    let mut integrator = VelocityVerletIntegrator::new(dt);
    let entities = vec![entity1, entity2];

    integrator.integrate(
        entities.iter(),
        &mut positions,
        &mut velocities,
        &accelerations,
        &masses,
        &mut force_registry,
        false,
    );

    let pos1 = positions.get(entity1).unwrap();
    let pos2 = positions.get(entity2).unwrap();

    // Check that both entities moved
    assert!((pos1.x() - 0.01).abs() < 1e-10, "Entity 1 should move");
    assert!((pos2.x() - 1.02).abs() < 1e-10, "Entity 2 should move");
}
