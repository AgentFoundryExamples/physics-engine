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

//! SIMD detection example
//!
//! Demonstrates how to detect the active SIMD backend at runtime.
//! This is useful for debugging performance and verifying that SIMD
//! acceleration is active on your CPU.
//!
//! # Usage
//!
//! ```bash
//! # Build without SIMD (will always use scalar backend)
//! cargo run --example simd_detection
//!
//! # Build with SIMD support (will use AVX2 if available)
//! cargo run --features simd --example simd_detection
//! ```

#[cfg(feature = "simd")]
use physics_engine::simd::{detect_cpu_features, select_backend};

fn main() {
    println!("=== SIMD Detection Example ===\n");

    #[cfg(feature = "simd")]
    {
        // Detect available CPU features
        let features = detect_cpu_features();
        println!("CPU Features Detected:");
        println!("  AVX2:       {}", features.has_avx2);
        println!("  AVX-512F:   {}", features.has_avx512f);
        println!("  AVX-512DQ:  {}", features.has_avx512dq);
        println!();

        // Get the active SIMD backend
        let backend = select_backend();
        println!("Active SIMD Backend: {}", backend.name());
        println!();

        // Explain what this means
        match backend.name() {
            "AVX-512" => {
                println!("✅ Using AVX-512 vectorization");
                println!("   - Processing 8 × f64 values per instruction");
                println!("   - Expected speedup: 4-6× for large arrays");
                println!("   - Requires Intel Skylake-X (2017+) or AMD Zen 4 (2022+)");
            }
            "AVX2" => {
                println!("✅ Using AVX2 vectorization");
                println!("   - Processing 4 × f64 values per instruction");
                println!("   - Expected speedup: 2-4× for large arrays");
                println!("   - Supported on Intel Haswell (2013+) and AMD Excavator (2015+)");
            }
            "Scalar" => {
                println!("⚠️  Using scalar (non-SIMD) backend");
                println!("   - No vectorization (processing one value at a time)");
                println!("   - This is normal on older CPUs (pre-2013)");
                println!("   - Or if CPU features couldn't be detected");
            }
            _ => {
                println!("Unknown backend: {}", backend.name());
            }
        }
    }

    #[cfg(not(feature = "simd"))]
    {
        println!("⚠️  SIMD feature not enabled at compile time");
        println!();
        println!("The physics engine was built without SIMD support.");
        println!("To enable SIMD acceleration:");
        println!("  cargo run --features simd --example simd_detection");
        println!();
        println!("Without the 'simd' feature, all computations use scalar code,");
        println!("regardless of CPU capabilities.");
    }

    println!();
    println!("=== Performance Expectations ===");
    println!();
    println!("For best performance:");
    println!("  1. Build with --release flag");
    println!("  2. Enable --features simd");
    println!("  3. Use entity counts > 100 (SIMD overhead dominates for small arrays)");
    println!("  4. Use SoAStorage or true SoA storage for cache-friendly data layout");
    println!();
    println!("Typical speedups with SIMD enabled (vs scalar):");
    println!("  - Velocity updates: 2-4×");
    println!("  - Position updates: 2-4×");
    println!("  - Force accumulation: 2-4×");
    println!();
    println!("Note: Actual speedup depends on:");
    println!("  - Entity count (larger = better amortization of SIMD overhead)");
    println!("  - Data layout (contiguous arrays = better cache utilization)");
    println!("  - CPU microarchitecture (newer CPUs have better SIMD units)");
    println!("  - Memory bandwidth (SIMD can be memory-bound for large datasets)");
}
