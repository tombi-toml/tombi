use std::hash::Hash;
use std::iter::{FromIterator, IntoIterator};

#[cfg(feature = "serde")]
use serde::{Serialize, Serializer};

use crate::Value;

/// A map implementation for JSON objects
#[derive(Debug, Clone)]
pub struct Map<K, V> {
    inner: tombi_hashmap::IndexMap<K, V>,
}

impl<K, V> PartialEq for Map<K, V>
where
    K: Hash + Eq,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<K, V> Map<K, V>
where
    K: Hash + Eq,
{
    /// Creates an empty Map
    pub fn new() -> Self {
        Map {
            inner: tombi_hashmap::IndexMap::new(),
        }
    }

    /// Creates an empty Map with the specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Map {
            inner: tombi_hashmap::IndexMap::with_capacity(capacity),
        }
    }

    /// Returns the number of elements in the map
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if the map contains no elements
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Clears the map, removing all elements
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Returns a reference to the value corresponding to the key
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        Q: ?Sized + Hash + tombi_hashmap::Equivalent<K>,
    {
        self.inner.get(key)
    }

    /// Returns a mutable reference to the value corresponding to the key
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.inner.get_mut(key)
    }

    /// Inserts a key-value pair into the map
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.inner.insert(key, value)
    }

    /// Removes a key from the map, returning the value if the key was previously in the map
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.inner.shift_remove(key)
    }

    /// Returns an iterator over the entries of the map
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.inner.iter()
    }

    /// Returns a mutable iterator over the entries of the map
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        self.inner.iter_mut()
    }

    /// Returns an iterator over the keys of the map
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.inner.keys()
    }

    /// Returns an iterator over the values of the map
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.inner.values()
    }

    /// Returns a mutable iterator over the values of the map
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.inner.values_mut()
    }

    /// Returns a reference to the underlying tombi_hashmap::IndexMap
    pub fn as_inner(&self) -> &tombi_hashmap::IndexMap<K, V> {
        &self.inner
    }

    /// Returns a mutable reference to the underlying tombi_hashmap::IndexMap
    pub fn as_inner_mut(&mut self) -> &mut tombi_hashmap::IndexMap<K, V> {
        &mut self.inner
    }

    /// Consumes the Map and returns the underlying tombi_hashmap::IndexMap
    pub fn into_inner(self) -> tombi_hashmap::IndexMap<K, V> {
        self.inner
    }

    /// Returns true if the map contains a key
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        Q: ?Sized + Hash + tombi_hashmap::Equivalent<K>,
    {
        self.inner.contains_key(key)
    }
}

impl Map<String, Value> {
    /// Returns a reference to the value corresponding to the string key
    pub fn get_str(&self, key: &str) -> Option<&Value> {
        self.inner.get(key)
    }

    /// Returns a mutable reference to the value corresponding to the string key
    pub fn get_str_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.inner.get_mut(key)
    }

    /// Removes a string key from the map, returning the value if the key was previously in the map
    pub fn remove_str(&mut self, key: &str) -> Option<Value> {
        self.inner.shift_remove(key)
    }
}

impl<K, V> Default for Map<K, V>
where
    K: Hash + Eq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> IntoIterator for Map<K, V> {
    type Item = (K, V);
    type IntoIter = tombi_hashmap::map::IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a, K, V> IntoIterator for &'a Map<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = tombi_hashmap::map::Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl<'a, K, V> IntoIterator for &'a mut Map<K, V> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = tombi_hashmap::map::IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter_mut()
    }
}

impl<K, V> FromIterator<(K, V)> for Map<K, V>
where
    K: Hash + Eq,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Map {
            inner: tombi_hashmap::IndexMap::from_iter(iter),
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for Map<String, Value> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.inner.serialize(serializer)
    }
}
