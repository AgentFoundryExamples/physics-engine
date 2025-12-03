// Copyright 2025 John Brosnihan
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//! # Physics Engine
//!
//! A high-performance ECS (Entity Component System) based physics engine
//! with support for parallel execution and plugin extensibility.
//!
//! ## Features
//!
//! - **ECS Architecture**: Efficient entity-component-system design
//! - **Newtonian Physics**: Components for position, velocity, acceleration, and mass
//! - **Force Accumulation**: Flexible system for applying forces from multiple sources
//! - **Parallelization**: Optional Rayon integration for multi-threaded execution
//! - **Extensibility**: Plugin system for custom components and systems
//!
//! ## Example
//!
//! ```rust
//! use physics_engine::ecs::{World, Entity};
//! use physics_engine::ecs::components::{Position, Velocity, Mass};
//!
//! let mut world = World::new();
//! let entity = world.create_entity();
//! 
//! let pos = Position::new(0.0, 0.0, 0.0);
//! let vel = Velocity::new(1.0, 0.0, 0.0);
//! let mass = Mass::new(10.0);
//! ```

#![warn(missing_docs)]

/// Entity Component System implementation
pub mod ecs;

/// Numerical integration methods
pub mod integration;

/// Plugin system for extensibility
pub mod plugins;

/// SIMD vectorization support
#[cfg(feature = "simd")]
pub mod simd;

/// Memory pooling for reducing allocation churn
pub mod pool;

pub use ecs::{World, Entity};
