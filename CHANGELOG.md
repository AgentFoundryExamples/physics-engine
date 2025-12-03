# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-12-03

### Added - Core Physics Engine Foundation

#### Entity Component System (ECS) Architecture
- **Entity Management** (`ecs/entity.rs`)
  - Generational entity IDs with safe reference handling
  - O(1) entity creation and destruction
  - Entity validity checking to prevent use-after-free
  
- **Component System** (`ecs/component.rs`, `ecs/components.rs`)
  - `Component` trait for type-safe component storage
  - `ComponentStorage` trait with `HashMapStorage` implementation
  - Newtonian physics components:
    - `Position`: Double-precision 3D coordinates
    - `Velocity`: Rate of change of position
    - `Acceleration`: Rate of change of velocity (computed from forces)
    - `Mass`: Entity mass with special handling for immovable bodies
  - SIMD-friendly 8-byte aligned data layouts
  - Validation helpers for detecting NaN/Inf values
  - Array conversion utilities for bulk operations

- **System Execution** (`ecs/system.rs`, `ecs/systems.rs`)
  - `System` trait for logic implementation
  - `SystemExecutor` for managing system execution order
  - Physics systems:
    - `ForceRegistry`: Accumulates forces from multiple providers
    - `ForceProvider` trait: Plugin interface for custom force generators
    - `apply_forces_to_acceleration()`: Applies F=ma with safeguards
    - `integrate_motion()`: Semi-implicit Euler integration
  - Overflow detection and NaN/Inf validation
  - Graceful handling of missing components and immovable bodies

- **Staged Scheduler** (`ecs/scheduler.rs`)
  - Deterministic stage-based execution (5 stages)
  - Stages: force accumulation, acceleration, integration, constraints, post-process
  - Parallel execution support via Rayon (optional)
  - Stage barriers for sequential ordering with intra-stage parallelism
  - Configurable sequential fallback for debugging

- **World Container** (`ecs/world.rs`)
  - Central ECS data container
  - Entity lifecycle management
  - Foundation for query interface

#### Numerical Integration Methods

- **Velocity Verlet Integrator** (`integration/verlet.rs`)
  - Symplectic integrator with excellent energy conservation
  - Second-order accurate: O(dt²) global error
  - Time-reversible algorithm
  - ~2 force evaluations per step
  - Configurable timestep with validation (warns if dt < 1e-9 or dt > 1.0)
  - Best for long-running simulations and oscillatory motion

- **RK4 Integrator** (`integration/rk4.rs`)
  - Fourth-order Runge-Kutta explicit integrator
  - Fourth-order accurate: O(dt⁴) global error
  - 4 force evaluations per step with intermediate stages
  - Internal buffer reuse to minimize allocations
  - Configurable timestep with validation
  - Best for high-precision simulations with smooth forces

- **Integrator Trait** (`integration/mod.rs`)
  - Unified interface for all integration methods
  - Timestep validation and warnings
  - Pluggable integrator system for easy algorithm swapping

#### Plugin System

- **Plugin API** (`plugins/api.rs`)
  - `Plugin` trait: Base interface for all plugins
  - `PluginContext`: Scoped access to engine state
  - `ObjectFactory`: Create entities with pre-configured components
  - `ForceProviderPlugin`: Compute custom forces
  - `ConstraintSystem`: Enforce geometric or physical constraints
  - API versioning with semantic versioning compatibility checks
  - Type-safe downcasting for plugin-specific functionality

- **Plugin Registry** (`plugins/registry.rs`)
  - Plugin registration and lifecycle management
  - Dependency resolution with topological sorting
  - Circular dependency detection
  - Initialization/update/shutdown hooks
  - Environment-based plugin discovery (informational, no dynamic loading yet)
  - Thread-safe plugin management

- **Gravitational N-Body Plugin** (`plugins/gravity.rs`)
  - Newton's law of universal gravitation implementation
  - Realistic gravitational constant (G = 6.67430 × 10⁻¹¹ m³/(kg⋅s²))
  - Configurable softening factor to prevent singularities
  - O(N²) pairwise force calculation with parallel execution
  - Handles zero distance, immovable bodies, and edge cases
  - Configurable G constant and chunk sizes for performance tuning
  - Comprehensive validation and test suite

#### Examples

- **Basic ECS Example** (`examples/basic.rs`)
  - Demonstrates entity creation and component management
  - Shows sequential vs parallel system execution
  - Educational introduction to ECS concepts

- **Solar System Simulation** (`examples/solar_system.rs`)
  - Realistic N-body simulation with Sun, Mercury, Venus, Earth, Mars
  - Accurate planetary masses, distances, and orbital velocities
  - Real-time energy conservation tracking
  - Configurable integrator selection (Verlet or RK4)
  - Adjustable timestep and simulation duration
  - Demonstrates long-term orbital stability
  - Command-line interface for parameter tuning

- **Particle Collision Simulation** (`examples/particle_collision.rs`)
  - N-body gravitational dynamics with configurable particle count
  - Deterministic seeding for reproducible results
  - Performance scaling demonstration (O(N²) complexity)
  - Random initial conditions within bounded volume
  - Energy conservation monitoring
  - Configurable integrator, timestep, and duration
  - Command-line interface for experimentation

#### Testing & Validation

- **Unit Tests** (93 tests passing)
  - Component creation and validation
  - Entity management and generational indices
  - System execution and scheduling
  - Force accumulation and provider system
  - Integration accuracy and stability
  - Plugin registration and dependency resolution
  - Gravitational force calculations with reference values

- **Conservation Tests** (`tests/conservation.rs`)
  - Energy conservation for free particles
  - Position accuracy vs analytical solutions
  - Constant acceleration tests (free fall)
  - Multi-entity interaction validation
  - Tests for both Verlet and RK4 integrators

- **Benchmarks** (`benches/integration.rs`)
  - Integrator throughput comparison (entities/second)
  - Accuracy benchmarks over one oscillation period
  - Free motion baseline for overhead measurement
  - Statistical analysis via Criterion.rs
  - Configurable entity counts (10, 100, 1000)

#### Documentation

- **Architecture Guide** (`docs/architecture.md`)
  - ECS design philosophy and principles
  - Entity, component, and system architecture
  - Component memory layout and cache considerations
  - Staged scheduler design with parallelization strategy
  - Newtonian mechanics framework
  - Force accumulation system
  - Plugin system architecture
  - Parallelization with Rayon
  - Performance considerations
  - Edge case handling

- **Integration Methods Guide** (`docs/integration.md`)
  - Velocity Verlet algorithm and properties
  - RK4 algorithm and properties
  - Choosing an integrator (decision guide)
  - Timestep selection guidelines and stability criteria
  - Usage examples and best practices
  - Integration with scheduler
  - Performance considerations
  - Common pitfalls and troubleshooting
  - Testing and validation approaches
  - Academic references

- **Plugin System Guide** (`docs/plugins.md`)
  - Plugin architecture and lifecycle
  - Plugin types: ObjectFactory, ForceProvider, ConstraintSystem
  - Static registration and discovery
  - Dependency management and resolution
  - Version compatibility rules
  - Plugin context and safety boundaries
  - Complete examples for each plugin type
  - Built-in gravitational N-body plugin documentation
  - Performance tips and best practices
  - Environment configuration
  - Troubleshooting common issues

- **Examples Guide** (`docs/examples.md`)
  - Detailed instructions for all examples
  - Command-line options and usage
  - Expected behavior and output interpretation
  - Performance characteristics
  - Timestep selection guidance
  - Energy conservation monitoring
  - Profiling and benchmarking instructions
  - Troubleshooting common issues
  - Extension and customization examples

- **Performance Analysis** (`docs/performance.md`)
  - Benchmark methodology and test environment
  - Hardware specifications and software configuration
  - Integrator throughput comparison results
  - Accuracy vs performance trade-offs
  - Memory overhead analysis
  - Parallel scaling efficiency
  - Performance by use case (small/medium/large systems)
  - Optimization guidelines
  - Known performance issues and mitigations
  - Platform-specific notes
  - Future performance enhancements

- **Project Roadmap** (`docs/roadmap.md`)
  - Project vision and core values
  - Version history (0.1.0 completed features)
  - Planned features by version (0.2.0 through 1.0.0)
  - GPU acceleration strategy (CUDA vs WebGPU)
  - Spatial acceleration structures (Barnes-Hut, octrees)
  - Collision detection and constraint systems
  - WebGPU + Three.js visualization integration plan
  - Advanced features exploration
  - Technology dependencies
  - Non-goals and scope boundaries
  - Risk mitigation strategies
  - Timeline disclaimer and contributing guidelines

- **API Documentation**
  - Comprehensive rustdoc for all public APIs
  - Code examples in documentation
  - Generate with `cargo doc --open --all-features`

#### Project Infrastructure

- **Build System**
  - Cargo workspace configuration
  - Release profile optimizations
  - Feature flags: `parallel` (default, enables Rayon)
  - MSRV: Rust 1.70+ (2021 edition)

- **Dependencies** (pinned versions)
  - `rayon = "1.10.0"`: Parallel execution framework (optional)
  - `semver = "1.0.23"`: Version compatibility checking
  - `criterion = "0.5.1"`: Benchmarking framework (dev)

- **Platform Support**
  - Primary: Linux (x86_64, aarch64)
  - Secondary: macOS, Windows
  - Experimental: WebAssembly (with `--no-default-features`)

### Fixed

- Proper handling of zero mass entities (treated as immovable)
- NaN/Inf detection and rejection at component and force levels
- Force magnitude overflow protection with configurable limits
- Numerical precision warnings for very small timesteps (dt < 1e-9)

### Security

- No unsafe code in public API
- Compile-time borrow checking prevents data races
- Validation for all numeric inputs (NaN/Inf checks)
- Bounded force magnitudes to prevent numerical overflow
- Generational entity indices prevent use-after-free

## Release Scope

This release (v0.1.0) completes the initial foundation scoped in issues ISS-1 through ISS-5:
- **ISS-1**: ECS Core Architecture
- **ISS-2**: Newtonian Physics Systems
- **ISS-3**: Numerical Integration Methods
- **ISS-4**: Plugin System and API
- **ISS-5**: Documentation and Examples

## Migration Guide

N/A - Initial release

## Future Releases

See [`docs/roadmap.md`](docs/roadmap.md) for planned features:
- **v0.2.0**: Structure-of-Arrays (SoA) storage, SIMD vectorization, memory pooling
- **v0.3.0**: Barnes-Hut tree, octree spatial partitioning, broad-phase collision
- **v0.4.0**: Collision detection, impulse-based response, joint constraints
- **v0.5.0**: GPU acceleration (WebGPU or CUDA)
- **v0.6.0**: WebGPU + Three.js visualization
- **v1.0.0**: Stable release

## Versioning Policy

This project follows [Semantic Versioning](https://semver.org/):
- **MAJOR** version: Incompatible API changes
- **MINOR** version: Backward-compatible functionality additions
- **PATCH** version: Backward-compatible bug fixes

## Contributing

See [README.md](README.md) for contribution guidelines.

## License

Apache 2.0 - See [LICENSE](LICENSE) for details.

## Authors

Created by Agent Foundry and John Brosnihan

[0.1.0]: https://github.com/AgentFoundryExamples/physics-engine/releases/tag/v0.1.0
