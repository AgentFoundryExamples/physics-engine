# Project Roadmap

## Overview

This document outlines the planned features, enhancements, and long-term vision for the physics engine. It covers near-term improvements, GPU acceleration strategies, advanced integration methods, collision systems, constraints, and visualization integration.

## Project Vision

**Mission**: Build a high-performance, extensible physics simulation engine that balances ease of use with computational efficiency, suitable for games, scientific visualization, and educational purposes.

**Core Values**:
- **Performance**: Optimize for modern hardware (multi-core CPUs, GPUs)
- **Correctness**: Accurate physics with well-tested numerical methods
- **Ergonomics**: Clean API that's easy to learn and use
- **Extensibility**: Plugin system for custom physics and constraints
- **Portability**: Target desktop, web (WebAssembly), and eventually mobile

## Version 0.1.0 - Foundation (Completed)

✅ **Released**: Initial ECS-based physics engine

**Completed Features** (ISS-1 through ISS-5):
- [x] Entity Component System (ECS) core architecture
- [x] Newtonian physics components (Position, Velocity, Acceleration, Mass)
- [x] Staged system scheduler with parallel execution support
- [x] Force accumulation and provider system
- [x] Velocity Verlet integrator (symplectic, 2nd-order accurate)
- [x] RK4 integrator (4th-order accurate)
- [x] Plugin system with dependency resolution
- [x] Gravitational N-body plugin with realistic physics
- [x] Comprehensive documentation (architecture, integration, plugins, examples)
- [x] Example programs (basic ECS, solar system, particle collision)
- [x] Benchmark suite comparing integrators
- [x] Conservation tests validating accuracy

**Deliverables**:
- Core library crate with ECS and integration
- Plugin API for extensibility
- Working examples demonstrating features
- Documentation suite covering architecture and usage

## Version 0.2.0 - Performance & Memory (Completed)

✅ **Released**: 2025-12-03

**Focus**: Memory layout optimization and cache efficiency improvements

### Completed Features

#### ✅ Dense Array Component Storage
**Status**: Implemented as `SoAStorage<T>` using dense Array-of-Structures layout

**Delivered Benefits**:
- ✅ 1.5-3× faster sequential iteration vs HashMap for 1000+ entities
- ✅ Direct array access via `components()` method
- ✅ Swap-remove prevents memory fragmentation
- ✅ Full API compatibility with `ComponentStorage` trait

**Implementation Notes**:
- Current implementation uses dense `Vec<T>` (Array-of-Structures)
- Name `SoAStorage` retained for API compatibility
- True Structure-of-Arrays (separate field vectors) deferred to future release
- Provides excellent cache locality for iteration-heavy workloads

**Usage**:
```rust
use physics_engine::ecs::SoAStorage;

// Create dense storage for 1000 entities
let mut positions = SoAStorage::<Position>::with_capacity(1000);

// Efficient bulk iteration
for pos in positions.components() {
    // Process with excellent cache locality
}
```

#### ✅ SIMD Vectorization
**Status**: Implemented with AVX2 support and runtime CPU detection

**Delivered Benefits**:
- ✅ 2-4× speedup for math operations on AVX2 CPUs
- ✅ Velocity updates: ~1.67 Gelem/s throughput
- ✅ Position updates: ~1.34 Gelem/s throughput
- ✅ Force accumulation: ~1.95 Gelem/s throughput
- ✅ Automatic scalar fallback for older CPUs

**Implementation**:
- AVX2 support for x86_64 (Intel Haswell 2013+, AMD Excavator 2015+)
- Runtime CPU feature detection via `raw-cpuid` crate
- Transparent backend selection (no user configuration)
- Enable with `--features simd` flag

**Performance** (AMD EPYC 7763):
- 4 × f64 operations per instruction (256-bit vectors)
- 2-4× speedup for entity counts >1000
- Zero overhead on CPUs without AVX2 (scalar fallback)

#### ✅ Memory Pooling
**Status**: Implemented for RK4 integrator with configurable pools

**Delivered Benefits**:
- ✅ 10-20% reduction in allocation overhead
- ✅ More consistent frame times (reduced GC pauses)
- ✅ Better performance for entity counts >100
- ✅ Generic `ObjectPool<T>` for reusable buffers

**Implementation**:
- Specialized `HashMapPool<K, V>` for HashMap reuse
- Thread-safe with `Mutex` synchronization
- Configurable via `PoolConfig` (capacity, max size, growth factor)
- Integrated into `RK4Integrator::with_pool_config()`

**Usage**:
```rust
use physics_engine::pool::PoolConfig;
use physics_engine::integration::RK4Integrator;

let pool_config = PoolConfig::new(256, 16)
    .with_growth_factor(1.5)
    .with_logging();
let integrator = RK4Integrator::with_pool_config(1.0 / 60.0, pool_config);

// Monitor pool performance
let (pos_stats, vel_stats, acc_stats) = integrator.pool_stats();
println!("Hit rate: {:.1}%", pos_stats.hit_rate());
```

### Deferred to Future Releases

#### ⏸️ Adaptive Chunk Sizing for Parallel Execution
**Reason**: Requires production workload data for effective tuning
**Target**: v0.3.0 or later based on community feedback

**Planned Approach**:
- Profile-guided chunk sizing based on entity count, thread count, cache size
- Automatic work distribution optimization for parallel execution
- Depends on real-world usage patterns and performance data

#### ⏸️ Query DSL for Component Access
**Reason**: Requires additional trait design for optimal integration with dense storage
**Target**: v0.3.0 or later

**Rationale**:
- Current `components()` method provides functional and performant access
- Query DSL design should leverage lessons learned from storage evolution
- Deferred to allow proper integration with true SoA in future

**Current Workaround**:
```rust
// Use direct array access (fully supported)
let positions = storage.components();
let velocities = velocity_storage.components();

for (pos, vel) in positions.iter().zip(velocities.iter()) {
    // Efficient iteration with current API
}
```

### Documentation Delivered

- ✅ **Updated** `docs/architecture.md` - Dense storage design and trade-offs
- ✅ **Updated** `docs/performance.md` - SIMD benchmarks, memory pooling tuning
- ✅ **Updated** `README.md` - Version 0.2.0 features and configuration
- ✅ **Updated** `.env.example` - Memory pooling reference configuration
- ✅ **Added** Storage benchmark suite (`cargo bench --bench storage`)
- ✅ **Added** Pooling benchmark suite (`cargo bench --bench pooling`)
- ✅ **Added** SIMD benchmark suite (`cargo bench --features simd`)

## Version 0.3.0 - Spatial Acceleration & Query Improvements (Planned)

**Target**: Q3-Q4 2025 (aspirational)

**Focus**: Scalability for large particle counts and improved API ergonomics

### High Priority

#### Query DSL for Component Access (Deferred from v0.2.0)
**Goal**: Ergonomic and efficient entity queries

**Motivation**: 
- Improve API ergonomics for common query patterns
- Leverage lessons learned from dense storage implementation
- Enable better integration with true Structure-of-Arrays in future

**Planned API**:
```rust
world.query::<(&Position, &mut Velocity, &Mass)>()
    .filter(|e| e.has::<Active>())
    .for_each(|(pos, vel, mass)| {
        // Process entities with type-safe access
    });

// Parallel iteration
world.query::<(&mut Position, &Velocity)>()
    .par_iter()
    .for_each(|(pos, vel)| {
        // Automatic parallelization
    });
```

**Benefits**:
- Type-safe compile-time checking
- Automatic parallelization hints
- Clear component dependency tracking
- Better optimization opportunities for compiler
- Preparation for true SoA migration

**Design Considerations**:
- Minimize runtime overhead vs direct array access
- Preserve cache-friendly access patterns
- Support both mutable and immutable queries
- Integration with Rayon for parallel execution

#### Adaptive Chunk Sizing for Parallel Execution (Deferred from v0.2.0)
**Goal**: Automatically tune work distribution for optimal parallel performance

**Motivation**:
- Default Rayon chunking may not be optimal for all workloads
- Entity count, force complexity, and cache size affect ideal chunk size
- Manual tuning is tedious and workload-specific

**Planned Approach**:
- Runtime profiling of system execution time per entity
- Dynamic adjustment based on:
  - Entity count (larger counts → larger chunks for better cache utilization)
  - Available threads (distribute work evenly)
  - L1/L2/L3 cache sizes (keep working set in cache)
  - Force provider complexity (heavier forces → smaller chunks for load balancing)

**Implementation Strategy**:
```rust
// Automatic tuning based on workload
let scheduler = Scheduler::new()
    .with_adaptive_chunking()
    .with_profiling_interval(100); // Adjust every 100 frames

// Manual override for specific systems
scheduler.set_chunk_size("gravity_system", 512);
```

**Expected Benefits**:
- 5-15% performance improvement over default chunking
- Better scaling on systems with many cores
- Reduced load imbalance for heterogeneous workloads

**Dependencies**:
- Requires performance counters (cycle counts, cache misses)
- May need platform-specific profiling APIs (perf on Linux, Instruments on macOS)
- Should degrade gracefully to fixed chunks if profiling unavailable

#### True Structure-of-Arrays Storage Evolution
**Goal**: Evolve from dense AoS to true SoA for additional performance gains

**Current State**: `SoAStorage` uses dense `Vec<T>` (Array-of-Structures)

**Target State**: Separate vectors per field
```rust
struct TrueSoAStorage<T> {
    // For Position component:
    x: Vec<f64>,  // All x coordinates contiguous
    y: Vec<f64>,  // All y coordinates contiguous
    z: Vec<f64>,  // All z coordinates contiguous
    entity_map: HashMap<Entity, usize>,
}
```

**Additional Benefits Over Current Dense Storage**:
- Better SIMD utilization (process single field across many entities)
- Reduced memory traffic when accessing subset of fields
- Enables field-level parallelism
- Better compiler auto-vectorization

**Challenges**:
- Trait redesign to expose per-field access
- Query DSL integration for ergonomic multi-field access
- Migration path from current `SoAStorage`
- Maintaining API compatibility where possible

**Estimated Impact**: Additional 1.5-2× speedup for field-heavy operations

#### Barnes-Hut Tree for Gravity
**Goal**: O(N log N) gravitational force computation

**Algorithm**:
- Hierarchical octree subdivision
- Approximate distant particles as single mass
- Configurable accuracy threshold (θ parameter)

**Expected Impact**: 10-100× speedup for N > 1000 particles

**Use Cases**:
- Large N-body simulations (galaxies, star clusters)
- Particle swarms with thousands of entities
- Grand strategy games with many gravitating objects

**Implementation Notes**:
- Tree construction: O(N log N)
- Force evaluation: O(N log N)
- Parallel tree traversal for additional speedup
- Configurable θ parameter (accuracy vs speed trade-off)

**Performance Targets**:
- N=1,000: ~10× faster than O(N²)
- N=10,000: ~100× faster than O(N²)
- N=100,000: Enable simulations impossible with brute force

#### Octree Spatial Partitioning
**Goal**: General-purpose spatial acceleration structure

**Use Cases**:
- Fast collision detection (O(log N) per query)
- Spatial queries (nearest neighbor, radius search)
- Level-of-detail selection
- Frustum culling for rendering integration

**Features**:
- Dynamic insertion/removal during simulation
- Configurable subdivision criteria
- Parallel construction for large scenes
- Support for moving entities (incremental updates)

**API Example**:
```rust
let mut octree = Octree::new(bounds);

// Insert entities
for entity in entities {
    octree.insert(entity, position);
}

// Query nearby entities
let nearby = octree.query_sphere(center, radius);

// Update moving entity
octree.update(entity, new_position);
```

### Medium Priority

#### Broad-Phase Collision Detection
**Goal**: Quickly identify potential collision pairs before narrow-phase

**Methods Under Consideration**:
- **Sweep and Prune** (SAP): Sort along axes, detect overlaps
- **Spatial Hashing**: Grid-based bucketing  
- **Bounding Volume Hierarchies** (BVH): Tree of bounding boxes
- **Octree-based**: Leverage octree from spatial partitioning

**Trade-offs**:
| Method | Construction | Query | Dynamic Updates | Memory |
|--------|--------------|-------|-----------------|--------|
| SAP    | O(N log N)   | O(N)  | Fast            | Low    |
| Spatial Hash | O(N)  | O(1) avg | Instant        | Medium |
| BVH    | O(N log N)   | O(log N) | Slow        | High   |
| Octree | O(N log N)   | O(log N) | Medium      | Medium |

**Decision**: Start with spatial hashing for simplicity, add BVH/octree for complex scenes

**Performance Target**: Reduce collision pairs by 90-99% vs brute force O(N²)

#### Profiling Integration
**Goal**: Built-in profiling tools for performance analysis

**Features**:
- Frame time breakdown by system
- Entity count tracking
- Memory allocation tracking
- Cache miss counters (where available)
- Performance regression detection

**Output Formats**:
- Console logging
- JSON export for external analysis
- Chrome tracing format for visualization
- Integration with `cargo flamegraph`

**Example**:
```rust
let profiler = Profiler::new()
    .with_sampling_interval(Duration::from_millis(16))
    .with_json_export("profile.json");

scheduler.attach_profiler(profiler);
```

### Documentation Planned

- [ ] Query DSL guide with examples and best practices
- [ ] Adaptive chunking tuning guide
- [ ] True SoA migration guide for v0.2.0 users
- [ ] Barnes-Hut tree parameter selection guide
- [ ] Octree usage examples and performance characteristics
- [ ] Broad-phase collision detection comparison and selection guide
- [ ] Profiling integration guide

## Version 0.4.0 - Collision & Constraints (Planned)

**Target**: Q4 2025 - Q1 2026 (aspirational)

**Focus**: Contact resolution and constraint systems

### High Priority

#### Narrow-Phase Collision Detection
**Goal**: Precise collision detection between shapes

**Supported Primitives**:
- Sphere-sphere (simple, fast)
- Box-box (SAT or GJK)
- Sphere-box (hybrid approach)
- Convex polyhedra (GJK/EPA)

**Output**: Contact points, normals, penetration depths

#### Impulse-Based Collision Response
**Goal**: Realistic collision resolution

**Algorithm**: Sequential impulse method
- Compute collision impulses from contact constraints
- Apply impulses to velocities
- Support friction and restitution coefficients

**Features**:
- Configurable elasticity (0 = inelastic, 1 = elastic)
- Friction modeling (static and kinetic)
- Contact point caching for stability

#### Joint Constraints
**Goal**: Connect entities with mechanical joints

**Joint Types**:
- **Ball-and-socket**: Point-to-point connection (3 DOF)
- **Hinge**: Revolute joint (1 DOF rotation)
- **Slider**: Prismatic joint (1 DOF translation)
- **Fixed**: Weld two bodies together (0 DOF)

**Solver**: Iterative constraint solver (PGS or similar)

### Medium Priority

#### Distance Constraints
**Goal**: Maintain fixed distances between entities

**Use Cases**:
- Rope simulation
- Cloth pinning
- Rigid body clustering

**Algorithm**: XPBD (Extended Position Based Dynamics) or Gauss-Seidel

#### SHAKE/RATTLE Algorithms
**Goal**: Preserve geometric constraints during integration

**SHAKE**: Position constraint projection
**RATTLE**: Velocity constraint projection (extension of SHAKE)

**Use Cases**:
- Rigid body constraints
- Fixed bond lengths in molecular dynamics
- Holonomic constraints

## Version 0.5.0 - GPU Acceleration (Planned)

**Target**: Q1-Q2 2026 (aspirational)

**Focus**: Massively parallel computation on GPUs

### GPU Backend Strategy

**Primary Options**:

#### Option 1: CUDA (NVIDIA-specific)
**Pros**:
- ✅ Mature ecosystem with extensive documentation
- ✅ Excellent performance on NVIDIA GPUs
- ✅ Large body of existing physics code to reference
- ✅ Comprehensive debugging and profiling tools

**Cons**:
- ❌ NVIDIA-only (excludes AMD, Intel, Mac)
- ❌ Requires CUDA SDK installation
- ❌ Not available in WebAssembly

**Best For**: Research, scientific computing, data centers with NVIDIA GPUs

#### Option 2: WebGPU Compute Shaders (Multi-platform)
**Pros**:
- ✅ Cross-platform (Windows, Linux, macOS, eventually Web)
- ✅ Works on NVIDIA, AMD, Intel, Apple GPUs
- ✅ Standard API (based on WebGPU spec)
- ✅ Rust support via `wgpu` crate
- ✅ Path to browser-based simulation

**Cons**:
- ❌ Less mature than CUDA
- ❌ May have performance gap vs CUDA on NVIDIA hardware
- ❌ Debugging tools less developed

**Best For**: Cross-platform applications, games, web integration

#### Option 3: Hybrid Approach
**Strategy**: Support both CUDA and WebGPU via abstraction layer

```rust
pub trait GpuBackend {
    fn allocate_buffer(&mut self, size: usize) -> BufferId;
    fn compute_forces(&mut self, positions: &[f64], masses: &[f64]) -> Vec<Force>;
    fn integrate(&mut self, dt: f64, positions: &mut [f64], velocities: &mut [f64]);
}

// Implementations:
// - CudaBackend (NVIDIA)
// - WebGpuBackend (cross-platform)
// - CpuBackend (fallback)
```

**Decision Criteria**:
- **Research use case**: CUDA for maximum performance
- **General use case**: WebGPU for portability
- **Library goal**: Likely prioritize WebGPU, add CUDA if demand exists

### GPU Implementation Plan

#### Phase 1: Force Computation on GPU
**Goal**: Offload O(N²) pairwise force calculations

**Algorithm**:
```
For each particle i (parallel):
    force_i = sum over j≠i of force(i, j)
```

**Challenges**:
- Memory transfer overhead (CPU ↔ GPU)
- Divergent execution for conditionals
- Atomic operations for force accumulation
- Load balancing for uneven distributions

**Expected Speedup**: 10-100× for N > 1000

#### Phase 2: Integration on GPU
**Goal**: Keep entire simulation on GPU

**Benefits**:
- Eliminate CPU-GPU transfer bottleneck
- Enables larger timesteps with faster computation
- Sustained performance for large N

**Challenges**:
- Complex integrators (RK4) require more GPU memory
- Constraint solvers are harder to parallelize
- Debugging on GPU is more difficult

#### Phase 3: Hybrid CPU/GPU Execution
**Goal**: Automatically partition work between CPU and GPU

**Heuristics**:
- N < 100: CPU only (GPU overhead not worth it)
- 100 < N < 500: Hybrid (forces on GPU, constraints on CPU)
- N > 500: GPU dominant

**Framework**: Similar to heterogeneous computing in HPC

### GPU Feature Roadmap

- [ ] WebGPU compute shader framework integration (`wgpu`)
- [ ] GPU buffer management and transfer optimization
- [ ] Parallel force computation kernel (gravity, springs)
- [ ] GPU integration kernels (Verlet, RK4)
- [ ] Bandwidth optimization (minimize CPU-GPU transfers)
- [ ] Hybrid CPU/GPU workload distribution
- [ ] Optional CUDA backend for NVIDIA-specific optimizations

### Platform-Specific Considerations

**Desktop** (Windows, Linux, macOS):
- ✅ Full GPU access via WebGPU or CUDA
- ✅ Large GPU memory (GB range)
- ✅ High bandwidth interconnects

**Web** (WebAssembly + WebGPU):
- ⚠️ WebGPU support varies by browser
- ⚠️ Memory limits more restrictive
- ⚠️ Shader compilation may be slower
- ✅ Still provides massive speedup vs CPU-only

**Mobile** (Future):
- ⚠️ GPU capabilities vary widely
- ⚠️ Power consumption concerns
- ⚠️ Thermal throttling
- ✅ Modern mobile GPUs quite capable for moderate N

## Version 0.6.0 - Visualization Integration (Planned)

**Target**: Q2-Q3 2026 (aspirational)

**Focus**: Real-time visualization and debugging tools

### WebGPU + Three.js Visualizer

**Architecture**:

```mermaid
graph TB
    subgraph "Physics Engine (Rust)"
        PE[Physics Simulation]
        GPU[GPU Compute]
    end
    
    subgraph "Visualization Layer"
        WASM[WebAssembly Bridge]
        Three[Three.js Renderer]
        UI[Control Panel]
    end
    
    subgraph "Browser"
        Canvas[WebGL/WebGPU Canvas]
    end
    
    PE --> GPU
    GPU --> WASM
    WASM --> Three
    Three --> Canvas
    UI --> PE
```

**Components**:

#### 1. Rust-WASM Bridge
**Responsibilities**:
- Compile physics engine to WebAssembly
- Expose API for JavaScript integration
- Efficient binary data transfer (SharedArrayBuffer when available)
- Handle threading constraints in WASM

**Technologies**:
- `wasm-bindgen` for Rust-JS interop
- `wasm-pack` for build tooling
- `web-sys` for DOM access

#### 2. Three.js Rendering
**Responsibilities**:
- Real-time 3D rendering of simulation
- Camera controls and scene navigation
- Lighting and materials for visual quality
- Particle systems and instanced rendering

**Features**:
- ✅ Sphere/box rendering for particles
- ✅ Trail rendering for orbits
- ✅ Glow effects for stars
- ✅ Skybox/background
- ✅ Dynamic lighting
- ✅ Shadow mapping (optional)

#### 3. Interactive Controls
**Responsibilities**:
- Simulation parameters (timestep, gravity, integrator)
- Camera controls (orbit, pan, zoom)
- Entity selection and inspection
- Play/pause/step controls
- Recording and replay

**UI Framework Options**:
- **React**: Full-featured, large ecosystem
- **Svelte**: Lightweight, good performance
- **Vanilla JS**: Minimal dependencies

**Decision**: Likely Svelte for balance of features and size

### Desktop Visualization (Optional)

**Alternative**: Native desktop visualization using:
- **Bevy**: Rust game engine with ECS integration
- **wgpu**: Cross-platform graphics API
- **egui**: Immediate-mode GUI for controls

**Advantages**:
- Better performance than browser
- No WASM compilation step
- Full GPU access

**Disadvantages**:
- Requires native installation
- Platform-specific builds
- Less accessible than web version

**Decision**: Web version is priority, desktop visualization as optional alternative

### Debugging and Profiling Tools

**Planned Features**:
- Real-time performance graphs (frame time, entity count)
- Energy conservation monitoring
- Force vector visualization
- Constraint violation highlighting
- Breakpoints on entity collision
- State recording and replay

### Example Use Cases

**Educational**:
- Interactive solar system with planet selection
- Particle collision demonstrations
- Constraint system experiments

**Gamedev**:
- Physics sandbox for prototyping
- Debug visualization for game physics
- Parameter tuning interface

**Research**:
- N-body simulation visualization
- Algorithm comparison tool
- Publication-quality renders

## Version 1.0.0 - Stable Release (Long-term)

**Target**: 2026 (aspirational)

**Criteria for 1.0**:
- [ ] Stable, well-tested API
- [ ] Comprehensive documentation
- [ ] Multiple backend support (CPU, GPU)
- [ ] Production-ready performance
- [ ] Active community and ecosystem
- [ ] Semantic versioning commitment
- [ ] Long-term support plan

## Advanced Features (Future Exploration)

### Soft Body Physics
**Complexity**: High
**Use Cases**: Cloth, deformable objects, fluid simulation (particles)

**Approaches**:
- Mass-spring systems
- Position-based dynamics (PBD)
- Finite element methods (FEM)

**Challenges**: Stability, stiffness, computational cost

### Fluid Simulation
**Complexity**: Very High
**Use Cases**: Water, smoke, gas dynamics

**Methods**:
- Smoothed Particle Hydrodynamics (SPH)
- Lattice Boltzmann Method (LBM)
- Navier-Stokes solvers

**Challenges**: Incompressibility, boundary conditions, visual plausibility

### Fracture and Destruction
**Complexity**: High
**Use Cases**: Breaking objects, terrain deformation

**Approaches**:
- Voronoi fracture patterns
- Progressive damage models
- Particle emission on fracture

**Challenges**: Real-time performance, visual quality, stability

### Haptic Feedback Integration
**Complexity**: Medium
**Use Cases**: VR/AR, surgical simulation, robotics

**Requirements**:
- Very high update rates (1kHz+)
- Low latency
- Stable force feedback

## Technology Dependencies

### Core Dependencies (Current)
- **rayon**: Parallel execution (v1.10.0)
- **semver**: Version checking (v1.0.23)

### Planned Dependencies

**Memory & Performance**:
- **`packed_simd`** or **`portable-simd`**: SIMD intrinsics (when SoA added)
- **`mimalloc`** or **`jemalloc`**: Alternative allocators (if needed)

**GPU Computing**:
- **`wgpu`**: WebGPU bindings (~0.17 when ready)
- **`bytemuck`**: Safe byte casting for GPU buffers
- **`cuda-sys`**: CUDA bindings (optional, NVIDIA-specific)

**Web Integration**:
- **`wasm-bindgen`**: Rust-JavaScript interop
- **`web-sys`**: Browser API access
- **`console_error_panic_hook`**: Better WASM error messages

**Serialization** (if needed):
- **`serde`**: Serialization framework
- **`bincode`**: Binary serialization for efficiency

**All dependencies will be pinned to specific versions in `Cargo.toml` to ensure reproducible builds.**

## Non-Goals

**What This Project Will NOT Do** (to maintain focus):

❌ **Full Game Engine**: Physics only, not rendering/audio/input
- Use with Bevy, ggez, or other game engines instead

❌ **Fluid Simulation**: Too complex for initial scope
- May reconsider in distant future

❌ **Robotics Simulation**: Requires specialized constraints and actuators
- Better served by dedicated robotics frameworks

❌ **Quantum Mechanics**: Classical physics only
- Out of scope for this project

❌ **General PDE Solver**: Focused on particle/rigid body physics
- Use FEM libraries for continuous fields

## Contributing

**How to Influence the Roadmap**:
1. Open GitHub issues with feature requests
2. Discuss in community forums or Discord (if established)
3. Submit pull requests for prototypes
4. Share use cases and requirements

**Prioritization Criteria**:
- Community demand and use cases
- Implementation complexity
- Alignment with core values
- Maintenance burden
- Ecosystem compatibility

## Risk Mitigation

### Technical Risks

**GPU Backend Selection**:
- **Risk**: Choose wrong backend, need to rewrite
- **Mitigation**: Abstract interface, prototype both CUDA and WebGPU
- **Fallback**: CPU-only remains fully functional

**Performance Targets**:
- **Risk**: Optimizations don't achieve expected speedups
- **Mitigation**: Early prototyping, benchmark-driven development
- **Fallback**: Document realistic expectations, focus on correctness

**WebAssembly Limitations**:
- **Risk**: WASM lacks features needed for visualization
- **Mitigation**: Keep desktop visualization as backup plan
- **Fallback**: Native desktop app if web proves insufficient

### Scope Creep

**Risk**: Feature requests exceed development capacity

**Mitigation**:
- Maintain clear non-goals list
- Focus on core competency (particle/rigid body physics)
- Defer advanced features (fluids, soft bodies) to future
- Community contributions for non-core features

### API Stability

**Risk**: Breaking changes frustrate users

**Mitigation**:
- Semantic versioning (major.minor.patch)
- Deprecation warnings before removal
- Migration guides for major versions
- Feature flags for experimental features

## Timeline Disclaimer

⚠️ **Important**: All dates and version numbers in this roadmap are **aspirational** and subject to change. This is a volunteer-driven open-source project with no guaranteed delivery dates.

**Factors Affecting Timeline**:
- Contributor availability
- Complexity of implementation
- Community feedback and changing priorities
- Ecosystem changes (Rust, WebGPU, WASM standards)
- Unforeseen technical challenges

**Commitment**: We commit to transparency about progress and realistic estimates, not to specific deadlines.

## Feedback and Discussion

**Contact**:
- GitHub Issues: Feature requests and bug reports
- GitHub Discussions: Roadmap feedback and design discussions
- Pull Requests: Contributions welcome!

**Questions to Consider**:
1. Which version features are most important to you?
2. What use cases are we missing?
3. Which GPU backend do you prefer (CUDA vs WebGPU)?
4. Interest in contributing to specific features?

## References

### GPU Computing
- Sanders, J., & Kandrot, E. (2010). *CUDA by Example*. Addison-Wesley.
- [WebGPU Specification](https://gpuweb.github.io/gpuweb/)
- [wgpu Documentation](https://wgpu.rs/)

### Physics Simulation
- Erleben, K., et al. (2005). *Physics-Based Animation*. Charles River Media.
- Bender, J., et al. (2014). "A Survey on Position-Based Simulation Methods in Computer Graphics". EG STAR.

### Spatial Data Structures
- Samet, H. (2006). *Foundations of Multidimensional and Metric Data Structures*. Morgan Kaufmann.
- Barnes, J., & Hut, P. (1986). "A hierarchical O(N log N) force-calculation algorithm". *Nature*, 324, 446-449.

### Visualization
- [Three.js Documentation](https://threejs.org/docs/)
- [Bevy Game Engine](https://bevyengine.org/)
- Marschner, S., & Shirley, P. (2021). *Fundamentals of Computer Graphics* (5th ed.). CRC Press.
