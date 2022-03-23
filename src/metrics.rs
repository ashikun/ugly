//! Metrics used in `ugly`.
//!
//! Generally, metrics are `i32`s, even when negative values make no sense.
//! This is primarily to avoid unnecessary casting and potential overflow/underflow corner cases.
//!
//! `ugly` itself doesn't specify the units used for lengths; they are backend-dependent.
//! In SDL2, for instance, they are logical pixel values.  Future text-based backends may instead
//! specify character cell units.

pub mod anchor;
pub mod axis;
pub mod point;
pub mod rect;
pub mod size;

pub use anchor::Anchor;
pub use axis::Axis;
pub use point::Point;
pub use rect::Rect;
pub use size::{Length, Size};
