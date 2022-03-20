//! Conversion functions from `ugly` metrics to SDL ones.

use crate::metrics;

/// Converts a rect from `ugly` to SDL.
#[must_use]
pub fn convert_rect(r: &metrics::Rect) -> sdl2::rect::Rect {
    let (w, h) = convert_size(&r.size);
    sdl2::rect::Rect::new(r.top_left.x, r.top_left.y, w, h)
}

/// Converts a size from `ugly` to SDL (pair of width and height).
///
/// Negative widths and heights will be clipped to zero.
#[must_use]
pub fn convert_size(s: &metrics::Size) -> (u32, u32) {
    (u32_or_zero(s.w), u32_or_zero(s.h))
}

/// Convert `x` to u32, set to 0 if negative.
pub(crate) fn u32_or_zero(x: impl TryInto<u32>) -> u32 {
    x.try_into().unwrap_or_default()
}
