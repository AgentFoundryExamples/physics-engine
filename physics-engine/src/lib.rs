//! # Physics Engine
//!
//! A high-performance ECS (Entity Component System) based physics engine
//! with support for parallel execution and plugin extensibility.
//!
//! ## Features
//!
//! - **ECS Architecture**: Efficient entity-component-system design
//! - **Parallelization**: Optional Rayon integration for multi-threaded execution
//! - **Extensibility**: Plugin system for custom components and systems
//!
//! ## Example
//!
//! ```rust
//! use physics_engine::ecs::{World, Entity};
//!
//! let mut world = World::new();
//! let entity = world.create_entity();
//! ```

#![warn(missing_docs)]

/// Entity Component System implementation
pub mod ecs;

pub use ecs::{World, Entity};
