//! The [Rect] struct and related functionality.

use super::{
    anchor::{self, Anchor},
    point::{Coord, Point},
    Length, Size,
};

/// Output-independent rectangle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Rect {
    /// Position of the top-left of this rectangle.
    pub top_left: Point,
    /// Size of the rectangle.
    pub size: Size,
}

impl Rect {
    /// Makes a [Rect] with top-left at (`x`, `y`), width `w`, and height `h`.
    #[must_use]
    pub fn new(x: Coord, y: Coord, w: Length, h: Length) -> Self {
        Self {
            top_left: Point { x, y },
            size: Size { w, h },
        }
    }

    /// Makes a [Rect] with top-left at `top_left` and bottom-right at `bottom_right`.
    ///
    /// If `bottom_right` is not to the bottom-right of `top_left`, the rectangle will become
    /// zero-sized.
    ///
    /// ```
    /// use ugly::metrics::{Anchor, Point, Rect};
    ///
    /// let tl = Point{x: 20, y: 45};
    /// let br = Point{x: 55, y: 70};
    /// let rect = Rect::from_points(tl, br);
    /// assert_eq!(tl, rect.anchor(Anchor::TOP_LEFT));
    /// assert_eq!(br, rect.anchor(Anchor::BOTTOM_RIGHT));
    /// ```
    #[must_use]
    pub fn from_points(top_left: Point, bottom_right: Point) -> Self {
        let w = bottom_right.x - top_left.x;
        let h = bottom_right.y - top_left.y;
        let size = Size { w, h }.clamp();
        Self { top_left, size }
    }

    /// Resolves a point within a rectangle, given an offset (`dx`, `dy`) from
    /// `anchor`.
    #[must_use]
    pub fn point(self, dx: Length, dy: Length, anchor: Anchor) -> Point {
        Point {
            x: self.x(dx, anchor.x),
            y: self.y(dy, anchor.y),
        }
    }

    /// Shorthand for getting an anchor point on a rect.
    #[must_use]
    pub fn anchor(self, anchor: Anchor) -> Point {
        self.point(0, 0, anchor)
    }

    /// Resolves an X coordinate within a rectangle, given an offset `dx` from
    /// `anchor`.
    #[must_use]
    pub fn x(self, dx: Length, anchor: anchor::X) -> i32 {
        self.top_left.x + dx + anchor.offset(self.size.w)
    }

    /// Resolves an Y coordinate within a rectangle, given an offset `dy` from
    /// `anchor`.
    #[must_use]
    pub fn y(self, dy: Length, anchor: anchor::Y) -> i32 {
        self.top_left.y + dy + anchor.offset(self.size.h)
    }

    /// Produces a new [Rect] by growing the given [Rect] by `amount` on each side.
    ///
    /// To shrink, grow by a negative amount.
    #[must_use]
    pub fn grow(self, amount: Length) -> Self {
        Self {
            top_left: self.top_left.offset(-amount, -amount),
            size: self.size.grow(amount * 2),
        }
    }
}
