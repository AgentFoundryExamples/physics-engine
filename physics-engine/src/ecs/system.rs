//! System execution framework
//!
//! Systems contain the logic that operates on entities and components.
//! This module provides traits and executors for running systems,
//! including support for parallel execution when the `parallel` feature is enabled.

use crate::ecs::World;

/// Trait for systems that operate on the ECS world
///
/// Systems should be stateless and operate purely on component data
/// for maximum parallelization potential.
pub trait System: Send + Sync {
    /// Execute the system on the world
    fn run(&mut self, world: &mut World);

    /// Get the name of this system for debugging
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

/// Executor for running systems
///
/// The executor manages system scheduling and execution order.
/// With the `parallel` feature enabled, it can run independent systems concurrently.
pub struct SystemExecutor {
    systems: Vec<Box<dyn System>>,
}

impl SystemExecutor {
    /// Create a new system executor
    pub fn new() -> Self {
        SystemExecutor {
            systems: Vec::new(),
        }
    }

    /// Add a system to the executor
    pub fn add_system<S: System + 'static>(&mut self, system: S) {
        self.systems.push(Box::new(system));
    }

    /// Run all systems sequentially
    ///
    /// TODO: Implement parallel execution when systems don't conflict
    pub fn run_sequential(&mut self, world: &mut World) {
        for system in &mut self.systems {
            system.run(world);
        }
    }

    /// Run all systems with parallelization support
    ///
    /// This method attempts to run independent systems in parallel when the
    /// `parallel` feature is enabled. Falls back to sequential execution otherwise.
    #[cfg(feature = "parallel")]
    pub fn run_parallel(&mut self, world: &mut World) {
        // TODO: Implement dependency analysis and parallel scheduling
        // For now, fall back to sequential execution
        self.run_sequential(world);
    }

    #[cfg(not(feature = "parallel"))]
    /// Run all systems (sequential fallback when parallel feature disabled)
    pub fn run_parallel(&mut self, world: &mut World) {
        self.run_sequential(world);
    }

    /// Get the number of registered systems
    pub fn system_count(&self) -> usize {
        self.systems.len()
    }
}

impl Default for SystemExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestSystem {
        run_count: usize,
    }

    impl System for TestSystem {
        fn run(&mut self, _world: &mut World) {
            self.run_count += 1;
        }

        fn name(&self) -> &str {
            "TestSystem"
        }
    }

    #[test]
    fn test_system_executor() {
        let mut executor = SystemExecutor::new();
        assert_eq!(executor.system_count(), 0);

        let system = TestSystem { run_count: 0 };
        executor.add_system(system);
        assert_eq!(executor.system_count(), 1);
    }
}
