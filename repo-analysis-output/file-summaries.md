# File Summaries

Heuristic summaries of source files based on filenames, extensions, and paths.

Schema Version: 2.0

Total files: 22

## physics-engine/benches/integration.rs
**Language:** Rust  
**Role:** implementation  
**Role Justification:** general implementation file (default classification)  
**Size:** 9.90 KB  
**LOC:** 204  
**TODOs/FIXMEs:** 0  
**Declarations:** 10  
**Top-level declarations:**
  - fn new
  - fn compute_force
  - fn name
  - fn setup_harmonic_oscillator
  - fn bench_integrator_throughput
  - fn bench_integrator_accuracy
  - fn bench_free_motion
  - struct SpringForce
  - impl SpringForce
  - impl ForceProvider

## physics-engine/examples/basic.rs
**Language:** Rust  
**Role:** implementation  
**Role Justification:** general implementation file (default classification)  
**Size:** 4.43 KB  
**LOC:** 81  
**TODOs/FIXMEs:** 0  
**Declarations:** 9  
**Top-level declarations:**
  - fn run
  - fn name
  - fn main
  - struct Position
  - struct Velocity
  - struct PhysicsSystem
  - impl Component
  - impl Component
  - impl System

## physics-engine/examples/particle_collision.rs
**Language:** Rust  
**Role:** implementation  
**Role Justification:** general implementation file (default classification)  
**Size:** 20.27 KB  
**LOC:** 431  
**TODOs/FIXMEs:** 0  
**Declarations:** 22  
**Top-level declarations:**
  - fn new
  - fn next_u64
  - fn next_f64
  - fn next_f64_range
  - fn default
  - fn create_particles
  - fn calculate_kinetic_energy
  - fn calculate_center_of_mass
  - fn calculate_spread
  - fn print_state
  - ... and 12 more

## physics-engine/examples/solar_system.rs
**Language:** Rust  
**Role:** implementation  
**Role Justification:** general implementation file (default classification)  
**Size:** 18.36 KB  
**LOC:** 392  
**TODOs/FIXMEs:** 0  
**Declarations:** 19  
**Top-level declarations:**
  - fn default
  - fn create_solar_system
  - fn calculate_kinetic_energy
  - fn calculate_potential_energy
  - fn print_state
  - fn print_diagnostics
  - fn main
  - fn name
  - fn integrate
  - struct CelestialBody
  - ... and 9 more

## physics-engine/src/ecs/component.rs
**Language:** Rust  
**Role:** component  
**Role Justification:** filename contains 'component'  
**Size:** 4.07 KB  
**LOC:** 71  
**TODOs/FIXMEs:** 0  
**Declarations:** 24  
**Top-level declarations:**
  - fn type_id
  - fn insert
  - fn remove
  - fn get
  - fn get_mut
  - fn contains
  - fn clear
  - fn new
  - fn default
  - fn insert
  - ... and 14 more

## physics-engine/src/ecs/components.rs
**Language:** Rust  
**Role:** component  
**Role Justification:** filename contains 'component'  
**Size:** 14.43 KB  
**LOC:** 318  
**TODOs/FIXMEs:** 0  
**Declarations:** 82  
**Top-level declarations:**
  - fn new
  - fn zero
  - fn x
  - fn y
  - fn z
  - fn set_x
  - fn set_y
  - fn set_z
  - fn is_valid
  - fn as_array
  - ... and 72 more

## physics-engine/src/ecs/entity.rs
**Language:** Rust  
**Role:** implementation  
**Role Justification:** general implementation file (default classification)  
**Size:** 2.39 KB  
**LOC:** 53  
**TODOs/FIXMEs:** 0  
**Declarations:** 15  
**Top-level declarations:**
  - fn new
  - fn raw
  - fn fmt
  - fn new
  - fn id
  - fn generation
  - fn fmt
  - fn test_entity_creation
  - fn test_entity_equality
  - struct EntityId
  - ... and 5 more

## physics-engine/src/ecs/mod.rs
**Language:** Rust  
**Role:** module-init  
**Role Justification:** module initialization file 'mod'  
**Size:** 1.71 KB  
**LOC:** 24  
**TODOs/FIXMEs:** 0  
**Declarations:** 2  
**Top-level declarations:**
  - fn test_world_creation
  - fn test_entity_creation

## physics-engine/src/ecs/scheduler.rs
**Language:** Rust  
**Role:** implementation  
**Role Justification:** general implementation file (default classification)  
**Size:** 10.18 KB  
**LOC:** 167  
**TODOs/FIXMEs:** 0  
**Declarations:** 38  
**Top-level declarations:**
  - fn new
  - fn new
  - fn with_stages
  - fn add_system
  - fn add_system_default
  - fn system_count
  - fn stage_count
  - fn run_sequential
  - fn run_parallel
  - fn run_parallel
  - ... and 28 more

## physics-engine/src/ecs/system.rs
**Language:** Rust  
**Role:** implementation  
**Role Justification:** general implementation file (default classification)  
**Size:** 3.72 KB  
**LOC:** 60  
**TODOs/FIXMEs:** 1  
**Declarations:** 18  
**Top-level declarations:**
  - fn run
  - fn name
  - fn new
  - fn add_system
  - fn run_sequential
  - fn run_parallel
  - fn run_parallel
  - fn system_count
  - fn default
  - fn run
  - ... and 8 more

## physics-engine/src/ecs/systems.rs
**Language:** Rust  
**Role:** implementation  
**Role Justification:** general implementation file (default classification)  
**Size:** 18.66 KB  
**LOC:** 385  
**TODOs/FIXMEs:** 0  
**Declarations:** 38  
**Top-level declarations:**
  - fn new
  - fn zero
  - fn is_valid
  - fn add
  - fn magnitude
  - fn compute_force
  - fn name
  - fn new
  - fn register_provider
  - fn clear_forces
  - ... and 28 more

## physics-engine/src/ecs/world.rs
**Language:** Rust  
**Role:** implementation  
**Role Justification:** general implementation file (default classification)  
**Size:** 5.34 KB  
**LOC:** 113  
**TODOs/FIXMEs:** 0  
**Declarations:** 15  
**Top-level declarations:**
  - fn new
  - fn create_entity
  - fn destroy_entity
  - fn is_entity_alive
  - fn entity_count
  - fn clear
  - fn entities
  - fn default
  - fn test_world_entity_lifecycle
  - fn test_entity_generation
  - ... and 5 more

## physics-engine/src/integration/mod.rs
**Language:** Rust  
**Role:** module-init  
**Role Justification:** module initialization file 'mod'  
**Size:** 6.96 KB  
**LOC:** 92  
**TODOs/FIXMEs:** 0  
**Declarations:** 15  
**Top-level declarations:**
  - fn name
  - fn timestep
  - fn set_timestep
  - fn validate_timestep
  - fn integrate
  - fn new
  - fn omega
  - fn amplitude
  - fn phase
  - fn position_at
  - ... and 5 more

## physics-engine/src/integration/rk4.rs
**Language:** Rust  
**Role:** implementation  
**Role Justification:** general implementation file (default classification)  
**Size:** 19.34 KB  
**LOC:** 419  
**TODOs/FIXMEs:** 0  
**Declarations:** 15  
**Top-level declarations:**
  - fn new
  - fn clear_buffers
  - fn compute_derivative
  - fn name
  - fn timestep
  - fn set_timestep
  - fn integrate
  - fn test_rk4_creation
  - fn test_rk4_invalid_timestep
  - fn test_rk4_set_timestep
  - ... and 5 more

## physics-engine/src/integration/verlet.rs
**Language:** Rust  
**Role:** implementation  
**Role Justification:** general implementation file (default classification)  
**Size:** 12.70 KB  
**LOC:** 222  
**TODOs/FIXMEs:** 0  
**Declarations:** 20  
**Top-level declarations:**
  - fn new
  - fn name
  - fn timestep
  - fn set_timestep
  - fn integrate
  - fn compute_force
  - fn name
  - fn test_verlet_creation
  - fn test_verlet_invalid_timestep
  - fn test_verlet_negative_timestep
  - ... and 10 more

## physics-engine/src/lib.rs
**Language:** Rust  
**Role:** implementation  
**Role Justification:** general implementation file (default classification)  
**Size:** 1.72 KB  
**LOC:** 4  
**TODOs/FIXMEs:** 0  

## physics-engine/src/plugins/api.rs
**Language:** Rust  
**Role:** api  
**Role Justification:** filename contains 'api'  
**Size:** 13.09 KB  
**LOC:** 168  
**TODOs/FIXMEs:** 0  
**Declarations:** 41  
**Top-level declarations:**
  - fn new
  - fn world
  - fn integrator_name
  - fn timestep
  - fn thread_count
  - fn is_parallel_enabled
  - fn name
  - fn version
  - fn api_version
  - fn dependencies
  - ... and 31 more

## physics-engine/src/plugins/gravity.rs
**Language:** Rust  
**Role:** implementation  
**Role Justification:** general implementation file (default classification)  
**Size:** 22.19 KB  
**LOC:** 356  
**TODOs/FIXMEs:** 0  
**Declarations:** 45  
**Top-level declarations:**
  - fn new
  - fn with_scaled_g
  - fn default_settings
  - fn set_softening
  - fn softening
  - fn set_chunk_size
  - fn set_warn_on_invalid
  - fn compute_pairwise_force
  - fn compute_force_for_entity
  - fn name
  - ... and 35 more

## physics-engine/src/plugins/mod.rs
**Language:** Rust  
**Role:** module-init  
**Role Justification:** module initialization file 'mod'  
**Size:** 7.07 KB  
**LOC:** 15  
**TODOs/FIXMEs:** 0  
**Declarations:** 1  
**Top-level declarations:**
  - fn test_module_exports

## physics-engine/src/plugins/registry.rs
**Language:** Rust  
**Role:** implementation  
**Role Justification:** general implementation file (default classification)  
**Size:** 22.99 KB  
**LOC:** 420  
**TODOs/FIXMEs:** 0  
**Declarations:** 42  
**Top-level declarations:**
  - fn new
  - fn register
  - fn discover_plugins
  - fn initialize_all
  - fn update_all
  - fn shutdown_all
  - fn get
  - fn get_mut
  - fn plugin_count
  - fn is_initialized
  - ... and 32 more

## physics-engine/tests/conservation.rs
**Language:** Rust  
**Role:** implementation  
**Role Justification:** general implementation file (default classification)  
**Size:** 12.79 KB  
**LOC:** 291  
**TODOs/FIXMEs:** 0  
**Declarations:** 16  
**Top-level declarations:**
  - fn compute_force
  - fn name
  - fn compute_energy
  - fn test_verlet_energy_conservation_free_particle
  - fn test_rk4_energy_conservation_free_particle
  - fn test_verlet_position_accuracy
  - fn test_rk4_position_accuracy
  - fn test_verlet_constant_acceleration
  - fn compute_force
  - fn name
  - ... and 6 more

## physics-engine/tests/integration_failures.rs
**Language:** Rust  
**Role:** implementation  
**Role Justification:** general implementation file (default classification)  
**Size:** 13.62 KB  
**LOC:** 289  
**TODOs/FIXMEs:** 0  
**Declarations:** 9  
**Top-level declarations:**
  - fn compute_force
  - fn name
  - fn calculate_potential_energy_two_body
  - fn test_verlet_kinetic_energy_changes_under_constant_force
  - fn test_rk4_kinetic_energy_changes_under_constant_force
  - fn test_verlet_circular_orbit_stability
  - fn test_verlet_energy_conservation_gravity
  - struct ConstantForce
  - impl ForceProvider
