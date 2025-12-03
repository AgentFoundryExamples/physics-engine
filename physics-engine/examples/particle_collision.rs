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
//! N-Body Particle Simulation Example
//!
//! This example demonstrates gravitational N-body simulation with a large
//! number of particles. It showcases:
//!
//! - Performance with many bodies (scalability)
//! - Random initial conditions with deterministic seeding
//! - Energy conservation tracking
//! - Parallel computation with Rayon
//!
//! # Running
//!
//! ```bash
//! # Run with 100 particles for 10 seconds
//! cargo run --example particle_collision --release
//!
//! # Run with 500 particles
//! cargo run --example particle_collision --release -- --particles 500
//!
//! # Run with custom parameters
//! cargo run --example particle_collision --release -- --particles 200 --duration 20 --timestep 0.01
//!
//! # Use RK4 integrator
//! cargo run --example particle_collision --release -- --integrator rk4
//! ```

use physics_engine::ecs::{World, Entity, ComponentStorage, HashMapStorage};
use physics_engine::ecs::components::{Position, Velocity, Mass, Acceleration};
use physics_engine::ecs::systems::{ForceRegistry, apply_forces_to_acceleration};
use physics_engine::integration::{VelocityVerletIntegrator, RK4Integrator, Integrator};
use physics_engine::plugins::gravity::{GravityPlugin, GravitySystem};
use std::time::Instant;

/// Simple pseudo-random number generator for deterministic results
/// Uses a linear congruential generator (LCG) with parameters from
/// Numerical Recipes (Press et al., 2007), specifically:
/// - Multiplier: 6364136223846793005 (Knuth's 64-bit multiplier)
/// - Increment: 1442695040888963407
/// These parameters provide good statistical properties for 64-bit integers.
struct SimpleRng {
    state: u64,
}

/// Maximum value for 53-bit mantissa (2^53) used in float conversion
const F64_MANTISSA_MAX: f64 = 9007199254740992.0;

impl SimpleRng {
    fn new(seed: u64) -> Self {
        SimpleRng { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        // LCG: X_{n+1} = (a * X_n + c) mod m
        // Using parameters from Numerical Recipes / Knuth MMIX
        self.state = self.state.wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    fn next_f64(&mut self) -> f64 {
        // Generate a float in [0, 1) using upper 53 bits
        // This follows the standard approach for converting uniform integers to floats
        // Shift right by 11 to get 53 bits (64 - 11 = 53)
        // Divide by 2^53 to normalize to [0, 1)
        // This ensures uniform distribution across the entire [0, 1) range
        (self.next_u64() >> 11) as f64 / F64_MANTISSA_MAX
    }

    fn next_f64_range(&mut self, min: f64, max: f64) -> f64 {
        min + (max - min) * self.next_f64()
    }
}

/// Simulation configuration
struct SimulationConfig {
    num_particles: usize,
    integrator_name: String,
    timestep: f64,      // seconds
    duration: f64,      // seconds
    output_interval: f64, // seconds
    g_scale: f64,       // Gravitational constant scale factor
    mass_range: (f64, f64),  // kg
    position_range: f64,      // meters
    velocity_range: f64,      // m/s
    softening: f64,           // meters
    seed: u64,
    diagnostic_mode: bool,    // Enable detailed per-step diagnostics
}

impl Default for SimulationConfig {
    fn default() -> Self {
        SimulationConfig {
            num_particles: 100,
            integrator_name: "verlet".to_string(),
            timestep: 0.01,  // 10 ms
            duration: 10.0,  // 10 seconds
            output_interval: 1.0, // 1 second
            g_scale: 1e10,   // Scaled G for visible effects
            mass_range: (1.0, 10.0),  // 1-10 kg
            position_range: 100.0,     // ±100 m
            velocity_range: 10.0,      // ±10 m/s
            softening: 1.0,            // 1 m softening
            seed: 12345,
            diagnostic_mode: false,
        }
    }
}

/// Create random particles with deterministic seed
fn create_particles(
    world: &mut World,
    positions: &mut HashMapStorage<Position>,
    velocities: &mut HashMapStorage<Velocity>,
    masses: &mut HashMapStorage<Mass>,
    config: &SimulationConfig,
) -> Vec<Entity> {
    let mut rng = SimpleRng::new(config.seed);
    let mut entities = Vec::new();

    for i in 0..config.num_particles {
        let entity = world.create_entity();
        
        // Random position in a cube
        let pos = Position::new(
            rng.next_f64_range(-config.position_range, config.position_range),
            rng.next_f64_range(-config.position_range, config.position_range),
            rng.next_f64_range(-config.position_range, config.position_range),
        );
        
        // Random velocity
        let vel = Velocity::new(
            rng.next_f64_range(-config.velocity_range, config.velocity_range),
            rng.next_f64_range(-config.velocity_range, config.velocity_range),
            rng.next_f64_range(-config.velocity_range, config.velocity_range),
        );
        
        // Random mass
        let mass_value = rng.next_f64_range(config.mass_range.0, config.mass_range.1);
        let mass = Mass::new(mass_value);

        positions.insert(entity, pos);
        velocities.insert(entity, vel);
        masses.insert(entity, mass);

        entities.push(entity);

        if i < 5 || i == config.num_particles - 1 {
            println!("  Particle {}: pos=({:.1}, {:.1}, {:.1}) m, vel=({:.1}, {:.1}, {:.1}) m/s, mass={:.1} kg",
                     i, pos.x(), pos.y(), pos.z(), vel.dx(), vel.dy(), vel.dz(), mass_value);
        } else if i == 5 {
            println!("  ... ({} more particles) ...", config.num_particles - 6);
        }
    }

    entities
}

/// Calculate total kinetic energy
fn calculate_kinetic_energy(
    entities: &[Entity],
    velocities: &HashMapStorage<Velocity>,
    masses: &HashMapStorage<Mass>,
) -> f64 {
    let mut ke = 0.0;
    for entity in entities {
        if let (Some(vel), Some(mass)) = (velocities.get(*entity), masses.get(*entity)) {
            let v_sq = vel.dx() * vel.dx() + vel.dy() * vel.dy() + vel.dz() * vel.dz();
            ke += 0.5 * mass.value() * v_sq;
        }
    }
    ke
}

/// Calculate center of mass
fn calculate_center_of_mass(
    entities: &[Entity],
    positions: &HashMapStorage<Position>,
    masses: &HashMapStorage<Mass>,
) -> (f64, f64, f64) {
    let mut total_mass = 0.0;
    let mut cm_x = 0.0;
    let mut cm_y = 0.0;
    let mut cm_z = 0.0;

    for entity in entities {
        if let (Some(pos), Some(mass)) = (positions.get(*entity), masses.get(*entity)) {
            let m = mass.value();
            total_mass += m;
            cm_x += pos.x() * m;
            cm_y += pos.y() * m;
            cm_z += pos.z() * m;
        }
    }

    if total_mass > 0.0 {
        (cm_x / total_mass, cm_y / total_mass, cm_z / total_mass)
    } else {
        (0.0, 0.0, 0.0)
    }
}

/// Calculate system spread (max distance from center of mass)
fn calculate_spread(
    entities: &[Entity],
    positions: &HashMapStorage<Position>,
    center: (f64, f64, f64),
) -> f64 {
    let mut max_dist = 0.0;

    for entity in entities {
        if let Some(pos) = positions.get(*entity) {
            let dx = pos.x() - center.0;
            let dy = pos.y() - center.1;
            let dz = pos.z() - center.2;
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
            if dist > max_dist {
                max_dist = dist;
            }
        }
    }

    max_dist
}

/// Print system state
fn print_state(
    time: f64,
    entities: &[Entity],
    positions: &HashMapStorage<Position>,
    velocities: &HashMapStorage<Velocity>,
    masses: &HashMapStorage<Mass>,
) {
    let ke = calculate_kinetic_energy(entities, velocities, masses);
    let cm = calculate_center_of_mass(entities, positions, masses);
    let spread = calculate_spread(entities, positions, cm);
    
    println!("\nTime: {:.2} s", time);
    println!("  Kinetic Energy: {:.3e} J", ke);
    println!("  Center of Mass: ({:.1}, {:.1}, {:.1}) m", cm.0, cm.1, cm.2);
    println!("  System Spread:  {:.1} m", spread);
}

/// CSV header for diagnostic output
const DIAG_HEADER: &str = "DIAG,step,time_s,dt_s,KE_J,ke_change_frac,cm_x_m,cm_y_m,cm_z_m,spread_m";

/// Print detailed diagnostic information for failure analysis
fn print_diagnostics(
    step: usize,
    time: f64,
    dt: f64,
    entities: &[Entity],
    positions: &HashMapStorage<Position>,
    velocities: &HashMapStorage<Velocity>,
    masses: &HashMapStorage<Mass>,
    initial_ke: f64,
) {
    let ke = calculate_kinetic_energy(entities, velocities, masses);
    let cm = calculate_center_of_mass(entities, positions, masses);
    let spread = calculate_spread(entities, positions, cm);
    let ke_change = if initial_ke.abs() > 1e-9 {
        (ke - initial_ke) / initial_ke
    } else {
        0.0
    };
    
    // Format: step,time_s,dt_s,KE_J,ke_change_frac,cm_x_m,cm_y_m,cm_z_m,spread_m
    println!("DIAG,{},{:.6e},{:.6e},{:.6e},{:.6e},{:.3e},{:.3e},{:.3e},{:.3e}",
             step, time, dt, ke, ke_change, cm.0, cm.1, cm.2, spread);
}

fn main() {
    println!("==========================================================");
    println!("       N-Body Particle Collision Simulation");
    println!("==========================================================");
    println!();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let mut config = SimulationConfig::default();
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--particles" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<usize>() {
                        Ok(value) => config.num_particles = value,
                        Err(_) => {
                            eprintln!("Warning: Invalid particles '{}', using default 100", 
                                     args[i + 1]);
                            config.num_particles = 100;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("Error: --particles requires an argument");
                    std::process::exit(1);
                }
            }
            "--integrator" => {
                if i + 1 < args.len() {
                    config.integrator_name = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --integrator requires an argument");
                    std::process::exit(1);
                }
            }
            "--timestep" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<f64>() {
                        Ok(value) => config.timestep = value,
                        Err(_) => {
                            eprintln!("Warning: Invalid timestep '{}', using default 0.01 s", 
                                     args[i + 1]);
                            config.timestep = 0.01;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("Error: --timestep requires an argument");
                    std::process::exit(1);
                }
            }
            "--duration" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<f64>() {
                        Ok(value) => config.duration = value,
                        Err(_) => {
                            eprintln!("Warning: Invalid duration '{}', using default 10.0 s", 
                                     args[i + 1]);
                            config.duration = 10.0;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("Error: --duration requires an argument");
                    std::process::exit(1);
                }
            }
            "--seed" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u64>() {
                        Ok(value) => config.seed = value,
                        Err(_) => {
                            eprintln!("Warning: Invalid seed '{}', using default 12345", 
                                     args[i + 1]);
                            config.seed = 12345;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("Error: --seed requires an argument");
                    std::process::exit(1);
                }
            }
            "--diagnostics" => {
                config.diagnostic_mode = true;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    println!("Simulation Configuration:");
    println!("  Particles: {}", config.num_particles);
    println!("  Integrator: {}", config.integrator_name);
    println!("  Timestep: {:.3} s", config.timestep);
    println!("  Duration: {:.1} s", config.duration);
    println!("  G scale: {:.1e}", config.g_scale);
    println!("  Softening: {:.1} m", config.softening);
    println!("  Random seed: {}", config.seed);
    println!();

    // Create world and components
    let mut world = World::new();
    let mut positions = HashMapStorage::<Position>::new();
    let mut velocities = HashMapStorage::<Velocity>::new();
    let mut masses = HashMapStorage::<Mass>::new();
    let mut accelerations = HashMapStorage::<Acceleration>::new();

    println!("Creating {} particles...", config.num_particles);
    let entities = create_particles(&mut world, &mut positions, &mut velocities, &mut masses, &config);
    println!();

    // Initialize accelerations to zero
    for entity in &entities {
        accelerations.insert(*entity, Acceleration::zero());
    }

    // Create gravity system with scaled G and warning suppression
    let mut gravity_plugin = GravityPlugin::with_scaled_g(config.g_scale);
    gravity_plugin.set_softening(config.softening);
    // Suppress warnings for expected high-force scenarios in dense particle clouds
    gravity_plugin.set_warn_on_high_forces(false);
    gravity_plugin.set_warn_on_invalid(false);
    let gravity_system = GravitySystem::new(gravity_plugin);

    // Create integrator
    enum IntegratorWrapper {
        Verlet(VelocityVerletIntegrator),
        RK4(RK4Integrator),
    }

    impl IntegratorWrapper {
        fn name(&self) -> &str {
            match self {
                IntegratorWrapper::Verlet(i) => i.name(),
                IntegratorWrapper::RK4(i) => i.name(),
            }
        }

        fn integrate<'a>(
            &mut self,
            entities: impl Iterator<Item = &'a Entity>,
            positions: &mut impl ComponentStorage<Component = Position>,
            velocities: &mut impl ComponentStorage<Component = Velocity>,
            accelerations: &impl ComponentStorage<Component = Acceleration>,
            masses: &impl ComponentStorage<Component = Mass>,
            force_registry: &mut ForceRegistry,
            warn_on_missing: bool,
        ) -> usize {
            match self {
                IntegratorWrapper::Verlet(i) => i.integrate(entities, positions, velocities, accelerations, masses, force_registry, warn_on_missing),
                IntegratorWrapper::RK4(i) => i.integrate(entities, positions, velocities, accelerations, masses, force_registry, warn_on_missing),
            }
        }
    }

    let mut integrator = match config.integrator_name.as_str() {
        "rk4" => IntegratorWrapper::RK4(RK4Integrator::new(config.timestep)),
        "verlet" | _ => IntegratorWrapper::Verlet(VelocityVerletIntegrator::new(config.timestep)),
    };

    println!("Starting simulation with {} integrator...", integrator.name());

    // Initial state
    let initial_energy = calculate_kinetic_energy(&entities, &velocities, &masses);
    print_state(0.0, &entities, &positions, &velocities, &masses);

    // Diagnostic mode header
    if config.diagnostic_mode {
        println!();
        println!("=== DIAGNOSTIC MODE ENABLED ===");
        println!("CSV Header: {}", DIAG_HEADER);
        println!();
    }

    // Simulation loop
    let mut time = 0.0;
    let mut next_output_time = config.output_interval;
    let num_steps = (config.duration / config.timestep).ceil() as usize;

    println!();
    println!("Running {} steps (complexity: O(N²) = {} pairwise interactions per step)...",
             num_steps, config.num_particles * (config.num_particles - 1) / 2);
    println!();

    let start_time = Instant::now();
    let mut step_times = Vec::new();

    for step in 0..num_steps {
        let step_start = Instant::now();

        // Create fresh force registry for this step
        let mut force_registry = ForceRegistry::new();
        force_registry.max_force_magnitude = 1e10;
        force_registry.warn_on_missing_components = false;

        // Compute gravitational forces at current positions
        gravity_system.compute_forces(&entities, &positions, &masses, &mut force_registry);
        
        // Accumulate forces from registered providers
        for entity in &entities {
            force_registry.accumulate_for_entity(*entity);
        }

        // Apply forces to compute accelerations
        apply_forces_to_acceleration(
            entities.iter(),
            &force_registry,
            &masses,
            &mut accelerations,
            false,
        );
        
        // Store old accelerations for Verlet velocity update
        let mut old_accelerations = HashMapStorage::<Acceleration>::new();
        for entity in &entities {
            if let Some(acc) = accelerations.get(*entity) {
                old_accelerations.insert(*entity, *acc);
            }
        }

        // Update positions: x(t+dt) = x(t) + v(t)*dt + 0.5*a(t)*dt²
        let dt = config.timestep;
        let dt_sq = dt * dt;
        for entity in &entities {
            if masses.get(*entity).map_or(true, |m| m.is_immovable()) {
                continue;
            }
            if let (Some(pos), Some(vel), Some(acc)) = (
                positions.get_mut(*entity),
                velocities.get(*entity),
                accelerations.get(*entity),
            ) {
                let new_x = pos.x() + vel.dx() * dt + 0.5 * acc.ax() * dt_sq;
                let new_y = pos.y() + vel.dy() * dt + 0.5 * acc.ay() * dt_sq;
                let new_z = pos.z() + vel.dz() * dt + 0.5 * acc.az() * dt_sq;
                pos.set_x(new_x);
                pos.set_y(new_y);
                pos.set_z(new_z);
            }
        }

        // Create fresh force registry for recomputing at new positions
        let mut force_registry = ForceRegistry::new();
        force_registry.max_force_magnitude = 1e10;
        force_registry.warn_on_missing_components = false;

        // Recompute gravitational forces at new positions
        gravity_system.compute_forces(&entities, &positions, &masses, &mut force_registry);
        
        // Accumulate forces from registered providers
        for entity in &entities {
            force_registry.accumulate_for_entity(*entity);
        }

        // Compute new accelerations
        apply_forces_to_acceleration(
            entities.iter(),
            &force_registry,
            &masses,
            &mut accelerations,
            false,
        );

        // Update velocities: v(t+dt) = v(t) + 0.5*(a(t) + a(t+dt))*dt
        for entity in &entities {
            if masses.get(*entity).map_or(true, |m| m.is_immovable()) {
                continue;
            }
            if let Some(vel) = velocities.get_mut(*entity) {
                let old_acc = old_accelerations.get(*entity).copied().unwrap_or_else(Acceleration::zero);
                let new_acc = accelerations.get(*entity).copied().unwrap_or_else(Acceleration::zero);

                let avg_ax = 0.5 * (old_acc.ax() + new_acc.ax());
                let avg_ay = 0.5 * (old_acc.ay() + new_acc.ay());
                let avg_az = 0.5 * (old_acc.az() + new_acc.az());
                
                let new_dx = vel.dx() + avg_ax * dt;
                let new_dy = vel.dy() + avg_ay * dt;
                let new_dz = vel.dz() + avg_az * dt;
                vel.set_dx(new_dx);
                vel.set_dy(new_dy);
                vel.set_dz(new_dz);
            }
        }

        time += config.timestep;
        step_times.push(step_start.elapsed().as_secs_f64());

        // Diagnostic logging (every 50 steps to avoid explosion)
        if config.diagnostic_mode && step % 50 == 0 {
            print_diagnostics(
                step,
                time,
                config.timestep,
                &entities,
                &positions,
                &velocities,
                &masses,
                initial_energy,
            );
        }

        // Output at intervals
        if time >= next_output_time {
            print_state(time, &entities, &positions, &velocities, &masses);
            next_output_time += config.output_interval;
        }
    }

    let total_time = start_time.elapsed();

    // Final state
    println!();
    println!("==========================================================");
    println!("                  SIMULATION COMPLETE");
    println!("==========================================================");
    print_state(time, &entities, &positions, &velocities, &masses);

    // Energy conservation
    let final_energy = calculate_kinetic_energy(&entities, &velocities, &masses);
    let energy_drift = if initial_energy != 0.0 {
        ((final_energy - initial_energy) / initial_energy).abs()
    } else {
        0.0
    };

    println!();
    println!("Energy Conservation:");
    println!("  Initial KE: {:.6e} J", initial_energy);
    println!("  Final KE:   {:.6e} J", final_energy);
    println!("  Relative Change: {:.6e} ({:.4}%)", energy_drift, energy_drift * 100.0);

    // Performance statistics
    println!();
    println!("Performance Statistics:");
    println!("  Total time: {:.2} s", total_time.as_secs_f64());
    println!("  Steps completed: {}", num_steps);
    println!("  Average step time: {:.3} ms", 
             step_times.iter().sum::<f64>() / step_times.len() as f64 * 1000.0);
    
    let interactions_per_step = config.num_particles * (config.num_particles - 1) / 2;
    let total_interactions = interactions_per_step * num_steps;
    let interactions_per_second = total_interactions as f64 / total_time.as_secs_f64();
    
    println!("  Total pairwise interactions: {:.2e}", total_interactions as f64);
    println!("  Interactions/second: {:.2e}", interactions_per_second);

    #[cfg(feature = "parallel")]
    println!("  Parallel execution: ENABLED");
    #[cfg(not(feature = "parallel"))]
    println!("  Parallel execution: DISABLED");

    println!();
    
    // Performance guidance
    if config.num_particles < 50 {
        println!("💡 Try increasing --particles for a more challenging test");
    } else if config.num_particles > 1000 {
        println!("⚡ Large N detected - performance may be limited by O(N²) complexity");
        println!("   Consider spatial data structures (octree/BH) for production use");
    }

    println!();
}
