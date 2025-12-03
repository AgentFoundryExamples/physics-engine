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
- 📊 **Cache-Friendly**: Data-oriented design with SIMD-friendly component layouts
- 🦀 **Pure Rust**: Memory-safe implementation without runtime overhead

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
# Simulate 1 Earth year with default settings
cargo run --example solar_system --release

# Compare Verlet vs RK4 integrators
cargo run --example solar_system --release -- --integrator rk4

# Simulate 10 years with hourly timesteps
cargo run --example solar_system --release -- --years 10 --timestep 3600
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
    
    // Add physics components
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

#### Feature Flags

The engine supports the following Cargo features:

- **`parallel`** (default): Enables parallel system execution via Rayon
  ```bash
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

# Run benchmarks
cargo bench
```

### Code Quality

The project enforces:
- Compiler warnings as errors in CI
- Documentation for public APIs
- Comprehensive test coverage

### Future Roadmap

- [ ] Archetype-based entity organization
- [ ] Query DSL for ergonomic component access
- [ ] Automatic system scheduling and dependency resolution
- [x] Advanced integrators (Verlet, RK4) for better accuracy
- [ ] Adaptive timestepping for automatic dt adjustment
- [ ] Collision detection and response systems
- [ ] Constraint solvers for joints and contacts
- [ ] Integration examples with graphics libraries

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
