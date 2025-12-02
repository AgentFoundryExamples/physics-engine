# Integration Methods

## Overview

The physics engine provides multiple numerical integration methods for updating entity positions and velocities based on forces. Each integrator has different accuracy, stability, and performance characteristics suitable for different use cases.

## Available Integrators

### Velocity Verlet

The Velocity Verlet method is a symplectic integrator that provides excellent energy conservation for Hamiltonian systems like orbital mechanics and molecular dynamics.

**Algorithm:**
```text
x(t + dt) = x(t) + v(t)*dt + 0.5*a(t)*dt²
v(t + dt) = v(t) + 0.5*(a(t) + a(t + dt))*dt
```

**Properties:**
- **Symplectic**: Preserves phase space volume (Liouville's theorem)
- **Time-reversible**: Running forward then backward returns to start
- **Energy conservation**: Bounded energy error over long periods
- **Second-order accurate**: Local error O(dt³), global error O(dt²)
- **Performance**: ~2x force evaluations per step (one for current, one for next position)

**Best for:**
- Long-running simulations requiring energy conservation
- Oscillatory motion (springs, pendulums)
- Orbital mechanics
- Molecular dynamics

### Runge-Kutta 4th Order (RK4)

RK4 is a classical explicit integrator providing fourth-order accuracy for smooth ordinary differential equations.

**Algorithm:**
```text
k1 = f(t, y)
k2 = f(t + dt/2, y + k1*dt/2)
k3 = f(t + dt/2, y + k2*dt/2)
k4 = f(t + dt, y + k3*dt)
y(t + dt) = y(t) + (k1 + 2*k2 + 2*k3 + k4)*dt/6
```

**Properties:**
- **Fourth-order accurate**: Local error O(dt⁵), global error O(dt⁴)
- **Explicit method**: Easy to implement, no implicit solve needed
- **Not symplectic**: Energy may drift over very long simulations
- **Performance**: 4x force evaluations per step

**Best for:**
- Simulations requiring high accuracy with smooth forces
- Short to medium duration simulations
- Systems with nonlinear forces that vary smoothly
- When energy drift is acceptable for improved accuracy

## Choosing an Integrator

| Criterion | Velocity Verlet | RK4 |
|-----------|----------------|-----|
| **Accuracy** | O(dt²) | O(dt⁴) |
| **Energy Conservation** | Excellent | Good |
| **Performance** | ~2x evals/step | 4x evals/step |
| **Stability** | High | High |
| **Best Use Case** | Long simulations, oscillatory | High precision, smooth forces |

### Decision Guide

**Use Velocity Verlet when:**
- Running long simulations (>1000 timesteps)
- Energy conservation is critical
- Forces are position-dependent (springs, gravity)
- Performance matters (2x cheaper than RK4)

**Use RK4 when:**
- Maximum accuracy is needed
- Forces vary smoothly and nonlinearly
- Simulation duration is short to medium
- Energy drift is acceptable

## Timestep Selection

Choosing the right timestep is crucial for both accuracy and stability:

### General Guidelines

- **Too small**: Numerical precision issues, wasted computation, potential underflow
- **Too large**: Instability, poor accuracy, potential explosion
- **Recommended starting point**: dt = 1/60 (60 FPS game loop) or dt = 0.01

### Stability Criteria

For oscillatory systems with frequency ω:
- **Verlet**: dt < 2/ω (typically dt < period/3 is safe)
- **RK4**: dt < 2.8/ω (more stable than Verlet)

### Validation

The integrator trait provides validation warnings:

```rust
use physics_engine::integration::{VelocityVerletIntegrator, Integrator};

let integrator = VelocityVerletIntegrator::new(1e-10);
match integrator.validate_timestep() {
    Ok(()) => println!("Timestep OK"),
    Err(msg) => eprintln!("Warning: {}", msg),
}
```

**Warnings issued for:**
- dt < 1e-9: May cause precision loss with f64
- dt > 1.0: May cause instability
- dt ≤ 0 or NaN/Inf: Invalid timestep

## Usage Examples

### Basic Integration

```rust
use physics_engine::ecs::{World, Entity, HashMapStorage};
use physics_engine::ecs::components::{Position, Velocity, Mass, Acceleration};
use physics_engine::ecs::systems::ForceRegistry;
use physics_engine::integration::{VelocityVerletIntegrator, Integrator};

// Setup entities with physics components
let mut world = World::new();
let entity = world.create_entity();

let mut positions = HashMapStorage::<Position>::new();
positions.insert(entity, Position::new(0.0, 0.0, 0.0));

let mut velocities = HashMapStorage::<Velocity>::new();
velocities.insert(entity, Velocity::new(1.0, 0.0, 0.0));

let accelerations = HashMapStorage::<Acceleration>::new();
let mut masses = HashMapStorage::<Mass>::new();
masses.insert(entity, Mass::new(10.0));

let mut force_registry = ForceRegistry::new();
// Register force providers here

// Create and use integrator
let mut integrator = VelocityVerletIntegrator::new(1.0 / 60.0);
let entities = vec![entity];

integrator.integrate(
    entities.iter(),
    &mut positions,
    &mut velocities,
    &accelerations,
    &masses,
    &mut force_registry,
    true, // warn on missing components
);
```

### Switching Integrators

```rust
use physics_engine::integration::{Integrator, VelocityVerletIntegrator, RK4Integrator};

enum IntegratorChoice {
    Verlet,
    RK4,
}

fn create_integrator(choice: IntegratorChoice, dt: f64) -> Box<dyn Integrator> {
    match choice {
        IntegratorChoice::Verlet => Box::new(VelocityVerletIntegrator::new(dt)),
        IntegratorChoice::RK4 => Box::new(RK4Integrator::new(dt)),
    }
}

// Runtime selection
let integrator = create_integrator(IntegratorChoice::Verlet, 0.01);
```

### Adaptive Timestep (Manual)

The engine doesn't currently provide automatic adaptive timestepping, but you can implement it manually:

```rust
let mut dt = 0.01;
let max_dt = 0.1;
let min_dt = 0.001;

loop {
    // Integrate
    integrator.set_timestep(dt);
    integrator.integrate(/* ... */);
    
    // Estimate error (simplified - compute position difference with half-step)
    let error_estimate = compute_error();
    
    // Adjust timestep
    if error_estimate > threshold {
        dt = (dt * 0.5).max(min_dt); // Decrease
    } else if error_estimate < threshold * 0.1 {
        dt = (dt * 2.0).min(max_dt); // Increase
    }
}
```

## Integration with Scheduler

The integrators can be integrated into the ECS scheduler for organized simulation:

```rust
use physics_engine::ecs::scheduler::{Scheduler, stages};
use physics_engine::ecs::{System, World};
use physics_engine::integration::{Integrator, VelocityVerletIntegrator};

struct IntegrationSystem {
    integrator: Box<dyn Integrator>,
}

impl System for IntegrationSystem {
    fn run(&mut self, world: &mut World) {
        // Get components from world
        // Run integration
        // self.integrator.integrate(...)
    }
    
    fn name(&self) -> &str {
        "IntegrationSystem"
    }
}

let mut scheduler = Scheduler::new();
let integration_system = IntegrationSystem {
    integrator: Box::new(VelocityVerletIntegrator::new(0.01)),
};
scheduler.add_system(integration_system, stages::INTEGRATION);
```

## Performance Considerations

### Benchmark Results

Based on benchmarks in `benches/integration.rs`:

**Throughput (entities/second):**
- Verlet: ~2x faster than RK4 for same accuracy level
- RK4: More expensive but higher accuracy per step

**Memory:**
- Verlet: O(1) additional memory per integration
- RK4: Reuses internal buffers (8 HashMaps) to minimize allocations

### Optimization Tips

1. **Batch sizing**: Process entities in batches to improve cache locality
2. **Component layout**: Future SoA storage will improve SIMD utilization
3. **Force evaluation**: Cache forces when possible, especially for RK4
4. **Parallel execution**: Use parallel feature for multi-threaded force evaluation

## Force Sampling

Both integrators support custom force providers through the `ForceRegistry`:

```rust
use physics_engine::ecs::systems::{ForceProvider, Force, ForceRegistry};
use physics_engine::ecs::Entity;

struct GravityForce {
    g: f64,
}

impl ForceProvider for GravityForce {
    fn compute_force(&self, entity: Entity, registry: &ForceRegistry) -> Option<Force> {
        // Get mass from storage (simplified)
        let mass = 10.0;
        Some(Force::new(0.0, -mass * self.g, 0.0))
    }
    
    fn name(&self) -> &str {
        "Gravity"
    }
}
```

### Force Evaluation Hooks

- **Verlet**: Evaluates forces twice per step (at current and next position)
- **RK4**: Evaluates forces four times per step (at intermediate RK stages)

Ensure force providers are thread-safe (`Send + Sync`) for parallel execution.

## Common Pitfalls

### 1. Timestep Too Large

**Problem:** Simulation explodes or becomes unstable

**Solution:** Reduce timestep or use more stable integrator
```rust
// Check stability condition
let omega = (k / m).sqrt(); // System frequency
let dt_max = 2.0 / omega; // Nyquist limit
let dt = dt_max / 3.0; // Safety factor
```

### 2. Missing Components

**Problem:** Entities skip integration silently

**Solution:** Enable warnings and ensure all entities have required components
```rust
integrator.integrate(
    entities.iter(),
    &mut positions,
    &mut velocities,
    &accelerations,
    &masses,
    &mut force_registry,
    true, // Enable warnings
);
```

### 3. Force Provider State

**Problem:** RK4 evaluates forces at intermediate states, but force providers don't see updated positions

**Solution:** RK4 temporarily updates positions during evaluation. Force providers should read from the storage, not cache values.

### 4. Energy Drift

**Problem:** Energy slowly increases/decreases over time

**Solution:**
- Use Verlet for better energy conservation
- Reduce timestep
- Add energy correction/damping if needed
- Monitor energy: `E = 0.5*m*v² + potential_energy`

## Testing and Validation

### Conservation Tests

Verify energy conservation for simple systems:

```rust
// Simple harmonic oscillator
let k = 100.0; // spring constant
let m = 1.0; // mass
let x0 = 1.0; // initial displacement

let initial_energy = 0.5 * k * x0 * x0;

// Run simulation
for _ in 0..1000 {
    integrator.integrate(/* ... */);
}

let final_energy = compute_energy(&positions, &velocities, k, m);
let energy_error = (final_energy - initial_energy).abs() / initial_energy;

assert!(energy_error < 0.01); // 1% error tolerance for Verlet
```

### Accuracy Tests

Compare against analytical solutions when available:

```rust
// Free fall: x(t) = x0 + v0*t - 0.5*g*t²
let analytical_position = x0 + v0 * t - 0.5 * g * t * t;
let simulated_position = positions.get(entity).unwrap().y();
let error = (analytical_position - simulated_position).abs();

assert!(error < tolerance);
```

## References

### Velocity Verlet
- Hairer, E., Lubich, C., & Wanner, G. (2006). *Geometric Numerical Integration: Structure-Preserving Algorithms for Ordinary Differential Equations* (2nd ed.). Springer.
- Swope, W. C., et al. (1982). *A computer simulation method for the calculation of equilibrium constants*. J. Chem. Phys., 76(1), 637-649.
- Verlet, L. (1967). *Computer "Experiments" on Classical Fluids*. Physical Review, 159(1), 98-103.

### Runge-Kutta Methods
- Butcher, J. C. (2016). *Numerical Methods for Ordinary Differential Equations* (3rd ed.). Wiley.
- Press, W. H., et al. (2007). *Numerical Recipes: The Art of Scientific Computing* (3rd ed.). Cambridge University Press.
- Kutta, W. (1901). *Beitrag zur näherungsweisen Integration totaler Differentialgleichungen*. Z. Math. Phys., 46, 435-453.

### General Numerical Integration
- Ascher, U. M., & Petzold, L. R. (1998). *Computer Methods for Ordinary Differential Equations and Differential-Algebraic Equations*. SIAM.
- Lambert, J. D. (1991). *Numerical Methods for Ordinary Differential Systems*. Wiley.

## Future Enhancements

Planned features for future releases:

- **Adaptive timestepping**: Automatic dt adjustment based on error estimates
- **Implicit integrators**: For stiff systems (e.g., backward Euler)
- **Symplectic integrators**: Additional methods (leapfrog, Störmer-Verlet)
- **Multi-step methods**: Adams-Bashforth, Adams-Moulton
- **Constraint preservation**: SHAKE/RATTLE algorithms for constrained dynamics
- **Variable-order methods**: Automatic order selection based on smoothness
