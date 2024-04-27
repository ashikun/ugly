//! The [Map] trait for resource maps.

use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;

/// Trait of resource maps used in `ugly`.
///
/// This is effectively an inverted form of `Index` where we care more about what comes out of the
/// map than what goes into it, and we expect the key to be an enum or something else
/// trivially copiable.
pub trait Map<Resource> {
    /// Type of key used by this map.
    ///
    /// Identifiers must be trivially copiable, and generally will be lightweight enums.  There
    /// must additionally be a default identifier, used to denote the default resource when no
    /// other resource has been specified; for example: a transparent background colour, a
    /// black foreground colour, and a medium-sized font.
    type Id: Copy + Clone + Default + Eq + Hash;

    /// Gets the resource at ID `k`.
    fn get(&self, k: Self::Id) -> &Resource;
}

/// A resource map that can be modified.
pub trait MutableMap<Resource>: Map<Resource> {
    /// Sets the resource at ID `k` to `v`.
    fn set(&mut self, k: Self::Id, v: Resource);
}

/// A `HashMap` with a default value attached, for use when a requested key is not available.
///
/// This map can be used as a resource map, so long as the key satisfies the various requirements
/// to be both a [Map] key and a `HashMap` key.
#[derive(Debug, Clone, Default)]
pub struct DefaultingHashMap<K, V> {
    map: HashMap<K, V>,
    default: V,
    phantom: PhantomData<K>,
}

impl<K: Copy + Clone + Default + Eq + Hash, V> Map<V> for DefaultingHashMap<K, V> {
    type Id = K;

    fn get(&self, k: K) -> &V {
        self.map.get(&k).unwrap_or(&self.default)
    }
}

impl<K: Copy + Clone + Default + Eq + Hash, V> MutableMap<V> for DefaultingHashMap<K, V> {
    fn set(&mut self, k: K, v: V) {
        self.map.insert(k, v);
    }
}

impl<K: Eq + Hash, V> DefaultingHashMap<K, V> {
    /// Constructs a `DefaultingHashMap` by wrapping `map` and supplying the default `default`.
    #[must_use]
    pub fn new(map: HashMap<K, V>, default: V) -> DefaultingHashMap<K, V> {
        DefaultingHashMap {
            map,
            default,
            phantom: PhantomData {},
        }
    }

    /// Gets the underlying map's borrowing iterator.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.map.iter()
    }
}
