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
        self.entries.get(index).unwrap_or(&self.default)
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
        let rights = Some(kern.into_iter().collect());
        if let Some(entry) = table.get_mut(char) {
            entry.rights = rights;
        } else {
            table.insert(char, Entry { rights, ..*default });
        }
    }
}

fn add_width(table: &mut Subtable<Entry>, width: width::Map, default: &Entry) {
    for (char, width) in width {
        if let Some(entry) = table.get_mut(char) {
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
/// This table is optimised for speed in looking up ASCII values
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Subtable<T> {
    /// Array of ASCII character values.
    ascii: [Option<Box<T>>; 128],
    /// Map of non-ASCII character values.
    non_ascii: BTreeMap<char, T>,
}

impl<T> Subtable<T> {
    #[must_use]
    pub fn new() -> Self {
        // https://www.joshmcguigan.com/blog/array-initialization-rust/
        let mut uninit: [std::mem::MaybeUninit<Option<Box<T>>>; 128] =
            unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        for elem in &mut uninit {
            unsafe {
                std::ptr::write(elem.as_mut_ptr(), None);
            }
        }
        let ascii = unsafe { std::mem::transmute(uninit) };

        Self {
            ascii,
            non_ascii: BTreeMap::new(),
        }
    }

    /// Inserts value `val` into character `key`.
    pub fn insert(&mut self, key: char, val: T) {
        if let Some(x) = as_ascii(key) {
            self.ascii[x] = Some(Box::new(val));
        } else {
            self.non_ascii.insert(key, val);
        }
    }

    /// Gets the value for character `key`.
    #[must_use]
    pub fn get(&self, key: char) -> Option<&T> {
        if let Some(x) = as_ascii(key) {
            self.ascii[x].as_deref()
        } else {
            self.non_ascii.get(&key)
        }
    }

    /// Gets a mutable reference to the value for character `key`.
    #[must_use]
    pub fn get_mut(&mut self, key: char) -> Option<&mut T> {
        if let Some(x) = as_ascii(key) {
            self.ascii[x].as_deref_mut()
        } else {
            self.non_ascii.get_mut(&key)
        }
    }
}

impl<T: Default> Default for Subtable<T> {
    fn default() -> Self {
        Self::new()
    }
}

fn as_ascii(key: char) -> Option<usize> {
    key.is_ascii().then(|| {
        let mut b = [0; 1];
        key.encode_utf8(&mut b);
        usize::from(b[0])
    })
}

impl<T> FromIterator<(char, T)> for Subtable<T> {
    fn from_iter<I: IntoIterator<Item = (char, T)>>(iter: I) -> Self {
        let mut stab = Subtable::new();
        for (k, v) in iter {
            stab.insert(k, v);
        }
        stab
    }
}

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
            .and_then(|r| r.get(right).copied())
            .unwrap_or(self.default_kerning)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn subtable_get_ascii() {
        let mut t = Subtable::new();
        t.insert('a', 12);
        t.insert('b', 39);
        assert_eq!(Some(&12), t.get('a'));
        assert_eq!(Some(&39), t.get('b'));
    }

    #[test]
    fn subtable_get_non_ascii() {
        let mut t = Subtable::new();
        t.insert('コ', 12);
        t.insert('ヒ', 39);
        assert_eq!(Some(&12), t.get('コ'));
        assert_eq!(Some(&39), t.get('ヒ'));
    }
}
