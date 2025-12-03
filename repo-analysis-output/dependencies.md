# Dependency Graph

Multi-language intra-repository dependency analysis.

Supports Python, JavaScript/TypeScript, C/C++, Rust, Go, Java, C#, Swift, HTML/CSS, and SQL.

Includes classification of external dependencies as stdlib vs third-party.

## Statistics

- **Total files**: 29
- **Intra-repo dependencies**: 10
- **External stdlib dependencies**: 10
- **External third-party dependencies**: 13

## External Dependencies

### Standard Library / Core Modules

Total: 10 unique modules

- `std::any::Any`
- `std::any::TypeId`
- `std::arch::x86_64::`
- `std::collections::`
- `std::collections::BTreeMap`
- `std::collections::HashMap`
- `std::fmt`
- `std::sync::Arc`
- `std::sync::OnceLock`
- `std::time::Instant`

### Third-Party Packages

Total: 13 unique packages

- `criterion::`
- `physics_engine::ecs::`
- `physics_engine::ecs::components::`
- `physics_engine::ecs::components::Position`
- `physics_engine::ecs::systems::`
- `physics_engine::ecs::systems::ForceRegistry`
- `physics_engine::integration::`
- `physics_engine::plugins::gravity::`
- `raw_cpuid::CpuId`
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
- `physics-engine/src/integration/simd_helpers.rs` (1 dependents)
- `physics-engine/src/simd/dispatch.rs` (1 dependents)
- `physics-engine/src/simd/scalar.rs` (1 dependents)
- `physics-engine/src/simd/avx2.rs` (1 dependents)

## Files with Most Dependencies (Intra-Repo)

- `physics-engine/src/ecs/mod.rs` (4 dependencies)
- `physics-engine/src/integration/mod.rs` (3 dependencies)
- `physics-engine/src/simd/mod.rs` (3 dependencies)
