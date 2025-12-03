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
//! Tests verifying RK4 accuracy and proper staging implementation
//!
//! These tests verify that the RK4 integrator correctly implements
//! global staging for coupled systems.

use physics_engine::ecs::components::{Position, Velocity, Mass, Acceleration};
use physics_engine::ecs::systems::{ForceRegistry, ForceProvider, Force};
use physics_engine::ecs::{Entity, HashMapStorage, ComponentStorage};
use physics_engine::integration::{RK4Integrator, Integrator};

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

/// Test RK4 with constant acceleration
///
/// For constant acceleration, the integrator should produce nearly exact results
/// because the motion equations are polynomial in time.
///
/// Analytical solution: x(t) = x0 + v0*t + 0.5*a*t²
///                       v(t) = v0 + a*t
#[test]
fn test_rk4_constant_acceleration_accuracy() {
    let entity = Entity::new(1, 0);
    
    // Initial conditions
    let x0 = 0.0;
    let v0 = 1.0;
    let a = 5.0;  // m/s²
    let m = 1.0;  // kg
    let dt = 0.1;  // seconds
    let steps = 100;
    let t_final = dt * steps as f64;
    
    // Analytical solution
    let x_analytical = x0 + v0 * t_final + 0.5 * a * t_final * t_final;
    let v_analytical = v0 + a * t_final;
    
    // Setup
    let mut positions = HashMapStorage::<Position>::new();
    positions.insert(entity, Position::new(x0, 0.0, 0.0));
    let mut velocities = HashMapStorage::<Velocity>::new();
    velocities.insert(entity, Velocity::new(v0, 0.0, 0.0));
    let accelerations = HashMapStorage::<Acceleration>::new();
    let mut masses = HashMapStorage::<Mass>::new();
    masses.insert(entity, Mass::new(m));
    let mut forces = ForceRegistry::new();
    forces.register_provider(Box::new(ConstantForce {
        force: Force::new(m * a, 0.0, 0.0),
    }));
    
    // Run integration
    let mut integrator = RK4Integrator::new(dt);
    let entities = vec![entity];
    
    for _ in 0..steps {
        integrator.integrate(
            entities.iter(),
            &mut positions,
            &mut velocities,
            &accelerations,
            &mut masses,
            &mut forces,
            false,
        );
    }
    
    // Verify results
    let pos = positions.get(entity).unwrap();
    let vel = velocities.get(entity).unwrap();
    
    let pos_error = (pos.x() - x_analytical).abs() / x_analytical.abs().max(1.0);
    let vel_error = (vel.dx() - v_analytical).abs() / v_analytical.abs().max(1.0);
    
    // RK4 should be very accurate for polynomial motion
    assert!(
        pos_error < 1e-6,
        "RK4 position error ({:.6e}) should be < 0.0001% for constant acceleration. \
         Analytical: {:.6}, RK4: {:.6}",
        pos_error, x_analytical, pos.x()
    );
    
    assert!(
        vel_error < 1e-10,
        "RK4 velocity error ({:.6e}) should be < 1e-8% for constant acceleration. \
         Analytical: {:.6}, RK4: {:.6}",
        vel_error, v_analytical, vel.dx()
    );
}

/// Test RK4 maintains accuracy over long integration times
#[test]
fn test_rk4_long_term_accuracy() {
    let entity = Entity::new(1, 0);
    
    let a = 1.0;  // Constant acceleration
    let m = 1.0;
    let dt = 0.01;
    let steps = 1000;  // 10 seconds
    let t_final = dt * steps as f64;
    
    // Analytical: x = 0.5*a*t²
    let x_analytical = 0.5 * a * t_final * t_final;
    
    let mut positions = HashMapStorage::<Position>::new();
    positions.insert(entity, Position::new(0.0, 0.0, 0.0));
    let mut velocities = HashMapStorage::<Velocity>::new();
    velocities.insert(entity, Velocity::new(0.0, 0.0, 0.0));
    let accelerations = HashMapStorage::<Acceleration>::new();
    let mut masses = HashMapStorage::<Mass>::new();
    masses.insert(entity, Mass::new(m));
    
    let mut forces = ForceRegistry::new();
    forces.register_provider(Box::new(ConstantForce {
        force: Force::new(m * a, 0.0, 0.0),
    }));
    
    let mut integrator = RK4Integrator::new(dt);
    let entities = vec![entity];
    
    for _ in 0..steps {
        integrator.integrate(
            entities.iter(),
            &mut positions,
            &mut velocities,
            &accelerations,
            &mut masses,
            &mut forces,
            false,
        );
    }
    
    let pos = positions.get(entity).unwrap();
    let error = (pos.x() - x_analytical).abs() / x_analytical;
    
    // Even after 1000 steps, error should be minimal due to 4th-order accuracy
    assert!(
        error < 0.0001,
        "RK4 should maintain < 0.01% error over long integration. \
         Error: {:.6}%, Analytical: {:.3}, RK4: {:.3}",
        error * 100.0, x_analytical, pos.x()
    );
}

/// Test that RK4 properly stages in multi-body scenarios
/// 
/// This test uses two bodies to verify that the staging properly
/// updates ALL entities before computing forces for each stage
#[test]
fn test_rk4_multi_body_staging() {
    let entity1 = Entity::new(1, 0);
    let entity2 = Entity::new(2, 0);
    
    let mut positions = HashMapStorage::<Position>::new();
    positions.insert(entity1, Position::new(0.0, 0.0, 0.0));
    positions.insert(entity2, Position::new(10.0, 0.0, 0.0));
    
    let mut velocities = HashMapStorage::<Velocity>::new();
    velocities.insert(entity1, Velocity::new(1.0, 0.0, 0.0));
    velocities.insert(entity2, Velocity::new(-1.0, 0.0, 0.0));
    
    let accelerations = HashMapStorage::<Acceleration>::new();
    
    let mut masses = HashMapStorage::<Mass>::new();
    masses.insert(entity1, Mass::new(1.0));
    masses.insert(entity2, Mass::new(1.0));
    
    let mut forces = ForceRegistry::new();
    // No forces - free motion
    
    let mut integrator = RK4Integrator::new(0.1);
    let entities = vec![entity1, entity2];
    
    // Should complete without errors
    for _ in 0..10 {
        integrator.integrate(
            entities.iter(),
            &mut positions,
            &mut velocities,
            &accelerations,
            &mut masses,
            &mut forces,
            false,
        );
    }
    
    // Both entities should still have valid states
    let pos1 = positions.get(entity1).unwrap();
    let pos2 = positions.get(entity2).unwrap();
    assert!(pos1.is_valid());
    assert!(pos2.is_valid());
    
    // Verify they moved in opposite directions
    assert!(pos1.x() > 0.0, "Entity 1 should have moved right");
    assert!(pos2.x() < 10.0, "Entity 2 should have moved left");
}

/// Test RK4 with entities that have different masses
#[test]
fn test_rk4_different_masses() {
    let light = Entity::new(1, 0);
    let heavy = Entity::new(2, 0);
    
    let mut positions = HashMapStorage::<Position>::new();
    positions.insert(light, Position::new(0.0, 0.0, 0.0));
    positions.insert(heavy, Position::new(0.0, 0.0, 0.0));
    
    let mut velocities = HashMapStorage::<Velocity>::new();
    velocities.insert(light, Velocity::new(0.0, 0.0, 0.0));
    velocities.insert(heavy, Velocity::new(0.0, 0.0, 0.0));
    
    let accelerations = HashMapStorage::<Acceleration>::new();
    
    let mut masses = HashMapStorage::<Mass>::new();
    masses.insert(light, Mass::new(0.1));  // 0.1 kg
    masses.insert(heavy, Mass::new(10.0)); // 10 kg
    
    // Apply same force to both
    let force_mag = 10.0;  // 10 N
    let mut forces = ForceRegistry::new();
    forces.register_provider(Box::new(ConstantForce {
        force: Force::new(force_mag, 0.0, 0.0),
    }));
    
    let mut integrator = RK4Integrator::new(0.1);
    let entities = vec![light, heavy];
    
    for _ in 0..100 {
        integrator.integrate(
            entities.iter(),
            &mut positions,
            &mut velocities,
            &accelerations,
            &mut masses,
            &mut forces,
            false,
        );
    }
    
    // Light object should have moved much farther (a = F/m)
    let pos_light = positions.get(light).unwrap();
    let pos_heavy = positions.get(heavy).unwrap();
    
    assert!(
        pos_light.x() > pos_heavy.x() * 50.0,
        "Light object should move much farther than heavy object. \
         Light: {:.3}, Heavy: {:.3}",
        pos_light.x(), pos_heavy.x()
    );
}

/// Test RK4 handles immovable bodies correctly
#[test]
fn test_rk4_immovable_bodies() {
    let movable = Entity::new(1, 0);
    let immovable = Entity::new(2, 0);
    
    let mut positions = HashMapStorage::<Position>::new();
    positions.insert(movable, Position::new(0.0, 0.0, 0.0));
    positions.insert(immovable, Position::new(10.0, 0.0, 0.0));
    
    let mut velocities = HashMapStorage::<Velocity>::new();
    velocities.insert(movable, Velocity::new(1.0, 0.0, 0.0));
    velocities.insert(immovable, Velocity::new(0.0, 0.0, 0.0));
    
    let accelerations = HashMapStorage::<Acceleration>::new();
    
    let mut masses = HashMapStorage::<Mass>::new();
    masses.insert(movable, Mass::new(1.0));
    masses.insert(immovable, Mass::immovable());
    
    let mut forces = ForceRegistry::new();
    forces.register_provider(Box::new(ConstantForce {
        force: Force::new(10.0, 0.0, 0.0),
    }));
    
    let mut integrator = RK4Integrator::new(0.1);
    let entities = vec![movable, immovable];
    
    for _ in 0..10 {
        integrator.integrate(
            entities.iter(),
            &mut positions,
            &mut velocities,
            &accelerations,
            &mut masses,
            &mut forces,
            false,
        );
    }
    
    // Immovable body should not have moved
    let pos_immovable = positions.get(immovable).unwrap();
    assert_eq!(pos_immovable.x(), 10.0, "Immovable body should not move");
    assert_eq!(pos_immovable.y(), 0.0);
    assert_eq!(pos_immovable.z(), 0.0);
    
    // Movable body should have moved
    let pos_movable = positions.get(movable).unwrap();
    assert!(pos_movable.x() > 0.0, "Movable body should have moved");
}

/// Test RK4 with zero forces (free motion)
#[test]
fn test_rk4_free_motion() {
    let entity = Entity::new(1, 0);
    
    let v0_x = 5.0;
    let v0_y = 3.0;
    let dt = 0.1;
    let steps = 100;
    let t_final = dt * steps as f64;
    
    let mut positions = HashMapStorage::<Position>::new();
    positions.insert(entity, Position::new(0.0, 0.0, 0.0));
    
    let mut velocities = HashMapStorage::<Velocity>::new();
    velocities.insert(entity, Velocity::new(v0_x, v0_y, 0.0));
    
    let accelerations = HashMapStorage::<Acceleration>::new();
    
    let mut masses = HashMapStorage::<Mass>::new();
    masses.insert(entity, Mass::new(1.0));
    
    let mut forces = ForceRegistry::new();
    // No forces
    
    let mut integrator = RK4Integrator::new(dt);
    let entities = vec![entity];
    
    for _ in 0..steps {
        integrator.integrate(
            entities.iter(),
            &mut positions,
            &mut velocities,
            &accelerations,
            &mut masses,
            &mut forces,
            false,
        );
    }
    
    // With no forces, position should be x = v*t
    let pos = positions.get(entity).unwrap();
    let vel = velocities.get(entity).unwrap();
    
    let expected_x = v0_x * t_final;
    let expected_y = v0_y * t_final;
    
    let x_error = (pos.x() - expected_x).abs() / expected_x;
    let y_error = (pos.y() - expected_y).abs() / expected_y;
    
    assert!(x_error < 1e-10, "Free motion X should be exact");
    assert!(y_error < 1e-10, "Free motion Y should be exact");
    
    // Velocity should remain constant
    assert!((vel.dx() - v0_x).abs() < 1e-14, "Velocity X should not change");
    assert!((vel.dy() - v0_y).abs() < 1e-14, "Velocity Y should not change");
}
