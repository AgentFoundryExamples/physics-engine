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
//! SIMD vectorization support for physics computations
//!
//! This module provides vectorized implementations of integration and force
//! accumulation using AVX2/AVX-512 instructions with automatic runtime dispatch
//! and scalar fallback for unsupported CPUs.
//!
//! # Architecture
//!
//! - **Runtime Detection**: Automatically detects available CPU features at startup
//! - **Dispatch**: Selects best available implementation (AVX-512 > AVX2 > scalar)
//! - **Deterministic**: SIMD and scalar paths produce identical results (within FP tolerance)
//! - **Stable Rust**: Uses `std::arch` intrinsics, no nightly features required
//!
//! # Performance
//!
//! - **AVX-512**: Process 8 × f64 values per instruction (512-bit vectors)
//! - **AVX2**: Process 4 × f64 values per instruction (256-bit vectors)
//! - **Expected Speedup**: 2-6× for integration loops with sufficient entities
//!
//! # Safety
//!
//! All SIMD code uses Rust's `target_feature` and runtime checks to ensure
//! instructions are only executed on CPUs that support them. Tail handling
//! ensures correctness for entity counts not divisible by SIMD width.

mod dispatch;
mod scalar;

#[cfg(target_arch = "x86_64")]
mod avx2;

#[cfg(target_arch = "x86_64")]
mod avx512;

pub use dispatch::{CpuFeatures, detect_cpu_features};
pub use scalar::ScalarBackend;

#[cfg(target_arch = "x86_64")]
pub use avx2::Avx2Backend;

#[cfg(target_arch = "x86_64")]
pub use avx512::Avx512Backend;

/// SIMD width for different instruction sets
pub const AVX2_WIDTH: usize = 4;  // 256-bit / 64-bit per f64
pub const AVX512_WIDTH: usize = 8; // 512-bit / 64-bit per f64

/// Backend for vectorized physics computations
///
/// Implementations provide vectorized versions of integration and force
/// accumulation that process multiple entities per instruction.
pub trait SimdBackend: Send + Sync {
    /// Get the name of this SIMD backend
    fn name(&self) -> &str;
    
    /// Get the vector width (number of f64 values per operation)
    fn width(&self) -> usize;
    
    /// Check if this backend is supported on the current CPU
    fn is_supported(&self) -> bool;
    
    /// Vectorized velocity update: v' = v + a * dt
    ///
    /// Processes `width()` entities at a time.
    ///
    /// # Safety
    ///
    /// - `velocities` and `accelerations` must have the same length
    /// - Length should be divisible by `width()` for optimal performance
    /// - Caller must ensure CPU supports required instructions
    /// - Implementation handles any length safely, processing full chunks only
    unsafe fn update_velocity_vectorized(
        &self,
        velocities: &mut [f64],
        accelerations: &[f64],
        dt: f64,
    );
    
    /// Vectorized position update: p' = p + v * dt + 0.5 * a * dt²
    ///
    /// Processes `width()` entities at a time.
    ///
    /// # Safety
    ///
    /// - All slices must have the same length
    /// - Length should be divisible by `width()` for optimal performance
    /// - Caller must ensure CPU supports required instructions
    /// - Implementation handles any length safely, processing full chunks only
    unsafe fn update_position_vectorized(
        &self,
        positions: &mut [f64],
        velocities: &[f64],
        accelerations: &[f64],
        dt: f64,
        dt_sq_half: f64,
    );
    
    /// Vectorized force accumulation: f_total += f
    ///
    /// Processes `width()` force components at a time.
    ///
    /// # Safety
    ///
    /// - `total_forces` and `forces` must have the same length
    /// - Length should be divisible by `width()` for optimal performance
    /// - Caller must ensure CPU supports required instructions
    /// - Implementation handles any length safely, processing full chunks only
    unsafe fn accumulate_forces_vectorized(
        &self,
        total_forces: &mut [f64],
        forces: &[f64],
    );
}

/// Select the best available SIMD backend for the current CPU
///
/// Selects backends in priority order:
/// - **AVX-512**: If available (Intel Skylake-X 2017+, AMD Zen 4 2022+)
/// - **AVX2**: If available (Intel Haswell 2013+, AMD Excavator 2015+)
/// - **Scalar**: Always available as fallback
///
/// Selection is cached globally for thread-safe access.
pub fn select_backend() -> Box<dyn SimdBackend> {
    let features = detect_cpu_features();
    
    #[cfg(target_arch = "x86_64")]
    {
        // Prefer AVX-512 if available
        if features.has_avx512f && features.has_avx512dq {
            return Box::new(Avx512Backend);
        }
        
        // Fall back to AVX2
        if features.has_avx2 {
            return Box::new(Avx2Backend);
        }
    }
    
    // Fallback to scalar
    Box::new(ScalarBackend)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_backend_selection() {
        let backend = select_backend();
        // Should always return a valid backend
        assert!(backend.width() >= 1);
    }
    
    #[test]
    fn test_backend_selection_priority() {
        let backend = select_backend();
        let features = detect_cpu_features();
        
        #[cfg(target_arch = "x86_64")]
        {
            // Verify selection priority
            if features.has_avx512f && features.has_avx512dq {
                assert_eq!(backend.name(), "AVX-512", "Should select AVX-512 when available");
                assert_eq!(backend.width(), 8);
            } else if features.has_avx2 {
                assert_eq!(backend.name(), "AVX2", "Should select AVX2 when AVX-512 not available");
                assert_eq!(backend.width(), 4);
            } else {
                assert_eq!(backend.name(), "Scalar", "Should fall back to scalar");
                assert_eq!(backend.width(), 1);
            }
        }
        
        #[cfg(not(target_arch = "x86_64"))]
        {
            assert_eq!(backend.name(), "Scalar", "Non-x86_64 should use scalar");
            assert_eq!(backend.width(), 1);
        }
    }
    
    #[test]
    fn test_scalar_backend_always_supported() {
        let backend = ScalarBackend;
        assert!(backend.is_supported());
    }
    
    #[test]
    fn test_cpu_feature_detection() {
        let features = detect_cpu_features();
        // Basic sanity check
        #[cfg(target_arch = "x86_64")]
        {
            // x86_64 architecture requires SSE2
            assert!(features.has_sse2);
        }
    }
    
    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_backend_correctness_across_implementations() {
        // Test that all backends produce the same results
        let mut velocities_scalar = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let mut velocities_avx2 = velocities_scalar.clone();
        let mut velocities_avx512 = velocities_scalar.clone();
        let accelerations = vec![0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0];
        let dt = 0.1;
        
        let scalar = ScalarBackend;
        let avx2 = Avx2Backend;
        let avx512 = Avx512Backend;
        
        unsafe {
            scalar.update_velocity_vectorized(&mut velocities_scalar, &accelerations, dt);
            
            if avx2.is_supported() {
                avx2.update_velocity_vectorized(&mut velocities_avx2, &accelerations, dt);
                
                // Check AVX2 matches scalar
                for i in 0..velocities_scalar.len() {
                    assert!((velocities_avx2[i] - velocities_scalar[i]).abs() < 1e-14,
                            "AVX2 mismatch at {}: AVX2={}, Scalar={}", i, velocities_avx2[i], velocities_scalar[i]);
                }
            }
            
            if avx512.is_supported() {
                avx512.update_velocity_vectorized(&mut velocities_avx512, &accelerations, dt);
                
                // Check AVX-512 matches scalar
                for i in 0..velocities_scalar.len() {
                    assert!((velocities_avx512[i] - velocities_scalar[i]).abs() < 1e-14,
                            "AVX-512 mismatch at {}: AVX512={}, Scalar={}", i, velocities_avx512[i], velocities_scalar[i]);
                }
            }
        }
    }
}
