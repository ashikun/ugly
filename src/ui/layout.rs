//! Trait and context for things that can be laid out.

use crate::{font, metrics, resource};
use std::hash::Hash;

// TODO: move this
/// A layout context.
pub trait LayoutContext<FontId>
where
    FontId: Copy + Clone + Default + Eq + Hash,
{
    /// Gets an immutable reference to this renderer's font metrics.
    fn font_metrics(&self) -> &impl resource::Map<font::Metrics, Id = FontId>;
}

/// Trait for things whose bounding box can be set.
pub trait Boundable {
    /// Updates the bounding box of this item.
    /// This does not trigger a re-layout.
    fn set_bounds(&mut self, bounds: metrics::Rect);
}

/// Trait for things that can be laid out into the space defined by a context.
///
/// Layout is decoupled from rendering, and typically happens once at the start of the view creation
/// followed by occasional follow-ups if the size of the window changes.
pub trait Layoutable<Ctx> {
    /// Precalculate a minimal bounding box size, given the layout context.
    fn min_bounds(&self, ctx: &Ctx) -> metrics::Size;

    /// Calculates and stores a layout based on the context `ctx` and current bounding box.
    fn layout(&mut self, ctx: &Ctx);
}
