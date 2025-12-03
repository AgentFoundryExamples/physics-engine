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

The engine provides two storage implementations with different performance characteristics:

**`HashMapStorage<Component>`** (Simple, Flexible)
- ✅ Simple implementation
- ✅ Sparse entity support
- ✅ No pre-allocation required
- ❌ Poor cache locality (HashMap indirection)
- ❌ No SIMD vectorization
- **Best for**: Small entity counts (< 100), prototyping

**`SoAStorage<Component>`** (Dense Array Storage) ✨ **New in v0.2.0**

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
- ✅ Direct array iteration support
- ✅ Swap-remove prevents fragmentation
- ⚠️ Loads entire components even if only one field needed (AoS limitation)
- ⚠️ True field-level SIMD requires separate field arrays (not implemented)

**Benchmark Results** (see `cargo bench --bench storage`):

The benchmarks now use fair comparisons with proper setup isolation:

| Operation | Entity Count | HashMap | Dense Storage | Notes |
|-----------|--------------|---------|---------------|-------|
| Via Entities | 1,000 | 100% | **Similar** | Both use entity lookup |
| Direct Array | 1,000 | N/A | **150-200%** | Dense storage advantage |
| Via Entities | 10,000 | 100% | **Similar** | Lookup overhead dominates |
| Direct Array | 10,000 | N/A | **200-300%** | Cache benefits increase |

**Cache Performance Analysis**:

1. **Direct Array Iteration**: Maximum benefit when API allows
   - HashMap: Must iterate via entity list with lookups
   - Dense: Can iterate directly over `Vec<T>` with sequential access
   - Sequential memory layout loads entire cache lines efficiently
   - Hardware prefetcher maximally utilized

2. **Via Entity Lookup**: Similar performance
   - Both require HashMap/index lookup per entity
   - Dense array access after lookup is slightly faster than HashMap
   - Lookup overhead dominates for small components

3. **Memory Bandwidth**:
   - Dense storage: One contiguous allocation, better for iteration
   - HashMap: Scattered allocations, more cache line loads
   - Benefit increases with entity count and iteration frequency

**Profiling Evidence**:

The performance benefits are most pronounced when using direct array iteration
(not available with HashMap). The benchmarks now properly separate:
- Via-entities iteration: Both storages similar (lookup overhead)
- Direct array iteration: Dense storage shows 1.5-3× improvement

**Key Observations**:
- Dense storage excels when systems can use direct array iteration
- Via-entity access shows similar performance (lookup overhead dominates)
- Dense packing provides cache benefits but not as dramatic as true SoA would
- True Structure-of-Arrays (separate field vectors) would need new trait design

**Usage Example**:
```rust
use physics_engine::ecs::{SoAStorage, ComponentStorage};
use physics_engine::ecs::components::Position;

let mut positions = SoAStorage::<Position>::new();

// Insert components as usual
positions.insert(entity, Position::new(1.0, 2.0, 3.0));

// Efficient bulk iteration (this is where SoA shines)
for pos in positions.components() {
    // Process with excellent cache locality
    // ~2-3× faster than HashMap iteration for 10k+ entities
}
```

**When to Use SoA**:
- ✅ Systems that iterate over many components
- ✅ Medium to large entity counts (> 100)
- ✅ Performance-critical update loops
- ✅ When memory bandwidth is a bottleneck

**When to Use HashMap**:
- ✅ Small entity counts (< 100)
- ✅ Sparse component access patterns
- ✅ Prototyping and development
- ✅ When simplicity is more important than performance

**Future SIMD Enhancement**:
With explicit SIMD operations (AVX2/AVX-512), SoA storage could achieve:
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

## Future Performance Enhancements

### Near-Term (Next Release)

- [ ] **Structure-of-Arrays (SoA) Layout**: 2-4× speedup for integration
- [ ] **SIMD Vectorization**: 2-4× additional speedup with AVX2
- [ ] **Adaptive Chunk Sizing**: Better parallel efficiency
- [ ] **Memory Pooling**: Reduce allocation overhead

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
