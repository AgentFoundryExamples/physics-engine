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
//! Memory pooling for reducing allocation churn
//!
//! This module provides thread-safe buffer pools for reusing temporary
//! allocations in integrators and force computation. Pools help reduce
//! per-frame allocation overhead and improve cache locality.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Configuration for buffer pool behavior
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Initial capacity for each buffer in the pool
    pub initial_capacity: usize,
    /// Maximum number of buffers to keep in the pool
    pub max_pool_size: usize,
    /// Growth factor when allocating new capacity (e.g., 2.0 for doubling)
    pub growth_factor: f64,
    /// Whether to log when the pool grows or shrinks
    pub log_resize_events: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        PoolConfig {
            initial_capacity: 64,
            max_pool_size: 8,
            growth_factor: 2.0,
            log_resize_events: false,
        }
    }
}

impl PoolConfig {
    /// Create a new pool configuration with custom settings
    pub fn new(initial_capacity: usize, max_pool_size: usize) -> Self {
        PoolConfig {
            initial_capacity,
            max_pool_size,
            growth_factor: 2.0,
            log_resize_events: false,
        }
    }

    /// Enable logging for resize events
    pub fn with_logging(mut self) -> Self {
        self.log_resize_events = true;
        self
    }

    /// Set the growth factor for buffer capacity expansion
    pub fn with_growth_factor(mut self, factor: f64) -> Self {
        assert!(factor >= 1.0, "Growth factor must be >= 1.0");
        self.growth_factor = factor;
        self
    }
}

/// Statistics for monitoring pool performance
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// Number of times a buffer was successfully borrowed from the pool
    pub hits: usize,
    /// Number of times a new buffer had to be allocated
    pub misses: usize,
    /// Number of times the pool was resized
    pub resize_count: usize,
    /// Current number of buffers in the pool
    pub pool_size: usize,
    /// Peak number of buffers ever allocated
    pub peak_size: usize,
}

impl PoolStats {
    /// Calculate the hit rate as a percentage
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }
}

/// A thread-safe pool for HashMap buffers
///
/// This pool manages reusable HashMaps to reduce allocation overhead
/// in hot paths like integrator intermediate steps and force accumulation.
pub struct HashMapPool<K, V> {
    pool: Arc<Mutex<Vec<HashMap<K, V>>>>,
    config: PoolConfig,
    stats: Arc<Mutex<PoolStats>>,
}

impl<K, V> HashMapPool<K, V>
where
    K: std::cmp::Eq + std::hash::Hash,
{
    /// Create a new HashMap pool with default configuration
    pub fn new() -> Self {
        Self::with_config(PoolConfig::default())
    }

    /// Create a new HashMap pool with custom configuration
    pub fn with_config(config: PoolConfig) -> Self {
        HashMapPool {
            pool: Arc::new(Mutex::new(Vec::new())),
            config,
            stats: Arc::new(Mutex::new(PoolStats::default())),
        }
    }

    /// Acquire a buffer from the pool
    ///
    /// If the pool is empty, allocates a new buffer. The buffer is
    /// automatically returned to the pool when the guard is dropped.
    pub fn acquire(&self) -> HashMapGuard<K, V> {
        // LOCK ORDERING: Acquire pool lock, get buffer, release lock, then update stats
        let (buffer, was_hit, pool_len) = {
            let mut pool = self.pool.lock().unwrap();
            let was_hit = !pool.is_empty();
            let buf = if let Some(mut b) = pool.pop() {
                b.clear();
                b
            } else {
                HashMap::with_capacity(self.config.initial_capacity)
            };
            let len = pool.len();
            (buf, was_hit, len)
        }; // pool lock released here
        
        // Update stats with separate lock (no overlap with pool lock)
        {
            let mut stats = self.stats.lock().unwrap();
            if was_hit {
                stats.hits += 1;
            } else {
                stats.misses += 1;
                if self.config.log_resize_events {
                    eprintln!("HashMapPool: Allocating new buffer (hit rate: {:.1}%)", stats.hit_rate());
                }
            }
            stats.pool_size = pool_len;
        } // stats lock released here

        HashMapGuard {
            buffer: Some(buffer),
            pool: Arc::clone(&self.pool),
            stats: Arc::clone(&self.stats),
            max_pool_size: self.config.max_pool_size,
        }
    }

    /// Get current pool statistics
    pub fn stats(&self) -> PoolStats {
        self.stats.lock().unwrap().clone()
    }

    /// Clear all buffers from the pool (useful for shutdown)
    pub fn clear(&self) {
        // LOCK ORDERING: Acquire pool lock, clear, release, then update stats
        {
            let mut pool = self.pool.lock().unwrap();
            pool.clear();
        } // pool lock released here
        
        {
            let mut stats = self.stats.lock().unwrap();
            stats.pool_size = 0;
        } // stats lock released here
    }

    /// Get the current number of buffers in the pool
    pub fn len(&self) -> usize {
        self.pool.lock().unwrap().len()
    }

    /// Check if the pool is empty
    pub fn is_empty(&self) -> bool {
        self.pool.lock().unwrap().is_empty()
    }
}

impl<K, V> Default for HashMapPool<K, V>
where
    K: std::cmp::Eq + std::hash::Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> Clone for HashMapPool<K, V>
where
    K: std::cmp::Eq + std::hash::Hash,
{
    fn clone(&self) -> Self {
        HashMapPool {
            pool: Arc::clone(&self.pool),
            config: self.config.clone(),
            stats: Arc::clone(&self.stats),
        }
    }
}

/// RAII guard for a pooled HashMap
///
/// When dropped, returns the buffer to the pool for reuse.
pub struct HashMapGuard<K, V> {
    buffer: Option<HashMap<K, V>>,
    pool: Arc<Mutex<Vec<HashMap<K, V>>>>,
    stats: Arc<Mutex<PoolStats>>,
    max_pool_size: usize,
}

impl<K, V> HashMapGuard<K, V> {
    /// Get a reference to the underlying HashMap
    pub fn as_hashmap(&self) -> &HashMap<K, V> {
        self.buffer.as_ref().unwrap()
    }

    /// Get a mutable reference to the underlying HashMap
    pub fn as_hashmap_mut(&mut self) -> &mut HashMap<K, V> {
        self.buffer.as_mut().unwrap()
    }
}

impl<K, V> std::ops::Deref for HashMapGuard<K, V> {
    type Target = HashMap<K, V>;

    fn deref(&self) -> &Self::Target {
        self.buffer.as_ref().unwrap()
    }
}

impl<K, V> std::ops::DerefMut for HashMapGuard<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buffer.as_mut().unwrap()
    }
}

impl<K, V> Drop for HashMapGuard<K, V> {
    fn drop(&mut self) {
        if let Some(buffer) = self.buffer.take() {
            let mut pool = self.pool.lock().unwrap();
            if pool.len() < self.max_pool_size {
                pool.push(buffer);
                
                let mut stats = self.stats.lock().unwrap();
                stats.pool_size = pool.len();
                if stats.pool_size > stats.peak_size {
                    stats.peak_size = stats.pool_size;
                }
            }
            // If pool is full, buffer is dropped (deallocated)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::Entity;

    #[test]
    fn test_pool_config_defaults() {
        let config = PoolConfig::default();
        assert_eq!(config.initial_capacity, 64);
        assert_eq!(config.max_pool_size, 8);
        assert_eq!(config.growth_factor, 2.0);
        assert!(!config.log_resize_events);
    }

    #[test]
    fn test_pool_config_custom() {
        let config = PoolConfig::new(128, 16)
            .with_growth_factor(1.5)
            .with_logging();
        
        assert_eq!(config.initial_capacity, 128);
        assert_eq!(config.max_pool_size, 16);
        assert_eq!(config.growth_factor, 1.5);
        assert!(config.log_resize_events);
    }

    #[test]
    fn test_hashmap_pool_creation() {
        let pool: HashMapPool<Entity, i32> = HashMapPool::new();
        assert_eq!(pool.len(), 0);
        assert!(pool.is_empty());
    }

    #[test]
    fn test_hashmap_pool_acquire_and_return() {
        let pool: HashMapPool<Entity, i32> = HashMapPool::new();
        
        {
            let mut guard = pool.acquire();
            guard.insert(Entity::new(1, 0), 42);
            assert_eq!(guard.len(), 1);
        } // Guard dropped, buffer returned to pool
        
        assert_eq!(pool.len(), 1);
        
        let stats = pool.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_hashmap_pool_reuse() {
        let pool: HashMapPool<Entity, i32> = HashMapPool::new();
        
        // Acquire and release first buffer
        {
            let mut guard = pool.acquire();
            guard.insert(Entity::new(1, 0), 42);
        }
        
        // Acquire again - should reuse the buffer
        {
            let guard = pool.acquire();
            assert_eq!(guard.len(), 0); // Should be cleared
        }
        
        let stats = pool.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate(), 50.0);
    }

    #[test]
    fn test_hashmap_pool_max_size() {
        let config = PoolConfig::new(32, 2); // Max 2 buffers
        let pool: HashMapPool<Entity, i32> = HashMapPool::with_config(config);
        
        // Create 3 buffers
        {
            let _g1 = pool.acquire();
            let _g2 = pool.acquire();
            let _g3 = pool.acquire();
        } // All dropped
        
        // Pool should only keep 2 (max_pool_size)
        assert_eq!(pool.len(), 2);
    }

    #[test]
    fn test_hashmap_pool_concurrent_access() {
        use std::thread;
        
        let pool: HashMapPool<usize, i32> = HashMapPool::new();
        let pool_clone = pool.clone();
        
        let handle = thread::spawn(move || {
            let mut guard = pool_clone.acquire();
            guard.insert(1, 100);
        });
        
        let mut guard = pool.acquire();
        guard.insert(2, 200);
        
        handle.join().unwrap();
        
        let stats = pool.stats();
        assert_eq!(stats.misses, 2); // Both threads allocated new buffers
    }

    #[test]
    fn test_hashmap_pool_clear() {
        let pool: HashMapPool<Entity, i32> = HashMapPool::new();
        
        // Add some buffers to the pool
        {
            let _g1 = pool.acquire();
            let _g2 = pool.acquire();
        }
        
        assert_eq!(pool.len(), 2);
        
        pool.clear();
        assert_eq!(pool.len(), 0);
        assert!(pool.is_empty());
    }

    #[test]
    fn test_pool_stats_tracking() {
        let pool: HashMapPool<usize, i32> = HashMapPool::new();
        
        // First acquisition - miss
        { let _ = pool.acquire(); }
        
        // Second acquisition - hit
        { let _ = pool.acquire(); }
        
        // Third acquisition - hit
        { let _ = pool.acquire(); }
        
        let stats = pool.stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.pool_size, 1);
        assert_eq!(stats.peak_size, 1);
    }

    #[test]
    fn test_guard_deref() {
        let pool: HashMapPool<usize, i32> = HashMapPool::new();
        let mut guard = pool.acquire();
        
        // Test Deref and DerefMut traits through HashMap operations
        guard.insert(1, 42);
        assert_eq!(guard.get(&1), Some(&42));
        
        // Test DerefMut trait
        if let Some(v) = guard.get_mut(&1) {
            *v = 100;
        }
        assert_eq!(guard.get(&1), Some(&100));
    }
}
