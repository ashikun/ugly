//! Compiled character tables for width and kerning pairs.

use std::collections::BTreeMap;
use std::ops::Index;

use super::{
    super::{metrics::Length, Result},
    kerning, width,
};

/// Character table.
///
/// A default character table maps every character's metrics to zero, and is likely not what you
/// want in most circumstances.
#[derive(Clone, Debug, Default)]
pub struct Table {
    entries: Subtable<Entry>,
    default: Entry,
}

impl Index<char> for Table {
    type Output = Entry;

    fn index(&self, index: char) -> &Self::Output {
        self.entries.get(&index).unwrap_or(&self.default)
    }
}

impl Table {
    /// Compiles a character table from width and kerning specifications.
    ///
    /// # Errors
    ///
    /// Fails if any of the kerning or width specifications are invalid.
    pub fn new(
        width: width::Spec,
        default_width: Length,
        kerning: kerning::Spec,
        default_kerning: Length,
    ) -> Result<Self> {
        let mut table = Subtable::new();

        let default = Entry {
            width: default_width,
            rights: None,
            default_kerning,
        };

        add_kerning(&mut table, kerning.into_map()?, &default);
        add_width(&mut table, width.into_map(default_width)?, &default);

        Ok(Self {
            entries: table,
            default,
        })
    }
}

fn add_kerning(table: &mut Subtable<Entry>, kerning: kerning::Map, default: &Entry) {
    for (char, kern) in kerning {
        if let Some(entry) = table.get_mut(&char) {
            entry.rights = Some(kern);
        } else {
            table.insert(
                char,
                Entry {
                    rights: Some(kern),
                    ..*default
                },
            );
        }
    }
}

fn add_width(table: &mut Subtable<Entry>, width: width::Map, default: &Entry) {
    for (char, width) in width {
        if let Some(entry) = table.get_mut(&char) {
            entry.width = width;
        } else {
            table.insert(
                char,
                Entry {
                    width,
                    rights: default.rights.clone(),
                    default_kerning: default.default_kerning,
                },
            );
        }
    }
}

/// Type of character tables.
///
/// The exact type here is subject to change.
type Subtable<T> = BTreeMap<char, T>;

/// A character left-table.
///
/// The left-table contains the width of the character (used for advancing past it to the next
/// character in the layout algorithm) as well as the right kerning table.
/// Both aspects are merged into one table to avoid having to do multiple lookups.
///
/// Note that the default character entry has everything zeroed.  This is useful for the initial
/// state of the layout algorithm and little else.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Entry {
    /// The calculated width of this character.
    pub width: Length,

    /// The right kerning table of this character, if any.
    ///
    /// This maps characters on the right-hand side of a pair to kerning adjustments with respect to the
    /// character to which the parent [Left] table belongs.  Each kerning adjustment is absolute (ie,
    /// it completely replaces the default spacing).
    pub rights: Option<Subtable<Length>>,

    /// The default spacing for any character not in the kerning table.
    pub default_kerning: Length,
}

impl Entry {
    /// Gets the kerning between this map's leftward character and `right`.
    #[must_use]
    pub fn kerning(&self, right: char) -> Length {
        self.rights
            .as_ref()
            .and_then(|r| r.get(&right).copied())
            .unwrap_or(self.default_kerning)
    }
}
