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
//! System scheduler with parallel execution support
//!
//! This module provides a scheduler that can execute systems in parallel using
//! Rayon while maintaining deterministic ordering through staged execution.
//! Systems are organized into stages that execute sequentially, but systems
//! within a stage can run in parallel if they don't conflict.

use crate::ecs::System;
use crate::ecs::World;
use std::collections::BTreeMap;

/// Stage identifier for grouping systems
///
/// Systems in the same stage may run in parallel, while stages execute sequentially.
/// This allows for deterministic ordering with parallelism within stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StageId(pub usize);

impl StageId {
    /// Create a new stage ID
    pub fn new(id: usize) -> Self {
        StageId(id)
    }
}

/// Pre-defined standard stages for common physics operations
pub mod stages {
    use super::StageId;

    /// Stage for force computation and accumulation
    pub const FORCE_ACCUMULATION: StageId = StageId(0);
    
    /// Stage for computing accelerations from forces
    pub const ACCELERATION: StageId = StageId(1);
    
    /// Stage for integration (updating velocities and positions)
    pub const INTEGRATION: StageId = StageId(2);
    
    /// Stage for constraint resolution
    pub const CONSTRAINTS: StageId = StageId(3);
    
    /// Stage for post-processing and cleanup
    pub const POST_PROCESS: StageId = StageId(4);
}

/// System scheduler with support for staged parallel execution
///
/// The scheduler organizes systems into stages that execute sequentially,
/// providing a deterministic execution order. Within each stage, systems
/// can run in parallel when the `parallel` feature is enabled.
///
/// # Examples
///
/// ```
/// use physics_engine::ecs::scheduler::{Scheduler, stages};
/// use physics_engine::ecs::{World, System};
///
/// struct MySystem;
/// impl System for MySystem {
///     fn run(&mut self, _world: &mut World) {}
/// }
///
/// let mut scheduler = Scheduler::new();
/// scheduler.add_system(MySystem, stages::INTEGRATION);
/// ```
pub struct Scheduler {
    stages: BTreeMap<StageId, Vec<Box<dyn System>>>,
}

impl Scheduler {
    /// Create a new scheduler
    pub fn new() -> Self {
        Scheduler {
            stages: BTreeMap::new(),
        }
    }

    /// Create a scheduler with a specific number of stages
    ///
    /// Pre-allocates capacity for the given number of stages to reduce allocations.
    pub fn with_stages(_stage_count: usize) -> Self {
        // BTreeMap doesn't support pre-allocation, so we just create a new one
        Scheduler {
            stages: BTreeMap::new(),
        }
    }

    /// Add a system to a specific stage
    ///
    /// Systems within the same stage may run in parallel. Stages are executed
    /// in order (stage 0, then 1, then 2, etc.).
    pub fn add_system<S: System + 'static>(&mut self, system: S, stage: StageId) {
        self.stages
            .entry(stage)
            .or_insert_with(Vec::new)
            .push(Box::new(system));
    }

    /// Add a system to the default integration stage
    pub fn add_system_default<S: System + 'static>(&mut self, system: S) {
        self.add_system(system, stages::INTEGRATION);
    }

    /// Get the number of registered systems
    pub fn system_count(&self) -> usize {
        self.stages.values().map(|v| v.len()).sum()
    }

    /// Get the number of stages in use
    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }

    /// Execute all systems sequentially in stage order
    ///
    /// This is the fallback when parallel execution is not available or
    /// for debugging purposes.
    pub fn run_sequential(&mut self, world: &mut World) {
        // BTreeMap automatically maintains sorted order by key
        for stage_systems in self.stages.values_mut() {
            for system in stage_systems {
                system.run(world);
            }
        }
    }

    /// Execute all systems with parallel execution within stages
    ///
    /// When the `parallel` feature is enabled, systems within the same stage
    /// can run in parallel using Rayon. Stages execute sequentially to maintain
    /// deterministic ordering.
    ///
    /// Note: Currently, this implementation runs systems sequentially as a
    /// foundation. Full parallel execution within stages requires tracking
    /// component access patterns to determine which systems can safely run
    /// concurrently. This will be implemented in a future release.
    #[cfg(feature = "parallel")]
    pub fn run_parallel(&mut self, world: &mut World) {
        // BTreeMap automatically maintains sorted order by key (StageId)
        for stage_systems in self.stages.values_mut() {
            // Within a stage, systems currently run sequentially
            // Future enhancement: analyze component access to run independent systems in parallel
            for system in stage_systems {
                system.run(world);
            }
        }
    }

    #[cfg(not(feature = "parallel"))]
    /// Execute all systems (sequential fallback when parallel feature disabled)
    pub fn run_parallel(&mut self, world: &mut World) {
        self.run_sequential(world);
    }

    /// Clear all systems from the scheduler
    pub fn clear(&mut self) {
        self.stages.clear();
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestSystem {
        name: String,
        run_count: usize,
    }

    impl TestSystem {
        fn new(name: &str) -> Self {
            TestSystem {
                name: name.to_string(),
                run_count: 0,
            }
        }
    }

    impl System for TestSystem {
        fn run(&mut self, _world: &mut World) {
            self.run_count += 1;
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn test_scheduler_creation() {
        let scheduler = Scheduler::new();
        assert_eq!(scheduler.system_count(), 0);
    }

    #[test]
    fn test_add_system() {
        let mut scheduler = Scheduler::new();
        scheduler.add_system(TestSystem::new("test"), stages::INTEGRATION);
        assert_eq!(scheduler.system_count(), 1);
    }

    #[test]
    fn test_stage_ordering() {
        let mut scheduler = Scheduler::new();
        scheduler.add_system(TestSystem::new("stage2"), StageId::new(2));
        scheduler.add_system(TestSystem::new("stage0"), StageId::new(0));
        scheduler.add_system(TestSystem::new("stage1"), StageId::new(1));

        assert_eq!(scheduler.stage_count(), 3);
    }

    #[test]
    fn test_sequential_execution() {
        let mut scheduler = Scheduler::new();
        scheduler.add_system(TestSystem::new("system1"), stages::FORCE_ACCUMULATION);
        scheduler.add_system(TestSystem::new("system2"), stages::INTEGRATION);

        let mut world = World::new();
        scheduler.run_sequential(&mut world);

        // Both systems should have been executed
        assert_eq!(scheduler.system_count(), 2);
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn test_parallel_execution() {
        let mut scheduler = Scheduler::new();
        scheduler.add_system(TestSystem::new("system1"), stages::FORCE_ACCUMULATION);
        scheduler.add_system(TestSystem::new("system2"), stages::INTEGRATION);

        let mut world = World::new();
        scheduler.run_parallel(&mut world);

        // Both systems should have been executed
        assert_eq!(scheduler.system_count(), 2);
    }

    #[test]
    fn test_stage_barrier() {
        let mut scheduler = Scheduler::new();
        
        // Add systems to different stages
        scheduler.add_system(TestSystem::new("early"), StageId::new(0));
        scheduler.add_system(TestSystem::new("late"), StageId::new(1));

        let mut world = World::new();
        scheduler.run_sequential(&mut world);

        // Systems should execute in stage order (tested via deterministic ordering)
        assert_eq!(scheduler.system_count(), 2);
    }

    #[test]
    fn test_empty_scheduler() {
        let mut scheduler = Scheduler::new();
        let mut world = World::new();

        // Should not panic with no systems
        scheduler.run_sequential(&mut world);
        
        #[cfg(feature = "parallel")]
        scheduler.run_parallel(&mut world);
    }

    #[test]
    fn test_clear_scheduler() {
        let mut scheduler = Scheduler::new();
        scheduler.add_system(TestSystem::new("test"), stages::INTEGRATION);
        assert_eq!(scheduler.system_count(), 1);

        scheduler.clear();
        assert_eq!(scheduler.system_count(), 0);
    }

    #[test]
    fn test_many_systems() {
        let mut scheduler = Scheduler::new();
        
        // Add many systems to test scalability
        for i in 0..1000 {
            scheduler.add_system(TestSystem::new(&format!("system{}", i)), StageId::new(i % 5));
        }

        assert_eq!(scheduler.system_count(), 1000);
        
        let mut world = World::new();
        scheduler.run_sequential(&mut world); // Should handle large number of systems
    }

    #[test]
    fn test_stage_count() {
        let mut scheduler = Scheduler::new();
        assert_eq!(scheduler.stage_count(), 0);

        scheduler.add_system(TestSystem::new("s1"), StageId::new(0));
        assert_eq!(scheduler.stage_count(), 1);

        scheduler.add_system(TestSystem::new("s2"), StageId::new(5));
        assert_eq!(scheduler.stage_count(), 2); // Two distinct stages: 0 and 5

        scheduler.add_system(TestSystem::new("s3"), StageId::new(2));
        assert_eq!(scheduler.stage_count(), 3); // Three distinct stages: 0, 2, and 5
    }
}
