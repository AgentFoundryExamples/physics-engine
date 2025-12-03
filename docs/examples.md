# Examples Guide

This guide provides detailed instructions for running and understanding the example programs included with the physics engine.

## Available Examples

### 1. Basic ECS Example (`basic.rs`)

**Purpose**: Demonstrates the fundamental Entity Component System (ECS) architecture.

**Topics Covered**:
- Creating entities and worlds
- Adding components to entities
- Basic system execution
- Sequential vs parallel execution modes

**Running**:
```bash
cargo run --example basic
```

**Expected Output**:
- Entity creation and component assignment
- Simple position/velocity updates
- System execution demonstration

---

### 2. Solar System Simulation (`solar_system.rs`)

**Purpose**: Demonstrates gravitational N-body simulation with realistic solar system parameters.

**Topics Covered**:
- Newton's law of universal gravitation
- Realistic physical constants (G, AU, planetary masses)
- Comparison between Verlet and RK4 integrators
- Energy conservation tracking
- Long-term orbital stability

**Physical Constants**:
- Gravitational constant G = 6.67430 × 10⁻¹¹ m³/(kg⋅s²)
- 1 Astronomical Unit (AU) = 1.496 × 10¹¹ m
- Bodies: Sun, Mercury, Venus, Earth, Mars

**Running**:

```bash
# Basic run with default settings (1 year, Verlet integrator)
cargo run --example solar_system --release

# Run with RK4 integrator for comparison
cargo run --example solar_system --release -- --integrator rk4

# Simulate 10 years
cargo run --example solar_system --release -- --years 10

# Use smaller timestep for better accuracy
cargo run --example solar_system --release -- --timestep 43200

# Combine options
cargo run --example solar_system --release -- --integrator rk4 --years 5 --timestep 3600
```

**Command-Line Options**:
- `--integrator <name>`: Choose integrator (`verlet` or `rk4`)
- `--timestep <seconds>`: Set timestep in seconds (default: 86400 = 1 day)
- `--years <number>`: Duration in Earth years (default: 1.0)

**Expected Behavior**:
- Planets should maintain stable orbits
- Energy drift should be < 10% for reasonable timesteps
- Earth should remain approximately 1 AU from the Sun
- Faster integrators (larger timesteps) = less accurate but faster
- RK4 generally shows better energy conservation than Verlet for same timestep

**Interpreting Results**:

The simulation outputs:
- **Kinetic Energy**: Energy due to motion (½mv²)
- **Potential Energy**: Gravitational potential energy (-Gm₁m₂/r)
- **Total Energy**: Should remain approximately constant
- **Energy Drift**: Relative change in total energy (should be small)

Good results:
- Energy drift < 1%: Excellent
- Energy drift < 10%: Good
- Energy drift > 10%: Consider smaller timestep

**Timestep Selection**:

For solar system simulations:
- **1 day (86400 s)**: Fast but moderate accuracy (~1-10% energy drift over 1 year)
- **12 hours (43200 s)**: Better accuracy (~0.1-1% energy drift)
- **1 hour (3600 s)**: High accuracy (< 0.1% energy drift) but slower
- **RK4 at 1 day**: Similar accuracy to Verlet at 12 hours

**Performance**:
- Complexity: O(N²) for N bodies (N=5 for inner solar system)
- Typical performance: 1000+ steps/second on modern hardware
- Negligible overhead for small N

---

### 3. Particle Collision Simulation (`particle_collision.rs`)

**Purpose**: Demonstrates N-body simulation with many particles and performance characteristics.

**Topics Covered**:
- Random initial conditions with deterministic seeding
- Performance scaling with particle count
- Parallel force computation
- Energy conservation in chaotic systems
- System clustering and dynamics

**Running**:

```bash
# Basic run with 100 particles for 10 seconds
cargo run --example particle_collision --release

# Test with 500 particles (much slower)
cargo run --example particle_collision --release -- --particles 500

# Short test with custom parameters
cargo run --example particle_collision --release -- --particles 200 --duration 5 --timestep 0.01

# Use RK4 integrator
cargo run --example particle_collision --release -- --integrator rk4 --particles 100

# Deterministic testing with specific seed
cargo run --example particle_collision --release -- --seed 42 --particles 50
```

**Command-Line Options**:
- `--particles <n>`: Number of particles (default: 100)
- `--integrator <name>`: Choose integrator (`verlet` or `rk4`)
- `--timestep <seconds>`: Set timestep (default: 0.01 s)
- `--duration <seconds>`: Simulation duration (default: 10 s)
- `--seed <n>`: Random seed for reproducibility (default: 12345)

**Expected Behavior**:
- Particles gravitate toward each other
- System spread decreases over time (clustering)
- Kinetic energy increases as potential energy decreases
- Center of mass may drift slightly due to numerical errors

**Performance Characteristics**:

Complexity: O(N²) pairwise interactions
- N = 50: ~1-2 ms/step, ~1.25k interactions/step
- N = 100: ~4-8 ms/step, ~5k interactions/step  
- N = 500: ~100-200 ms/step, ~125k interactions/step
- N = 1000: ~400-800 ms/step, ~500k interactions/step

**Parallel Execution**:
- Enabled by default with `--features parallel` (Rayon)
- Scales well up to ~500 particles
- Chunk-based work distribution for load balancing

**Performance Guidance**:
- **N < 50**: Overhead dominated, consider single-threaded
- **N = 100-500**: Sweet spot for parallel execution
- **N > 1000**: O(N²) becomes limiting factor
  - Consider spatial data structures (octree, Barnes-Hut)
  - Or GPU acceleration for production use

**Deterministic Results**:

Same seed → identical results:
```bash
# These two runs produce identical output
cargo run --release --example particle_collision -- --seed 42 --particles 50
cargo run --release --example particle_collision -- --seed 42 --particles 50
```

This is crucial for:
- Debugging and validation
- Regression testing
- Reproducible scientific results

---

## General Tips

### Building for Performance

Always use release mode for realistic performance:
```bash
cargo build --release --examples
cargo run --release --example <name>
```

Debug builds are ~10-100x slower.

### Choosing an Integrator

**Velocity Verlet**:
- ✅ Best for long-running simulations
- ✅ Excellent energy conservation (symplectic)
- ✅ Good for oscillatory motion and orbits
- ⚠️ Less accurate for nonlinear forces
- 📊 2 force evaluations per step

**RK4 (Runge-Kutta 4th order)**:
- ✅ Higher accuracy for smooth forces
- ✅ Fourth-order accurate (vs second-order for Verlet)
- ✅ Better for nonlinear dynamics
- ⚠️ Not symplectic (energy may drift more)
- ⚠️ 4 force evaluations per step (2x slower)

**Recommendation**:
- Solar system / orbital mechanics → Velocity Verlet
- Smooth nonlinear forces → RK4
- When in doubt → Try both and compare

### Timestep Selection Guidelines

General rule: **dt < T_min / 20**
where T_min is the shortest timescale in your system.

**Solar System**:
- Mercury period: ~88 days → dt < 4 days
- Use dt = 1 day for good results
- Use dt = 1 hour for publication-quality

**Particle Collisions**:
- Typical collision time: ~1 second
- Use dt = 0.01 s for good results
- Use dt = 0.001 s if needed for accuracy

**Warning Signs**:
- Particles escaping to infinity: dt too large
- Energy growing exponentially: dt too large
- No visible change: dt too small (wasting computation)

### Monitoring Energy Conservation

Total energy should remain approximately constant:

```
E_total = E_kinetic + E_potential
ΔE / E_initial < tolerance
```

**Tolerance Guidelines**:
- **< 0.1%**: Excellent (publication quality)
- **< 1%**: Good (suitable for demonstrations)
- **< 10%**: Acceptable (qualitative behavior correct)
- **> 10%**: Poor (reduce timestep or check for bugs)

### Performance Profiling

To measure performance:
```bash
# Time the entire run
time cargo run --release --example particle_collision -- --particles 500

# Profile with perf (Linux)
cargo build --release --example particle_collision
perf record target/release/examples/particle_collision --particles 500
perf report

# Profile with flamegraph
cargo flamegraph --example particle_collision -- --particles 500
```

---

## Extending the Examples

### Adding New Bodies to Solar System

Edit `solar_system.rs` and add to `SOLAR_BODIES`:

```rust
CelestialBody {
    name: "Jupiter",
    mass: 1.898e27,
    distance: 5.2 * AU,
    orbital_velocity: 13070.0,
    color: "orange",
},
```

### Modifying Particle Properties

Edit `particle_collision.rs` `SimulationConfig`:

```rust
mass_range: (10.0, 100.0),  // Heavier particles
position_range: 200.0,       // Larger volume
velocity_range: 20.0,        // Faster initial speeds
softening: 5.0,              // More softening
g_scale: 1e11,               // Stronger gravity
```

### Adding Visualization

The examples output text data. To visualize:

1. **Save data to file**:
```rust
// Add to print_state()
writeln!(file, "{:.6},{:.6},{:.6}", pos.x(), pos.y(), pos.z())?;
```

2. **Plot with Python**:
```python
import matplotlib.pyplot as plt
import numpy as np

data = np.loadtxt('output.csv', delimiter=',')
plt.plot(data[:,0], data[:,1])
plt.show()
```

3. **Real-time visualization** (advanced):
   - Use `minifb` for window/pixel buffer
   - Use `nannou` or `ggez` for 2D graphics
   - Use `bevy` or `wgpu` for 3D graphics

---

## Troubleshooting

### "Force magnitude exceeds limit" warnings

**Cause**: Gravitational forces are very large for massive objects.

**Solution**: Increase the force limit in the example:
```rust
force_registry.max_force_magnitude = 1e25; // Or larger
```

This is not an error, just a safety check.

### Energy exploding

**Causes**:
1. Timestep too large
2. Particles too close (singularity)
3. Numerical instability

**Solutions**:
1. Reduce timestep (e.g., `--timestep 0.001`)
2. Increase softening parameter
3. Try different integrator (RK4 vs Verlet)

### Poor performance

**Causes**:
1. Running in debug mode
2. Too many particles (O(N²) complexity)
3. Parallel overhead with small N

**Solutions**:
1. Always use `--release` flag
2. Reduce particle count or use spatial acceleration
3. Disable parallel for N < 50

### Different results between runs

**Causes**:
1. Random seed not specified
2. Floating-point non-determinism in parallel mode
3. Platform differences

**Solutions**:
1. Use `--seed <fixed_value>`
2. Use single-threaded for determinism
3. Accept small differences across platforms

---

## Further Reading

- [Plugin Guide](plugins.md) - Creating custom force providers
- [Integration Guide](integration.md) - Numerical integration methods
- [Architecture Guide](architecture.md) - ECS design and parallelism

## References

### Physics
- Goldstein, H., Poole, C., & Safko, J. (2002). *Classical Mechanics* (3rd ed.)
- Aarseth, S. J. (2003). *Gravitational N-Body Simulations*
- [NASA Planetary Fact Sheet](https://nssdc.gsfc.nasa.gov/planetary/factsheet/)

### Numerical Methods
- Hairer, E., Lubich, C., & Wanner, G. (2006). *Geometric Numerical Integration*
- Press, W. H., et al. (2007). *Numerical Recipes* (3rd ed.)

### N-Body Algorithms
- Barnes, J., & Hut, P. (1986). "A hierarchical O(N log N) force-calculation algorithm"
- Dehnen, W. (2001). "Towards optimal softening in three-dimensional N-body codes"
