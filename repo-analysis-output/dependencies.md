# Dependency Graph

Multi-language intra-repository dependency analysis.

Supports Python, JavaScript/TypeScript, C/C++, Rust, Go, Java, C#, Swift, HTML/CSS, and SQL.

Includes classification of external dependencies as stdlib vs third-party.

## Statistics

- **Total files**: 23
- **Intra-repo dependencies**: 6
- **External stdlib dependencies**: 9
- **External third-party dependencies**: 11

## External Dependencies

### Standard Library / Core Modules

Total: 9 unique modules

- `std::any::Any`
- `std::any::TypeId`
- `std::collections::`
- `std::collections::BTreeMap`
- `std::collections::HashMap`
- `std::fmt`
- `std::sync::Arc`
- `std::sync::Mutex`
- `std::time::Instant`

### Third-Party Packages

Total: 11 unique packages

- `criterion::`
- `physics_engine::ecs::`
- `physics_engine::ecs::components::`
- `physics_engine::ecs::systems::`
- `physics_engine::ecs::systems::ForceRegistry`
- `physics_engine::integration::`
- `physics_engine::plugins::gravity::`
- `rayon::ThreadPool`
- `rayon::prelude::`
- `semver::Version`
- `tests`

## Most Depended Upon Files (Intra-Repo)

- `physics-engine/src/ecs/entity.rs` (1 dependents)
- `physics-engine/src/ecs/component.rs` (1 dependents)
- `physics-engine/src/ecs/system.rs` (1 dependents)
- `physics-engine/src/ecs/world.rs` (1 dependents)
- `physics-engine/src/integration/verlet.rs` (1 dependents)
- `physics-engine/src/integration/rk4.rs` (1 dependents)

## Files with Most Dependencies (Intra-Repo)

- `physics-engine/src/ecs/mod.rs` (4 dependencies)
- `physics-engine/src/integration/mod.rs` (2 dependencies)
