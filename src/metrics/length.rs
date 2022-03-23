//! Helper functions for dealing with lengths in `ugly`'s metric space.

/// Type of pixel lengths (and deltas on lengths).
///
/// We keep both lengths and deltas signed to avoid needing to do a lot of type conversion.
pub type Length = i32;

/// Const-able maximum between two [Length]s.
///
/// # Example
///
/// ```
/// use ugly::metrics::length::max;
/// assert_eq!(42, max(24, 42));
/// assert_eq!(42, max(42, 24));
/// ```
#[must_use]
pub const fn max(x: Length, y: Length) -> Length {
    if x < y {
        y
    } else {
        x
    }
}

/// Clamps a [Length] to 0 if it is negative.
///
/// # Example
///
/// ```
/// use ugly::metrics::length::clamp;
/// assert_eq!(5, clamp(5));
/// assert_eq!(0, clamp(0));
/// assert_eq!(0, clamp(-5));
/// ```
#[must_use]
pub const fn clamp(l: Length) -> Length {
    max(l, 0)
}
