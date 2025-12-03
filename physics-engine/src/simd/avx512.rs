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
//! AVX-512 vectorized implementation for x86_64 CPUs
//!
//! This module provides AVX-512-accelerated physics computations that process
//! 8 × f64 values per instruction (512-bit vectors).
//!
//! # Requirements
//!
//! - x86_64 CPU with AVX-512F and AVX-512DQ support
//! - Detected automatically at runtime
//!
//! # Performance
//!
//! - Processes 8 entities per SIMD instruction
//! - Expected 4-8× speedup vs scalar for aligned workloads
//! - Expected 2× speedup vs AVX2 for aligned workloads
//! - Best performance with entity counts divisible by 8

use super::SimdBackend;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// AVX-512 backend for x86_64 CPUs
///
/// Processes 8 × f64 values per instruction using 512-bit AVX-512 vectors.
pub struct Avx512Backend;

impl SimdBackend for Avx512Backend {
    fn name(&self) -> &str {
        "AVX-512"
    }
    
    fn width(&self) -> usize {
        8 // Process 8 f64 values at once
    }
    
    fn is_supported(&self) -> bool {
        #[cfg(target_arch = "x86_64")]
        {
            is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512dq")
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            false
        }
    }
    
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx512f")]
    #[target_feature(enable = "avx512dq")]
    unsafe fn update_velocity_vectorized(
        &self,
        velocities: &mut [f64],
        accelerations: &[f64],
        dt: f64,
    ) {
        // v' = v + a * dt
        let dt_vec = _mm512_set1_pd(dt);
        
        // Process 8 elements at a time using zip for safety
        for (v_chunk, a_chunk) in velocities.chunks_exact_mut(8).zip(accelerations.chunks_exact(8)) {
            // Load 8 velocity values
            let v = _mm512_loadu_pd(v_chunk.as_ptr());
            
            // Load 8 acceleration values
            let a = _mm512_loadu_pd(a_chunk.as_ptr());
            
            // Compute: v' = v + a * dt
            let a_dt = _mm512_mul_pd(a, dt_vec);
            let v_new = _mm512_add_pd(v, a_dt);
            
            // Store result
            _mm512_storeu_pd(v_chunk.as_mut_ptr(), v_new);
        }
    }
    
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx512f")]
    #[target_feature(enable = "avx512dq")]
    unsafe fn update_position_vectorized(
        &self,
        positions: &mut [f64],
        velocities: &[f64],
        accelerations: &[f64],
        dt: f64,
        dt_sq_half: f64,
    ) {
        // p' = p + v * dt + 0.5 * a * dt²
        let dt_vec = _mm512_set1_pd(dt);
        let dt_sq_half_vec = _mm512_set1_pd(dt_sq_half);
        
        // Process 8 elements at a time using zip for safety
        for ((p_chunk, v_chunk), a_chunk) in positions.chunks_exact_mut(8)
            .zip(velocities.chunks_exact(8))
            .zip(accelerations.chunks_exact(8))
        {
            // Load 8 position values
            let p = _mm512_loadu_pd(p_chunk.as_ptr());
            
            // Load 8 velocity values
            let v = _mm512_loadu_pd(v_chunk.as_ptr());
            
            // Load 8 acceleration values
            let a = _mm512_loadu_pd(a_chunk.as_ptr());
            
            // Compute: v * dt
            let v_dt = _mm512_mul_pd(v, dt_vec);
            
            // Compute: a * dt_sq_half
            let a_term = _mm512_mul_pd(a, dt_sq_half_vec);
            
            // Compute: p + v * dt + a * dt_sq_half
            let p_new = _mm512_add_pd(p, v_dt);
            let p_new = _mm512_add_pd(p_new, a_term);
            
            // Store result
            _mm512_storeu_pd(p_chunk.as_mut_ptr(), p_new);
        }
    }
    
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx512f")]
    #[target_feature(enable = "avx512dq")]
    unsafe fn accumulate_forces_vectorized(
        &self,
        total_forces: &mut [f64],
        forces: &[f64],
    ) {
        // f_total += f
        
        // Process 8 elements at a time using zip for safety
        for (f_total_chunk, f_chunk) in total_forces.chunks_exact_mut(8).zip(forces.chunks_exact(8)) {
            // Load 8 total force values
            let f_total = _mm512_loadu_pd(f_total_chunk.as_ptr());
            
            // Load 8 force values
            let f = _mm512_loadu_pd(f_chunk.as_ptr());
            
            // Add: f_total += f
            let f_new = _mm512_add_pd(f_total, f);
            
            // Store result
            _mm512_storeu_pd(f_total_chunk.as_mut_ptr(), f_new);
        }
    }
    
    #[cfg(not(target_arch = "x86_64"))]
    unsafe fn update_velocity_vectorized(
        &self,
        _velocities: &mut [f64],
        _accelerations: &[f64],
        _dt: f64,
    ) {
        panic!("AVX-512 backend is not available on non-x86_64 platforms. Use ScalarBackend instead or check is_supported() before use.");
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
        panic!("AVX-512 backend is not available on non-x86_64 platforms. Use ScalarBackend instead or check is_supported() before use.");
    }
    
    #[cfg(not(target_arch = "x86_64"))]
    unsafe fn accumulate_forces_vectorized(
        &self,
        _total_forces: &mut [f64],
        _forces: &[f64],
    ) {
        panic!("AVX-512 backend is not available on non-x86_64 platforms. Use ScalarBackend instead or check is_supported() before use.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_avx512_detection() {
        let backend = Avx512Backend;
        // Just check that the detection doesn't crash
        let _supported = backend.is_supported();
    }
    
    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_avx512_update_velocity() {
        let backend = Avx512Backend;
        if !backend.is_supported() {
            eprintln!("Skipping AVX-512 test - not supported on this CPU");
            return;
        }
        let mut velocities = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let accelerations = vec![0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0];
        let dt = 0.1;
        
        unsafe {
            backend.update_velocity_vectorized(&mut velocities, &accelerations, dt);
        }
        
        // v' = v + a * dt
        assert!((velocities[0] - 1.05).abs() < 1e-10);
        assert!((velocities[1] - 2.1).abs() < 1e-10);
        assert!((velocities[2] - 3.15).abs() < 1e-10);
        assert!((velocities[3] - 4.2).abs() < 1e-10);
        assert!((velocities[4] - 5.25).abs() < 1e-10);
        assert!((velocities[5] - 6.3).abs() < 1e-10);
        assert!((velocities[6] - 7.35).abs() < 1e-10);
        assert!((velocities[7] - 8.4).abs() < 1e-10);
    }
    
    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_avx512_update_position() {
        let backend = Avx512Backend;
        if !backend.is_supported() {
            eprintln!("Skipping AVX-512 test - not supported on this CPU");
            return;
        }
        let mut positions = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let velocities = vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0];
        let accelerations = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
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
        assert!((positions[4] - (4.0 + 50.0 * 0.1 + 5.0 * dt_sq_half)).abs() < 1e-10);
        assert!((positions[5] - (5.0 + 60.0 * 0.1 + 6.0 * dt_sq_half)).abs() < 1e-10);
        assert!((positions[6] - (6.0 + 70.0 * 0.1 + 7.0 * dt_sq_half)).abs() < 1e-10);
        assert!((positions[7] - (7.0 + 80.0 * 0.1 + 8.0 * dt_sq_half)).abs() < 1e-10);
    }
    
    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_avx512_accumulate_forces() {
        let backend = Avx512Backend;
        if !backend.is_supported() {
            eprintln!("Skipping AVX-512 test - not supported on this CPU");
            return;
        }
        let mut total_forces = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let forces = vec![0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0];
        
        unsafe {
            backend.accumulate_forces_vectorized(&mut total_forces, &forces);
        }
        
        assert_eq!(total_forces[0], 1.5);
        assert_eq!(total_forces[1], 3.0);
        assert_eq!(total_forces[2], 4.5);
        assert_eq!(total_forces[3], 6.0);
        assert_eq!(total_forces[4], 7.5);
        assert_eq!(total_forces[5], 9.0);
        assert_eq!(total_forces[6], 10.5);
        assert_eq!(total_forces[7], 12.0);
    }
    
    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_avx512_correctness_vs_scalar() {
        let backend_avx512 = Avx512Backend;
        let backend_scalar = crate::simd::ScalarBackend;
        
        if !backend_avx512.is_supported() {
            eprintln!("Skipping AVX-512 correctness test - not supported on this CPU");
            return;
        }
        
        // Test data with multiple of 8 elements
        let mut velocities_avx512 = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0];
        let mut velocities_scalar = velocities_avx512.clone();
        let accelerations = vec![0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0, 5.5, 6.0, 6.5, 7.0, 7.5, 8.0];
        let dt = 0.1;
        
        unsafe {
            backend_avx512.update_velocity_vectorized(&mut velocities_avx512, &accelerations, dt);
            backend_scalar.update_velocity_vectorized(&mut velocities_scalar, &accelerations, dt);
        }
        
        // Check that AVX-512 and scalar produce identical results
        for i in 0..velocities_avx512.len() {
            assert!((velocities_avx512[i] - velocities_scalar[i]).abs() < 1e-14,
                    "Mismatch at index {}: AVX-512={}, Scalar={}", i, velocities_avx512[i], velocities_scalar[i]);
        }
    }
}
