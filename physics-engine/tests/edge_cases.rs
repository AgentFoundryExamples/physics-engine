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
//! Edge case tests for integrators
//!
//! Tests boundary conditions, extreme values, and unusual scenarios

use physics_engine::ecs::components::{Position, Velocity, Mass, Acceleration};
use physics_engine::ecs::systems::ForceRegistry;
use physics_engine::ecs::{Entity, HashMapStorage, ComponentStorage};
use physics_engine::integration::{VelocityVerletIntegrator, RK4Integrator, Integrator};

#[test]
#[should_panic(expected = "Timestep must be positive and finite")]
fn test_verlet_zero_timestep() {
    VelocityVerletIntegrator::new(0.0);
}

#[test]
#[should_panic(expected = "Timestep must be positive and finite")]
fn test_rk4_zero_timestep() {
    RK4Integrator::new(0.0);
}

#[test]
#[should_panic(expected = "Timestep must be positive and finite")]
fn test_verlet_negative_timestep() {
    VelocityVerletIntegrator::new(-0.01);
}

#[test]
#[should_panic(expected = "Timestep must be positive and finite")]
fn test_rk4_negative_timestep() {
    RK4Integrator::new(-0.01);
}

#[test]
#[should_panic(expected = "Timestep must be positive and finite")]
fn test_verlet_nan_timestep() {
    VelocityVerletIntegrator::new(f64::NAN);
}

#[test]
#[should_panic(expected = "Timestep must be positive and finite")]
fn test_rk4_nan_timestep() {
    RK4Integrator::new(f64::NAN);
}

#[test]
#[should_panic(expected = "Timestep must be positive and finite")]
fn test_verlet_infinite_timestep() {
    VelocityVerletIntegrator::new(f64::INFINITY);
}

#[test]
#[should_panic(expected = "Timestep must be positive and finite")]
fn test_rk4_infinite_timestep() {
    RK4Integrator::new(f64::INFINITY);
}

#[test]
fn test_verlet_very_small_timestep_validation() {
    let integrator = VelocityVerletIntegrator::new(1e-10);
    let result = integrator.validate_timestep();
    assert!(result.is_err(), "Very small timestep should trigger warning");
    assert!(result.unwrap_err().contains("extremely small"));
}

#[test]
fn test_rk4_very_small_timestep_validation() {
    let integrator = RK4Integrator::new(1e-10);
    let result = integrator.validate_timestep();
    assert!(result.is_err(), "Very small timestep should trigger warning");
    assert!(result.unwrap_err().contains("extremely small"));
}

#[test]
fn test_verlet_large_timestep_validation() {
    let integrator = VelocityVerletIntegrator::new(2.0);
    let result = integrator.validate_timestep();
    assert!(result.is_err(), "Large timestep should trigger warning");
    assert!(result.unwrap_err().contains("large"));
}

#[test]
fn test_rk4_large_timestep_validation() {
    let integrator = RK4Integrator::new(2.0);
    let result = integrator.validate_timestep();
    assert!(result.is_err(), "Large timestep should trigger warning");
    assert!(result.unwrap_err().contains("large"));
}

#[test]
fn test_verlet_reasonable_timestep_validation() {
    let integrator = VelocityVerletIntegrator::new(0.01);
    let result = integrator.validate_timestep();
    assert!(result.is_ok(), "Reasonable timestep should pass validation");
}

#[test]
fn test_rk4_reasonable_timestep_validation() {
    let integrator = RK4Integrator::new(0.01);
    let result = integrator.validate_timestep();
    assert!(result.is_ok(), "Reasonable timestep should pass validation");
}

#[test]
fn test_near_immovable_mass_consistency() {
    // Test that masses just above and below the immovable threshold behave consistently
    let entity_below = Entity::new(1, 0);
    let entity_above = Entity::new(2, 0);

    let mut positions = HashMapStorage::<Position>::new();
    positions.insert(entity_below, Position::new(0.0, 0.0, 0.0));
    positions.insert(entity_above, Position::new(0.0, 0.0, 0.0));

    let mut velocities = HashMapStorage::<Velocity>::new();
    velocities.insert(entity_below, Velocity::new(1.0, 0.0, 0.0));
    velocities.insert(entity_above, Velocity::new(1.0, 0.0, 0.0));

    let accelerations = HashMapStorage::<Acceleration>::new();
    
    let mut masses = HashMapStorage::<Mass>::new();
    masses.insert(entity_below, Mass::new(1e-11)); // Below Mass::IMMOVABLE_THRESHOLD
    masses.insert(entity_above, Mass::new(1e-9));  // Above Mass::IMMOVABLE_THRESHOLD

    let mut force_registry = ForceRegistry::new();
    let mut integrator = VelocityVerletIntegrator::new(0.01);

    let entities = vec![entity_below, entity_above];
    let count = integrator.integrate(
        entities.iter(),
        &mut positions,
        &mut velocities,
        &accelerations,
        &masses,
        &mut force_registry,
        false,
    );

    // Only entity_above should be integrated (entity_below is immovable)
    assert_eq!(count, 1, "Only one entity should be integrated");

    let pos_below = positions.get(entity_below).unwrap();
    let pos_above = positions.get(entity_above).unwrap();

    assert_eq!(pos_below.x(), 0.0, "Immovable entity should not move");
    assert!(pos_above.x() > 0.0, "Movable entity should move");
}

#[test]
fn test_empty_entity_list() {
    // Test that integrators handle empty entity lists gracefully
    let mut positions = HashMapStorage::<Position>::new();
    let mut velocities = HashMapStorage::<Velocity>::new();
    let accelerations = HashMapStorage::<Acceleration>::new();
    let masses = HashMapStorage::<Mass>::new();
    let mut force_registry = ForceRegistry::new();

    let mut integrator = VelocityVerletIntegrator::new(0.01);
    let entities: Vec<Entity> = vec![];

    let count = integrator.integrate(
        entities.iter(),
        &mut positions,
        &mut velocities,
        &accelerations,
        &masses,
        &mut force_registry,
        false,
    );

    assert_eq!(count, 0, "Empty entity list should result in zero updates");
}

#[test]
fn test_missing_components() {
    // Test that entities missing required components are skipped
    let entity = Entity::new(1, 0);

    let mut positions = HashMapStorage::<Position>::new();
    let mut velocities = HashMapStorage::<Velocity>::new();
    let accelerations = HashMapStorage::<Acceleration>::new();
    let masses = HashMapStorage::<Mass>::new();
    let mut force_registry = ForceRegistry::new();

    // Only add position, no velocity or mass
    positions.insert(entity, Position::new(0.0, 0.0, 0.0));

    let mut integrator = VelocityVerletIntegrator::new(0.01);
    let entities = vec![entity];

    let count = integrator.integrate(
        entities.iter(),
        &mut positions,
        &mut velocities,
        &accelerations,
        &masses,
        &mut force_registry,
        false, // Don't warn
    );

    assert_eq!(count, 0, "Entity with missing components should be skipped");
}

#[test]
fn test_rk4_buffer_reuse_thread_safety() {
    // Test that RK4 buffers are properly reused without conflicts
    let entity = Entity::new(1, 0);

    let mut positions = HashMapStorage::<Position>::new();
    positions.insert(entity, Position::new(0.0, 0.0, 0.0));

    let mut velocities = HashMapStorage::<Velocity>::new();
    velocities.insert(entity, Velocity::new(1.0, 0.0, 0.0));

    let accelerations = HashMapStorage::<Acceleration>::new();
    let mut masses = HashMapStorage::<Mass>::new();
    masses.insert(entity, Mass::new(1.0));

    let mut force_registry = ForceRegistry::new();
    let mut integrator = RK4Integrator::new(0.01);

    let entities = vec![entity];

    // Run multiple iterations to test buffer reuse
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

    // Should complete without errors
    let pos = positions.get(entity).unwrap();
    assert!(pos.is_valid(), "Position should remain valid after many iterations");
}

#[test]
fn test_entity_without_mass_treated_as_immovable() {
    // Test that entities without mass components are treated as immovable
    let entity = Entity::new(1, 0);

    let mut positions = HashMapStorage::<Position>::new();
    positions.insert(entity, Position::new(0.0, 0.0, 0.0));

    let mut velocities = HashMapStorage::<Velocity>::new();
    velocities.insert(entity, Velocity::new(10.0, 0.0, 0.0));

    let mut accelerations = HashMapStorage::<Acceleration>::new();
    accelerations.insert(entity, Acceleration::new(5.0, 0.0, 0.0));

    // No mass component inserted - entity should be treated as immovable
    let masses = HashMapStorage::<Mass>::new();

    let mut force_registry = ForceRegistry::new();
    let mut integrator = VelocityVerletIntegrator::new(0.01);

    let entities = vec![entity];
    let initial_pos = *positions.get(entity).unwrap();
    let initial_vel = *velocities.get(entity).unwrap();

    let count = integrator.integrate(
        entities.iter(),
        &mut positions,
        &mut velocities,
        &accelerations,
        &masses,
        &mut force_registry,
        false,
    );

    // Should skip entity without mass
    assert_eq!(count, 0, "Entity without mass should be skipped");

    let final_pos = positions.get(entity).unwrap();
    let final_vel = velocities.get(entity).unwrap();

    // Position and velocity should not change
    assert_eq!(final_pos.x(), initial_pos.x(), "Entity without mass should not move");
    assert_eq!(final_vel.dx(), initial_vel.dx(), "Entity without mass velocity should not change");
}

#[test]
fn test_extreme_velocity() {
    // Test that integrators handle very large velocities without overflow
    let entity = Entity::new(1, 0);

    let mut positions = HashMapStorage::<Position>::new();
    positions.insert(entity, Position::new(0.0, 0.0, 0.0));

    let mut velocities = HashMapStorage::<Velocity>::new();
    velocities.insert(entity, Velocity::new(1e10, 0.0, 0.0)); // Very large velocity

    let accelerations = HashMapStorage::<Acceleration>::new();
    let mut masses = HashMapStorage::<Mass>::new();
    masses.insert(entity, Mass::new(1.0));

    let mut force_registry = ForceRegistry::new();
    let mut integrator = VelocityVerletIntegrator::new(0.01);

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
    assert!(pos.is_valid(), "Position should remain valid with extreme velocity");
    assert!(pos.x().is_finite(), "Position should not overflow");
}
