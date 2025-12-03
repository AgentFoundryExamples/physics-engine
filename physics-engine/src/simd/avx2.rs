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
//! AVX2 vectorized implementation for x86_64 CPUs
//!
//! This module provides AVX2-accelerated physics computations that process
//! 4 × f64 values per instruction (256-bit vectors).
//!
//! # Requirements
//!
//! - x86_64 CPU with AVX2 support
//! - Detected automatically at runtime
//!
//! # Performance
//!
//! - Processes 4 entities per SIMD instruction
//! - Expected 2-4× speedup vs scalar for aligned workloads
//! - Best performance with entity counts divisible by 4

use super::SimdBackend;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// AVX2 backend for x86_64 CPUs
///
/// Processes 4 × f64 values per instruction using 256-bit AVX2 vectors.
pub struct Avx2Backend;

impl SimdBackend for Avx2Backend {
    fn name(&self) -> &str {
        "AVX2"
    }
    
    fn width(&self) -> usize {
        4 // Process 4 f64 values at once
    }
    
    fn is_supported(&self) -> bool {
        #[cfg(target_arch = "x86_64")]
        {
            is_x86_feature_detected!("avx2")
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            false
        }
    }
    
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn update_velocity_vectorized(
        &self,
        velocities: &mut [f64],
        accelerations: &[f64],
        dt: f64,
    ) {
        // v' = v + a * dt
        let dt_vec = _mm256_set1_pd(dt);
        let len = velocities.len();
        
        // Process 4 elements at a time
        let mut i = 0;
        while i + 4 <= len {
            // Load 4 velocity values
            let v = _mm256_loadu_pd(velocities.as_ptr().add(i));
            
            // Load 4 acceleration values
            let a = _mm256_loadu_pd(accelerations.as_ptr().add(i));
            
            // Compute: v' = v + a * dt
            let a_dt = _mm256_mul_pd(a, dt_vec);
            let v_new = _mm256_add_pd(v, a_dt);
            
            // Store result
            _mm256_storeu_pd(velocities.as_mut_ptr().add(i), v_new);
            
            i += 4;
        }
    }
    
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn update_position_vectorized(
        &self,
        positions: &mut [f64],
        velocities: &[f64],
        accelerations: &[f64],
        dt: f64,
        dt_sq_half: f64,
    ) {
        // p' = p + v * dt + 0.5 * a * dt²
        let dt_vec = _mm256_set1_pd(dt);
        let dt_sq_half_vec = _mm256_set1_pd(dt_sq_half);
        let len = positions.len();
        
        // Process 4 elements at a time
        let mut i = 0;
        while i + 4 <= len {
            // Load 4 position values
            let p = _mm256_loadu_pd(positions.as_ptr().add(i));
            
            // Load 4 velocity values
            let v = _mm256_loadu_pd(velocities.as_ptr().add(i));
            
            // Load 4 acceleration values
            let a = _mm256_loadu_pd(accelerations.as_ptr().add(i));
            
            // Compute: v * dt
            let v_dt = _mm256_mul_pd(v, dt_vec);
            
            // Compute: a * dt_sq_half
            let a_term = _mm256_mul_pd(a, dt_sq_half_vec);
            
            // Compute: p + v * dt + a * dt_sq_half
            let p_new = _mm256_add_pd(p, v_dt);
            let p_new = _mm256_add_pd(p_new, a_term);
            
            // Store result
            _mm256_storeu_pd(positions.as_mut_ptr().add(i), p_new);
            
            i += 4;
        }
    }
    
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn accumulate_forces_vectorized(
        &self,
        total_forces: &mut [f64],
        forces: &[f64],
    ) {
        // f_total += f
        let len = total_forces.len();
        
        // Process 4 elements at a time
        let mut i = 0;
        while i + 4 <= len {
            // Load 4 total force values
            let f_total = _mm256_loadu_pd(total_forces.as_ptr().add(i));
            
            // Load 4 force values
            let f = _mm256_loadu_pd(forces.as_ptr().add(i));
            
            // Add: f_total += f
            let f_new = _mm256_add_pd(f_total, f);
            
            // Store result
            _mm256_storeu_pd(total_forces.as_mut_ptr().add(i), f_new);
            
            i += 4;
        }
    }
    
    #[cfg(not(target_arch = "x86_64"))]
    unsafe fn update_velocity_vectorized(
        &self,
        _velocities: &mut [f64],
        _accelerations: &[f64],
        _dt: f64,
    ) {
        panic!("AVX2 backend not available on non-x86_64 platforms");
    }
    
    #[cfg(not(target_arch = "x86_64"))]
    unsafe fn update_position_vectorized(
        &self,
        _positions: &mut [f64],
        _velocities: &[f64],
        _accelerations: &[f64],
        _dt: f64,
        _dt_sq_half: f64,
    ) {
        panic!("AVX2 backend not available on non-x86_64 platforms");
    }
    
    #[cfg(not(target_arch = "x86_64"))]
    unsafe fn accumulate_forces_vectorized(
        &self,
        _total_forces: &mut [f64],
        _forces: &[f64],
    ) {
        panic!("AVX2 backend not available on non-x86_64 platforms");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_avx2_detection() {
        let backend = Avx2Backend;
        // Just check that the detection doesn't crash
        let _supported = backend.is_supported();
    }
    
    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_avx2_update_velocity() {
        let backend = Avx2Backend;
        if !backend.is_supported() {
            eprintln!("Skipping AVX2 test - not supported on this CPU");
            return;
        }
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
    #[cfg(target_arch = "x86_64")]
    fn test_avx2_update_position() {
        let backend = Avx2Backend;
        if !backend.is_supported() {
            eprintln!("Skipping AVX2 test - not supported on this CPU");
            return;
        }
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
    #[cfg(target_arch = "x86_64")]
    fn test_avx2_accumulate_forces() {
        let backend = Avx2Backend;
        if !backend.is_supported() {
            eprintln!("Skipping AVX2 test - not supported on this CPU");
            return;
        }
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
