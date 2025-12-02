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
- ⚡ **Parallel Execution**: Optional multi-threaded system execution with Rayon
- 🔌 **Plugin Architecture**: Extensible design for adding custom functionality
- 📊 **Cache-Friendly**: Data-oriented design for optimal performance
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

# Run the basic example
cargo run --example basic
```

### Example Usage

```rust
use physics_engine::ecs::{World, Entity, Component, ComponentStorage, HashMapStorage};

// Define a component
#[derive(Debug)]
struct Position { x: f32, y: f32, z: f32 }
impl Component for Position {}

fn main() {
    // Create a world and entities
    let mut world = World::new();
    let entity = world.create_entity();
    
    // Add components
    let mut positions = HashMapStorage::<Position>::new();
    positions.insert(entity, Position { x: 0.0, y: 0.0, z: 0.0 });
    
    // Query and update
    if let Some(pos) = positions.get_mut(entity) {
        pos.x += 1.0;
    }
}
```

## Configuration

### Feature Flags

The engine supports the following Cargo features:

- **`parallel`** (default): Enables parallel system execution via Rayon
  ```bash
  # Build without parallel support (e.g., for WASM)
  cargo build --no-default-features
  ```

### Platform-Specific Notes

- **WebAssembly**: Build with `--no-default-features` as threading support varies
- **Embedded/No-Std**: Not currently supported, but planned for future versions

## Documentation

Comprehensive documentation is available:

- **[Architecture Guide](docs/architecture.md)**: Detailed design overview, ECS concepts, and parallelization strategy
- **API Documentation**: Generate with `cargo doc --open --all-features`
- **Examples**: See the `examples/` directory for practical usage

### Key Concepts

- **Entities**: Lightweight identifiers with generational indices
- **Components**: Pure data structures (no behavior)
- **Systems**: Logic that operates on entities with specific components
- **World**: Central container managing all ECS data

## Project Structure

```
physics-engine/
├── physics-engine/       # Main library crate
│   ├── src/
│   │   ├── lib.rs       # Library root
│   │   └── ecs/         # ECS implementation
│   │       ├── mod.rs   # ECS module root
│   │       ├── entity.rs    # Entity management
│   │       ├── component.rs # Component storage
│   │       ├── system.rs    # System execution
│   │       └── world.rs     # World container
│   └── examples/        # Example programs
│       └── basic.rs     # Basic ECS demonstration
├── docs/                # Documentation
│   └── architecture.md  # Architecture overview
├── Cargo.toml          # Workspace configuration
└── README.md           # This file
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

### Code Quality

The project enforces:
- Compiler warnings as errors in CI
- Documentation for public APIs
- Comprehensive test coverage

### Future Roadmap

- [ ] Structure-of-Arrays (SoA) component storage for better cache performance
- [ ] Archetype-based entity organization
- [ ] Query DSL for ergonomic component access
- [ ] Automatic system scheduling and dependency resolution
- [ ] Physics components and systems (rigid bodies, collisions, constraints)
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
