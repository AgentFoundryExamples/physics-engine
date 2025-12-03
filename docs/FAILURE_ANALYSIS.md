# Integrator Failure Analysis

**Date**: 2025-12-03  
**Version**: 0.1.0  
**Status**: Investigation Phase - No Fixes Applied

## Executive Summary

This document captures the precise failure modes observed in both Velocity Verlet and RK4 integrators before code remediation. Simulation runs demonstrate massive energy drift and runaway trajectories in orbital mechanics scenarios. These failures are reproducible and documented here to guide future fixes.

## Methodology

### Test Environment

- **Hardware**: AMD EPYC 7763 64-Core Processor (x86_64)
- **OS**: Linux
- **Rust**: 1.70+ (2021 edition)
- **Build**: `--release` mode with optimizations enabled
- **Features**: `parallel` feature enabled (Rayon)

### Reproducibility

All tests use:
- **Deterministic seeds**: particle_collision uses seed=12345 by default
- **Fixed timesteps**: No adaptive timestepping
- **SI units**: meters, kilograms, seconds
- **Double precision**: f64 throughout

### Test Parameters

#### Solar System Simulation
- **Bodies**: Sun, Mercury, Venus, Earth, Mars
- **Initial Conditions**: Circular orbits at semi-major axis distances
- **Timestep**: 86400 seconds (1 day)
- **Duration**: 1 Earth year (365.25 days, 366 steps)
- **Gravitational Constant**: G = 6.67430 × 10⁻¹¹ m³/(kg⋅s²)
- **Softening Parameter**: ε = 1.0 m (DEFAULT_SOFTENING)

#### Particle Collision Simulation
- **Particles**: 100 bodies
- **Mass Range**: 1-10 kg
- **Position Range**: ±100 m cube
- **Velocity Range**: ±10 m/s
- **Timestep**: 0.01 seconds
- **Duration**: 5 seconds (500 steps)
- **G Scale**: 1×10¹⁰ (artificial enhancement for visible effects)
- **Softening**: 1.0 m
- **Seed**: 12345

## Observed Failures

### 1. Solar System: Massive Energy Drift

#### Velocity Verlet Integrator

**Command**: `target/release/examples/solar_system --years 1 --timestep 86400 --integrator verlet`

**Initial State (t=0)**:
```
Kinetic Energy:   6.196682e33 J
Potential Energy: -1.240375e34 J
Total Energy:     -6.207068e33 J
Earth distance:   1.496e11 m (1.000 AU)
```

**Progression**:
| Time (years) | KE (J) | PE (J) | Total Energy (J) | Earth Distance (AU) |
|--------------|--------|--------|------------------|---------------------|
| 0.00 | 6.197e33 | -1.240e34 | -6.207e33 | 1.000 |
| 0.08 | 6.197e33 | -9.966e33 | -3.769e33 | 1.125 |
| 0.16 | 6.197e33 | -7.245e33 | -1.048e33 | 1.437 |
| 0.25 | 6.197e33 | -5.486e33 | +7.108e32 | 1.843 |
| 1.00 | 6.197e33 | -1.548e33 | +4.648e33 | 6.374 |

**Final State (t=1 year)**:
```
Kinetic Energy:   6.196682e33 J  (UNCHANGED)
Potential Energy: -1.548260e33 J  (87.5% increase)
Total Energy:     4.648422e33 J   (175% drift from initial)
Earth distance:   9.535e11 m (6.374 AU)
```

**Key Observations**:
1. **Kinetic energy remains constant** at 6.197e33 J throughout simulation
2. **Potential energy increases dramatically** from -1.240e34 J to -1.548e33 J
3. **Total energy increases by 175%** (changes sign from negative to positive)
4. **Earth's orbital radius grows 6.4×** from 1.0 AU to 6.4 AU
5. **Energy drift is monotonic and accelerating**

#### RK4 Integrator

**Command**: `target/release/examples/solar_system --years 1 --timestep 86400 --integrator rk4`

**Behavior**: **IDENTICAL** to Velocity Verlet

**Key Observations**:
1. **Both integrators produce identical results** - suggests shared bug
2. **Same kinetic energy freeze** at 6.197e33 J
3. **Same runaway trajectory** - Earth reaches 6.4 AU
4. **Same energy drift pattern** - 175% error

### 2. Particle Collision: Energy Growth

#### Velocity Verlet Integrator

**Command**: `target/release/examples/particle_collision --particles 100 --duration 5 --seed 12345`

**Energy Evolution**:
| Time (s) | Kinetic Energy (J) | Change from t=0 | System Spread (m) |
|----------|-------------------|-----------------|-------------------|
| 0.00 | 2.878e4 | 0% | 158.7 |
| 1.00 | 2.902e4 | +0.8% | 160.0 |
| 2.00 | 3.111e4 | +8.1% | 160.6 |
| 3.01 | 3.901e4 | +35.5% | 159.8 |
| 4.01 | 5.728e4 | +99.0% | 156.9 |
| 5.00 | 9.130e4 | +217.2% | 153.7 |

**Key Observations**:
1. **Kinetic energy grows exponentially** - doubles by t=4s, triples by t=5s
2. **System spread decreases slightly** (clustering) - expected for gravitational attraction
3. **Center of mass drifts** from (10.4, -17.7, 1.7) to (9.5, -15.3, -2.1) m
4. **No momentum conservation** apparent

### 3. Shared Failure Pattern

**Common to Both Scenarios**:
1. ✅ **Deterministic**: Repeated runs with same parameters produce identical results
2. ❌ **Energy conservation violated**: Total energy not conserved
3. ❌ **Orbital instability**: Stable circular orbits become unstable
4. ❌ **Runaway trajectories**: Bodies escape to infinity or unrealistic distances

**Suspected Root Cause**:
- **Kinetic energy freeze in solar system** suggests velocity integration failure
- **Both integrators fail identically** suggests bug in shared force computation or acceleration application
- **Potential energy changes correctly** based on position changes
- **Force warnings** indicate forces are computed (though exceeding arbitrary limits)

## Hypotheses

### Hypothesis 1: Acceleration Not Applied to Velocities

**Evidence**:
- Kinetic energy remains perfectly constant in solar system
- KE = ½mv², so constant KE suggests constant velocity magnitudes
- Positions do change (Earth moves from 1 AU to 6.4 AU)
- This suggests positions are updated but velocities are not

**Test**: Check integrator velocity update logic

### Hypothesis 2: Force Application Bug

**Evidence**:
- Force warnings appear every step
- Forces are computed (evidenced by changing positions and PE)
- Both integrators fail identically
- May be in shared `apply_forces_to_acceleration` or integration step

**Test**: Verify force → acceleration → velocity pipeline

### Hypothesis 3: Timestep Too Large

**Evidence**:
- Solar system timestep (1 day = 86400 s) is very large
- Earth's orbital period ≈ 365 days, so dt ≈ T/365
- Standard guidance: dt < T/20 to T/100
- Current timestep may be 5-20× too large

**Counter-Evidence**:
- Problem manifests immediately (< 0.1 year)
- Kinetic energy should still vary even with large timestep
- RK4 should handle larger timesteps better than Verlet

### Hypothesis 4: Initial Velocity Direction Wrong

**Evidence**:
- Planets initialized with velocity in y-direction
- Positions initialized along x-axis
- For circular orbit: v ⊥ r (perpendicular)
- May be using wrong velocity magnitude or direction

**Test**: Verify circular orbit condition: v = √(GM/r)

## Diagnostic Data Collection

### Required Metrics

For future instrumentation, we need:

1. **Per-Step Diagnostics**:
   - Current timestep (dt)
   - Total kinetic energy
   - Total potential energy
   - Total energy and drift percentage
   - Position of reference body (e.g., Earth)
   - Velocity magnitude of reference body
   - Force magnitude on reference body
   - Acceleration magnitude on reference body

2. **Summary Statistics**:
   - Energy drift: |E_final - E_initial| / |E_initial|
   - Maximum orbital radius change
   - Center of mass drift
   - Momentum conservation: |Σmv|

3. **Integrator-Specific**:
   - Force evaluation count per step
   - Buffer allocation/reuse (RK4)
   - Symplectic error accumulation (Verlet)

### Logging Frequency

To avoid output explosion:
- **Solar system**: Log every 30 days (30 outputs per year)
- **Particle collision**: Log every 1 second (5 outputs for 5s run)
- **Tests**: Log initial, midpoint, final only

## Acceptance Targets

Based on literature and best practices:

### Energy Conservation
- **Excellent**: < 0.1% drift over simulation duration
- **Good**: < 1% drift
- **Acceptable**: < 10% drift
- **FAILING**: > 10% drift ❌ (currently 175%)

### Orbital Stability
- **Excellent**: Orbital radius variation < 1%
- **Good**: < 5% variation
- **Acceptable**: < 10% variation
- **FAILING**: > 100% variation ❌ (currently 540% for Earth)

### Momentum Conservation (for isolated systems)
- **Excellent**: |Δp| / |p_initial| < 1e-12
- **Good**: < 1e-8
- **FAILING**: Not measured ❌

## Next Steps

### Investigation Phase (Current)
- [x] Document baseline failure modes
- [x] Establish reproducible test parameters
- [x] Generate hypotheses
- [ ] Add detailed per-step logging to examples
- [ ] Create regression tests with expected failures
- [ ] Verify force computation correctness
- [ ] Verify acceleration computation
- [ ] Verify velocity integration
- [ ] Verify position integration

### Remediation Phase (Future)
- [ ] Identify root cause from diagnostics
- [ ] Implement minimal fix
- [ ] Verify fix with regression tests
- [ ] Re-run benchmarks
- [ ] Update documentation with findings

## References

### Expected Behavior

For solar system with dt = 1 day:
- **Verlet symplectic error**: O(dt²) ≈ (86400/T_earth)² ≈ 1.7e-7 per orbit
- **Expected drift**: < 0.01% per year for stable integrator
- **Observed drift**: 175% - **10,000× worse than expected**

### Literature

1. Hairer, E., Lubich, C., & Wanner, G. (2006). *Geometric Numerical Integration*. Springer.
   - Verlet expected energy drift: < 1e-4 for dt = T/100

2. Aarseth, S. J. (2003). *Gravitational N-Body Simulations*. Cambridge University Press.
   - N-body timestep recommendation: dt < 0.01 × T_min

3. Dehnen, W. (2001). "Towards optimal softening in three-dimensional N-body codes". *MNRAS*.
   - Softening parameter effects on energy conservation

## Appendix: Full Example Output

### Solar System (Verlet, 1 year, dt=1 day)

```
==========================================================
       Solar System N-Body Simulation
==========================================================

Physical Constants:
  G = 6.67430e-11 m³/(kg⋅s²)
  1 AU = 1.49598e11 m
  1 year = 3.15576e7 s

Simulation Configuration:
  Integrator: verlet
  Timestep: 86400 s (1.00 days)
  Duration: 1.00 years
  Output interval: 30 days

Creating solar system bodies...

Created Sun - Mass: 1.989e30 kg, Distance: 0.000e0 m (0.000 AU), Velocity: 0 m/s
Created Mercury - Mass: 3.301e23 kg, Distance: 5.789e10 m (0.387 AU), Velocity: 47870 m/s
Created Venus - Mass: 4.867e24 kg, Distance: 1.082e11 m (0.723 AU), Velocity: 35020 m/s
Created Earth - Mass: 5.972e24 kg, Distance: 1.496e11 m (1.000 AU), Velocity: 29780 m/s
Created Mars - Mass: 6.417e23 kg, Distance: 2.280e11 m (1.524 AU), Velocity: 24070 m/s

Starting simulation with Velocity Verlet integrator...

=== Time: 0.00 years (0.00e0 s) ===
Kinetic Energy:   6.196682e33 J
Potential Energy: -1.240375e34 J
Total Energy:     -6.207068e33 J
Earth distance from Sun: 1.496e11 m (1.000 AU)

[... many force warnings omitted ...]

=== Time: 1.00 years (3.16e7 s) ===
Kinetic Energy:   6.196682e33 J
Potential Energy: -1.548260e33 J
Total Energy:     4.648422e33 J
Earth distance from Sun: 9.535e11 m (6.374 AU)

Energy Conservation:
  Initial Energy: -6.207068e33 J
  Final Energy:   4.648422e33 J
  Relative Drift: 1.748892e0 (174.8892%)

⚠ Significant energy drift - consider smaller timestep

Completed 366 steps in 1.00 years
```

---

**Document Status**: Living document - to be updated as investigation progresses  
**Last Updated**: 2025-12-03  
**Next Review**: After instrumentation is added to examples
