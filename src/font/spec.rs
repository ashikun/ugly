//! Font specifications.

use std::{fmt::Debug, hash::Hash};

/// A font specification (ID and colour).
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
pub struct Spec<F, C> {
    /// The identifier of the font.
    pub id: F,
    /// The colour key for the font.
    pub colour: C,
}
