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
//! CPU feature detection and runtime dispatch
//!
//! This module provides runtime detection of CPU SIMD capabilities to
//! automatically select the best available implementation.

use std::sync::OnceLock;

/// CPU feature flags detected at runtime
#[derive(Debug, Clone, Copy)]
pub struct CpuFeatures {
    /// CPU supports SSE (Streaming SIMD Extensions)
    pub has_sse: bool,
    /// CPU supports SSE2
    pub has_sse2: bool,
    /// CPU supports SSE3
    pub has_sse3: bool,
    /// CPU supports SSSE3 (Supplemental SSE3)
    pub has_ssse3: bool,
    /// CPU supports SSE4.1
    pub has_sse4_1: bool,
    /// CPU supports SSE4.2
    pub has_sse4_2: bool,
    /// CPU supports AVX (Advanced Vector Extensions)
    pub has_avx: bool,
    /// CPU supports AVX2
    pub has_avx2: bool,
    /// CPU supports FMA (Fused Multiply-Add)
    pub has_fma: bool,
    /// CPU supports AVX-512 Foundation
    pub has_avx512f: bool,
    /// CPU supports AVX-512 Double/Quad Word instructions
    pub has_avx512dq: bool,
}

impl Default for CpuFeatures {
    fn default() -> Self {
        CpuFeatures {
            has_sse: false,
            has_sse2: false,
            has_sse3: false,
            has_ssse3: false,
            has_sse4_1: false,
            has_sse4_2: false,
            has_avx: false,
            has_avx2: false,
            has_fma: false,
            has_avx512f: false,
            has_avx512dq: false,
        }
    }
}

impl CpuFeatures {
    /// Create a new CpuFeatures with all features disabled
    pub fn none() -> Self {
        Self::default()
    }
    
    /// Create a CpuFeatures for testing with specific features enabled
    #[cfg(test)]
    pub fn with_avx2() -> Self {
        CpuFeatures {
            has_sse: true,
            has_sse2: true,
            has_sse3: true,
            has_ssse3: true,
            has_sse4_1: true,
            has_sse4_2: true,
            has_avx: true,
            has_avx2: true,
            has_fma: true,
            has_avx512f: false,
            has_avx512dq: false,
        }
    }
}

/// Global cache of detected CPU features
static CPU_FEATURES: OnceLock<CpuFeatures> = OnceLock::new();

/// Detect CPU features at runtime
///
/// Uses CPUID instruction to query CPU capabilities. Results are cached
/// globally to avoid repeated detection overhead.
///
/// # Platform Support
///
/// - **x86_64**: Full feature detection via CPUID
/// - **Other**: Returns default features (scalar only)
pub fn detect_cpu_features() -> CpuFeatures {
    *CPU_FEATURES.get_or_init(|| {
        detect_cpu_features_impl()
    })
}

#[cfg(target_arch = "x86_64")]
fn detect_cpu_features_impl() -> CpuFeatures {
    use raw_cpuid::CpuId;
    
    let cpuid = CpuId::new();
    let mut features = CpuFeatures::default();
    
    // Check feature info
    if let Some(feature_info) = cpuid.get_feature_info() {
        features.has_sse = feature_info.has_sse();
        features.has_sse2 = feature_info.has_sse2();
        features.has_sse3 = feature_info.has_sse3();
        features.has_ssse3 = feature_info.has_ssse3();
        features.has_sse4_1 = feature_info.has_sse41();
        features.has_sse4_2 = feature_info.has_sse42();
        features.has_avx = feature_info.has_avx();
        features.has_fma = feature_info.has_fma();
    }
    
    // Check extended features
    if let Some(extended_features) = cpuid.get_extended_feature_info() {
        features.has_avx2 = extended_features.has_avx2();
        features.has_avx512f = extended_features.has_avx512f();
        features.has_avx512dq = extended_features.has_avx512dq();
    }
    
    features
}

#[cfg(not(target_arch = "x86_64"))]
fn detect_cpu_features_impl() -> CpuFeatures {
    // Non-x86_64 platforms: return default (no SIMD)
    CpuFeatures::default()
}

/// Check if the current CPU supports AVX2
pub fn has_avx2() -> bool {
    detect_cpu_features().has_avx2
}

/// Check if the current CPU supports AVX-512
pub fn has_avx512() -> bool {
    let features = detect_cpu_features();
    features.has_avx512f && features.has_avx512dq
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feature_detection() {
        let features = detect_cpu_features();
        // On x86_64, we should detect at least SSE2 (required by x86_64)
        #[cfg(target_arch = "x86_64")]
        {
            assert!(features.has_sse2, "x86_64 requires SSE2");
        }
    }
    
    #[test]
    fn test_feature_caching() {
        // Multiple calls should return the same cached result
        let f1 = detect_cpu_features();
        let f2 = detect_cpu_features();
        
        assert_eq!(f1.has_avx2, f2.has_avx2);
        assert_eq!(f1.has_avx512f, f2.has_avx512f);
    }
    
    #[test]
    fn test_helper_functions() {
        let features = detect_cpu_features();
        
        assert_eq!(has_avx2(), features.has_avx2);
        assert_eq!(has_avx512(), features.has_avx512f && features.has_avx512dq);
    }
    
    #[test]
    fn test_default_features() {
        let features = CpuFeatures::default();
        assert!(!features.has_avx2);
        assert!(!features.has_avx512f);
    }
}
