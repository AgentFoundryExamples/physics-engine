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
//! Solar System N-Body Simulation Example
//!
//! This example demonstrates gravitational N-body simulation using the
//! gravity plugin with realistic solar system parameters. It showcases:
//!
//! - Newton's law of universal gravitation
//! - Comparison between Verlet and RK4 integrators
//! - Energy conservation tracking
//! - Deterministic simulation results
//!
//! # Physical Constants
//!
//! All values use SI units (meters, kilograms, seconds) based on:
//! - NASA Planetary Fact Sheet: https://nssdc.gsfc.nasa.gov/planetary/factsheet/
//! - JPL Solar System Dynamics: https://ssd.jpl.nasa.gov/
//!
//! # Running
//!
//! ```bash
//! # Run with default settings (Verlet integrator, 1 Earth year)
//! cargo run --example solar_system --release
//!
//! # Run with RK4 integrator
//! cargo run --example solar_system --release -- --integrator rk4
//!
//! # Run for 10 years with smaller timestep
//! cargo run --example solar_system --release -- --years 10 --timestep 3600
//! ```

use physics_engine::ecs::{World, Entity, ComponentStorage, HashMapStorage};
use physics_engine::ecs::components::{Position, Velocity, Mass, Acceleration};
use physics_engine::ecs::systems::{ForceRegistry, apply_forces_to_acceleration};
use physics_engine::integration::{VelocityVerletIntegrator, RK4Integrator, Integrator};
use physics_engine::plugins::gravity::{GravityPlugin, GravitySystem, GRAVITATIONAL_CONSTANT, DEFAULT_SOFTENING};

/// Astronomical Unit in meters (average Earth-Sun distance)
const AU: f64 = 1.495978707e11;

/// One Earth day in seconds
const DAY: f64 = 86400.0;

/// One Earth year in seconds (365.25 days)
const YEAR: f64 = 365.25 * DAY;

/// Celestial body data structure
struct CelestialBody {
    name: &'static str,
    mass: f64,              // kg
    distance: f64,          // m (semi-major axis)
    orbital_velocity: f64,  // m/s (approximate circular orbit velocity)
}

/// Solar system body data from NASA
/// Source: https://nssdc.gsfc.nasa.gov/planetary/factsheet/
const SOLAR_BODIES: &[CelestialBody] = &[
    CelestialBody {
        name: "Sun",
        mass: 1.989e30,
        distance: 0.0,
        orbital_velocity: 0.0,
    },
    CelestialBody {
        name: "Mercury",
        mass: 3.301e23,
        distance: 0.387 * AU,
        orbital_velocity: 47870.0,
    },
    CelestialBody {
        name: "Venus",
        mass: 4.867e24,
        distance: 0.723 * AU,
        orbital_velocity: 35020.0,
    },
    CelestialBody {
        name: "Earth",
        mass: 5.972e24,
        distance: 1.0 * AU,
        orbital_velocity: 29780.0,
    },
    CelestialBody {
        name: "Mars",
        mass: 6.417e23,
        distance: 1.524 * AU,
        orbital_velocity: 24070.0,
    },
];

/// Simulation configuration
struct SimulationConfig {
    integrator_name: String,
    timestep: f64,      // seconds
    duration: f64,      // seconds
    output_interval: f64, // seconds
    diagnostic_mode: bool, // Enable detailed per-step diagnostics
}

impl Default for SimulationConfig {
    fn default() -> Self {
        SimulationConfig {
            integrator_name: "verlet".to_string(),
            timestep: 86400.0,  // 1 day
            duration: YEAR,     // 1 year
            output_interval: 30.0 * DAY, // Once per month
            diagnostic_mode: false,
        }
    }
}

/// Create entities for solar system bodies
fn create_solar_system(
    world: &mut World,
    positions: &mut HashMapStorage<Position>,
    velocities: &mut HashMapStorage<Velocity>,
    masses: &mut HashMapStorage<Mass>,
) -> Vec<(Entity, &'static str)> {
    let mut entities = Vec::new();

    for body in SOLAR_BODIES {
        let entity = world.create_entity();
        
        // Place bodies at their orbital distances along x-axis
        // Sun at origin, planets on positive x-axis
        let pos = Position::new(body.distance, 0.0, 0.0);
        
        // Initial velocity in y-direction for circular orbit
        // v = sqrt(G*M_sun/r) for circular orbit
        let vel = Velocity::new(0.0, body.orbital_velocity, 0.0);
        
        let mass = Mass::new(body.mass);

        positions.insert(entity, pos);
        velocities.insert(entity, vel);
        masses.insert(entity, mass);

        entities.push((entity, body.name));
        
        println!("Created {} - Mass: {:.3e} kg, Distance: {:.3e} m ({:.3} AU), Velocity: {:.0} m/s",
                 body.name, body.mass, body.distance, body.distance / AU, body.orbital_velocity);
    }

    entities
}

/// Calculate total kinetic energy of the system
fn calculate_kinetic_energy(
    entities: &[(Entity, &str)],
    velocities: &HashMapStorage<Velocity>,
    masses: &HashMapStorage<Mass>,
) -> f64 {
    let mut ke = 0.0;
    for (entity, _) in entities {
        if let (Some(vel), Some(mass)) = (velocities.get(*entity), masses.get(*entity)) {
            let v_sq = vel.dx() * vel.dx() + vel.dy() * vel.dy() + vel.dz() * vel.dz();
            ke += 0.5 * mass.value() * v_sq;
        }
    }
    ke
}

/// Calculate total potential energy of the system
fn calculate_potential_energy(
    entities: &[(Entity, &str)],
    positions: &HashMapStorage<Position>,
    masses: &HashMapStorage<Mass>,
) -> f64 {
    let mut pe = 0.0;
    let n = entities.len();
    
    // Sum pairwise potential energies: U = -G * m1 * m2 / sqrt(r² + ε²)
    // Using the same softening as force calculation for consistency
    // This ensures energy conservation is properly tracked
    let softening_squared = DEFAULT_SOFTENING * DEFAULT_SOFTENING;
    
    for i in 0..n {
        for j in (i + 1)..n {
            let entity1 = entities[i].0;
            let entity2 = entities[j].0;
            
            if let (Some(pos1), Some(pos2), Some(m1), Some(m2)) = (
                positions.get(entity1),
                positions.get(entity2),
                masses.get(entity1),
                masses.get(entity2),
            ) {
                let dx = pos2.x() - pos1.x();
                let dy = pos2.y() - pos1.y();
                let dz = pos2.z() - pos1.z();
                let r_squared = dx * dx + dy * dy + dz * dz;
                let softened_r = (r_squared + softening_squared).sqrt();
                
                if softened_r > 0.0 {
                    pe -= GRAVITATIONAL_CONSTANT * m1.value() * m2.value() / softened_r;
                }
            }
        }
    }
    
    pe
}

/// Print system state with optional diagnostics
fn print_state(
    time: f64,
    entities: &[(Entity, &str)],
    positions: &HashMapStorage<Position>,
    velocities: &HashMapStorage<Velocity>,
    masses: &HashMapStorage<Mass>,
) {
    let ke = calculate_kinetic_energy(entities, velocities, masses);
    let pe = calculate_potential_energy(entities, positions, masses);
    let total_energy = ke + pe;
    
    println!("\n=== Time: {:.2} years ({:.2e} s) ===", time / YEAR, time);
    println!("Kinetic Energy:   {:.6e} J", ke);
    println!("Potential Energy: {:.6e} J", pe);
    println!("Total Energy:     {:.6e} J", total_energy);
    
    // Print Earth's position for reference
    if let Some((entity, _)) = entities.iter().find(|(_, name)| *name == "Earth") {
        if let Some(pos) = positions.get(*entity) {
            let r = (pos.x() * pos.x() + pos.y() * pos.y() + pos.z() * pos.z()).sqrt();
            println!("Earth distance from Sun: {:.3e} m ({:.3} AU)", r, r / AU);
        }
    }
}

/// CSV header for diagnostic output
const DIAG_HEADER: &str = "DIAG,step,time_s,dt_s,KE_J,PE_J,E_total_J,drift_frac,earth_AU,earth_v_ms,earth_a_ms2";

/// Print detailed diagnostic information for failure analysis
fn print_diagnostics(
    step: usize,
    time: f64,
    dt: f64,
    entities: &[(Entity, &str)],
    positions: &HashMapStorage<Position>,
    velocities: &HashMapStorage<Velocity>,
    accelerations: &HashMapStorage<Acceleration>,
    masses: &HashMapStorage<Mass>,
    initial_energy: f64,
) {
    let ke = calculate_kinetic_energy(entities, velocities, masses);
    let pe = calculate_potential_energy(entities, positions, masses);
    let total_energy = ke + pe;
    let energy_drift = if initial_energy != 0.0 {
        ((total_energy - initial_energy) / initial_energy).abs()
    } else {
        0.0
    };
    
    // Find Earth for detailed tracking
    if let Some((entity, _)) = entities.iter().find(|(_, name)| *name == "Earth") {
        if let (Some(pos), Some(vel), Some(acc)) = (
            positions.get(*entity),
            velocities.get(*entity),
            accelerations.get(*entity),
        ) {
            let r = (pos.x() * pos.x() + pos.y() * pos.y() + pos.z() * pos.z()).sqrt();
            let v_mag = (vel.dx() * vel.dx() + vel.dy() * vel.dy() + vel.dz() * vel.dz()).sqrt();
            let a_mag = (acc.ax() * acc.ax() + acc.ay() * acc.ay() + acc.az() * acc.az()).sqrt();
            
            // Format: step,time_s,dt_s,KE_J,PE_J,E_total_J,drift_frac,earth_AU,earth_v_ms,earth_a_ms2
            println!("DIAG,{},{:.6e},{:.6e},{:.6e},{:.6e},{:.6e},{:.6e},{:.6e},{:.6e},{:.6e}",
                     step, time, dt, ke, pe, total_energy, energy_drift, r / AU, v_mag, a_mag);
        }
    }
}

fn main() {
    println!("==========================================================");
    println!("       Solar System N-Body Simulation");
    println!("==========================================================");
    println!();
    println!("This example demonstrates gravitational N-body simulation");
    println!("using Newton's law of universal gravitation.");
    println!();
    println!("Physical Constants:");
    println!("  G = {:.5e} m³/(kg⋅s²)", GRAVITATIONAL_CONSTANT);
    println!("  1 AU = {:.5e} m", AU);
    println!("  1 year = {:.5e} s", YEAR);
    println!();

    // Parse command line arguments (simple)
    let args: Vec<String> = std::env::args().collect();
    let mut config = SimulationConfig::default();
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
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
                            eprintln!("Warning: Invalid timestep '{}', using default {:.0} s", 
                                     args[i + 1], DAY);
                            config.timestep = DAY;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("Error: --timestep requires an argument");
                    std::process::exit(1);
                }
            }
            "--years" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<f64>() {
                        Ok(years) => config.duration = years * YEAR,
                        Err(_) => {
                            eprintln!("Warning: Invalid years '{}', using default 1.0 year", 
                                     args[i + 1]);
                            config.duration = YEAR;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("Error: --years requires an argument");
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
    println!("  Integrator: {}", config.integrator_name);
    println!("  Timestep: {:.0} s ({:.2} days)", config.timestep, config.timestep / DAY);
    println!("  Duration: {:.2} years", config.duration / YEAR);
    println!("  Output interval: {:.0} days", config.output_interval / DAY);
    println!();

    // Create world and components
    let mut world = World::new();
    let mut positions = HashMapStorage::<Position>::new();
    let mut velocities = HashMapStorage::<Velocity>::new();
    let mut masses = HashMapStorage::<Mass>::new();
    let mut accelerations = HashMapStorage::<Acceleration>::new();

    println!("Creating solar system bodies...");
    println!();
    let entities = create_solar_system(&mut world, &mut positions, &mut velocities, &mut masses);

    // Initialize accelerations to zero
    for (entity, _) in &entities {
        accelerations.insert(*entity, Acceleration::zero());
    }

    // Create gravity system
    let gravity_plugin = GravityPlugin::new(GRAVITATIONAL_CONSTANT);
    let gravity_system = GravitySystem::new(gravity_plugin);

    // Create force registry
    let mut force_registry = ForceRegistry::new();

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

    println!();
    println!("Starting simulation with {} integrator...", integrator.name());
    println!();

    // Initial state
    let initial_energy = {
        let ke = calculate_kinetic_energy(&entities, &velocities, &masses);
        let pe = calculate_potential_energy(&entities, &positions, &masses);
        ke + pe
    };

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
    println!("Running {} steps...", num_steps);
    println!();

    for step in 0..num_steps {
        // Compute gravitational forces
        let entity_vec: Vec<Entity> = entities.iter().map(|(e, _)| *e).collect();
        gravity_system.compute_forces(&entity_vec, &positions, &masses, &mut force_registry);

        // Apply forces to compute accelerations
        apply_forces_to_acceleration(
            entity_vec.iter(),
            &force_registry,
            &masses,
            &mut accelerations,
            false,
        );

        // Integrate motion
        integrator.integrate(
            entity_vec.iter(),
            &mut positions,
            &mut velocities,
            &accelerations,
            &masses,
            &mut force_registry,
            false,
        );

        // Clear forces for next step
        force_registry.clear_forces();

        time += config.timestep;

        // Diagnostic logging (every 10 steps to avoid explosion)
        if config.diagnostic_mode && step % 10 == 0 {
            print_diagnostics(
                step,
                time,
                config.timestep,
                &entities,
                &positions,
                &velocities,
                &accelerations,
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

    // Final state
    println!();
    println!("==========================================================");
    println!("                  SIMULATION COMPLETE");
    println!("==========================================================");
    print_state(time, &entities, &positions, &velocities, &masses);

    // Energy conservation check
    let final_energy = {
        let ke = calculate_kinetic_energy(&entities, &velocities, &masses);
        let pe = calculate_potential_energy(&entities, &positions, &masses);
        ke + pe
    };

    let energy_drift = ((final_energy - initial_energy) / initial_energy).abs();
    println!();
    println!("Energy Conservation:");
    println!("  Initial Energy: {:.6e} J", initial_energy);
    println!("  Final Energy:   {:.6e} J", final_energy);
    println!("  Relative Drift: {:.6e} ({:.4}%)", energy_drift, energy_drift * 100.0);
    println!();

    if energy_drift < 0.01 {
        println!("✓ Excellent energy conservation (< 1% drift)");
    } else if energy_drift < 0.1 {
        println!("✓ Good energy conservation (< 10% drift)");
    } else {
        println!("⚠ Significant energy drift - consider smaller timestep");
    }

    println!();
    println!("Completed {} steps in {:.2} years", num_steps, time / YEAR);
    println!();
}
