//! Kerning tables.
//!
//! The general class-based kerning approach here is vaguely similar to `OpenType` class-based
//! kerning: we have a left-table, a right-table, and pairwise adjustments between them.

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::metrics::Length;

/// Identifier type for kerning classes.
pub type Class = String;

/// A map containing kerning classes (mapping from identifiers to character sets).
pub type ClassTable = HashMap<Class, String>;

/// A map providing spacing overrides for pairs.
///
/// Note that these are *absolute* spaces, not adjustments.  This may change.
pub type PairTable = HashMap<(Class, Class), Length>;

/// A complete kerning specification.
///
/// Note that there is a crossing over of terminology here: the names of the classes refer to their
/// characters' positions on either side of the space being kerned, but the classes are usually
/// set up such that the left class depends on the right edge of the character and vice versa.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Spec {
    /// The left-character class table, grouping characters by how they are kerned on the right.
    pub left: ClassTable,
    /// The right-character class table, grouping characters by how they are kerned on the left.
    pub right: ClassTable,
    /// The pair table, mapping left/right classes to spacing overrides.
    pub pairs: PairTable,
}

impl Spec {
    /// Compiles this specification into a full kerning pairs map.
    ///
    /// # Errors
    ///
    /// Fails if a kerning pair in the spec refers to a missing table.
    pub fn into_map(self, default_spacing: Length) -> Result<Map> {
        let mut kerning_pairs: BTreeMap<char, BTreeMap<char, Length>> = BTreeMap::new();
        for ((lclass, rclass), length) in &self.pairs {
            let lefts = self.class(Direction::Left, lclass)?;
            let rights = self.class(Direction::Right, rclass)?;
            for l in lefts.chars() {
                if let Some(lmap) = kerning_pairs.get_mut(&l) {
                    lmap.extend(rights.chars().map(|r| (r, *length)));
                } else {
                    kerning_pairs.insert(l, rights.chars().map(|r| (r, *length)).collect());
                }
            }
        }

        Ok(Map {
            kerning_pairs,
            default_spacing,
        })
    }

    fn class(&self, dir: Direction, class: &str) -> Result<String> {
        self.class_table(dir)
            .get(class)
            .cloned()
            .ok_or(Error::MissingClass {
                dir,
                class: class.to_string(),
            })
    }

    fn class_table(&self, dir: Direction) -> &ClassTable {
        match dir {
            Direction::Left => &self.left,
            Direction::Right => &self.right,
        }
    }
}

/// A kerning map.
///
/// This is very similar in principle to a width override table, but for spacing between character
/// pairs.
#[derive(Clone, Debug, Default)]
pub struct Map {
    /// The specified kerning pairs.
    kerning_pairs: BTreeMap<char, BTreeMap<char, Length>>,
    /// The default spacing between glyphs.
    default_spacing: Length,
}

impl Map {
    /// Gets the spacing between `left` and `right`.
    #[must_use]
    pub fn spacing(&self, left: char, right: char) -> Length {
        self.kerning_pairs
            .get(&left)
            .and_then(|l| l.get(&right))
            .copied()
            .unwrap_or(self.default_spacing)
    }
}

/// Enumeration of possible errors when compiling a kerning list.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The given kerning class was missing.
    #[error("Missing kerning class")]
    MissingClass { dir: Direction, class: Class },
}

/// Shorthand for `Result`s over `Error`.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Left class.
    Left,
    /// Right class.
    Right,
}
