//! Trait and context for things that can be laid out.

use crate::metrics;

/// Trait for things that can be laid out into the space defined by a context.
///
/// Layout is decoupled from rendering, and typically happens once at the start of the view creation
/// followed by occasional follow-ups if the size of the window changes.
pub trait Layoutable<Ctx> {
    /// Precalculate a minimal bounding box size, given the layout context.
    fn min_bounds(&self, ctx: &Ctx) -> metrics::Size;

    /// Calculates and stores a layout based on the context `ctx` and bounding box `bounds`.
    fn layout(&mut self, ctx: &Ctx, bounds: metrics::Rect);
}
