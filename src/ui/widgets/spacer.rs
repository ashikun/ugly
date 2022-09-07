//! The [Spacer] widget and its implementations.

use super::super::{
    super::{metrics, Result},
    layout::Layoutable,
    render::Renderable,
    update::Updatable,
};
use std::marker::PhantomData;

/// Dummy widget that renders nothing but consumes some minimum size provided by a function of the
/// context.
///
/// Useful for testing, forcing gaps between other widgets, etc.
#[derive(Copy, Clone)]
pub struct Spacer<C, S> {
    /// Bounds actually consumed by the spacer.
    bounds: metrics::Rect,
    /// Source providing the minimum size required by the spacer.
    min_bounds_source: BoundsSource<C>,
    /// Phantom type for the state ignored by the spacer.
    state: PhantomData<S>,
}

/// Constructs the default (empty) stack.
impl<C, S> Default for Spacer<C, S> {
    fn default() -> Self {
        // We can't derive this, because it would require S to be impl Default.
        Spacer {
            bounds: metrics::Rect::default(),
            min_bounds_source: BoundsSource::Static(metrics::Size::default()),
            state: PhantomData::default(),
        }
    }
}

/// We can layout a spacer using its minimum bounds.
impl<C, S> Layoutable<C> for Spacer<C, S> {
    fn min_bounds(&self, ctx: &C) -> metrics::Size {
        match self.min_bounds_source {
            BoundsSource::Static(s) => s,
            BoundsSource::Context(f) => (f)(ctx),
        }
    }

    fn actual_bounds(&self) -> metrics::Rect {
        self.bounds
    }

    fn layout(&mut self, _ctx: &C, bounds: metrics::Rect) {
        self.bounds = bounds;
    }
}

/// Spacers are vacuously updatable.
impl<C, S> Updatable<C> for Spacer<C, S> {
    type State = S;

    fn update(&mut self, _ctx: &C, _s: &Self::State) {}
}

/// Spacers are vacuously updatable.
impl<R, C, S> Renderable<R> for Spacer<C, S> {
    fn render(&self, _r: &mut R) -> Result<()> {
        Ok(())
    }
}

impl<C, S> Spacer<C, S> {
    /// Constructs a [Spacer] with the given minimum bounds callback.
    #[must_use]
    pub fn new_from_fn(f: fn(&C) -> metrics::Size) -> Self {
        Self::new(BoundsSource::Context(f))
    }

    /// Constructs a [Spacer] with the given minimum bounds.
    #[must_use]
    pub fn new_with_bounds(bounds: metrics::Size) -> Self {
        Self::new(BoundsSource::Static(bounds))
    }

    #[must_use]
    pub fn new(src: BoundsSource<C>) -> Self {
        Self {
            min_bounds_source: src,
            ..Self::default()
        }
    }
}

/// Type of bounds supported by a spacer.
///
/// This is needed to avoid injecting a closure type into the spacer.
#[derive(Copy, Clone)]
pub enum BoundsSource<C> {
    /// Static bounds amount.
    Static(metrics::Size),
    /// Bounds amount determined entirely from the context.
    Context(fn(&C) -> metrics::Size),
}
