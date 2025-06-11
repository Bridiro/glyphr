//! # utils.rs
//!
//! Utility functions for float operations, used primarily for SDF rendering.
//!
//! Includes a custom `floor` implementation, `smoothstep`, and `mix` functions.

/// Trait extension for custom floor operation on `f32`.
///
/// This version ensures correct flooring for negative values.
pub trait ExtFloor {
    /// Returns the largest integer less than or equal to the number.
    ///
    /// # Examples
    /// ```
    /// use glyphr::utils::ExtFloor;
    /// assert_eq!(1.9f32.floor(), 1.0);
    /// assert_eq!((-1.1f32).floor(), -2.0);
    /// ```
    fn floor(self) -> f32;
}

impl ExtFloor for f32 {
    fn floor(self) -> f32 {
        let mut xi = self as i32;
        if self < 0.0 && self != xi as f32 {
            xi -= 1;
        }
        xi as f32
    }
}

/// Performs a smooth Hermite interpolation between 0 and 1.
///
/// # Examples
/// ```
/// use glyphr::utils::smoothstep;
/// let result = smoothstep(0.0, 1.0, 0.5);
/// assert!((result - 0.5).abs() < 1e-6);
/// ```
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Linearly interpolates between two values.
///
/// # Examples
/// ```
/// use glyphr::utils::mix;
/// assert_eq!(mix(10.0, 20.0, 0.5), 15.0);
/// ```
pub fn mix(v1: f32, v2: f32, weight: f32) -> f32 {
    v1 + (v2 - v1) * weight
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_ext_floor_behavior() {
        assert_eq!(1.9f32.floor(), 1.0);
        assert_eq!((-1.1f32).floor(), -2.0);
        assert_eq!(0.0f32.floor(), 0.0);
        assert_eq!((-0.999f32).floor(), -1.0);
    }

    #[test]
    fn test_smoothstep_behavior() {
        assert_eq!(super::smoothstep(0.0, 1.0, -1.0), 0.0);
        assert_eq!(super::smoothstep(0.0, 1.0, 0.0), 0.0);
        assert_eq!(super::smoothstep(0.0, 1.0, 0.5), 0.5);
        assert_eq!(super::smoothstep(0.0, 1.0, 1.0), 1.0);
        assert_eq!(super::smoothstep(0.0, 1.0, 2.0), 1.0);
    }

    #[test]
    fn test_mix_behavior() {
        assert_eq!(super::mix(0.0, 10.0, 0.0), 0.0);
        assert_eq!(super::mix(0.0, 10.0, 0.5), 5.0);
        assert_eq!(super::mix(0.0, 10.0, 1.0), 10.0);
    }
}

