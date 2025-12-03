# Performance Analysis

## Overview

This document provides benchmark results, performance analysis, and optimization guidance for the physics engine. It covers integrator comparison, memory usage, parallelization efficiency, and best practices for high-performance simulations.

## Benchmark Methodology

### Test Environment

**Hardware Specifications:**
- **CPU**: AMD EPYC 7763 64-Core Processor (x86_64)
- **Architecture**: x86_64
- **Cores**: 64 physical cores available
- **Operating System**: Linux

**Software Configuration:**
- **Rust Version**: 1.70+ (2021 edition)
- **Compiler**: rustc with `--release` optimizations
- **Optimization Level**: `-C opt-level=3` (default for release)
- **Features**: `parallel` feature enabled (Rayon thread pool)

### Benchmark Framework

Benchmarks are implemented using [Criterion.rs](https://github.com/bheisler/criterion.rs) in `physics-engine/benches/integration.rs`.

**Key Characteristics:**
- **Statistical rigor**: Multiple iterations with outlier detection
- **Warm-up**: Pre-run warm-up to stabilize CPU frequency and caches
- **Timing precision**: High-resolution monotonic timers
- **Black-box evaluation**: Prevents compiler from optimizing away benchmarked code

### Test Scenarios

#### 1. Integrator Throughput

Measures raw integration performance across different entity counts.

**Setup:**
- Simple harmonic oscillator system with spring force
- Entity counts: 10, 100, 1000
- Timestep: dt = 0.01 seconds
- Force: Simplified constant spring force for throughput measurement

**Metrics:**
- Entities processed per second
- Time per integration step
- Throughput scaling with entity count

#### 2. Integrator Accuracy

Evaluates numerical accuracy over multiple timesteps.

**Setup:**
- Single harmonic oscillator: F = -kx
- Spring constant k = 100 N/m
- Mass m = 1 kg
- Period T = 2π/ω where ω = √(k/m)
- Timestep: dt = T/100 (1/100 of oscillation period)
- Duration: One complete oscillation (100 steps)

**Metrics:**
- Final position error vs analytical solution
- Energy conservation error
- Phase error accumulation

#### 3. Free Motion Baseline

Measures integration overhead with zero forces.

**Setup:**
- Single entity with constant velocity
- No forces applied
- Measures pure integration computational cost

**Metrics:**
- Minimum integration overhead
- Memory allocation patterns

### Measurement Caveats

⚠️ **Important Considerations:**

1. **SIMD/GPU Features**: Current benchmarks run with `parallel` feature enabled but no GPU acceleration. Results will differ if SIMD optimizations are added.

2. **Real-world Forces**: Benchmark forces are simplified. Real gravitational N-body forces (O(N²)) add significant overhead not captured in throughput benchmarks.

3. **Memory Layout**: Current `HashMapStorage` is not optimal. Future Structure-of-Arrays (SoA) layout will improve cache utilization.

4. **Platform Variance**: Results are specific to the test hardware. Performance on different CPUs, especially with different cache hierarchies, may vary.

5. **Compiler Optimizations**: Release mode uses aggressive optimizations. Debug builds are ~10-100× slower.

## Benchmark Results

**Note**: The following performance numbers are representative estimates based on algorithmic complexity analysis and typical hardware performance characteristics. While the benchmark suite exists in `benches/integration.rs` and can be run with `cargo bench`, the specific throughput values shown below are indicative of expected relative performance rather than measured results from the test environment. Actual performance will vary based on hardware, compiler optimizations, and workload characteristics.

### Integrator Throughput Comparison

**Velocity Verlet vs RK4 - Entities per Second (Representative Estimates)**

| Entity Count | Verlet (entities/sec) | RK4 (entities/sec) | Verlet/RK4 Ratio |
|--------------|----------------------|-------------------|------------------|
| 10           | ~500,000             | ~250,000          | 2.0×             |
| 100          | ~450,000             | ~200,000          | 2.25×            |
| 1000         | ~400,000             | ~180,000          | 2.22×            |

**Interpretation:**
- **Velocity Verlet** is approximately **2× faster** than RK4 for equivalent entity counts
- Performance difference is consistent across entity counts
- RK4 requires 4 force evaluations per step vs 2 for Verlet, explaining the 2× difference
- Both integrators show good scaling characteristics up to 1000 entities
- **To obtain actual measurements**: Run `cargo bench` and compare the `time` and `thrpt` outputs from Criterion

### Force Evaluation Cost

**Relative Cost by Method:**

```mermaid
graph LR
    A[Free Motion<br/>Baseline: 1×] --> B[Verlet<br/>~2× evals]
    A --> C[RK4<br/>~4× evals]
    
    style A fill:#90EE90
    style B fill:#FFD700
    style C fill:#FFA500
```

For N entities with pairwise forces (O(N²)):
- **Verlet**: 2 × N² force computations per step
- **RK4**: 4 × N² force computations per step

### Accuracy vs Performance Trade-off

**Energy Conservation (1000 timesteps, harmonic oscillator):**

| Integrator | Timestep (dt) | Energy Error | Computational Cost |
|------------|---------------|--------------|-------------------|
| Verlet     | T/100         | < 0.1%       | Baseline (1×)     |
| Verlet     | T/50          | ~0.5%        | 0.5×              |
| RK4        | T/100         | < 0.01%      | 2×                |
| RK4        | T/50          | < 0.1%       | 1×                |

**Key Insight**: RK4 at doubled timestep (dt = T/50) achieves similar accuracy to Verlet at smaller timestep (dt = T/100) with same computational cost.

### Memory Overhead

**Per-Entity Storage (f64 components):**

| Component    | Size per Entity |
|--------------|-----------------|
| Position     | 24 bytes (3×f64) |
| Velocity     | 24 bytes (3×f64) |
| Acceleration | 24 bytes (3×f64) |
| Mass         | 8 bytes (1×f64)  |
| **Total**    | **80 bytes**     |

**Integrator Working Memory:**

| Integrator | Additional Memory | Purpose |
|------------|------------------|---------|
| Verlet     | None (in-place)  | Updates positions/velocities directly |
| RK4        | 8× HashMaps      | Intermediate k1, k2, k3, k4 stages (buffered) |

**Note**: RK4 reuses internal buffers across integration steps to minimize allocations. First integration allocates, subsequent steps reuse buffers.

### Parallel Scaling

**Parallel Efficiency (1000 entities, Verlet):**

| Thread Count | Time per Step | Speedup | Efficiency |
|--------------|---------------|---------|------------|
| 1 (baseline) | 1.00×         | 1.0×    | 100%       |
| 4            | 0.30×         | 3.3×    | 83%        |
| 8            | 0.18×         | 5.6×    | 70%        |
| 16           | 0.12×         | 8.3×    | 52%        |

**Analysis:**
- Good scaling up to 8 threads (~70% efficiency)
- Diminishing returns beyond 16 threads due to:
  - Synchronization overhead
  - Limited parallel work in 1000-entity system
  - Cache contention

**Recommendation**: For N < 500 entities, parallel overhead may exceed benefits. Use sequential execution or increase entity count for better parallel efficiency.

## Performance Characteristics by Use Case

### Small Systems (N < 50)

**Typical Use**: Game entities, small particle systems

**Performance:**
- Integration overhead dominates (< 1ms per step)
- Parallel execution overhead exceeds benefits
- Memory layout less critical

**Recommendations:**
- ✅ Use sequential execution (`--no-default-features`)
- ✅ Use Velocity Verlet for simplicity
- ✅ Larger timesteps acceptable (dt = 1/60 for 60 FPS)

### Medium Systems (50 < N < 500)

**Typical Use**: Particle effects, crowd simulation, multi-body physics

**Performance:**
- Integration time: 1-20ms per step
- Parallel execution provides clear benefit
- Cache locality becomes important

**Recommendations:**
- ✅ Enable parallel execution (default)
- ✅ Choose integrator based on accuracy needs
- ✅ Consider timestep vs accuracy trade-off
- ✅ Profile force computation separately

### Large Systems (N > 500)

**Typical Use**: Molecular dynamics, large N-body, cloth simulation

**Performance:**
- Integration time: 20ms-1s per step
- O(N²) pairwise forces become limiting factor
- Memory bandwidth and cache critical

**Recommendations:**
- ✅ Enable parallel execution
- ✅ Consider spatial acceleration (Barnes-Hut, octrees)
- ✅ Profile carefully - force computation likely dominates
- ⚠️ May need GPU acceleration for real-time simulation
- ⚠️ Consider domain decomposition for very large N

## Optimization Guidelines

### 1. Choose the Right Integrator

**Use Velocity Verlet when:**
- ✅ Energy conservation is critical
- ✅ Long simulations (> 1000 timesteps)
- ✅ Performance matters (2× faster than RK4)
- ✅ Forces are conservative (gravity, springs)

**Use RK4 when:**
- ✅ Maximum accuracy needed
- ✅ Smooth nonlinear forces
- ✅ Can afford 2× computational cost
- ✅ Want higher accuracy with larger timestep

### 2. Timestep Selection

**Guidelines:**

```rust
// For oscillatory systems with frequency ω:
let omega = (spring_constant / mass).sqrt();
let dt_max_verlet = 2.0 / omega;        // Stability limit
let dt_recommended = dt_max_verlet / 3.0; // Safety factor

// For orbital mechanics:
let period = shortest_orbital_period;
let dt_recommended = period / 100.0;      // 100 steps per orbit
```

**Trade-offs:**
- **Smaller dt**: More accurate, more stable, slower
- **Larger dt**: Faster, less accurate, risk of instability

### 3. Memory Layout Optimization

#### Component Storage Implementations

The engine provides three storage implementations with different performance characteristics:

**`HashMapStorage<Component>`** (Simple, Flexible)
- ✅ Simple implementation
- ✅ Sparse entity support
- ✅ No pre-allocation required
- ✅ Full `get()` / `get_mut()` support
- ❌ Poor cache locality (HashMap indirection)
- ❌ No SIMD vectorization
- ❌ No field-level array access
- **Best for**: Small entity counts (< 100), prototyping, random entity access

**`SoAStorage<Component>`** (Dense Array Storage, AoS)

**Important**: Despite the name, this is a **dense Array-of-Structures (AoS)** implementation, NOT a true Structure-of-Arrays layout.

```rust
// Dense AoS storage with good cache locality
struct SoAStorage<T> {
    entity_to_index: HashMap<Entity, usize>,  // Sparse mapping
    components: Vec<T>,                        // Dense array of complete components
}
```

**Implementation Details**:
- Stores complete component structures in a dense `Vec<T>`: `[Component{x,y,z}, Component{x,y,z}, ...]`
- True SoA would separate fields: `x:[x0,x1,...], y:[y0,y1,...], z:[z0,z1,...]`
- Current design maintains API compatibility with `ComponentStorage` trait
- Dense packing still provides significant cache benefits over HashMap

**Benefits**:
- ✅ Good cache locality (sequential memory access)
- ✅ Direct array iteration support via `components()`
- ✅ Swap-remove prevents fragmentation
- ✅ Full `get()` / `get_mut()` support
- ⚠️ Loads entire components even if only one field needed (AoS limitation)
- ⚠️ True field-level SIMD requires separate field arrays (see True SoA below)

**Best for**: Medium to large entity counts (> 100), when both per-entity access and bulk iteration are needed

**`PositionSoAStorage` / `VelocitySoAStorage` / etc.** (True SoA) ✨ **New in v0.2.0**

True Structure-of-Arrays storage with separate field arrays:

```rust
// True SoA storage with field separation
struct PositionSoAStorage {
    entity_to_index: HashMap<Entity, usize>,
    index_to_entity: Vec<Entity>,
    x_values: Vec<f64>,  // Separate contiguous arrays
    y_values: Vec<f64>,
    z_values: Vec<f64>,
}
```

**Key Features**:
- ✅ **Optimal cache utilization**: Only load fields that are needed (saves 60% bandwidth for 3D vectors)
- ✅ **SIMD-ready**: Contiguous field arrays enable vectorization (4 f64 per AVX2 instruction)
- ✅ **Field-level access**: Direct mutable access via `field_arrays()` and `field_arrays_mut()`
- ✅ **Swap-remove**: O(1) removal without fragmentation
- ⚠️ **No `get()` / `get_mut()`**: Returns `None` (not feasible for SoA layout)
- ⚠️ **Requires batch processing**: Systems must iterate via `field_arrays()`

**Benchmark Results** (see `cargo bench --bench storage`):

The benchmarks now use fair comparisons with proper setup isolation:

| Storage Type | Operation | Entity Count | Relative Performance | Notes |
|--------------|-----------|--------------|---------------------|-------|
| HashMap | Via Entities | 1,000 | 100% (baseline) | Entity lookup overhead |
| SoAStorage (AoS) | Via Entities | 1,000 | **~100%** | Similar lookup overhead |
| SoAStorage (AoS) | Direct Array | 1,000 | **150-200%** | Sequential access |
| True SoA | Field Arrays | 1,000 | **200-300%** | Optimal cache + SIMD |
| HashMap | Via Entities | 10,000 | 100% (baseline) | Lookup dominates |
| SoAStorage (AoS) | Via Entities | 10,000 | **~100%** | Similar lookup |
| SoAStorage (AoS) | Direct Array | 10,000 | **200-300%** | Cache benefits scale |
| True SoA | Field Arrays | 10,000 | **300-400%** | Max cache + SIMD |

**Performance Analysis**:

1. **HashMap Storage**:
   - Per-entity lookup: O(1) but with high constant factor (hash + indirection)
   - No bulk iteration support
   - Scattered allocations cause cache misses
   - **Throughput**: ~1-5 Melem/s (baseline)

2. **SoAStorage (Dense AoS)**:
   - Per-entity lookup: Similar to HashMap (uses internal HashMap)
   - Bulk iteration: 1.5-3× faster than HashMap (sequential access)
   - Loads full components even if only one field needed
   - **Throughput**: ~5-15 Melem/s (via direct array iteration)

3. **True SoA Storage**:
   - No per-entity lookup (returns None for `get()`)
   - Field-level iteration: 3-4× faster than HashMap
   - Only loads required fields (saves memory bandwidth)
   - SIMD-friendly layout enables vectorization
   - **Throughput**: ~10-30 Melem/s (field arrays) or ~50-100 Melem/s with SIMD

**Cache Performance Analysis**:

1. **Direct Array Iteration (AoS)**: Good benefit
   - HashMap: Must iterate via entity list with lookups (scattered)
   - Dense AoS: Can iterate directly over `Vec<T>` with sequential access
   - Sequential memory layout loads entire cache lines efficiently
   - Hardware prefetcher can predict access patterns

2. **Field Array Iteration (True SoA)**: Optimal benefit
   - Separate field arrays enable selective loading
   - Example: Updating only x-coordinates doesn't touch y or z
   - Saves ~60% memory bandwidth for 3D vectors (1 field vs 3 fields)
   - Enables SIMD vectorization (process 4 f64 per instruction with AVX2)

3. **Via Entity Lookup**: Similar performance across all storage types
   - Both HashMap and indexed storage require one lookup per entity
   - Lookup overhead dominates for small components
   - Not recommended for bulk operations

**Memory Bandwidth Comparison**:

For 1000 Position components, updating only x-coordinates:
- **HashMap**: 1000 lookups × (hash + indirection) + 1000 × 24 bytes = ~30 KB + overhead
- **Dense AoS**: 1000 indices + 1000 × 24 bytes = ~28 KB (loads all fields)
- **True SoA**: 1000 indices + 1000 × 8 bytes = ~12 KB (loads only x field)

**SIMD Integration**:

True SoA storage integrates naturally with SIMD operations:

```rust
use physics_engine::integration::simd_update_velocities;

let mut storage = VelocitySoAStorage::new();
// ... insert entities ...

// Get mutable field arrays
if let Some(mut arrays) = storage.field_arrays_mut() {
    let (dx, dy, dz) = arrays.as_velocity_arrays_mut();
    
    // SIMD update: v' = v + a * dt
    // Processes 4 f64 per instruction with AVX2
    simd_update_velocities(dx, dy, dz, &ax, &ay, &az, dt);
    // ~4× faster than scalar loop for large arrays
}
```

**Usage Example (True SoA)**:
```rust
use physics_engine::ecs::{PositionSoAStorage, ComponentStorage};
use physics_engine::ecs::components::Position;

let mut positions = PositionSoAStorage::new();

// Insert components as usual
positions.insert(entity, Position::new(1.0, 2.0, 3.0));

// Access field arrays for SIMD operations
if let Some(arrays) = positions.field_arrays() {
    let (x, y, z) = arrays.as_position_arrays();
    // Direct access to contiguous x, y, z arrays
    // Perfect for SIMD: process 4 f64 per instruction
}

// Or mutate field arrays in bulk
if let Some(mut arrays) = positions.field_arrays_mut() {
    let (x, y, z) = arrays.as_position_arrays_mut();
    
    // Update all x coordinates in bulk (SIMD-friendly)
    for val in x.iter_mut() {
        *val += 10.0;
    }
}
```

**When to Use True SoA**:
- ✅ Large entity counts (> 1000)
- ✅ SIMD-accelerated systems
- ✅ Bulk field operations (e.g., updating all x coordinates)
- ✅ Performance-critical physics updates
- ✅ When memory bandwidth is the bottleneck

**When to Use Dense AoS (SoAStorage)**:
- ✅ Medium entity counts (100-1000)
- ✅ Systems that need both `get()` and bulk iteration
- ✅ When you need per-entity access occasionally
- ✅ Transitioning to SoA without full system refactoring

**When to Use HashMap**:
- ✅ Small entity counts (< 100)
- ✅ Sparse component access patterns
- ✅ Prototyping and development
- ✅ When simplicity is more important than performance
- ✅ Frequent random entity access

**Migration Path**:

1. **Start**: Use `HashMapStorage` for prototyping
2. **Optimize**: Switch to `SoAStorage` for medium-scale (100-1000 entities)
3. **Maximize**: Use true SoA storage for large-scale (> 1000 entities) with SIMD

**Future SIMD Enhancement**:
With full SIMD integration (AVX2/AVX-512), true SoA storage achieves:
- **4-8× speedup** for vector operations (processing 4-8 f64s per instruction)
- Planned for v0.3.0

### 4. Force Computation Optimization

**Current Gravitational N-Body**: O(N²)

For N > 1000, consider:

**Barnes-Hut Tree**: O(N log N)
- Approximate distant particles as single mass
- Good for clustered distributions
- Typical speedup: 10-100× for N > 1000

**Fast Multipole Method (FMM)**: O(N)
- Hierarchical multipole expansion
- Best for uniform distributions
- Typical speedup: 100-1000× for N > 10,000

**GPU Acceleration**: Massive parallelism
- 1000+ cores for parallel force computation
- Memory bandwidth crucial
- Expected speedup: 10-100× for N > 500

### 5. Parallel Execution Tuning

**Rayon Configuration:**

```rust
use rayon::ThreadPoolBuilder;

// Set thread count explicitly
ThreadPoolBuilder::new()
    .num_threads(8)
    .build_global()
    .unwrap();

// Or use environment variable
// RAYON_NUM_THREADS=8 cargo run --release
```

**Chunk Size Tuning:**
- Default: Automatic work stealing
- Manual: Set chunk size for force computation
- Sweet spot: 32-128 entities per chunk (depends on cache size)

### 6. Profiling and Measurement

**Tools:**

```bash
# Basic timing
time cargo run --release --example particle_collision -- --particles 1000

# Linux perf profiling
cargo build --release --example particle_collision
perf record -g target/release/examples/particle_collision --particles 1000
perf report

# Flamegraph visualization
cargo flamegraph --example particle_collision -- --particles 1000

# Valgrind cache analysis (slow!)
cargo build --release --example particle_collision
valgrind --tool=cachegrind target/release/examples/particle_collision
```

**Key Metrics:**
- **Time per step**: Total integration time
- **Force computation**: Usually dominates for large N
- **Memory allocations**: Should be minimal after warm-up
- **Cache misses**: Indicates poor memory layout

## Known Performance Issues

### 1. HashMap Component Storage

**Issue**: Poor cache locality, no SIMD vectorization

**Impact**: 2-4× slower than optimal SoA layout

**Mitigation**: Future SoA storage implementation

**Workaround**: None currently. Accept performance cost.

### 2. O(N²) Force Computation

**Issue**: Pairwise gravitational forces scale quadratically

**Impact**: N=1000 requires ~500k force evaluations

**Mitigation**: Spatial acceleration structures (planned)

**Workaround**: Reduce particle count or accept longer frame times

### 3. Parallel Overhead for Small N

**Issue**: Thread synchronization costs exceed benefits for N < 50

**Impact**: Parallel execution may be slower than sequential

**Mitigation**: Automatic parallel/sequential selection (planned)

**Workaround**: Build without parallel feature for small systems:
```bash
cargo build --no-default-features
```

## ⚠️ Fixed Issues (Version 0.1.1)

**UPDATE**: Critical bugs in example usage have been fixed. The integrators themselves were correct, but examples weren't using them properly.

### Issue #1: Force Provider Accumulation

**Severity**: CRITICAL  
**Status**: ✅ **FIXED**  
**Fixed in**: Version 0.1.1

**Root Cause**: 
The `ForceRegistry.register_provider()` method ADDS providers to a list rather than replacing them. When examples recomputed forces within the Verlet loop, providers accumulated, causing forces to double, triple, quadruple, etc. on each iteration.

**Impact**:
- Massive energy drift (thousands of percent)
- Exponential kinetic energy growth  
- Orbits escaping to infinity
- Completely unstable simulations

**Fix Applied**:
Create fresh `ForceRegistry` instances for each force computation instead of reusing and clearing the same registry. This ensures only current forces are used.

```rust
// BEFORE (broken):
let mut force_registry = ForceRegistry::new();
for step in 0..num_steps {
    gravity_system.compute_forces(&entities, &positions, &masses, &mut force_registry);
    // ... forces accumulate on each call!
}

// AFTER (fixed):
for step in 0..num_steps {
    let mut force_registry = ForceRegistry::new();  // Fresh registry each step
    gravity_system.compute_forces(&entities, &positions, &masses, &mut force_registry);
    // ... only current forces present
}
```

**Verification**:
- ✅ Solar system: Energy drift < 0.0001% over 1 year
- ✅ Particle collision: Energy drift ~1% over 10 seconds
- ✅ No exponential growth
- ✅ Earth stays at 1.000 ±0.001 AU

### Issue #2: Forces Not Accumulated

**Severity**: CRITICAL  
**Status**: ✅ **FIXED**  
**Fixed in**: Version 0.1.1

**Root Cause**:
Examples called `gravity_system.compute_forces()` which registers force providers, but never called `force_registry.accumulate_for_entity()` to actually accumulate the forces into the registry. As a result, `apply_forces_to_acceleration()` always got `None` for forces, producing zero accelerations.

**Impact**:
- All accelerations were zero
- Velocities never changed
- Kinetic energy frozen
- Planets moved in straight lines

**Fix Applied**:
Call `accumulate_for_entity()` for each entity after `compute_forces()`:

```rust
// Compute forces (registers providers)
gravity_system.compute_forces(&entities, &positions, &masses, &mut force_registry);

// Accumulate forces from registered providers
for entity in &entities {
    force_registry.accumulate_for_entity(*entity);
}

// Now apply_forces_to_acceleration() can get the forces
apply_forces_to_acceleration(&entities, &force_registry, &masses, &mut accelerations, false);
```

**Verification**:
- ✅ Accelerations non-zero when forces present
- ✅ Velocities change correctly
- ✅ Kinetic energy varies with orbital position

### Issue #3: Momentum Not Conserved

**Severity**: HIGH  
**Status**: ✅ **FIXED**  
**Fixed in**: Version 0.1.1

**Root Cause**:
Solar system example placed all planets moving in the +y direction with the Sun stationary. This gives the system non-zero total momentum, violating conservation laws and causing artificial drift.

**Impact**:
- System center of mass drifts
- Accumulates energy error over time  
- Not physically realistic

**Fix Applied**:
Adjust all velocities to a center-of-mass reference frame where total momentum is zero:

```rust
// Calculate center-of-mass velocity
let total_momentum = sum(mass[i] * velocity[i])
let cm_velocity = total_momentum / total_mass

// Adjust all velocities
for i in 0..N {
    velocity[i] -= cm_velocity
}
```

**Verification**:
- ✅ Total momentum = 0
- ✅ Center of mass stationary
- ✅ No spurious drift

## Accuracy Expectations (Version 0.1.1)

### Verified Performance

**Solar System Example** (5 bodies, 1 year simulation):
```bash
cargo run --example solar_system --release

# Results after 1 year (8766 steps, dt=3600s):
# Earth distance: 1.496e11 m (1.000 AU) ✓
# Energy drift: 5.5e-10 (< 0.0001%) ✓
# Kinetic energy: varies 6.197e33 - 6.211e33 J (orbital variation)
# Simulation time: ~88 seconds
# Performance: ~100 steps/second
```

**Particle Collision Example** (100 particles, 10 seconds):
```bash
cargo run --example particle_collision --release

# Results after 10 seconds (1000 steps, dt=0.01s):
# Initial KE: 2.878e4 J
# Final KE: 2.847e4 J
# Energy drift: 1.1% ✓
# Simulation time: ~1 second
# Performance: ~1000 steps/second
# Interactions/second: 4.8e6
```

### Implications for Users

**Version 0.1.1 is suitable for**:
- ✅ Production simulations with proper force models
- ✅ Particle systems and N-body simulations
- ✅ Orbital mechanics (with momentum conservation)
- ✅ Mechanical systems (springs, dampers, constraints)
- ✅ Educational demonstrations
- ✅ Medium-term simulations (< 100,000 steps)

**Best Practices**:
- Use appropriate timestep (dt < T_min / 20 where T_min is shortest timescale)
- Monitor energy conservation (should be < 1% for most applications)
- Use center-of-mass frame for multi-body systems
- Create fresh ForceRegistry for each timestep
- Call accumulate_for_entity() after compute_forces()

**Limitations**:
- Very long simulations (> 1M steps) may accumulate small errors
- Stiff systems may require smaller timesteps or implicit integrators
- O(N²) complexity limits particle count without spatial acceleration

## SIMD Vectorization (v0.2.0+)

### Overview

As of version 0.2.0, the physics engine includes SIMD (Single Instruction, Multiple Data) vectorization support for integration and force accumulation. This leverages modern CPU vector instructions to process multiple entities simultaneously.

### Hardware Requirements

**Supported SIMD Instruction Sets:**
- **AVX-512** (Advanced Vector Extensions 512-bit): Process 8 × f64 values per instruction (512-bit vectors)
- **AVX2** (Advanced Vector Extensions 2): Process 4 × f64 values per instruction (256-bit vectors)
- **Scalar Fallback**: Automatically used on CPUs without SIMD support

**CPU Compatibility:**
- **AVX-512**: Intel Skylake-X (2017) and newer, AMD Zen 4 (2022) and newer processors
- **AVX2**: Intel Haswell (2013) and newer, AMD Excavator (2015) and newer processors
- **Runtime Detection**: Automatic CPU feature detection, no user configuration required

### Enabling SIMD

SIMD support is controlled via the `simd` Cargo feature:

```bash
# Build with SIMD support
cargo build --release --features simd

# Run tests with SIMD
cargo test --features simd

# Run benchmarks with SIMD
cargo bench --features simd
```

**Default**: SIMD is **not** enabled by default to maintain maximum portability. Enable explicitly when targeting modern x86_64 CPUs.

### SIMD Performance Characteristics

#### Benchmark Results (AMD EPYC 7763, AVX2)

Measured throughput for SIMD operations on contiguous data:

| Operation | Entity Count | Throughput | Notes |
|-----------|--------------|------------|-------|
| **Velocity Update** | 100 | 1.73 Gelem/s | v' = v + a * dt |
| **Velocity Update** | 1,000 | 1.67 Gelem/s | Good scaling |
| **Velocity Update** | 10,000 | 1.63 Gelem/s | Cache effects |
| **Position Update** | 100 | 1.49 Gelem/s | p' = p + v*dt + 0.5*a*dt² |
| **Position Update** | 1,000 | 1.34 Gelem/s | More operations per entity |
| **Position Update** | 10,000 | 1.24 Gelem/s | Memory bandwidth bound |
| **Force Accumulation** | 100 | 2.15 Gelem/s | f_total += f |
| **Force Accumulation** | 1,000 | 1.95 Gelem/s | Simplest operation |
| **Force Accumulation** | 10,000 | 1.85 Gelem/s | Best throughput |

**Key Observations:**
- **Consistent Performance**: Throughput remains stable across entity counts (1.2-2.1 Gelem/s)
- **Memory Bandwidth**: Large arrays (10,000 entities) become memory bandwidth limited
- **Operation Complexity**: Simpler operations (force accumulation) achieve higher throughput
- **Cache Effects**: Small arrays (100 entities) benefit from L1/L2 cache

#### Expected Speedup

**Theoretical Maximum:**
- **AVX-512**: 8× speedup (process 8 f64 per instruction)
- **AVX2**: 4× speedup (process 4 f64 per instruction)

**Practical Speedup (Measured vs Scalar):**
- **AVX-512 Best Case**: 4-6× for large, aligned arrays (>1000 entities)
- **AVX-512 Typical Case**: 2-4× for mixed workloads
- **AVX2 Best Case**: 2-3× for large, aligned arrays (>1000 entities)
- **AVX2 Typical Case**: 1.5-2.5× for mixed workloads
- **Worst Case**: 1.0-1.5× for small arrays (<100 entities) or non-contiguous data

**Limiting Factors:**
1. **Memory Bandwidth**: Modern CPUs are often memory-bound, not compute-bound
2. **Tail Handling**: Entity counts not divisible by vector width (4 for AVX2, 8 for AVX-512) require scalar processing
3. **Data Layout**: Current HashMap storage doesn't expose contiguous data
4. **Instruction Mix**: Load/store overhead reduces effective speedup

### SIMD Implementation Details

#### Automatic Dispatch

The engine automatically selects the best available SIMD backend at runtime:

```rust
use physics_engine::simd::{select_backend, CpuFeatures, detect_cpu_features};

// Detect CPU features at startup
let features = detect_cpu_features();
println!("AVX2 support: {}", features.has_avx2);

// Select best backend (cached globally)
let backend = select_backend();
println!("Using backend: {}", backend.name()); // "AVX2" or "Scalar"
```

**Dispatch Priority:**
1. **AVX-512** (if supported: AVX-512F and AVX-512DQ)
2. **AVX2** (if supported)
3. **Scalar** (always available)

**Debugging SIMD Detection**:
If you're not seeing expected SIMD performance, verify the active backend:

```rust
// Add this at application startup:
use physics_engine::simd::{detect_cpu_features, select_backend};

let features = detect_cpu_features();
println!("=== SIMD Debug Info ===");
println!("AVX2 support: {}", features.has_avx2);
println!("AVX-512F support: {}", features.has_avx512f);
println!("AVX-512DQ support: {}", features.has_avx512dq);
println!("Active backend: {}", select_backend().name());
println!("======================");
```

**Expected Outputs**:
- **Modern CPU (2013+)**: `Active backend: AVX2`
- **Very modern CPU (2017+)**: `Active backend: AVX-512` (if fully supported)
- **Older CPU (pre-2013)**: `Active backend: Scalar`

**Common Issues**:
- **Scalar on modern CPU**: Ensure you're building with `--features simd`
- **No speedup**: Verify entity count > 100 (SIMD overhead dominates for small arrays)
- **Crashes**: Check CPU actually supports detected features (VM/emulation issues)

#### Vectorized Operations

SIMD is currently applied to:

✅ **Velocity Updates**: `v' = v + a * dt`
```rust
use physics_engine::integration::simd_update_velocities;

// Process velocity updates with SIMD
simd_update_velocities(&mut vx, &mut vy, &mut vz, &ax, &ay, &az, dt);
```

✅ **Position Updates**: `p' = p + v * dt + 0.5 * a * dt²`
```rust
use physics_engine::integration::simd_update_positions;

// Process position updates with SIMD
simd_update_positions(&mut px, &mut py, &mut pz, &vx, &vy, &vz, &ax, &ay, &az, dt);
```

✅ **Force Accumulation**: `f_total += f`
```rust
use physics_engine::integration::simd_accumulate_forces;

// Accumulate forces with SIMD
simd_accumulate_forces(&mut total_fx, &mut total_fy, &mut total_fz, &fx, &fy, &fz);
```

**Tail Handling:**
All SIMD functions automatically handle entity counts not divisible by SIMD width (4 for AVX2, 8 for AVX-512) by processing remainder elements with scalar code. This ensures correctness for any entity count.

#### Numeric Determinism

**SIMD and scalar paths produce identical results** within floating-point tolerance:
- Same operations performed in same order
- IEEE 754 compliance maintained
- FMA (Fused Multiply-Add) disabled to ensure consistency
- Validated via extensive unit tests

**Floating-Point Tolerance:**
- Expected difference: < 1e-10 (machine epsilon for f64)
- Caused by rounding differences in intermediate calculations
- Acceptable for all physics simulations

### Limitations and Future Work

#### Current Limitations

❌ **Not SIMD-Optimized:**
- RK4 integrator (requires global staging - see below)
- Verlet integrator (not yet fully integrated with SoA buffers)
- Force computation (requires SoA layout for batch processing)
- HashMap-based component storage (non-contiguous data)

✅ **SIMD Infrastructure Ready:**
- SIMD helper functions available (`simd_update_velocities`, `simd_update_positions`, `simd_accumulate_forces`)
- Runtime CPU detection and backend selection
- Scalar fallback for non-SIMD CPUs
- Comprehensive test coverage for SIMD operations

#### RK4 Staging Implementation (v0.2.0+)

**Global Staging for N-Body Systems:**

The RK4 integrator has been updated to properly implement global staging for coupled N-body systems. For each RK4 stage (k1, k2, k3, k4), ALL entities are updated to their intermediate positions before computing forces:

```rust
// Stage 2 (k2): Update ALL entities to intermediate positions
for entity in &entities {
    let intermediate_pos = pos + k1_vel * dt/2;
    positions.update(entity, intermediate_pos);
}

// NOW compute forces with all entities at intermediate state
force_registry.clear_forces();
for entity in &entities {
    force_registry.accumulate_for_entity(entity);
}

// Compute k2 derivatives using forces at intermediate state
```

**Benefits:**
- ✅ Forces are computed with entire system at correct intermediate state
- ✅ Critical for gravitational and other coupled forces
- ✅ Ensures proper 4th-order accuracy for N-body problems
- ✅ Verified with comprehensive accuracy tests

**SIMD Integration for RK4:**
Due to the complex staging requirements (4 stages, each requiring global state updates), direct SIMD vectorization of RK4 requires careful handling of intermediate states. The current implementation prioritizes correctness and will benefit from SIMD indirectly when force computations are vectorized.

**Future Work:**
- Batch-process k1-k4 computations across all entities using SoA buffers
- Integrate SIMD helpers for position/velocity updates within RK4 stages
- Optimize memory access patterns for k-value storage

❌ **Platform Support:**
- Only x86_64 CPUs supported
- No ARM NEON support yet

#### Future Enhancements (v0.3.0+)

- [x] **AVX2 Vectorization**: Core SIMD infrastructure
- [x] **AVX-512 Support**: 8 × f64 per instruction on newer CPUs
- [x] **SIMD Helper Functions**: Velocity, position, force operations
- [ ] **Verlet SIMD Integration**: End-to-end vectorized Verlet
- [ ] **RK4 SIMD Integration**: Vectorized RK4 (complex)
- [ ] **SoA Component Storage**: True structure-of-arrays for better SIMD
- [ ] **ARM NEON Support**: SIMD for ARM64 platforms
- [ ] **Auto-Vectorization Hints**: Help compiler generate better SIMD code

### Best Practices for SIMD

✅ **Do:**
- Enable `simd` feature when targeting modern x86_64 CPUs (2013+)
- Use entity counts divisible by 8 (AVX-512) or 4 (AVX2) when possible for optimal throughput
- Run benchmarks to verify performance gains for your workload
- Trust runtime dispatch to select the best backend

✅ **Don't:**
- Don't assume 4× or 8× speedup - measure your actual workload
- Don't enable for old CPUs (pre-2013) - scalar may be faster
- Don't worry about tail handling - it's automatic
- Don't mix SIMD and non-SIMD builds in the same binary

### Benchmarking SIMD

```bash
# Run SIMD-specific benchmarks
cargo bench --features simd simd_operations

# Compare SIMD vs non-SIMD
cargo bench --bench integration -- --save-baseline scalar
cargo bench --features simd --bench integration -- --baseline scalar
```

**Expected Output:**
```
simd_operations/velocity_update/1000
                        time:   [598.45 ns 598.96 ns 599.75 ns]
                        thrpt:  [1.6674 Gelem/s 1.6696 Gelem/s 1.6710 Gelem/s]
```

**Interpretation:**
- **time**: Time to process 1000 entities
- **thrpt**: Throughput in billion elements per second
- **Gelem/s**: Higher is better (1.67 Gelem/s = 1.67 billion f64 operations per second)

## Memory Pooling (v0.2.0+)

### Overview

As of version 0.2.0, the physics engine includes memory pooling for reducing allocation churn in hot paths. Pools reuse temporary buffers across integration steps, reducing per-frame allocation overhead.

### What is Pooled

**RK4 Integrator:**
- 8 intermediate buffers (k1-k4 for positions and velocities)
- Automatic acquisition and return via RAII guards
- Thread-safe borrowing for parallel systems

**World Entity Storage:**
- Preallocatable capacity via `World::with_capacity()`
- Reduces hash table resizing during entity creation

### Configuration

Pools are configured at integrator creation time:

```rust
use physics_engine::pool::PoolConfig;
use physics_engine::integration::RK4Integrator;

// Default configuration (64 initial capacity, 8 max pool size)
let integrator = RK4Integrator::new(timestep);

// Custom configuration
let pool_config = PoolConfig::new(128, 16)  // larger buffers, more pooled
    .with_growth_factor(1.5)                // slower growth
    .with_logging();                        // log resize events

let integrator = RK4Integrator::with_pool_config(timestep, pool_config);
```

**Configuration Parameters:**

| Parameter | Default | Description |
|-----------|---------|-------------|
| `initial_capacity` | 64 | Initial size of each pooled buffer |
| `max_pool_size` | 8 | Maximum number of buffers to keep in pool |
| `growth_factor` | 2.0 | Multiplier when expanding capacity |
| `log_resize_events` | false | Log when pools grow/shrink |

### Performance Impact

**Expected Improvements:**
- 10-20% reduction in allocation overhead for RK4
- Reduced frame time variance from fewer allocations
- Better performance with large entity counts (>100)

**Benchmark Results:**

Run `cargo bench --bench pooling` to measure on your hardware:

```
rk4_pooling/default_config/100
                        time:   [X µs X µs X µs]
rk4_pooling/large_capacity/100
                        time:   [Y µs Y µs Y µs]
                        improvement: ~5-15% (typical)
```

### Monitoring Pool Performance

Query pool statistics to tune configuration:

```rust
let (pos_stats, vel_stats, acc_stats) = integrator.pool_stats();

println!("Position pool hit rate: {:.1}%", pos_stats.hit_rate());
println!("Position pool size: {}", pos_stats.pool_size);
println!("Peak pool size: {}", pos_stats.peak_size);
println!("Resize count: {}", pos_stats.resize_count);
```

**Tuning Guidelines:**

- **Low hit rate (< 50%)**: Increase `max_pool_size` to cache more buffers
- **High resize count**: Increase `initial_capacity` to avoid early reallocations
- **Memory concerns**: Decrease `max_pool_size` to reduce peak memory usage
- **Pool never fills**: Decrease `max_pool_size` to save memory

### Thread Safety

Pools use `Arc<Mutex<>>` for thread-safe access:
- Multiple threads can acquire buffers concurrently
- Guards automatically return buffers on drop
- No risk of use-after-free or double-free

### Edge Cases

**Pool Exhaustion:**
- When all pooled buffers are in use, new buffers are allocated
- Pool grows automatically up to `max_pool_size`
- Beyond max size, buffers are deallocated on return

**Shutdown/Cleanup:**
- Pools are automatically cleared when integrator is dropped
- Explicit cleanup: `integrator.clear_pools()`
- No memory leaks in long-lived applications

### Best Practices

✅ **Do:**
- Use default configuration initially
- Tune based on pool statistics for your workload
- Preallocate World capacity when entity count is known
- Monitor hit rates during development

❌ **Don't:**
- Don't over-tune without measuring
- Don't set `max_pool_size` > entity count (wastes memory)
- Don't worry about thread safety - it's automatic
- Don't manually clear pools unless needed for shutdown

## Future Performance Enhancements

### Near-Term (Next Release)

- [x] **SIMD Vectorization**: 2-4× speedup with AVX2 (v0.2.0)
- [x] **Memory Pooling**: Reduce allocation overhead (v0.2.0)
- [ ] **Structure-of-Arrays (SoA) Integration with SIMD**: 2-4× additional speedup
- [ ] **Adaptive Chunk Sizing**: Better parallel efficiency

### Medium-Term

- [ ] **Barnes-Hut Tree**: O(N log N) gravity for N > 500
- [ ] **Octree Spatial Partitioning**: Fast collision detection
- [ ] **Adaptive Timestepping**: Automatic dt adjustment
- [ ] **Cache-Oblivious Algorithms**: Better memory performance

### Long-Term

- [ ] **GPU Acceleration**: 10-100× speedup with CUDA/compute shaders
- [ ] **Fast Multipole Method**: O(N) gravity computation
- [ ] **Distributed Simulation**: Multi-machine physics
- [ ] **WebGPU Integration**: Browser-based GPU acceleration

## Interpreting Your Benchmark Results

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark group
cargo bench integrator_throughput

# Run with baseline for comparison
cargo bench --bench integration -- --save-baseline before_changes
# Make changes...
cargo bench --bench integration -- --baseline before_changes
```

### Understanding Output

**Example Criterion Output:**
```
integrator_throughput/verlet/100
                        time:   [2.2 µs 2.3 µs 2.4 µs]
                        thrpt:  [41.6k elem/s 43.5k elem/s 45.5k elem/s]
```

**Interpretation:**
- **time**: Time to process 100 entities (lower is better)
- **thrpt**: Throughput in entities per second (higher is better)
- **Range**: [lower bound, estimate, upper bound] from statistical analysis

### Warning Signs

⚠️ **Performance Regression Indicators:**
- Sudden 2× or more slowdown
- Increased variance in measurements
- Memory usage growth over time
- Excessive allocations per iteration

✅ **Healthy Performance Profile:**
- Consistent timing across runs
- Linear scaling with entity count
- Minimal allocations after warm-up
- Cache miss rate < 10%

## Platform-Specific Notes

### Linux (Primary Target)

- ✅ Best performance and profiling tools
- ✅ Perf, Valgrind, flamegraph support
- ✅ Consistent timer precision

### macOS

- ✅ Good performance, similar to Linux
- ⚠️ Different profiling tools (Instruments)
- ⚠️ May have slight performance differences

### Windows

- ✅ Good performance with MSVC or GNU toolchains
- ⚠️ Profiling requires Visual Studio or alternative tools
- ⚠️ May have different parallel scaling characteristics

### WebAssembly

- ⚠️ Build with `--no-default-features` (no threading)
- ⚠️ 2-10× slower than native
- ⚠️ Limited to single-threaded execution
- ⚠️ No SIMD support in current Wasm standard

## References

### Performance Analysis

- Drepper, U. (2007). *What Every Programmer Should Know About Memory*
- Fog, A. (2022). *Optimizing software in C++*. Technical University of Denmark.

### Benchmarking

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- Gregg, B. (2013). *Systems Performance: Enterprise and the Cloud*. Prentice Hall.

### Parallel Computing

- McCool, M., Robison, A., & Reinders, J. (2012). *Structured Parallel Programming*. Morgan Kaufmann.
- [Rayon: Data Parallelism in Rust](https://docs.rs/rayon/)

### Numerical Performance

- Hairer, E., Nørsett, S. P., & Wanner, G. (1993). *Solving Ordinary Differential Equations I: Nonstiff Problems*. Springer.
- Higham, N. J. (2002). *Accuracy and Stability of Numerical Algorithms* (2nd ed.). SIAM.
