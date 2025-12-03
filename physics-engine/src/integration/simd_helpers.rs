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
//! SIMD-optimized integration helpers
//!
//! This module provides SIMD-accelerated versions of integration operations
//! that work with contiguous component data.

#[cfg(feature = "simd")]
use crate::simd::select_backend;

/// Process velocity updates with SIMD acceleration
///
/// Updates velocities: v' = v + a * dt
///
/// Uses SIMD when available and entity count is sufficient. Falls back to
/// scalar processing for remainder elements or when SIMD is not available.
#[cfg_attr(not(feature = "simd"), allow(unused_variables))]
pub fn simd_update_velocities(
    vx: &mut [f64],
    vy: &mut [f64],
    vz: &mut [f64],
    ax: &[f64],
    ay: &[f64],
    az: &[f64],
    dt: f64,
) {
    #[cfg(feature = "simd")]
    {
        let backend = select_backend();
        let width = backend.width();
        let count = vx.len();
        
        // Process full SIMD chunks
        let simd_count = (count / width) * width;
        
        if simd_count > 0 {
            unsafe {
                backend.update_velocity_vectorized(&mut vx[..simd_count], &ax[..simd_count], dt);
                backend.update_velocity_vectorized(&mut vy[..simd_count], &ay[..simd_count], dt);
                backend.update_velocity_vectorized(&mut vz[..simd_count], &az[..simd_count], dt);
            }
        }
        
        // Process remainder with scalar code
        for i in simd_count..count {
            vx[i] += ax[i] * dt;
            vy[i] += ay[i] * dt;
            vz[i] += az[i] * dt;
        }
    }
    
    #[cfg(not(feature = "simd"))]
    {
        // Scalar fallback when SIMD feature is not enabled
        for i in 0..vx.len() {
            vx[i] += ax[i] * dt;
            vy[i] += ay[i] * dt;
            vz[i] += az[i] * dt;
        }
    }
}

/// Process position updates with SIMD acceleration
///
/// Updates positions: p' = p + v * dt + 0.5 * a * dt²
///
/// Uses SIMD when available and entity count is sufficient. Falls back to
/// scalar processing for remainder elements or when SIMD is not available.
#[cfg_attr(not(feature = "simd"), allow(unused_variables))]
pub fn simd_update_positions(
    px: &mut [f64],
    py: &mut [f64],
    pz: &mut [f64],
    vx: &[f64],
    vy: &[f64],
    vz: &[f64],
    ax: &[f64],
    ay: &[f64],
    az: &[f64],
    dt: f64,
) {
    let dt_sq_half = 0.5 * dt * dt;
    
    #[cfg(feature = "simd")]
    {
        let backend = select_backend();
        let width = backend.width();
        let count = px.len();
        
        // Process full SIMD chunks
        let simd_count = (count / width) * width;
        
        if simd_count > 0 {
            unsafe {
                backend.update_position_vectorized(
                    &mut px[..simd_count],
                    &vx[..simd_count],
                    &ax[..simd_count],
                    dt,
                    dt_sq_half,
                );
                backend.update_position_vectorized(
                    &mut py[..simd_count],
                    &vy[..simd_count],
                    &ay[..simd_count],
                    dt,
                    dt_sq_half,
                );
                backend.update_position_vectorized(
                    &mut pz[..simd_count],
                    &vz[..simd_count],
                    &az[..simd_count],
                    dt,
                    dt_sq_half,
                );
            }
        }
        
        // Process remainder with scalar code
        for i in simd_count..count {
            px[i] += vx[i] * dt + ax[i] * dt_sq_half;
            py[i] += vy[i] * dt + ay[i] * dt_sq_half;
            pz[i] += vz[i] * dt + az[i] * dt_sq_half;
        }
    }
    
    #[cfg(not(feature = "simd"))]
    {
        // Scalar fallback when SIMD feature is not enabled
        for i in 0..px.len() {
            px[i] += vx[i] * dt + ax[i] * dt_sq_half;
            py[i] += vy[i] * dt + ay[i] * dt_sq_half;
            pz[i] += vz[i] * dt + az[i] * dt_sq_half;
        }
    }
}

/// Accumulate forces with SIMD acceleration
///
/// Adds forces: f_total += f
///
/// Uses SIMD when available and entity count is sufficient. Falls back to
/// scalar processing for remainder elements or when SIMD is not available.
#[cfg_attr(not(feature = "simd"), allow(unused_variables))]
pub fn simd_accumulate_forces(
    total_fx: &mut [f64],
    total_fy: &mut [f64],
    total_fz: &mut [f64],
    fx: &[f64],
    fy: &[f64],
    fz: &[f64],
) {
    #[cfg(feature = "simd")]
    {
        let backend = select_backend();
        let width = backend.width();
        let count = total_fx.len();
        
        // Process full SIMD chunks
        let simd_count = (count / width) * width;
        
        if simd_count > 0 {
            unsafe {
                backend.accumulate_forces_vectorized(&mut total_fx[..simd_count], &fx[..simd_count]);
                backend.accumulate_forces_vectorized(&mut total_fy[..simd_count], &fy[..simd_count]);
                backend.accumulate_forces_vectorized(&mut total_fz[..simd_count], &fz[..simd_count]);
            }
        }
        
        // Process remainder with scalar code
        for i in simd_count..count {
            total_fx[i] += fx[i];
            total_fy[i] += fy[i];
            total_fz[i] += fz[i];
        }
    }
    
    #[cfg(not(feature = "simd"))]
    {
        // Scalar fallback when SIMD feature is not enabled
        for i in 0..total_fx.len() {
            total_fx[i] += fx[i];
            total_fy[i] += fy[i];
            total_fz[i] += fz[i];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simd_update_velocities() {
        let mut vx = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mut vy = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let mut vz = vec![0.0, 0.0, 0.0, 0.0, 0.0];
        
        let ax = vec![0.5, 1.0, 1.5, 2.0, 2.5];
        let ay = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let az = vec![0.0, 0.0, 0.0, 0.0, 0.0];
        
        let dt = 0.1;
        
        simd_update_velocities(&mut vx, &mut vy, &mut vz, &ax, &ay, &az, dt);
        
        // Check that velocities were updated correctly
        assert!((vx[0] - 1.05).abs() < 1e-10);
        assert!((vx[1] - 2.1).abs() < 1e-10);
        assert!((vy[0] - 0.1).abs() < 1e-10);
        assert!((vy[1] - 1.2).abs() < 1e-10);
    }
    
    #[test]
    fn test_simd_update_positions() {
        let mut px = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let mut py = vec![0.0, 0.0, 0.0, 0.0, 0.0];
        let mut pz = vec![0.0, 0.0, 0.0, 0.0, 0.0];
        
        let vx = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let vy = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let vz = vec![0.0, 0.0, 0.0, 0.0, 0.0];
        
        let ax = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let ay = vec![0.5, 1.0, 1.5, 2.0, 2.5];
        let az = vec![0.0, 0.0, 0.0, 0.0, 0.0];
        
        let dt = 0.1;
        
        simd_update_positions(&mut px, &mut py, &mut pz, &vx, &vy, &vz, &ax, &ay, &az, dt);
        
        // Check that positions were updated correctly
        let dt_sq_half = 0.5 * dt * dt;
        assert!((px[0] - (0.0 + 10.0 * 0.1 + 1.0 * dt_sq_half)).abs() < 1e-10);
        assert!((px[1] - (1.0 + 20.0 * 0.1 + 2.0 * dt_sq_half)).abs() < 1e-10);
    }
    
    #[test]
    fn test_simd_accumulate_forces() {
        let mut total_fx = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mut total_fy = vec![0.5, 1.0, 1.5, 2.0, 2.5];
        let mut total_fz = vec![0.0, 0.0, 0.0, 0.0, 0.0];
        
        let fx = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let fy = vec![0.05, 0.1, 0.15, 0.2, 0.25];
        let fz = vec![0.0, 0.0, 0.0, 0.0, 0.0];
        
        simd_accumulate_forces(&mut total_fx, &mut total_fy, &mut total_fz, &fx, &fy, &fz);
        
        // Check that forces were accumulated correctly
        assert!((total_fx[0] - 1.1).abs() < 1e-10);
        assert!((total_fx[1] - 2.2).abs() < 1e-10);
        assert!((total_fy[0] - 0.55).abs() < 1e-10);
        assert!((total_fy[1] - 1.1).abs() < 1e-10);
    }
    
    #[test]
    fn test_non_aligned_counts() {
        // Test with counts not divisible by SIMD width
        let mut vx = vec![1.0, 2.0, 3.0];
        let mut vy = vec![0.0, 1.0, 2.0];
        let mut vz = vec![0.0, 0.0, 0.0];
        
        let ax = vec![0.5, 1.0, 1.5];
        let ay = vec![1.0, 2.0, 3.0];
        let az = vec![0.0, 0.0, 0.0];
        
        let dt = 0.1;
        
        simd_update_velocities(&mut vx, &mut vy, &mut vz, &ax, &ay, &az, dt);
        
        // All elements should be processed correctly
        assert!((vx[0] - 1.05).abs() < 1e-10);
        assert!((vx[1] - 2.1).abs() < 1e-10);
        assert!((vx[2] - 3.15).abs() < 1e-10);
    }
}
