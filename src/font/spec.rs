//! Font specifications.

use std::{fmt::Debug, hash::Hash};

/// Trait containing all functionality a font identifier should support.
///
/// Font IDs should be copiable, hashable, and equatable; they should also have a default (eg, the
/// default font to use if no other font is specified).
pub trait Id: Copy + Clone + Debug + Default + Eq + Hash {}

/// A font specification (ID and colour).
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
pub struct Spec<F, C> {
    /// The identifier of the font.
    pub id: F,
    /// The colour key for the font.
    pub colour: C,
}
