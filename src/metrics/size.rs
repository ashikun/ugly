//! The [Size] struct and related functionality.

use serde::{Deserialize, Serialize};

use super::length::{self, Length};

/// A two-dimensional size.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Size {
    /// Width in pixels.
    pub w: Length,
    /// Height in pixels.
    pub h: Length,
}

impl Size {
    /// Grows the size in both dimensions by `amount`.
    ///
    /// To shrink, grow by a negative amount.  Neither dimension will shrink past 0.
    ///
    /// ```
    /// use ugly::metrics::Size;
    ///
    /// assert_eq!(Size{w: 42, h: 22}, Size{w: 40, h:20}.grow(2));
    /// assert_eq!(Size{w: 40, h: 20}, Size{w: 42, h:22}.grow(-2));
    /// ```
    #[must_use]
    pub const fn grow(self, amount: Length) -> Self {
        Self {
            w: length::clamp(self.w + amount),
            h: length::clamp(self.h + amount),
        }
    }

    /// Returns a size that is the maximum of `self` and `other` horizontally, and their sum
    /// vertically.
    ///
    /// ```
    /// use ugly::metrics::Size;
    ///
    /// assert_eq!(Size{w: 42, h: 32}, Size{w: 42, h:10}.stack_vertically(Size{w: 20, h: 22}));
    /// ```
    #[must_use]
    pub fn stack_vertically(self, other: Self) -> Self {
        Self {
            w: self.w.max(other.w),
            h: self.h + other.h,
        }
    }

    /// Returns a size that is the maximum of `self` and `other` vertically, and their sum
    /// horizontally.
    ///
    /// ```
    /// use ugly::metrics::Size;
    ///
    /// assert_eq!(Size{w: 62, h: 22}, Size{w: 42, h:10}.stack_horizontally(Size{w: 20, h: 22}));
    /// ```
    #[must_use]
    pub fn stack_horizontally(self, other: Self) -> Self {
        Self {
            w: self.w + other.w,
            h: self.h.max(other.h),
        }
    }

    /// Gets whether either dimension of this size is zero.
    ///
    /// ```
    /// use ugly::metrics::Size;
    ///
    /// assert!(Size{w: 0, h: 10}.is_zero());
    /// assert!(Size{w: 10, h: 0}.is_zero());
    /// assert!(!Size{w: 10, h: 10}.is_zero());
    /// ```
    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.w <= 0 || self.h <= 0
    }

    /// Gets whether both dimensions of this size are non-negative.
    ///
    /// ```
    /// use ugly::metrics::Size;
    ///
    /// assert!(Size{w: 10, h: 10}.is_normal());
    /// assert!(Size{w: 5, h: 0}.is_normal());
    /// assert!(!Size{w: -5, h: 10}.is_normal());
    /// ```
    #[must_use]
    pub const fn is_normal(&self) -> bool {
        0 <= self.w && 0 <= self.h
    }

    /// Clamps negative dimensions to zero.
    ///
    /// # Example
    ///
    /// ```
    /// use ugly::metrics::Size;
    ///
    /// assert!(Size{w: -5, h: 10}.clamp().is_normal())
    /// ```
    #[must_use]
    pub const fn clamp(&self) -> Self {
        // `grow` clamps as a side-effect, so growing by 0 is akin to clamping
        self.grow(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Sizes must not be made negative by growing them by negative amounts.
    #[test]
    fn grow_negative_clamp() {
        assert_eq!(Size::default(), Size::default().grow(-1));
    }
}
