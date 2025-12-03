# Physics Engine

A high-performance Entity Component System (ECS) based physics engine with parallel execution support.

## Overview

This physics engine provides a flexible and efficient foundation for simulating physics in games and simulations. Built on modern ECS architecture principles, it emphasizes:

- **Performance**: Cache-friendly data layouts and optional parallel execution via Rayon
- **Extensibility**: Plugin system for custom components and systems
- **Safety**: Rust's type system and borrow checker prevent common bugs
- **Portability**: Runs on Linux, macOS, Windows, and experimentally on WebAssembly

## Features

- ✨ **Entity Component System**: Clean separation of data and logic
- 🎯 **Newtonian Physics**: Components for position, velocity, acceleration, and mass with double-precision
- ⚡ **Parallel Execution**: Optional multi-threaded system execution with Rayon
- 🔌 **Plugin Architecture**: Extensible system for custom objects, forces, and constraints
- 🔄 **Force Accumulation**: Generic system for applying forces without hardcoded simulation logic
- 🔢 **Advanced Integrators**: Velocity Verlet and RK4 for accurate physics simulation
- 📊 **Dense Array Storage**: Cache-friendly dense component storage for optimal memory access patterns  
- 🚀 **High Performance**: Direct array iteration enables efficient bulk operations
- ⚡ **SIMD Vectorization**: AVX2 acceleration for 2-4× speedup on modern CPUs (optional)
- 🔬 **Diagnostics**: Built-in diagnostic tools for physics validation and debugging
- 🦀 **Pure Rust**: Memory-safe implementation without runtime overhead

## Version 0.2.0 - Dense Array Storage & SIMD (Current)

This release implements cache-friendly dense array component storage and SIMD vectorization for improved performance:

### 🚀 What's New in 0.2.0

- **SIMD Vectorization** ⚡ **NEW**
  - AVX2 support for x86_64 CPUs (Haswell 2013+)
  - **Automatic runtime CPU detection** - no configuration needed
  - Process 4 × f64 values per instruction (256-bit vectors)
  - 2-4× speedup for velocity updates, position updates, and force accumulation
  - **Scalar fallback for older CPUs** - works on all x86_64 systems, no compatibility issues
  - Enable with `--features simd` flag
  - Measured throughput: 1.2-2.1 Gelem/s on AMD EPYC 7763
  - **Verify active backend**: Use `select_backend().name()` to check "AVX2" or "Scalar"
  
- **Dense Array Component Storage**: New `SoAStorage<T>` implementation (name retained for API compatibility)
  - **Important**: This is a dense Array-of-Structures (AoS), not true Structure-of-Arrays
  - Dense `Vec<T>` packing for better cache locality than HashMap
  - Direct array access for efficient bulk iteration
  - Swap-remove prevents fragmentation
  
- **Comprehensive Benchmarks**: New storage and SIMD benchmark suites
  - Separate benchmarks for via-entity and direct-array iteration
  - SIMD operation benchmarks at multiple scales (100, 1000, 10000 entities)
  - Run with `cargo bench --bench storage` or `cargo bench --features simd`
  
- **Full API Compatibility**: Dense storage implements the same `ComponentStorage` trait as HashMap
- **No Breaking Changes**: Existing code continues to work; opt-in for performance gains
- **Updated Documentation**: Architecture and performance docs explain design, trade-offs, and SIMD requirements

**When to Use SIMD**: Targeting modern x86_64 CPUs (2013+), processing many entities (>100), performance-critical simulations.

**SIMD Compatibility**: Works on all x86_64 systems. Automatically uses AVX2 on supported CPUs (Intel Haswell 2013+, AMD Excavator 2015+) and falls back to scalar code on older CPUs. No performance penalty on systems without SIMD support.

**Detecting Active Backend**: Add this to your code to verify SIMD is active:
```rust
use physics_engine::simd::select_backend;
println!("SIMD backend: {}", select_backend().name()); // "AVX2" or "Scalar"
```

**Example**: See `examples/simd_detection.rs` for a complete demonstration. Run with:
```bash
cargo run --features simd --example simd_detection
```

**When to Use Dense Storage**: Systems that can use direct array iteration, medium-large entity counts (>100), performance-critical paths.

**When to Use HashMap**: Small entity counts (<100), sparse access patterns, prototyping.

**Note**: True Structure-of-Arrays (separate field vectors) would provide additional benefits but requires different trait design. Planned for v0.3.0.

See [docs/architecture.md](docs/architecture.md) and [docs/performance.md](docs/performance.md) for details.

## Version 0.1.1 - Critical Bug Fixes and Diagnostics

This patch release fixes critical physics bugs and introduces diagnostic capabilities:

### 🔧 What's New in 0.1.1

- **Critical Bug Fixes**: Fixed force accumulation, force application, and momentum conservation bugs that were causing massive energy drift
- **Enhanced Diagnostics**: CSV output with `--diagnostics` flag for solar_system and particle_collision examples
- **Warning Controls**: Configurable force magnitude warnings in gravity plugin (`warn_on_high_forces`, `max_expected_force`)
- **Historical Documentation**: `docs/FAILURE_ANALYSIS.md` preserves the investigation that led to bug discovery (educational reference)
- **Improved Examples**: Better parameter validation and deterministic seeding for reproducibility
- **Verified Accuracy**: Energy conservation now < 0.0001% for solar system simulations

**Result**: Physics simulations now work correctly with excellent energy conservation and orbital stability. See [CHANGELOG.md](CHANGELOG.md) for detailed bug fixes and verification results.

### Version 0.1.0 - Foundation Release

Initial release completing the foundational scope (ISS-1 through ISS-5):

### ✅ ISS-1: ECS Core Architecture
- Entity management with generational indices
- Component storage traits with HashMap and SoA implementations
- System execution framework with staged scheduler
- World container for centralized entity/component management
- Parallel execution support via Rayon (optional)

### ✅ ISS-2: Newtonian Physics Systems  
- Physics components: Position, Velocity, Acceleration, Mass
- Force accumulation and provider system
- F=ma acceleration computation with safeguards
- Semi-implicit Euler integration
- Edge case handling (NaN/Inf, zero mass, overflow)

### ✅ ISS-3: Numerical Integration Methods
- Velocity Verlet integrator (symplectic, 2nd-order accurate)
- RK4 integrator (4th-order accurate)
- Unified Integrator trait for pluggable algorithms
- Timestep validation and warnings
- Energy conservation for long-term stability

### ✅ ISS-4: Plugin System and API
- Plugin trait with initialization/update/shutdown lifecycle
- Dependency resolution with topological sorting
- API versioning with semantic compatibility checks
- Plugin types: ObjectFactory, ForceProvider, ConstraintSystem
- Gravitational N-body plugin with realistic physics

### ✅ ISS-5: Documentation and Examples
- Architecture guide with ECS design and parallelization
- Integration methods guide with algorithm comparison
- Plugin guide with API reference and examples
- Performance analysis with benchmark results
- Project roadmap with GPU and visualization plans
- Changelog documenting all features
- Three working examples: basic ECS, solar system, particle collision
- Comprehensive test suite (93 unit tests + conservation tests)
- Benchmark suite comparing integrators

See [CHANGELOG.md](CHANGELOG.md) for detailed release notes.

## Quick Start

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)

### Building

```bash
# Clone the repository
git clone https://github.com/AgentFoundryExamples/physics-engine.git
cd physics-engine

# Build the library
cargo build --release

# Run tests
cargo test

# Run examples
cargo run --example basic          # Basic ECS demonstration
cargo run --example solar_system --release   # Solar system N-body simulation
cargo run --example particle_collision --release   # Particle dynamics
```

### Example: Solar System Simulation

```bash
# Simulate 1 Earth year with default settings (Velocity Verlet)
cargo run --example solar_system --release

# Compare Verlet vs RK4 integrators
cargo run --example solar_system --release -- --integrator rk4

# Simulate 10 years with hourly timesteps
cargo run --example solar_system --release -- --years 10 --timestep 3600

# Note: Unknown integrators produce clear error messages
cargo run --example solar_system --release -- --integrator unknown
# Error: Unknown integrator 'unknown'. Valid options: verlet, rk4
```

### Example: Using SoA Storage for High Performance

```rust
use physics_engine::ecs::{World, Entity, ComponentStorage, SoAStorage};
use physics_engine::ecs::components::{Position, Velocity, Mass};
use physics_engine::ecs::systems::ForceRegistry;
use physics_engine::integration::VelocityVerletIntegrator;
use physics_engine::plugins::gravity::{GravityPlugin, GravitySystem, GRAVITATIONAL_CONSTANT};

fn main() {
    // Create world and entities
    let mut world = World::new();
    
    // Use SoA storage for cache-friendly component access
    let mut positions = SoAStorage::<Position>::with_capacity(1000);
    let mut velocities = SoAStorage::<Velocity>::with_capacity(1000);
    let mut masses = SoAStorage::<Mass>::with_capacity(1000);
    
    // Create 1000 entities
    for i in 0..1000 {
        let entity = world.create_entity();
        positions.insert(entity, Position::new(i as f64, 0.0, 0.0));
        velocities.insert(entity, Velocity::zero());
        masses.insert(entity, Mass::new(1.0));
    }
    
    // Efficient bulk iteration with SoA (1.5-3× faster than HashMap for 1000+ entities)
    for pos in positions.components() {
        // Process with excellent cache locality
        println!("Position: ({}, {}, {})", pos.x(), pos.y(), pos.z());
    }
    
    // Systems can iterate over dense arrays efficiently
    let pos_array = positions.components();
    let vel_array = velocities.components();
    
    for (pos, vel) in pos_array.iter().zip(vel_array.iter()) {
        // SIMD-friendly iteration over contiguous memory
    }
}
```

### Example: Gravitational N-Body Plugin

```rust
use physics_engine::ecs::{World, Entity, ComponentStorage, HashMapStorage};
use physics_engine::ecs::components::{Position, Velocity, Mass};
use physics_engine::ecs::systems::ForceRegistry;
use physics_engine::integration::VelocityVerletIntegrator;
use physics_engine::plugins::gravity::{GravityPlugin, GravitySystem, GRAVITATIONAL_CONSTANT};

fn main() {
    // Create world and entities
    let mut world = World::new();
    let entity = world.create_entity();
    
    // Add physics components (can use HashMapStorage or SoAStorage)
    let mut positions = HashMapStorage::<Position>::new();
    positions.insert(entity, Position::new(0.0, 0.0, 0.0));
    
    let mut velocities = HashMapStorage::<Velocity>::new();
    velocities.insert(entity, Velocity::new(1000.0, 0.0, 0.0));
    
    let mut masses = HashMapStorage::<Mass>::new();
    masses.insert(entity, Mass::new(5.972e24)); // Earth mass
    
    // Create gravity plugin with realistic G
    let gravity_plugin = GravityPlugin::new(GRAVITATIONAL_CONSTANT);
    let gravity_system = GravitySystem::new(gravity_plugin);
    
    // Compute gravitational forces
    let mut force_registry = ForceRegistry::new();
    let entities = vec![entity];
    gravity_system.compute_forces(&entities, &positions, &masses, &mut force_registry);
    
    // Integrate with Verlet
    let mut integrator = VelocityVerletIntegrator::new(1.0 / 60.0);
    // ... integration loop
}
```

See [`docs/examples.md`](docs/examples.md) for detailed usage instructions and parameter tuning.

## Configuration

### Configuration

#### Numerical Integration

The engine provides multiple integrators with different accuracy/performance tradeoffs:

- **Velocity Verlet** (recommended): Symplectic integrator with excellent energy conservation
  - Best for long-running simulations, oscillatory motion, orbital mechanics
  - Second-order accurate: O(dt²)
  - ~2x force evaluations per step

- **RK4**: Fourth-order Runge-Kutta for high-precision simulation
  - Best for smooth nonlinear forces requiring high accuracy
  - Fourth-order accurate: O(dt⁴)
  - 4x force evaluations per step

```rust
use physics_engine::integration::{VelocityVerletIntegrator, RK4Integrator, Integrator};

// For long simulations with oscillatory motion
let verlet = VelocityVerletIntegrator::new(1.0 / 60.0);

// For high-precision with smooth forces
let rk4 = RK4Integrator::new(1.0 / 60.0);
```

**Timestep Selection:**
- Start with dt = 1/60 (60 FPS) or dt = 0.01
- For oscillatory systems: dt < 2/ω where ω is the angular frequency
- Too small: precision issues (dt < 1e-9 will warn)
- Too large: instability (dt > 1.0 will warn)

See [Integration Documentation](docs/integration.md) for detailed guidance.

#### Memory Pooling (New in 0.2.0)

The engine uses memory pooling to reduce allocation churn in hot paths:

```rust
use physics_engine::integration::RK4Integrator;
use physics_engine::pool::PoolConfig;
use physics_engine::ecs::World;

// RK4 with default pooling (64 capacity, 8 max size)
let integrator = RK4Integrator::new(1.0 / 60.0);

// Custom pool configuration for large simulations
let pool_config = PoolConfig::new(256, 16)
    .with_growth_factor(1.5)
    .with_logging();
let integrator = RK4Integrator::with_pool_config(1.0 / 60.0, pool_config);

// Preallocate World for known entity counts
let world = World::with_capacity(1000);  // Avoids hash table resizing

// Monitor pool performance
let (pos_stats, vel_stats, acc_stats) = integrator.pool_stats();
println!("Hit rate: {:.1}%", pos_stats.hit_rate());
```

**Benefits:**
- 10-20% reduction in allocation overhead (RK4)
- More consistent frame times
- Better performance for entity counts > 100

See [Performance Documentation](docs/performance.md#memory-pooling-v020) for tuning guidance.

#### Warning Controls (New in 0.1.1)

The gravity plugin now supports configurable warning controls for high-force scenarios:

```rust
use physics_engine::plugins::gravity::GravityPlugin;

// Create gravity plugin with custom warning thresholds
let mut gravity = GravityPlugin::new(6.67430e-11);

// Set maximum expected force (default: 1e10 N)
gravity.set_max_expected_force(1e12); // 1 trillion Newtons

// Disable high-force warnings for known extreme scenarios
gravity.set_warn_on_high_forces(false);
```

**Configuration via Code:**
- `set_max_expected_force(f64)`: Set threshold for force magnitude warnings
- `set_warn_on_high_forces(bool)`: Enable/disable warnings for forces exceeding threshold
- `set_softening(f64)`: Configure softening factor to prevent singularities (default: 1e3 m)

See [Plugin Guide](docs/plugins.md) for complete API reference.

#### Diagnostics (New in 0.1.1)

Examples now support detailed diagnostic output for physics validation:

```bash
# Solar system with CSV diagnostics
cargo run --release --example solar_system -- --diagnostics --years 1 > diagnostics.csv

# Particle collision with diagnostics
cargo run --release --example particle_collision -- --diagnostics --duration 5
```

**Diagnostic Output Includes:**
- Timestep and simulation time
- Kinetic energy, potential energy, total energy
- Energy drift percentage from initial state
- Reference body position, velocity, and acceleration
- System center of mass and spread

See [docs/DIAGNOSTICS_README.md](docs/DIAGNOSTICS_README.md) for methodology and analysis tools.

#### Feature Flags

The engine supports the following Cargo features:

- **`parallel`** (default): Enables parallel system execution via Rayon
  ```bash
  # Build without parallel support (e.g., for WASM)
  cargo build --no-default-features
  ```

- **`simd`** (optional): Enables SIMD vectorization with AVX2
  ```bash
  # Build with SIMD support for 2-4× performance improvement
  cargo build --release --features simd
  
  # Run tests with SIMD
  cargo test --features simd
  
  # Run benchmarks with SIMD
  cargo bench --features simd
  ```
  
  **SIMD Requirements:**
  - x86_64 CPU with AVX2 support (Intel Haswell 2013+, AMD Excavator 2015+)
  - **Runtime detection**: Automatically falls back to scalar on older CPUs - works everywhere!
  - No user configuration needed - backend selected automatically at startup
  - **Debugging**: Check active backend with `select_backend().name()`
  
  **SIMD Performance:**
  - Velocity updates: ~1.67 Gelem/s (1.67 billion f64 operations per second)
  - Position updates: ~1.34 Gelem/s
  - Force accumulation: ~1.95 Gelem/s
  - Expected speedup: 2-4× for large entity counts (>1000)
  
  See [docs/performance.md](docs/performance.md) for detailed SIMD benchmarks and best practices.
  # Build without parallel support (e.g., for WASM)
  cargo build --no-default-features
  ```

### Platform-Specific Notes

- **WebAssembly**: Build with `--no-default-features` as threading support varies
- **Embedded/No-Std**: Not currently supported, but planned for future versions

## Plugin System

The physics engine provides a comprehensive plugin API for extending functionality without modifying the core engine. Plugins can define custom objects, forces, and constraints.

### Plugin Types

1. **Object Factories**: Create entities with pre-configured components
2. **Force Providers**: Compute custom forces (gravity, springs, drag, etc.)
3. **Constraint Systems**: Enforce physical or geometric constraints

### Quick Example

```rust
use physics_engine::plugins::{Plugin, ForceProviderPlugin, PluginRegistry};
use physics_engine::ecs::Entity;
use physics_engine::ecs::systems::{Force, ForceRegistry, ForceProvider};
use std::any::Any;

// Define a custom gravity plugin
struct GravityPlugin {
    acceleration: f64,
}

impl Plugin for GravityPlugin {
    fn name(&self) -> &str { "gravity" }
    fn version(&self) -> &str { "1.0.0" }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl ForceProvider for GravityPlugin {
    fn compute_force(&self, _entity: Entity, _registry: &ForceRegistry) -> Option<Force> {
        Some(Force::new(0.0, self.acceleration, 0.0))
    }
    fn name(&self) -> &str { "gravity" }
}

impl ForceProviderPlugin for GravityPlugin {
    fn as_force_provider(&self) -> &dyn ForceProvider { self }
}

// Register and use the plugin
fn main() {
    let mut registry = PluginRegistry::new();
    registry.register(Box::new(GravityPlugin { acceleration: -9.81 }))
        .expect("Failed to register plugin");
    
    // Initialize and use plugins...
}
```

### Features

- ✅ **Type-safe API**: Compile-time safety guarantees
- ✅ **Dependency resolution**: Automatic plugin ordering with cycle detection
- ✅ **Version checking**: Semantic versioning compatibility validation
- ✅ **Static registration**: Zero runtime overhead
- ✅ **Thread-safe**: Safe for parallel execution
- 🔄 **Dynamic loading**: Planned for future versions

### Configuration

Set the `PHYSICS_ENGINE_PLUGIN_PATH` environment variable to configure plugin search paths:

```bash
# Copy the example configuration
cp .env.example .env

# Edit to set your plugin paths
export PHYSICS_ENGINE_PLUGIN_PATH=/path/to/plugins
```

**Note**: Dynamic plugin loading is not currently implemented. Use static registration via `PluginRegistry::register()`.

### Learn More

See the **[Plugin Guide](docs/plugins.md)** for detailed documentation, examples, and best practices.

## Documentation

Comprehensive documentation is available:

- **[Examples Guide](docs/examples.md)**: Detailed walkthroughs of solar system, particle collision, and basic ECS examples
- **[Architecture Guide](docs/architecture.md)**: Detailed design overview, ECS concepts, and parallelization strategy
- **[Integration Methods](docs/integration.md)**: Guide to numerical integrators, timestep selection, and accuracy considerations
- **[Plugin Guide](docs/plugins.md)**: Plugin system architecture, API reference, and extension examples
- **[Performance Analysis](docs/performance.md)**: Benchmark results, optimization guidelines, and performance characteristics
- **[Project Roadmap](docs/roadmap.md)**: Future plans for GPU acceleration, collision systems, and visualization
- **[Changelog](CHANGELOG.md)**: Detailed version history and release notes
- **API Documentation**: Generate with `cargo doc --open --all-features`

### Key Concepts

- **Entities**: Lightweight identifiers with generational indices
- **Components**: Pure data structures (no behavior)
  - **Position**: 3D coordinates with double-precision
  - **Velocity**: Rate of change of position
  - **Acceleration**: Rate of change of velocity (computed from forces)
  - **Mass**: Entity mass with special handling for immovable bodies
- **Systems**: Logic that operates on entities with specific components
- **Force Registry**: Accumulates forces from multiple providers for Newtonian mechanics
- **Integrators**: Numerical methods for updating motion (Verlet, RK4)
- **Scheduler**: Executes systems in deterministic stages with parallel support
- **World**: Central container managing all ECS data

## Project Structure

```
physics-engine/
├── physics-engine/       # Main library crate
│   ├── src/
│   │   ├── lib.rs        # Library root
│   │   ├── ecs/          # ECS implementation
│   │   │   ├── mod.rs         # ECS module root
│   │   │   ├── entity.rs      # Entity management
│   │   │   ├── component.rs   # Component storage
│   │   │   ├── components.rs  # Newtonian physics components
│   │   │   ├── system.rs      # System execution
│   │   │   ├── systems.rs     # Newtonian physics systems
│   │   │   ├── scheduler.rs   # Staged parallel scheduler
│   │   │   └── world.rs       # World container
│   │   ├── integration/  # Numerical integrators
│   │   │   ├── mod.rs         # Integration module root
│   │   │   ├── verlet.rs      # Velocity Verlet integrator
│   │   │   └── rk4.rs         # Runge-Kutta 4 integrator
│   │   └── plugins/      # Plugin system
│   │       ├── mod.rs         # Plugin module root
│   │       ├── api.rs         # Plugin traits and context
│   │       ├── registry.rs    # Plugin registry and loader
│   │       └── gravity.rs     # Gravitational N-body plugin
│   ├── benches/          # Performance benchmarks
│   │   └── integration.rs # Integrator benchmarks
│   └── examples/         # Example programs
│       ├── basic.rs      # Basic ECS demonstration
│       ├── solar_system.rs    # Solar system N-body simulation
│       └── particle_collision.rs  # N-body particle dynamics
├── docs/                 # Documentation
│   ├── architecture.md   # Architecture overview
│   ├── integration.md    # Integration methods guide
│   ├── plugins.md        # Plugin system guide
│   └── examples.md       # Examples usage guide
├── .env.example         # Environment configuration template
├── Cargo.toml           # Workspace configuration
└── README.md            # This file
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with all features
cargo test --all-features

# Run tests without parallel support
cargo test --no-default-features
```

### Running Benchmarks

The engine includes comprehensive benchmarks for both integrators and storage systems:

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark group
cargo bench integrator_throughput
cargo bench integrator_accuracy
cargo bench free_motion

# Run storage benchmarks
cargo bench --bench storage

# Save baseline for comparison
cargo bench -- --save-baseline my_baseline

# Compare against baseline
cargo bench -- --baseline my_baseline
```

**Benchmark Categories:**
- **Integrator Throughput**: Measures entities processed per second (10, 100, 1000 entities)
- **Integrator Accuracy**: Evaluates numerical accuracy over one oscillation period
- **Free Motion**: Baseline integration overhead with zero forces
- **Storage Performance**: Compares HashMap vs SoA storage for various operations
  - Insert, remove, random access, sequential iteration, bulk updates
  - Entity counts: 100, 1000, 10000

**Key Results** (see [Performance Analysis](docs/performance.md) for details):
- Velocity Verlet: ~2× faster than RK4 for equivalent entity counts
- SoA Storage: 1.5-3× faster sequential iteration than HashMap for 1000+ entities
- Both integrators scale well up to 1000+ entities
- RK4 provides higher accuracy (O(dt⁴)) at cost of 2× more force evaluations

**Interpreting Results:**

Criterion outputs statistical analysis including:
- **time**: [lower bound, estimate, upper bound] for benchmark duration
- **thrpt**: Throughput in entities/second
- **change**: Performance change vs previous run or baseline

For detailed methodology, hardware specifications, and optimization guidance, see [`docs/performance.md`](docs/performance.md).

### Code Quality

The project enforces:
- Compiler warnings as errors in CI
- Documentation for public APIs
- Comprehensive test coverage

### Future Roadmap

See [`docs/roadmap.md`](docs/roadmap.md) for comprehensive future plans. Highlights include:

**Version 0.2.0 - Performance & Memory** (Released 2025-12-03):
- [x] Structure-of-Arrays (SoA) component storage for 1.5-3× iteration speedup
- [x] Storage benchmarks comparing HashMap vs SoA
- [x] SIMD vectorization (AVX2) for explicit parallel computation
- [x] Memory pooling to reduce allocation overhead

**Version 0.3.0 - Spatial Acceleration & Query Improvements** (Q3-Q4 2025):
- [ ] Query DSL for ergonomic component access (deferred from v0.2.0)
- [ ] Adaptive chunk sizing for optimal parallelism (deferred from v0.2.0)
- [ ] True Structure-of-Arrays storage evolution
- [ ] Barnes-Hut tree for O(N log N) gravitational forces
- [ ] Octree spatial partitioning for collision detection
- [ ] Broad-phase collision detection (sweep-and-prune or spatial hashing)
- [ ] Profiling integration and performance analysis tools

**Version 0.4.0 - Collision & Constraints** (Q4 2025 - Q1 2026):
- [ ] Narrow-phase collision detection (sphere, box, convex polyhedra)
- [ ] Impulse-based collision response with friction
- [ ] Joint constraints (ball-and-socket, hinge, slider)
- [ ] Distance constraints and SHAKE/RATTLE algorithms

**Version 0.5.0 - GPU Acceleration** (Q1-Q2 2026):
- [ ] WebGPU compute shader integration (cross-platform)
- [ ] Optional CUDA backend for NVIDIA GPUs
- [ ] GPU buffer management and transfer optimization
- [ ] Hybrid CPU/GPU workload distribution

**Version 0.6.0 - Visualization** (Q2-Q3 2026):
- [ ] WebGPU + Three.js real-time 3D visualization
- [ ] Interactive controls for simulation parameters
- [ ] Rust-WASM bridge for browser integration
- [ ] Debug visualization tools (force vectors, energy graphs)

**Version 1.0.0 - Stable Release** (2026+):
- [ ] Stable, well-tested API with semantic versioning commitment
- [ ] Multiple backend support (CPU, GPU)
- [ ] Production-ready performance
- [ ] Long-term support plan

**Note**: All dates are aspirational and subject to change. This is a volunteer-driven open-source project with no guaranteed delivery dates. See [`docs/roadmap.md`](docs/roadmap.md) for details, risk mitigation, and dependency considerations.

## Performance

The engine is designed for high-performance simulations:

- **Data-oriented**: Component storage optimized for cache-friendly access
- **Parallel-ready**: Systems can run concurrently when independent
- **Zero-cost abstractions**: Rust's compile-time guarantees without runtime overhead

Benchmarks and profiling results will be added as the project matures.

## Troubleshooting

### Common Issues

**Build fails with "rayon not found":**
- Ensure you're building with default features: `cargo build`
- Or explicitly enable: `cargo build --features parallel`

**Tests fail on older Rust versions:**
- Update to Rust 1.70 or later: `rustup update`

**Performance issues:**
- Build in release mode: `cargo build --release`
- Enable parallel feature if not already: `cargo build --features parallel`



# Permanents (License, Contributing, Author)

Do not change any of the below sections

## License

This Agent Foundry Project is licensed under the Apache 2.0 License - see the LICENSE file for details.

## Contributing

Feel free to submit issues and enhancement requests!

## Author

Created by Agent Foundry and John Brosnihan
