//! Enumerations for anchoring a position to a rectangle.

/// A two-dimensional anchor.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Anchor {
    /// The X component of the anchor.
    pub x: X,
    /// The Y component of the anchor.
    pub y: Y,
}

/// The default anchor is top-left.
///
/// This is considered the default anchor because, for instance, font rendering in
/// left-to-right languages naturally proceeds from the top-left.
impl Default for Anchor {
    fn default() -> Self {
        // TODO: when Rust supports generalised enum default derivations, use that instead
        Self::TOP_LEFT
    }
}

impl Anchor {
    /// Top-left anchoring.
    pub const TOP_LEFT: Self = Anchor {
        x: X::Left,
        y: Y::Top,
    };

    /// Top-right anchoring.
    pub const TOP_RIGHT: Self = Anchor {
        x: X::Right,
        y: Y::Top,
    };

    /// Bottom-left anchoring.
    pub const BOTTOM_LEFT: Self = Anchor {
        x: X::Left,
        y: Y::Bottom,
    };

    /// Bottom-right anchoring.
    pub const BOTTOM_RIGHT: Self = Anchor {
        x: X::Right,
        y: Y::Bottom,
    };
}

/// An anchor for the X co-ordinate.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum X {
    /// Anchoring to the left edge.
    ///
    /// This is considered the default X-anchor because, for instance, font rendering in
    /// left-to-right languages naturally proceeds from the top-left.
    #[default]
    Left,
    /// Anchoring to the right edge.
    Right,
}

impl X {
    /// Calculates the offset from left of this anchor in an object of width `width`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ugly::metrics::anchor;
    ///
    /// assert_eq!(0, anchor::X::Left.offset(320));
    /// assert_eq!(320, anchor::X::Right.offset(320));
    /// ```
    #[must_use]
    pub fn offset(self, width: i32) -> i32 {
        match self {
            Self::Left => 0,
            Self::Right => width,
        }
    }
}

/// An anchor for the Y co-ordinate.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum Y {
    /// Anchoring to the top edge.
    ///
    /// This is considered the default Y-anchor because, for instance, font rendering in
    /// left-to-right languages naturally proceeds from the top-left.
    #[default]
    Top,
    /// Anchoring to the bottom edge.
    Bottom,
}

impl Y {
    /// Calculates the offset from top of this anchor in an object of height `height`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ugly::metrics::anchor;
    ///
    /// assert_eq!(0, anchor::Y::Top.offset(240));
    /// assert_eq!(240, anchor::Y::Bottom.offset(240));
    /// ```
    #[must_use]
    pub fn offset(self, height: i32) -> i32 {
        match self {
            Self::Top => 0,
            Self::Bottom => height,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Checks that the default anchor is made up of the default X and Y anchors.
    #[test]
    fn default_agreement() {
        assert_eq!(
            Anchor::default(),
            Anchor {
                x: X::default(),
                y: Y::default()
            }
        );
    }
}
