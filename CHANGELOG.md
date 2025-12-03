# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2025-12-03

### Fixed

**Critical Physics Bugs**

This patch release fixes critical bugs in force computation and example implementation that were causing massive energy drift and orbital instability. The integrators themselves were correct, but the way examples used them had several critical flaws.

#### Force Provider Accumulation Bug (CRITICAL)
- **Root Cause**: `ForceRegistry.register_provider()` ADDS providers to a list rather than replacing them. When examples recomputed forces within the integration loop, providers accumulated, causing forces to multiply (2x, 3x, 4x...) on each iteration.
- **Impact**: Massive energy drift (175%+ over 1 year), exponential kinetic energy growth, orbits escaping to infinity
- **Fix**: Create fresh `ForceRegistry` instances for each force computation instead of reusing the same registry
- **Verification**: Solar system energy drift now < 0.0001% over 1 year, Earth stays at 1.000 ±0.001 AU

#### Forces Not Accumulated Bug (CRITICAL)
- **Root Cause**: Examples called `gravity_system.compute_forces()` which registers force providers, but never called `force_registry.accumulate_for_entity()` to actually accumulate the forces. As a result, `apply_forces_to_acceleration()` always got `None` for forces, producing zero accelerations.
- **Impact**: All accelerations were zero, velocities frozen, kinetic energy constant despite changing positions
- **Fix**: Explicitly call `accumulate_for_entity()` after `compute_forces()` in integration loop
- **Verification**: Velocities now update correctly, kinetic energy varies with orbital position

#### Momentum Conservation Bug
- **Root Cause**: Initial conditions placed all planets moving in same direction with Sun stationary, resulting in non-zero total system momentum
- **Impact**: Artificial drift of system center of mass, spurious energy changes
- **Fix**: Adjust all velocities to center-of-mass reference frame in solar_system example
- **Verification**: System center of mass remains stationary (< 1 m drift over simulation)

### Added

#### Gravity Plugin Enhancements
- Added configurable warning suppression for high-force scenarios
- Introduced `max_expected_force` parameter (default: 1e10 N) to control force magnitude thresholds
- Added `warn_on_high_forces` flag to disable warnings in known high-force environments
- Improved numerical stability with configurable softening factor (default: 1 km)
- Enhanced documentation of gravitational constant usage (CODATA 2018 value)

#### Diagnostic Capabilities
- Added `--diagnostics` flag to solar_system example for CSV output
- Added `--diagnostics` flag to particle_collision example
- Diagnostic output includes: timestep, kinetic energy, potential energy, total energy, drift percentage, reference body position/velocity/acceleration
- Logging frequency optimized to prevent output explosion (every 10 steps for solar system, every 50 steps for particles)
- Created `docs/DIAGNOSTICS_README.md` with methodology and usage instructions

#### Example Improvements
- Enhanced solar_system example with energy conservation tracking and warnings
- Improved particle_collision example with deterministic seeding (default: 12345)
- Added command-line parameter validation
- Clarified timestep selection guidance in documentation
- Better error messages and user feedback

### Changed

- Version bumped from 0.1.0 to 0.1.1 (patch release)
- Updated `docs/performance.md` documenting the fixed issues
- Updated `docs/examples.md` with verification results
- Improved `.env.example` documentation for warning control configurations
- Enhanced README with diagnostics and warning control information

### Documentation

- **NEW**: `docs/FAILURE_ANALYSIS.md` - Historical document preserving the investigation that led to bug discovery (before fixes were applied)
- **NEW**: `docs/DIAGNOSTICS_README.md` - Diagnostic tools usage guide and methodology documentation
- **UPDATED**: `docs/examples.md` - Added "Recent Improvements" section documenting fixed bugs and verified behavior
- **UPDATED**: `docs/performance.md` - Added "Fixed Issues" section explaining the root causes and fixes
- **UPDATED**: README.md - Added configuration section for warning controls and diagnostics

### Notes

**Important**: The FAILURE_ANALYSIS.md document is a historical record of the bugs as they existed in version 0.1.0, preserved for educational purposes and to document the investigation process. All issues described in that document have been fixed in version 0.1.1.

**Current Status**:
- ✅ Energy conservation: < 0.0001% drift in solar system over 1 Earth year
- ✅ Orbital stability: Earth stays at 1.000 ±0.001 AU
- ✅ Momentum conservation: Center of mass drift < 1 m
- ✅ Kinetic energy: Varies correctly with orbital position
- ✅ No exponential growth or runaway trajectories

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

[0.1.1]: https://github.com/AgentFoundryExamples/physics-engine/releases/tag/v0.1.1
[0.1.0]: https://github.com/AgentFoundryExamples/physics-engine/releases/tag/v0.1.0
