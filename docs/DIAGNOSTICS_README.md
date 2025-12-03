# Integration Audit - Diagnostic Tools and Methodology

This directory contains diagnostic tools and documentation for auditing the physics engine integrators and reproducing simulation failures.

## Quick Links

- **[FAILURE_ANALYSIS.md](FAILURE_ANALYSIS.md)** - Comprehensive failure documentation with measurements, hypotheses, and analysis
- **[performance.md](performance.md#critical-known-issues-version-010)** - Known issues section for users
- **[examples.md](examples.md#known-issues-version-010)** - Usage warnings and limitations

## Running Diagnostics

### Solar System Simulation

Generate detailed CSV diagnostics for the solar system simulation:

```bash
# Short run with diagnostics (0.1 years)
cargo run --release --example solar_system -- --diagnostics --years 0.1

# Full year with diagnostics (generates ~37 diagnostic lines)
cargo run --release --example solar_system -- --diagnostics --years 1 > solar_diagnostics.csv

# Compare integrators
cargo run --release --example solar_system -- --diagnostics --integrator verlet --years 1 > verlet_diag.csv
cargo run --release --example solar_system -- --diagnostics --integrator rk4 --years 1 > rk4_diag.csv
```

**CSV Format**: `DIAG,step,time_s,dt_s,KE_J,PE_J,E_total_J,drift_frac,earth_AU,earth_v_ms,earth_a_ms2`

**Logging Frequency**: Every 10 steps (prevents output explosion)

### Particle Collision Simulation

Generate diagnostics for particle systems:

```bash
# Standard run with diagnostics
cargo run --release --example particle_collision -- --diagnostics --duration 5

# With specific seed for reproducibility
cargo run --release --example particle_collision -- --diagnostics --seed 12345 --duration 10 > particle_diag.csv

# Vary particle count
cargo run --release --example particle_collision -- --diagnostics --particles 50 --duration 5
```

**CSV Format**: `DIAG,step,time_s,dt_s,KE_J,ke_change_frac,cm_x_m,cm_y_m,cm_z_m,spread_m`

**Logging Frequency**: Every 50 steps

## Running Regression Tests

The failing behavior is captured in regression tests marked with `#[ignore]`:

```bash
# Run all failing tests
cargo test --test integration_failures -- --ignored

# Run specific test
cargo test --test integration_failures test_verlet_circular_orbit_stability -- --ignored

# Show output for investigation
cargo test --test integration_failures -- --ignored --nocapture
```

**Test Suite** (`tests/integration_failures.rs`):
- `test_verlet_kinetic_energy_changes_under_constant_force` - KE should increase under constant force
- `test_rk4_kinetic_energy_changes_under_constant_force` - Same for RK4
- `test_verlet_circular_orbit_stability` - Circular orbits should remain stable
- `test_verlet_energy_conservation_gravity` - Energy should be conserved in gravity simulation

Once integrators are fixed, remove `#[ignore]` attributes and these tests will serve as validation.

## Diagnostic Data Analysis

### Analyzing Solar System Diagnostics

```bash
# Extract diagnostic lines only
grep "^DIAG" solar_diagnostics.csv > data.csv

# Plot energy drift over time (requires Python + matplotlib)
python3 << 'EOF'
import matplotlib.pyplot as plt
import numpy as np

data = np.loadtxt('data.csv', delimiter=',', usecols=(2,7), skiprows=0)
time_years = data[:,0] / 31557600.0  # Convert seconds to years
drift_pct = data[:,1] * 100.0

plt.figure(figsize=(10, 6))
plt.plot(time_years, drift_pct)
plt.xlabel('Time (years)')
plt.ylabel('Energy Drift (%)')
plt.title('Solar System Energy Conservation')
plt.grid(True)
plt.savefig('energy_drift.png')
print("Saved energy_drift.png")
EOF
```

### Key Diagnostic Observations

From `solar_diagnostics.csv`:
```
DIAG,0,8.640e4,8.640e4,6.197e33,-1.240e34,-6.202e33,0.0008,1.000,29780.0,0.0
DIAG,10,9.504e5,8.640e4,6.197e33,-1.188e34,-5.687e33,0.0838,1.018,29780.0,0.0
                                 ^^^^^^^^^                            ^^^^^^^ ^^^
                                 KE frozen                            v const a=0
```

**Evidence of bug**:
1. Kinetic energy (column 4): `6.197e33` J - constant throughout
2. Earth velocity (column 9): `29780.0` m/s - never changes
3. Earth acceleration (column 10): `0.0` m/s² - always zero despite gravitational forces
4. Earth distance (column 8): increases from 1.0 AU to 6+ AU - position DOES update

**Conclusion**: Position integration works, velocity integration does not.

## Deterministic Verification

Verify that simulations are deterministic:

```bash
# Run twice with same seed
cargo run --release --example particle_collision -- --seed 42 --duration 1 > run1.txt
cargo run --release --example particle_collision -- --seed 42 --duration 1 > run2.txt
diff run1.txt run2.txt
# Should show no differences

# Different seeds should produce different results
cargo run --release --example particle_collision -- --seed 1 --duration 1 > seed1.txt
cargo run --release --example particle_collision -- --seed 2 --duration 1 > seed2.txt
diff seed1.txt seed2.txt
# Should show differences
```

## Reproducing Published Failures

### Solar System - 175% Energy Drift

```bash
cargo run --release --example solar_system -- --years 1 --timestep 86400
```

Expected output (final state):
```
Energy Conservation:
  Initial Energy: -6.207068e33 J
  Final Energy:   4.648422e33 J
  Relative Drift: 1.748892e0 (174.8892%)

⚠ Significant energy drift - consider smaller timestep
```

Earth should move from 1.0 AU to ~6.4 AU.

### Particle Collision - 217% KE Growth

```bash
cargo run --release --example particle_collision -- --particles 100 --duration 5 --seed 12345
```

Expected final output:
```
Time: 5.00 s
  Kinetic Energy: 9.130e4 J
  
Energy Conservation:
  Initial KE: 2.878e4 J
  Final KE:   9.130e4 J
  Relative Change: 2.172148e0 (217.2148%)
```

## Methodology Documentation

### Test Parameters

All tests use:
- **Deterministic seeds**: Fixed RNG state for reproducibility
- **SI units**: meters, kilograms, seconds
- **Double precision**: f64 throughout
- **Fixed timesteps**: No adaptive timestepping

### Solar System Configuration

- **Bodies**: Sun, Mercury, Venus, Earth, Mars (5 bodies)
- **Initial conditions**: Circular orbits at semi-major axis distances
- **Gravitational constant**: G = 6.67430 × 10⁻¹¹ m³/(kg⋅s²)
- **Softening**: ε = 1.0 m (DEFAULT_SOFTENING)
- **Default timestep**: 86400 s (1 day)
- **Default duration**: 1 Earth year (365.25 days)

### Particle Collision Configuration

- **Particles**: 100 (configurable)
- **Mass range**: 1-10 kg (uniform distribution)
- **Position range**: ±100 m cube
- **Velocity range**: ±10 m/s
- **G scale**: 1×10¹⁰ (enhanced for visible effects)
- **Softening**: 1.0 m
- **Timestep**: 0.01 s
- **Duration**: 10 s (default)
- **Default seed**: 12345

### Expected Behavior (Literature)

For solar system with dt = 1 day:
- **Verlet symplectic error**: O(dt²) ≈ 1.7×10⁻⁷ per orbit
- **Expected energy drift**: < 0.01% per year
- **Orbital radius variation**: < 1% per year

References:
- Hairer et al. (2006). *Geometric Numerical Integration*. Springer.
- Aarseth (2003). *Gravitational N-Body Simulations*. Cambridge University Press.

### Acceptance Criteria

Once integrators are fixed, expect:

**Energy Conservation**:
- Excellent: < 0.1% drift
- Good: < 1% drift
- Acceptable: < 10% drift
- Current (FAILING): 175% drift ❌

**Orbital Stability**:
- Excellent: < 1% radius variation
- Good: < 5% variation
- Acceptable: < 10% variation
- Current (FAILING): 540% variation ❌

## Next Steps

1. **Investigation**: Use diagnostic data to confirm root cause hypothesis
2. **Fix Implementation**: Correct acceleration → velocity integration
3. **Validation**: Run regression tests (should pass after fix)
4. **Re-benchmark**: Verify performance not degraded
5. **Documentation**: Update FAILURE_ANALYSIS.md with resolution

## Files Added

- `docs/FAILURE_ANALYSIS.md` - Comprehensive technical failure analysis
- `docs/DIAGNOSTICS_README.md` - This file
- `physics-engine/tests/integration_failures.rs` - Regression test suite
- Enhanced `physics-engine/examples/solar_system.rs` with --diagnostics flag
- Enhanced `physics-engine/examples/particle_collision.rs` with --diagnostics flag
- Updated `docs/performance.md` with critical known issues section
- Updated `docs/examples.md` with known issues and warnings

## Files Modified

All changes preserve original functionality while adding diagnostic capabilities. No fixes attempted (per requirements).

---

**Last Updated**: 2025-12-03  
**Status**: Investigation phase complete, awaiting remediation  
**Version**: 0.1.0 (with known issues documented)
