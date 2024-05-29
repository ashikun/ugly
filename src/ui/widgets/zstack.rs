//! The [`ZStack`] widget and its implementations.

use crate::{metrics, Result};

use super::super::{
    layout::{Boundable, Layoutable},
    render::Renderable,
    update::Updatable,
};

/// Homogeneous Z-index stack of widgets.
///
/// This is analogous to a `Stack`, but with the dimension being the Z-axis; that is, all of the
/// widgets will be laid out on top of each other with the same bounding box.
#[derive(Clone, Debug)]
pub struct ZStack<W> {
    /// The contents of the stack, with their ratios.
    contents: Vec<W>,
}

/// Constructs the default (empty) stack.
impl<W> Default for ZStack<W> {
    fn default() -> Self {
        // We can't derive this, because it would require W to be impl Default.
        ZStack { contents: vec![] }
    }
}

/// We can set bounds on a Z-stack by setting bounds on its individual components.
impl<W: Boundable> Boundable for ZStack<W> {
    fn set_bounds(&mut self, bounds: metrics::Rect) {
        for c in &mut self.contents {
            c.set_bounds(bounds);
        }
    }
}

/// We can layout a stack by laying out its individual components, with some flexing.
impl<C, W: Layoutable<C>> Layoutable<C> for ZStack<W> {
    fn min_bounds(&self, ctx: &C) -> metrics::Size {
        self.contents.iter().fold(metrics::Size::default(), |b, c| {
            b.superimpose(c.min_bounds(ctx))
        })
    }

    fn layout(&mut self, ctx: &C) {
        for c in &mut self.contents {
            c.layout(ctx);
        }
    }
}

/// Z-stacks are updatable, distributing updates to their children.
///
/// Each child widget must have the same state.
impl<S, W: Updatable<State = S>> Updatable for ZStack<W> {
    type State = S;

    fn update(&mut self, s: &Self::State) {
        for c in &mut self.contents {
            c.update(s);
        }
    }
}

/// Z-stacks are renderable, distributing rendering to their children.
/// Rendering occurs in insertion order.
///
/// Each child widget must have the same rendering state.
impl<R, W: Renderable<R>> Renderable<R> for ZStack<W> {
    fn render(&self, r: &mut R) -> Result<()> {
        self.contents.iter().try_for_each(|c| c.render(r))
    }
}

impl<W> ZStack<W> {
    /// Constructs a z-stack of widgets with the given orientation.
    ///
    /// This is an alias for `default`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Pushes a widget onto the end of the stack.
    pub fn push(&mut self, widget: W) {
        self.contents.push(widget);
    }

    /// Extends the stack with the given iterable of widget/ratio pairs.
    pub fn extend(&mut self, widgets: impl IntoIterator<Item = W>) {
        self.contents.extend(widgets);
    }
}
