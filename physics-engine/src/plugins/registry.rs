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
//! Plugin registry and loader
//!
//! This module provides the registry for managing plugins, including:
//! - Static registration via direct API calls
//! - Dependency resolution and circular dependency detection
//! - Version compatibility checking
//! - Optional dynamic plugin discovery via environment variables
//!
//! # Environment Configuration
//!
//! Set `PHYSICS_ENGINE_PLUGIN_PATH` to enable dynamic plugin discovery:
//! ```bash
//! export PHYSICS_ENGINE_PLUGIN_PATH=/path/to/plugins:/another/path
//! ```

use crate::plugins::api::{Plugin, PLUGIN_API_VERSION};
use std::collections::{HashMap, VecDeque};

/// Plugin registry for managing and executing plugins
///
/// The registry maintains the collection of registered plugins, handles
/// dependency resolution, and ensures plugins are initialized in the correct order.
///
/// # Thread Safety
///
/// The registry is Send + Sync, allowing it to be used in multi-threaded contexts.
/// However, plugin registration and initialization should typically be done during
/// engine setup, not during simulation updates.
pub struct PluginRegistry {
    /// Registered plugins indexed by name
    plugins: HashMap<String, Box<dyn Plugin>>,
    /// Plugin initialization order (topologically sorted by dependencies)
    load_order: Vec<String>,
    /// Whether the registry has been initialized
    initialized: bool,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        PluginRegistry {
            plugins: HashMap::new(),
            load_order: Vec::new(),
            initialized: false,
        }
    }

    /// Register a plugin statically
    ///
    /// Adds the plugin to the registry. The plugin will be initialized when
    /// `initialize_all()` is called.
    ///
    /// # Arguments
    ///
    /// * `plugin` - The plugin to register
    ///
    /// # Returns
    ///
    /// Ok(()) on success, or an error if:
    /// - A plugin with the same name is already registered
    /// - The plugin API version is incompatible
    /// - The registry has already been initialized
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut registry = PluginRegistry::new();
    /// registry.register(Box::new(MyPlugin::new()))?;
    /// ```
    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<(), String> {
        if self.initialized {
            return Err("Cannot register plugins after initialization".to_string());
        }

        let name = plugin.name().to_string();

        // Check if plugin already exists
        if self.plugins.contains_key(&name) {
            return Err(format!("Plugin '{}' is already registered", name));
        }

        // Verify API version compatibility
        let plugin_api_version = plugin.api_version();
        if !is_version_compatible(plugin_api_version, PLUGIN_API_VERSION) {
            return Err(format!(
                "Plugin '{}' API version {} is incompatible with engine API version {}",
                name, plugin_api_version, PLUGIN_API_VERSION
            ));
        }

        self.plugins.insert(name, plugin);
        Ok(())
    }

    /// Discover and register plugins from environment-configured paths
    ///
    /// Reads the `PHYSICS_ENGINE_PLUGIN_PATH` environment variable and attempts
    /// to load plugins from the specified directories. Paths should be separated
    /// by colons (':') on Unix or semicolons (';') on Windows.
    ///
    /// This is a placeholder for dynamic plugin loading. Full implementation
    /// would require libloading or similar for dynamic library loading.
    ///
    /// # Returns
    ///
    /// Ok with the number of plugins discovered, or Err with error message.
    ///
    /// # Note
    ///
    /// Dynamic plugin loading is not fully implemented to avoid requiring
    /// nightly Rust or unstable features. This function currently only
    /// checks for the environment variable and provides descriptive errors.
    pub fn discover_plugins(&mut self) -> Result<usize, String> {
        if self.initialized {
            return Err("Cannot discover plugins after initialization".to_string());
        }

        match std::env::var("PHYSICS_ENGINE_PLUGIN_PATH") {
            Ok(paths) => {
                // Split paths by platform-specific separator
                let separator = if cfg!(windows) { ';' } else { ':' };
                let path_list: Vec<&str> = paths.split(separator).collect();

                eprintln!(
                    "Info: PHYSICS_ENGINE_PLUGIN_PATH found with {} path(s), but dynamic loading not implemented",
                    path_list.len()
                );
                eprintln!("Info: Use static registration via PluginRegistry::register() instead");

                // Return 0 since we don't actually load anything
                Ok(0)
            }
            Err(_) => {
                // Environment variable not set, use built-in plugins only
                Ok(0)
            }
        }
    }

    /// Initialize all registered plugins
    ///
    /// Resolves dependencies, checks for circular dependencies, determines
    /// initialization order, and initializes all plugins.
    ///
    /// # Arguments
    ///
    /// * `context` - The plugin context to pass to each plugin's initialize method
    ///
    /// # Returns
    ///
    /// Ok(()) if all plugins initialized successfully, or Err with first error encountered.
    ///
    /// # Errors
    ///
    /// - Missing dependencies
    /// - Circular dependencies detected
    /// - Plugin initialization failure
    pub fn initialize_all(
        &mut self,
        context: &crate::plugins::api::PluginContext,
    ) -> Result<(), String> {
        if self.initialized {
            return Err("Registry already initialized".to_string());
        }

        // Build dependency graph and check for missing dependencies
        let mut dependencies: HashMap<String, Vec<String>> = HashMap::new();
        for (name, plugin) in &self.plugins {
            let deps: Vec<String> = plugin
                .dependencies()
                .iter()
                .map(|s| s.to_string())
                .collect();

            // Check if all dependencies are registered
            for dep in &deps {
                if !self.plugins.contains_key(dep) {
                    return Err(format!(
                        "Plugin '{}' depends on '{}' which is not registered",
                        name, dep
                    ));
                }
            }

            dependencies.insert(name.clone(), deps);
        }

        // Topological sort to determine load order and detect cycles
        self.load_order = topological_sort(&dependencies)?;

        // Initialize plugins in dependency order
        for name in &self.load_order.clone() {
            if let Some(plugin) = self.plugins.get_mut(name) {
                plugin.initialize(context).map_err(|e| {
                    format!("Failed to initialize plugin '{}': {}", name, e)
                })?;
            }
        }

        self.initialized = true;
        Ok(())
    }

    /// Update all plugins
    ///
    /// Calls the update method on all initialized plugins in load order.
    ///
    /// # Arguments
    ///
    /// * `context` - The plugin context to pass to each plugin's update method
    ///
    /// # Returns
    ///
    /// Ok(()) if all plugins updated successfully, or Err with first error encountered.
    pub fn update_all(
        &mut self,
        context: &crate::plugins::api::PluginContext,
    ) -> Result<(), String> {
        if !self.initialized {
            return Err("Registry not initialized".to_string());
        }

        for name in &self.load_order.clone() {
            if let Some(plugin) = self.plugins.get_mut(name) {
                plugin.update(context).map_err(|e| {
                    format!("Failed to update plugin '{}': {}", name, e)
                })?;
            }
        }

        Ok(())
    }

    /// Shutdown all plugins
    ///
    /// Calls the shutdown method on all plugins in reverse load order.
    pub fn shutdown_all(&mut self) -> Result<(), String> {
        if !self.initialized {
            return Ok(()); // Nothing to shutdown
        }

        // Shutdown in reverse order
        for name in self.load_order.iter().rev() {
            if let Some(plugin) = self.plugins.get_mut(name) {
                plugin.shutdown().map_err(|e| {
                    format!("Failed to shutdown plugin '{}': {}", name, e)
                })?;
            }
        }

        self.initialized = false;
        Ok(())
    }

    /// Get a plugin by name
    ///
    /// Returns an immutable reference to the plugin if it exists.
    pub fn get(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }

    /// Get a mutable plugin by name
    ///
    /// Returns a mutable reference to the plugin if it exists.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut (dyn Plugin + '_)> {
        self.plugins.get_mut(name).map(|p| &mut **p as &mut (dyn Plugin + '_))
    }

    /// Get the number of registered plugins
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    /// Check if the registry is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get the plugin load order
    ///
    /// Returns the names of plugins in the order they were initialized.
    pub fn load_order(&self) -> &[String] {
        &self.load_order
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a plugin API version is compatible with the engine
///
/// Uses semantic versioning rules:
/// - Major version must match
/// - Minor version can be less than or equal
/// - Patch version is ignored
fn is_version_compatible(plugin_version: &str, engine_version: &str) -> bool {
    let parse_version = |v: &str| -> Option<(u32, u32, u32)> {
        let parts: Vec<&str> = v.split('.').collect();
        if parts.len() != 3 {
            return None;
        }
        Some((
            parts[0].parse().ok()?,
            parts[1].parse().ok()?,
            parts[2].parse().ok()?,
        ))
    };

    let plugin_ver = parse_version(plugin_version);
    let engine_ver = parse_version(engine_version);

    match (plugin_ver, engine_ver) {
        (Some((p_major, p_minor, _)), Some((e_major, e_minor, _))) => {
            // Major version must match
            if p_major != e_major {
                return false;
            }
            // Plugin minor version must be <= engine minor version
            p_minor <= e_minor
        }
        _ => false, // Invalid version format
    }
}

/// Perform topological sort on dependency graph
///
/// Returns the sorted list of plugin names, or an error if a cycle is detected.
fn topological_sort(dependencies: &HashMap<String, Vec<String>>) -> Result<Vec<String>, String> {
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();

    // Initialize in-degree for all nodes
    for name in dependencies.keys() {
        in_degree.entry(name.clone()).or_insert(0);
    }

    // Build adjacency list and compute in-degrees
    for (dependent, deps) in dependencies {
        for dep in deps {
            adj_list
                .entry(dep.clone())
                .or_default()
                .push(dependent.clone());
            *in_degree.entry(dependent.clone()).or_insert(0) += 1;
        }
    }

    // Find all nodes with in-degree 0
    let mut queue: VecDeque<String> = in_degree
        .iter()
        .filter(|(_, &degree)| degree == 0)
        .map(|(name, _)| name.clone())
        .collect();

    let mut sorted = Vec::new();

    while let Some(node) = queue.pop_front() {
        sorted.push(node.clone());

        // Reduce in-degree of neighbors
        if let Some(neighbors) = adj_list.get(&node) {
            for neighbor in neighbors {
                if let Some(degree) = in_degree.get_mut(neighbor) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
    }

    // If sorted list doesn't contain all nodes, there's a cycle
    if sorted.len() != dependencies.len() {
        return Err("Circular dependency detected in plugin dependencies".to_string());
    }

    Ok(sorted)
}

/// Macro for static plugin registration
///
/// Provides a convenient way to register plugins at compile time.
///
/// # Example
///
/// ```rust,ignore
/// register_plugin!(registry, MyPlugin::new());
/// ```
#[macro_export]
macro_rules! register_plugin {
    ($registry:expr, $plugin:expr) => {
        $registry
            .register(Box::new($plugin))
            .expect("Failed to register plugin")
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::api::{Plugin, PluginContext};
    use crate::ecs::World;
    use std::any::Any;

    struct TestPlugin {
        name: String,
        version: String,
        deps: Vec<String>,
        init_count: usize,
        update_count: usize,
        shutdown_count: usize,
    }

    impl TestPlugin {
        fn new(name: &str, deps: Vec<&str>) -> Self {
            TestPlugin {
                name: name.to_string(),
                version: "1.0.0".to_string(),
                deps: deps.iter().map(|s| s.to_string()).collect(),
                init_count: 0,
                update_count: 0,
                shutdown_count: 0,
            }
        }
    }

    impl Plugin for TestPlugin {
        fn name(&self) -> &str {
            &self.name
        }

        fn version(&self) -> &str {
            &self.version
        }

        fn dependencies(&self) -> Vec<&str> {
            self.deps.iter().map(|s| s.as_str()).collect()
        }

        fn initialize(&mut self, _context: &PluginContext) -> Result<(), String> {
            self.init_count += 1;
            Ok(())
        }

        fn update(&mut self, _context: &PluginContext) -> Result<(), String> {
            self.update_count += 1;
            Ok(())
        }

        fn shutdown(&mut self) -> Result<(), String> {
            self.shutdown_count += 1;
            Ok(())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    fn create_test_context() -> PluginContext<'static> {
        // Use a Box::leak to create a static reference
        let world = Box::leak(Box::new(World::new()));
        let integrator_name = Box::leak(Box::new("test".to_string()));

        #[cfg(feature = "parallel")]
        {
            PluginContext::new(world, integrator_name, 0.016, None)
        }
        #[cfg(not(feature = "parallel"))]
        {
            PluginContext::new(world, integrator_name, 0.016)
        }
    }

    #[test]
    fn test_registry_creation() {
        let registry = PluginRegistry::new();
        assert_eq!(registry.plugin_count(), 0);
        assert!(!registry.is_initialized());
    }

    #[test]
    fn test_plugin_registration() {
        let mut registry = PluginRegistry::new();
        let plugin = Box::new(TestPlugin::new("test", vec![]));

        assert!(registry.register(plugin).is_ok());
        assert_eq!(registry.plugin_count(), 1);
    }

    #[test]
    fn test_duplicate_plugin_registration() {
        let mut registry = PluginRegistry::new();
        registry
            .register(Box::new(TestPlugin::new("test", vec![])))
            .unwrap();

        let result = registry.register(Box::new(TestPlugin::new("test", vec![])));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already registered"));
    }

    #[test]
    fn test_plugin_initialization() {
        let mut registry = PluginRegistry::new();
        registry
            .register(Box::new(TestPlugin::new("test", vec![])))
            .unwrap();

        let context = create_test_context();
        assert!(registry.initialize_all(&context).is_ok());
        assert!(registry.is_initialized());
    }

    #[test]
    fn test_dependency_resolution() {
        let mut registry = PluginRegistry::new();
        registry
            .register(Box::new(TestPlugin::new("plugin_a", vec![])))
            .unwrap();
        registry
            .register(Box::new(TestPlugin::new("plugin_b", vec!["plugin_a"])))
            .unwrap();

        let context = create_test_context();
        assert!(registry.initialize_all(&context).is_ok());

        // Check that plugin_a is loaded before plugin_b
        let load_order = registry.load_order();
        let a_index = load_order.iter().position(|s| s == "plugin_a").unwrap();
        let b_index = load_order.iter().position(|s| s == "plugin_b").unwrap();
        assert!(a_index < b_index);
    }

    #[test]
    fn test_missing_dependency() {
        let mut registry = PluginRegistry::new();
        registry
            .register(Box::new(TestPlugin::new("plugin_b", vec!["plugin_a"])))
            .unwrap();

        let context = create_test_context();
        let result = registry.initialize_all(&context);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not registered"));
    }

    #[test]
    fn test_circular_dependency() {
        let mut registry = PluginRegistry::new();

        // Create circular dependency: A -> B -> C -> A
        registry
            .register(Box::new(TestPlugin::new("plugin_a", vec!["plugin_c"])))
            .unwrap();
        registry
            .register(Box::new(TestPlugin::new("plugin_b", vec!["plugin_a"])))
            .unwrap();
        registry
            .register(Box::new(TestPlugin::new("plugin_c", vec!["plugin_b"])))
            .unwrap();

        let context = create_test_context();
        let result = registry.initialize_all(&context);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Circular dependency"));
    }

    #[test]
    fn test_plugin_update() {
        let mut registry = PluginRegistry::new();
        registry
            .register(Box::new(TestPlugin::new("test", vec![])))
            .unwrap();

        let context = create_test_context();
        registry.initialize_all(&context).unwrap();
        registry.update_all(&context).unwrap();

        let plugin = registry
            .get("test")
            .unwrap()
            .as_any()
            .downcast_ref::<TestPlugin>()
            .unwrap();
        assert_eq!(plugin.update_count, 1);
    }

    #[test]
    fn test_plugin_shutdown() {
        let mut registry = PluginRegistry::new();
        registry
            .register(Box::new(TestPlugin::new("test", vec![])))
            .unwrap();

        let context = create_test_context();
        registry.initialize_all(&context).unwrap();
        registry.shutdown_all().unwrap();

        let plugin = registry
            .get("test")
            .unwrap()
            .as_any()
            .downcast_ref::<TestPlugin>()
            .unwrap();
        assert_eq!(plugin.shutdown_count, 1);
        assert!(!registry.is_initialized());
    }

    #[test]
    fn test_version_compatibility() {
        assert!(is_version_compatible("0.1.0", "0.1.0"));
        assert!(is_version_compatible("0.1.0", "0.2.0")); // Minor upgrade ok
        assert!(!is_version_compatible("0.2.0", "0.1.0")); // Plugin newer
        assert!(!is_version_compatible("1.0.0", "0.1.0")); // Major mismatch
        assert!(!is_version_compatible("invalid", "0.1.0")); // Invalid format
    }

    #[test]
    fn test_topological_sort_simple() {
        let mut deps = HashMap::new();
        deps.insert("a".to_string(), vec![]);
        deps.insert("b".to_string(), vec!["a".to_string()]);
        deps.insert("c".to_string(), vec!["b".to_string()]);

        let sorted = topological_sort(&deps).unwrap();
        assert_eq!(sorted.len(), 3);

        // a must come before b, b must come before c
        let a_idx = sorted.iter().position(|s| s == "a").unwrap();
        let b_idx = sorted.iter().position(|s| s == "b").unwrap();
        let c_idx = sorted.iter().position(|s| s == "c").unwrap();
        assert!(a_idx < b_idx);
        assert!(b_idx < c_idx);
    }

    #[test]
    fn test_topological_sort_cycle() {
        let mut deps = HashMap::new();
        deps.insert("a".to_string(), vec!["b".to_string()]);
        deps.insert("b".to_string(), vec!["a".to_string()]);

        let result = topological_sort(&deps);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Circular dependency"));
    }

    #[test]
    fn test_discover_plugins() {
        let mut registry = PluginRegistry::new();
        
        // Should not fail even if env var not set
        let result = registry.discover_plugins();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}
