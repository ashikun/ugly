//! Width override specifications and tables.

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use super::super::{super::metrics::Length, Error, Result};

/// A class-based specification of width overrides.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Spec(HashMap<String, Length>);

/// We can construct a [Spec] by iterating over class/length pairs.
impl<S: ToString> FromIterator<(S, Length)> for Spec {
    fn from_iter<T: IntoIterator<Item = (S, Length)>>(iter: T) -> Self {
        Spec(
            iter.into_iter()
                .map(|(class, l)| (class.to_string(), l))
                .collect(),
        )
    }
}

impl Spec {
    /// Expands an override spec into a full width map, given the on-grid character width.
    ///
    /// # Errors
    ///
    /// Fails if the overrides try to make a width larger than the on-grid character width.
    pub fn into_map(self, grid_width: Length) -> Result<Map> {
        self.check(grid_width)?;
        let overrides = self.expand_map();
        Ok(Map {
            overrides,
            grid_width,
        })
    }

    /// Expands the overrides in this spec.
    fn expand_map(self) -> BTreeMap<char, Length> {
        self.0
            .into_iter()
            .flat_map(|(class, l)| class.chars().map(|c| (c, l)).collect::<Vec<_>>())
            .collect()
    }

    /// Checks that this specification is well-formed with respect to a grid.
    ///
    /// # Errors
    ///
    /// Fails if the overrides try to make a width larger than the on-grid character width.
    fn check(&self, grid_width: Length) -> Result<()> {
        for override_width in self.0.values().copied() {
            if grid_width < override_width {
                return Err(Error::OverlyLargeOverride {
                    grid_width,
                    override_width,
                });
            }
        }
        Ok(())
    }
}

/// Maps characters to their widths.
#[derive(Debug, Clone, Default)]
pub struct Map {
    /// The override map.
    overrides: BTreeMap<char, Length>,
    /// The default length, being the width of each cell in the font grid.
    grid_width: Length,
}

impl Map {
    /// Gets the length of a character.
    #[must_use]
    pub fn get(&self, c: char) -> Length {
        self.overrides.get(&c).copied().unwrap_or(self.grid_width)
    }
}
