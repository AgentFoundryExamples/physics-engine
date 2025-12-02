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
//! Newtonian physics components
//!
//! This module provides components for modeling physical entities with
//! Newtonian mechanics: position, velocity, acceleration, and mass.
//! Components use SIMD-friendly representations with double-precision
//! floats for accuracy in physics simulations.

use crate::ecs::Component;

/// 3D position component with double-precision coordinates
///
/// Represents the position of an entity in 3D space using double-precision
/// floats for high accuracy. The data layout is optimized for SIMD operations
/// with 8-byte aligned fields that can be processed in parallel.
///
/// # Examples
///
/// ```
/// use physics_engine::ecs::components::Position;
///
/// let pos = Position::new(1.0, 2.0, 3.0);
/// assert_eq!(pos.x(), 1.0);
/// assert!(pos.is_valid());
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    x: f64,
    y: f64,
    z: f64,
}

impl Position {
    /// Create a new position with the given coordinates
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Position { x, y, z }
    }

    /// Create a position at the origin (0, 0, 0)
    pub fn zero() -> Self {
        Position::new(0.0, 0.0, 0.0)
    }

    /// Get the x coordinate
    pub fn x(&self) -> f64 {
        self.x
    }

    /// Get the y coordinate
    pub fn y(&self) -> f64 {
        self.y
    }

    /// Get the z coordinate
    pub fn z(&self) -> f64 {
        self.z
    }

    /// Set the x coordinate
    pub fn set_x(&mut self, x: f64) {
        self.x = x;
    }

    /// Set the y coordinate
    pub fn set_y(&mut self, y: f64) {
        self.y = y;
    }

    /// Set the z coordinate
    pub fn set_z(&mut self, z: f64) {
        self.z = z;
    }

    /// Check if all coordinates are finite (not NaN or infinite)
    pub fn is_valid(&self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    /// Get the position as an array
    pub fn as_array(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }

    /// Create a position from an array
    pub fn from_array(arr: [f64; 3]) -> Self {
        Position::new(arr[0], arr[1], arr[2])
    }
}

impl Component for Position {}

impl Default for Position {
    fn default() -> Self {
        Position::zero()
    }
}

/// 3D velocity component with double-precision values
///
/// Represents the rate of change of position over time in meters per second.
/// Uses double-precision floats and SIMD-friendly layout.
///
/// # Examples
///
/// ```
/// use physics_engine::ecs::components::Velocity;
///
/// let vel = Velocity::new(10.0, 0.0, -5.0);
/// assert!(vel.is_valid());
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Velocity {
    dx: f64,
    dy: f64,
    dz: f64,
}

impl Velocity {
    /// Create a new velocity with the given components
    pub fn new(dx: f64, dy: f64, dz: f64) -> Self {
        Velocity { dx, dy, dz }
    }

    /// Create a zero velocity (at rest)
    pub fn zero() -> Self {
        Velocity::new(0.0, 0.0, 0.0)
    }

    /// Get the x component
    pub fn dx(&self) -> f64 {
        self.dx
    }

    /// Get the y component
    pub fn dy(&self) -> f64 {
        self.dy
    }

    /// Get the z component
    pub fn dz(&self) -> f64 {
        self.dz
    }

    /// Set the x component
    pub fn set_dx(&mut self, dx: f64) {
        self.dx = dx;
    }

    /// Set the y component
    pub fn set_dy(&mut self, dy: f64) {
        self.dy = dy;
    }

    /// Set the z component
    pub fn set_dz(&mut self, dz: f64) {
        self.dz = dz;
    }

    /// Check if all components are finite (not NaN or infinite)
    pub fn is_valid(&self) -> bool {
        self.dx.is_finite() && self.dy.is_finite() && self.dz.is_finite()
    }

    /// Get the velocity as an array
    pub fn as_array(&self) -> [f64; 3] {
        [self.dx, self.dy, self.dz]
    }

    /// Create a velocity from an array
    pub fn from_array(arr: [f64; 3]) -> Self {
        Velocity::new(arr[0], arr[1], arr[2])
    }

    /// Calculate the magnitude (speed) of the velocity vector
    pub fn magnitude(&self) -> f64 {
        (self.dx * self.dx + self.dy * self.dy + self.dz * self.dz).sqrt()
    }
}

impl Component for Velocity {}

impl Default for Velocity {
    fn default() -> Self {
        Velocity::zero()
    }
}

/// 3D acceleration component with double-precision values
///
/// Represents the rate of change of velocity over time in meters per second squared.
/// Typically computed by force accumulation systems based on F = ma.
///
/// # Examples
///
/// ```
/// use physics_engine::ecs::components::Acceleration;
///
/// let acc = Acceleration::new(0.0, -9.81, 0.0); // Gravity
/// assert!(acc.is_valid());
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Acceleration {
    ax: f64,
    ay: f64,
    az: f64,
}

impl Acceleration {
    /// Create a new acceleration with the given components
    pub fn new(ax: f64, ay: f64, az: f64) -> Self {
        Acceleration { ax, ay, az }
    }

    /// Create a zero acceleration
    pub fn zero() -> Self {
        Acceleration::new(0.0, 0.0, 0.0)
    }

    /// Get the x component
    pub fn ax(&self) -> f64 {
        self.ax
    }

    /// Get the y component
    pub fn ay(&self) -> f64 {
        self.ay
    }

    /// Get the z component
    pub fn az(&self) -> f64 {
        self.az
    }

    /// Set the x component
    pub fn set_ax(&mut self, ax: f64) {
        self.ax = ax;
    }

    /// Set the y component
    pub fn set_ay(&mut self, ay: f64) {
        self.ay = ay;
    }

    /// Set the z component
    pub fn set_az(&mut self, az: f64) {
        self.az = az;
    }

    /// Check if all components are finite (not NaN or infinite)
    pub fn is_valid(&self) -> bool {
        self.ax.is_finite() && self.ay.is_finite() && self.az.is_finite()
    }

    /// Get the acceleration as an array
    pub fn as_array(&self) -> [f64; 3] {
        [self.ax, self.ay, self.az]
    }

    /// Create an acceleration from an array
    pub fn from_array(arr: [f64; 3]) -> Self {
        Acceleration::new(arr[0], arr[1], arr[2])
    }
}

impl Component for Acceleration {}

impl Default for Acceleration {
    fn default() -> Self {
        Acceleration::zero()
    }
}

/// Mass component with double-precision value
///
/// Represents the mass of an entity in kilograms. Special handling is provided
/// for zero or near-zero mass values to prevent division-by-zero errors, treating
/// such entities as immovable bodies.
///
/// # Examples
///
/// ```
/// use physics_engine::ecs::components::Mass;
///
/// let mass = Mass::new(10.5);
/// assert!(mass.is_valid());
/// assert!(!mass.is_immovable());
///
/// let immovable = Mass::immovable();
/// assert!(immovable.is_immovable());
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mass {
    value: f64,
}

impl Mass {
    /// Threshold below which mass is considered effectively zero (immovable)
    pub const IMMOVABLE_THRESHOLD: f64 = 1e-10;

    /// Create a new mass with the given value in kilograms
    ///
    /// # Panics
    ///
    /// Panics if the mass is negative or NaN. This is appropriate for programming
    /// errors where invalid data should not be constructed. For fallible construction,
    /// use `try_new`.
    pub fn new(value: f64) -> Self {
        assert!(value >= 0.0 && value.is_finite(), "Mass must be non-negative and finite");
        Mass { value }
    }

    /// Try to create a new mass with the given value in kilograms
    ///
    /// Returns `None` if the value is negative or NaN.
    pub fn try_new(value: f64) -> Option<Self> {
        if value >= 0.0 && value.is_finite() {
            Some(Mass { value })
        } else {
            None
        }
    }

    /// Create an immovable mass (treated as infinite mass)
    pub fn immovable() -> Self {
        Mass { value: 0.0 }
    }

    /// Get the mass value
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Set the mass value
    ///
    /// # Panics
    ///
    /// Panics if the mass is negative or NaN.
    pub fn set_value(&mut self, value: f64) {
        assert!(value >= 0.0 && value.is_finite(), "Mass must be non-negative and finite");
        self.value = value;
    }

    /// Check if the mass is valid (non-negative and finite)
    pub fn is_valid(&self) -> bool {
        self.value >= 0.0 && self.value.is_finite()
    }

    /// Check if this is an immovable body (zero or near-zero mass)
    pub fn is_immovable(&self) -> bool {
        self.value < Self::IMMOVABLE_THRESHOLD
    }

    /// Get the inverse mass (1/m) for use in calculations
    ///
    /// Returns 0.0 for immovable bodies to prevent division by zero.
    pub fn inverse(&self) -> f64 {
        if self.is_immovable() {
            0.0
        } else {
            1.0 / self.value
        }
    }
}

impl Component for Mass {}

impl Default for Mass {
    fn default() -> Self {
        Mass::new(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let pos = Position::new(1.0, 2.0, 3.0);
        assert_eq!(pos.x(), 1.0);
        assert_eq!(pos.y(), 2.0);
        assert_eq!(pos.z(), 3.0);
    }

    #[test]
    fn test_position_zero() {
        let pos = Position::zero();
        assert_eq!(pos.x(), 0.0);
        assert_eq!(pos.y(), 0.0);
        assert_eq!(pos.z(), 0.0);
    }

    #[test]
    fn test_position_validation() {
        let valid = Position::new(1.0, 2.0, 3.0);
        assert!(valid.is_valid());

        let invalid = Position::new(f64::NAN, 2.0, 3.0);
        assert!(!invalid.is_valid());

        let infinite = Position::new(f64::INFINITY, 2.0, 3.0);
        assert!(!infinite.is_valid());
    }

    #[test]
    fn test_position_array_conversion() {
        let pos = Position::new(1.0, 2.0, 3.0);
        let arr = pos.as_array();
        assert_eq!(arr, [1.0, 2.0, 3.0]);

        let pos2 = Position::from_array([4.0, 5.0, 6.0]);
        assert_eq!(pos2.x(), 4.0);
        assert_eq!(pos2.y(), 5.0);
        assert_eq!(pos2.z(), 6.0);
    }

    #[test]
    fn test_velocity_creation() {
        let vel = Velocity::new(10.0, 20.0, 30.0);
        assert_eq!(vel.dx(), 10.0);
        assert_eq!(vel.dy(), 20.0);
        assert_eq!(vel.dz(), 30.0);
    }

    #[test]
    fn test_velocity_magnitude() {
        let vel = Velocity::new(3.0, 4.0, 0.0);
        assert_eq!(vel.magnitude(), 5.0); // 3-4-5 triangle
    }

    #[test]
    fn test_velocity_validation() {
        let valid = Velocity::new(1.0, 2.0, 3.0);
        assert!(valid.is_valid());

        let invalid = Velocity::new(f64::NAN, 2.0, 3.0);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_acceleration_creation() {
        let acc = Acceleration::new(0.0, -9.81, 0.0);
        assert_eq!(acc.ax(), 0.0);
        assert_eq!(acc.ay(), -9.81);
        assert_eq!(acc.az(), 0.0);
    }

    #[test]
    fn test_acceleration_validation() {
        let valid = Acceleration::new(1.0, 2.0, 3.0);
        assert!(valid.is_valid());

        let invalid = Acceleration::new(f64::INFINITY, 2.0, 3.0);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_mass_creation() {
        let mass = Mass::new(10.5);
        assert_eq!(mass.value(), 10.5);
        assert!(mass.is_valid());
    }

    #[test]
    fn test_mass_try_new() {
        let valid = Mass::try_new(10.5);
        assert!(valid.is_some());
        assert_eq!(valid.unwrap().value(), 10.5);

        let negative = Mass::try_new(-1.0);
        assert!(negative.is_none());

        let nan = Mass::try_new(f64::NAN);
        assert!(nan.is_none());

        let inf = Mass::try_new(f64::INFINITY);
        assert!(inf.is_none());
    }

    #[test]
    #[should_panic(expected = "Mass must be non-negative and finite")]
    fn test_mass_negative_panics() {
        Mass::new(-1.0);
    }

    #[test]
    #[should_panic(expected = "Mass must be non-negative and finite")]
    fn test_mass_nan_panics() {
        Mass::new(f64::NAN);
    }

    #[test]
    fn test_mass_immovable() {
        let immovable = Mass::immovable();
        assert!(immovable.is_immovable());
        assert_eq!(immovable.inverse(), 0.0);

        let near_zero = Mass::new(1e-15);
        assert!(near_zero.is_immovable());
        assert_eq!(near_zero.inverse(), 0.0);
    }

    #[test]
    fn test_mass_inverse() {
        let mass = Mass::new(2.0);
        assert_eq!(mass.inverse(), 0.5);

        let large_mass = Mass::new(100.0);
        assert_eq!(large_mass.inverse(), 0.01);
    }

    #[test]
    fn test_mass_zero_handling() {
        let zero_mass = Mass::new(0.0);
        assert!(zero_mass.is_immovable());
        assert_eq!(zero_mass.inverse(), 0.0); // Should not divide by zero
    }

    #[test]
    fn test_component_defaults() {
        let pos: Position = Default::default();
        assert_eq!(pos, Position::zero());

        let vel: Velocity = Default::default();
        assert_eq!(vel, Velocity::zero());

        let acc: Acceleration = Default::default();
        assert_eq!(acc, Acceleration::zero());

        let mass: Mass = Default::default();
        assert_eq!(mass.value(), 1.0);
    }
}
