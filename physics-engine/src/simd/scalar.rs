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
//! Scalar fallback implementation for SIMD operations
//!
//! This module provides a pure scalar implementation that serves as:
//! - Fallback for CPUs without SIMD support
//! - Reference implementation for testing SIMD correctness
//! - Tail handler for entity counts not divisible by SIMD width

use super::SimdBackend;

/// Scalar backend that processes one element at a time
///
/// Always available on all platforms. Used as fallback when no SIMD
/// instructions are available, or for processing remainder elements.
pub struct ScalarBackend;

impl SimdBackend for ScalarBackend {
    fn name(&self) -> &str {
        "Scalar"
    }
    
    fn width(&self) -> usize {
        1
    }
    
    fn is_supported(&self) -> bool {
        true // Always available
    }
    
    unsafe fn update_velocity_vectorized(
        &self,
        velocities: &mut [f64],
        accelerations: &[f64],
        dt: f64,
    ) {
        // v' = v + a * dt
        for i in 0..velocities.len() {
            velocities[i] += accelerations[i] * dt;
        }
    }
    
    unsafe fn update_position_vectorized(
        &self,
        positions: &mut [f64],
        velocities: &[f64],
        accelerations: &[f64],
        dt: f64,
        dt_sq_half: f64,
    ) {
        // p' = p + v * dt + 0.5 * a * dt²
        for i in 0..positions.len() {
            positions[i] += velocities[i] * dt + accelerations[i] * dt_sq_half;
        }
    }
    
    unsafe fn accumulate_forces_vectorized(
        &self,
        total_forces: &mut [f64],
        forces: &[f64],
    ) {
        // f_total += f
        for i in 0..total_forces.len() {
            total_forces[i] += forces[i];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_scalar_backend_always_supported() {
        let backend = ScalarBackend;
        assert!(backend.is_supported());
    }
    
    #[test]
    fn test_scalar_update_velocity() {
        let backend = ScalarBackend;
        let mut velocities = vec![1.0, 2.0, 3.0, 4.0];
        let accelerations = vec![0.5, 1.0, 1.5, 2.0];
        let dt = 0.1;
        
        unsafe {
            backend.update_velocity_vectorized(&mut velocities, &accelerations, dt);
        }
        
        // v' = v + a * dt
        assert!((velocities[0] - 1.05).abs() < 1e-10);
        assert!((velocities[1] - 2.1).abs() < 1e-10);
        assert!((velocities[2] - 3.15).abs() < 1e-10);
        assert!((velocities[3] - 4.2).abs() < 1e-10);
    }
    
    #[test]
    fn test_scalar_update_position() {
        let backend = ScalarBackend;
        let mut positions = vec![0.0, 1.0, 2.0, 3.0];
        let velocities = vec![10.0, 20.0, 30.0, 40.0];
        let accelerations = vec![1.0, 2.0, 3.0, 4.0];
        let dt = 0.1;
        let dt_sq_half = 0.5 * dt * dt;
        
        unsafe {
            backend.update_position_vectorized(
                &mut positions,
                &velocities,
                &accelerations,
                dt,
                dt_sq_half,
            );
        }
        
        // p' = p + v * dt + 0.5 * a * dt²
        assert!((positions[0] - (0.0 + 10.0 * 0.1 + 1.0 * dt_sq_half)).abs() < 1e-10);
        assert!((positions[1] - (1.0 + 20.0 * 0.1 + 2.0 * dt_sq_half)).abs() < 1e-10);
        assert!((positions[2] - (2.0 + 30.0 * 0.1 + 3.0 * dt_sq_half)).abs() < 1e-10);
        assert!((positions[3] - (3.0 + 40.0 * 0.1 + 4.0 * dt_sq_half)).abs() < 1e-10);
    }
    
    #[test]
    fn test_scalar_accumulate_forces() {
        let backend = ScalarBackend;
        let mut total_forces = vec![1.0, 2.0, 3.0, 4.0];
        let forces = vec![0.5, 1.0, 1.5, 2.0];
        
        unsafe {
            backend.accumulate_forces_vectorized(&mut total_forces, &forces);
        }
        
        assert_eq!(total_forces[0], 1.5);
        assert_eq!(total_forces[1], 3.0);
        assert_eq!(total_forces[2], 4.5);
        assert_eq!(total_forces[3], 6.0);
    }
}
