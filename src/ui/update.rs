//! The [Updatable] trait and its associated [Context].
//!
//! This module represents the updating phase of the widget lifecycle.  This phase passes the
//! current state to the widget and requests that it prepare its model of what it intends to send
//! for rendering.

/// Trait for things that can be updated using a state.
pub trait Updatable {
    /// Type of external state that this widget accepts.
    type State: ?Sized;

    /// Updates the widget's internal state according to the current external state.
    ///
    /// This will be called every cycle, before rendering.  This may change later.
    fn update(&mut self, s: &Self::State);
}
