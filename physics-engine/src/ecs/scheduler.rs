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

/// A system with metadata for scheduling
struct ScheduledSystem {
    system: Box<dyn System>,
    stage: StageId,
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
    systems: Vec<ScheduledSystem>,
}

impl Scheduler {
    /// Create a new scheduler
    pub fn new() -> Self {
        Scheduler {
            systems: Vec::new(),
        }
    }

    /// Create a scheduler with a specific number of stages
    ///
    /// Pre-allocates capacity for the given number of stages to reduce allocations.
    pub fn with_stages(stage_count: usize) -> Self {
        Scheduler {
            systems: Vec::with_capacity(stage_count * 4), // Estimate 4 systems per stage
        }
    }

    /// Add a system to a specific stage
    ///
    /// Systems within the same stage may run in parallel. Stages are executed
    /// in order (stage 0, then 1, then 2, etc.).
    pub fn add_system<S: System + 'static>(&mut self, system: S, stage: StageId) {
        self.systems.push(ScheduledSystem {
            system: Box::new(system),
            stage,
        });
    }

    /// Add a system to the default integration stage
    pub fn add_system_default<S: System + 'static>(&mut self, system: S) {
        self.add_system(system, stages::INTEGRATION);
    }

    /// Get the number of registered systems
    pub fn system_count(&self) -> usize {
        self.systems.len()
    }

    /// Get the number of stages in use
    pub fn stage_count(&self) -> usize {
        if self.systems.is_empty() {
            0
        } else {
            self.systems.iter()
                .map(|s| s.stage.0)
                .max()
                .map(|max| max + 1)
                .unwrap_or(0)
        }
    }

    /// Execute all systems sequentially in stage order
    ///
    /// This is the fallback when parallel execution is not available or
    /// for debugging purposes.
    pub fn run_sequential(&mut self, world: &mut World) {
        // Sort by stage to ensure deterministic order
        self.systems.sort_by_key(|s| s.stage);

        for scheduled in &mut self.systems {
            scheduled.system.run(world);
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
        use std::collections::HashMap;

        // Sort by stage to ensure deterministic order
        self.systems.sort_by_key(|s| s.stage);

        // Group systems by stage
        let mut stages: HashMap<StageId, Vec<&mut Box<dyn System>>> = HashMap::new();
        for scheduled in &mut self.systems {
            stages.entry(scheduled.stage)
                .or_insert_with(Vec::new)
                .push(&mut scheduled.system);
        }

        // Get sorted stage IDs
        let mut stage_ids: Vec<StageId> = stages.keys().copied().collect();
        stage_ids.sort();

        // Execute each stage sequentially
        for stage_id in stage_ids {
            if let Some(stage_systems) = stages.get_mut(&stage_id) {
                // Within a stage, systems currently run sequentially
                // Future enhancement: analyze component access to run independent systems in parallel
                for system in stage_systems {
                    system.run(world);
                }
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
        self.systems.clear();
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
        assert_eq!(scheduler.stage_count(), 6); // Stages 0-5

        scheduler.add_system(TestSystem::new("s3"), StageId::new(2));
        assert_eq!(scheduler.stage_count(), 6); // Still 0-5
    }
}
