//! Traits for colour identifiers.

use std::hash::Hash;

/// Trait containing all functionality a foreground colour identifier should support.
///
/// Foreground IDs should be copiable, hashable, and equatable; they should also have a default (eg,
/// the default colour to use for foreground text).
pub trait Fg: Copy + Clone + Default + Eq + Hash {}

/// Trait containing all functionality a background colour identifier should support.
///
/// Foreground IDs should be copiable, hashable, and equatable.  They do not need a background, as
/// generally the default background colour is transparency.
pub trait Bg: Copy + Clone + Eq + Hash {}
