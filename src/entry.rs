//! Note: Most of the documentation is taken from
//! rusts hashmap.rs and should be considered under
//! their copyright.

use super::*;
use core::hash::{Hash, Hasher};

use core::mem;
// use std::fmt::{self, Debug};

/////// General

/// A view into a single entry in a map, which may either be vacant or occupied.
///
/// This `enum` is constructed from the [`entry`] method on [`HashMap`].
///
/// [`HashMap`]: struct.HashMap.html
/// [`entry`]: struct.HashMap.html#method.entry
pub enum Entry<'a, K, V, H> {
    /// An occupied entry.
    Occupied(OccupiedEntry<'a, K, V, H>),

    /// A vacant entry.
    Vacant(VacantEntry<'a, K, V, H>),
}

impl<'a, K, V, H> Entry<'a, K, V, H>
where
    K: Clone,
    H: Hasher + Default,
{
    /// Ensures a value is in the entry by inserting the default if empty, and returns
    /// a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use ordnung::Map;
    ///
    /// let mut map: Map<&str, u32> = Map::new();
    ///
    /// map.entry("poneyland").or_insert(3);
    /// assert_eq!(map["poneyland"], 3);
    ///
    /// *map.entry("poneyland").or_insert(10) *= 2;
    /// assert_eq!(map["poneyland"], 6);
    /// ```
    #[inline]
    pub fn or_insert(self, default: V) -> &'a mut V
    where
        K: Eq + Hash,
    {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default function if empty,
    /// and returns a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use ordnung::Map;
    ///
    /// let mut map: Map<&str, String> = Map::new();
    /// let s = "hoho".to_string();
    ///
    /// map.entry("poneyland").or_insert_with(|| s);
    ///
    /// assert_eq!(map["poneyland"], "hoho".to_string());
    /// ```
    #[inline]
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V
    where
        K: Eq + Hash,
    {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default()),
        }
    }

    /// Returns a reference to this entry's key.
    ///
    /// # Examples
    ///
    /// ```
    /// use ordnung::Map;
    ///
    /// let mut map: Map<&str, u32> = Map::new();
    /// assert_eq!(map.entry("poneyland").key(), &"poneyland");
    /// ```
    #[inline]
    pub fn key(&self) -> &K {
        match *self {
            Entry::Occupied(ref entry) => entry.key(),
            Entry::Vacant(ref entry) => entry.key(),
        }
    }

    /// Provides in-place mutable access to an occupied entry before any
    /// potential inserts into the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use ordnung::Map;
    ///
    /// let mut map: Map<&str, u32> = Map::new();
    ///
    /// map.entry("poneyland")
    ///    .and_modify(|e| { *e += 1 })
    ///    .or_insert(42);
    /// assert_eq!(map["poneyland"], 42);
    ///
    /// map.entry("poneyland")
    ///    .and_modify(|e| { *e += 1 })
    ///    .or_insert(42);
    /// assert_eq!(map["poneyland"], 43);
    /// ```
    #[inline]
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        match self {
            Entry::Occupied(mut entry) => {
                f(entry.get_mut());
                Entry::Occupied(entry)
            }
            Entry::Vacant(entry) => Entry::Vacant(entry),
        }
    }
}

/*
impl<K: fmt::Debug, V: fmt::Debug, S> fmt::Debug for Entry<'_, K, V, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Entry::Vacant(ref v) => f.debug_tuple("Entry").field(v).finish(),
            Entry::Occupied(ref o) => f.debug_tuple("Entry").field(o).finish(),
        }
    }
}
*/

/// A view into an occupied entry in a `HashMap`.
/// It is part of the [`Entry`] enum.
///
/// [`Entry`]: enum.Entry.html
pub struct OccupiedEntry<'a, K, V, H> {
    idx: usize,
    key: Option<K>,
    map: &'a mut Map<K, V, H>,
}

unsafe impl<K, V, H> Send for OccupiedEntry<'_, K, V, H>
where
    K: Send,
    V: Send,
{
}
unsafe impl<K, V, H> Sync for OccupiedEntry<'_, K, V, H>
where
    K: Sync,
    V: Sync,
{
}

/*
impl<K: Debug, V: Debug, S> Debug for OccupiedEntry<'_, K, V, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OccupiedEntry")
            .field("key", self.key())
            .field("value", self.get())
            .finish()
    }
}
*/

impl<'a, K, V, H> OccupiedEntry<'a, K, V, H>
where
    K: Clone,
{
    pub(crate) fn new(idx: usize, key: K, map: &'a mut Map<K, V, H>) -> Self {
        Self {
            idx,
            key: Some(key),
            map,
        }
    }

    /// Gets a reference to the key in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use ordnung::Map;
    ///
    /// let mut map: Map<&str, u32> = Map::new();
    /// map.entry("poneyland").or_insert(12);
    /// assert_eq!(map.entry("poneyland").key(), &"poneyland");
    /// ```
    #[inline]
    pub fn key(&self) -> &K {
        unsafe { &self.map.store.get_unchecked(self.idx).key }
    }

    /// Take the ownership of the key and value from the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use ordnung::Map;
    /// use ordnung::Entry;
    ///
    /// let mut map: Map<&str, u32> = Map::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// if let Entry::Occupied(o) = map.entry("poneyland") {
    ///     // We delete the entry from the map.
    ///     o.remove_entry();
    /// }
    ///
    /// assert_eq!(map.contains_key("poneyland"), false);
    /// ```
    #[inline]
    pub fn remove_entry(self) -> (K, V) {
        let n = unsafe { self.map.store.get_unchecked_mut(self.idx) };

        (n.key.clone(), n.value.take().unwrap())
    }

    /// Gets a reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use ordnung::Map;
    /// use ordnung::Entry;
    ///
    /// let mut map: Map<&str, u32> = Map::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// if let Entry::Occupied(o) = map.entry("poneyland") {
    ///     assert_eq!(o.get(), &12);
    /// }
    /// ```
    #[inline]
    pub fn get(&self) -> &V {
        unsafe {
            if let Node { value: Some(v), .. } = self.map.store.get_unchecked(self.idx) {
                v
            } else {
                unreachable!()
            }
        }
    }

    /// Gets a mutable reference to the value in the entry.
    ///
    /// If you need a reference to the `OccupiedEntry` which may outlive the
    /// destruction of the `Entry` value, see [`into_mut`].
    ///
    /// [`into_mut`]: #method.into_mut
    ///
    /// # Examples
    ///
    /// ```
    /// use ordnung::Map;
    /// use ordnung::Entry;
    ///
    /// let mut map: Map<&str, u32> = Map::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// assert_eq!(map["poneyland"], 12);
    /// if let Entry::Occupied(mut o) = map.entry("poneyland") {
    ///     *o.get_mut() += 10;
    ///     assert_eq!(*o.get(), 22);
    ///
    ///     // We can use the same Entry multiple times.
    ///     *o.get_mut() += 2;
    /// }
    ///
    /// assert_eq!(map["poneyland"], 24);
    /// ```
    #[inline]
    pub fn get_mut(&mut self) -> &mut V {
        unsafe {
            if let Node { value: Some(v), .. } = self.map.store.get_unchecked_mut(self.idx) {
                v
            } else {
                unreachable!()
            }
        }
    }

    /// Converts the OccupiedEntry into a mutable reference to the value in the entry
    /// with a lifetime bound to the map itself.
    ///
    /// If you need multiple references to the `OccupiedEntry`, see [`get_mut`].
    ///
    /// [`get_mut`]: #method.get_mut
    ///
    /// # Examples
    ///
    /// ```
    /// use ordnung::Map;
    /// use ordnung::Entry;
    ///
    /// let mut map: Map<&str, u32> = Map::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// assert_eq!(map["poneyland"], 12);
    /// if let Entry::Occupied(o) = map.entry("poneyland") {
    ///     *o.into_mut() += 10;
    /// }
    ///
    /// assert_eq!(map["poneyland"], 22);
    /// ```
    #[inline]
    pub fn into_mut(self) -> &'a mut V {
        unsafe {
            if let Node { value: Some(v), .. } = self.map.store.get_unchecked_mut(self.idx) {
                v
            } else {
                unreachable!()
            }
        }
    }

    /// Sets the value of the entry, and returns the entry's old value.
    ///
    /// # Examples
    ///
    /// ```
    /// use ordnung::Map;
    /// use ordnung::Entry;
    ///
    /// let mut map: Map<&str, u32> = Map::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// if let Entry::Occupied(mut o) = map.entry("poneyland") {
    ///     assert_eq!(o.insert(15), 12);
    /// }
    ///
    /// assert_eq!(map["poneyland"], 15);
    /// ```
    #[inline]
    pub fn insert(&mut self, mut value: V) -> V {
        let old_value = self.get_mut();
        mem::swap(&mut value, old_value);
        value
    }

    /// Takes the value out of the entry, and returns it.
    ///
    /// # Examples
    ///
    /// ```
    /// use ordnung::Map;
    /// use ordnung::Entry;
    ///
    /// let mut map: Map<&str, u32> = Map::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// if let Entry::Occupied(o) = map.entry("poneyland") {
    ///     assert_eq!(o.remove(), 12);
    /// }
    ///
    /// assert_eq!(map.contains_key("poneyland"), false);
    /// ```
    #[inline]
    pub fn remove(self) -> V {
        self.remove_entry().1
    }

    /// Replaces the entry, returning the old key and value. The new key in the hash map will be
    /// the key used to create this entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use ordnung::{Entry, Map};
    /// use std::rc::Rc;
    ///
    /// let mut map: Map<Rc<String>, u32> = Map::new();
    /// map.insert(Rc::new("Stringthing".to_string()), 15);
    ///
    /// let my_key = Rc::new("Stringthing".to_string());
    ///
    /// if let Entry::Occupied(entry) = map.entry(my_key) {
    ///     // Also replace the key with a handle to our other key.
    ///     let (old_key, old_value): (Rc<String>, u32) = entry.replace_entry(16);
    /// }
    ///
    /// ```
    #[inline]
    pub fn replace_entry(self, value: V) -> (K, V) {
        if let Node {
            value: Some(cur_val),
            key: cur_key,
            ..
        } = unsafe { self.map.store.get_unchecked_mut(self.idx) }
        {
            let old_key = mem::replace(cur_key, self.key.unwrap());
            let old_value = mem::replace(cur_val, value);

            (old_key, old_value)
        } else {
            unreachable!()
        }
    }

    /// Replaces the key in the hash map with the key used to create this entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use ordnung::{Entry, Map};
    /// use std::rc::Rc;
    ///
    /// let mut map: Map<Rc<String>, u32> = Map::new();
    /// let mut known_strings: Vec<Rc<String>> = Vec::new();
    ///
    /// // Initialise known strings, run program, etc.
    ///
    /// reclaim_memory(&mut map, &known_strings);
    ///
    /// fn reclaim_memory(map: &mut Map<Rc<String>, u32>, known_strings: &[Rc<String>] ) {
    ///     for s in known_strings {
    ///         if let Entry::Occupied(entry) = map.entry(s.clone()) {
    ///             // Replaces the entry's key with our version of it in `known_strings`.
    ///             entry.replace_key();
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    pub fn replace_key(self) -> K {
        let key = unsafe { &mut self.map.store.get_unchecked_mut(self.idx).key };
        mem::replace(key, self.key.unwrap())
    }
}

/// A view into a vacant entry in a `HashMap`.
/// It is part of the [`Entry`] enum.
///
/// [`Entry`]: enum.Entry.html
pub struct VacantEntry<'a, K, V, H> {
    key: K,
    map: &'a mut Map<K, V, H>,
}

/*
impl<K: Debug, V, S> Debug for VacantEntry<'_, K, V, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("VacantEntry").field(self.key()).finish()
    }
}
*/

impl<'a, K, V, H> VacantEntry<'a, K, V, H>
where
    H: Hasher + Default,
{
    pub(crate) fn new(key: K, map: &'a mut Map<K, V, H>) -> Self {
        Self { key, map }
    }
    /// Gets a reference to the key that would be used when inserting a value
    /// through the `VacantEntry`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ordnung::Map;
    ///
    /// let mut map: Map<&str, u32> = Map::new();
    /// assert_eq!(map.entry("poneyland").key(), &"poneyland");
    /// ```
    #[inline]
    pub fn key(&self) -> &K {
        &self.key
    }

    /// Take ownership of the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use ordnung::Map;
    /// use ordnung::Entry;
    ///
    /// let mut map: Map<&str, u32> = Map::new();
    ///
    /// if let Entry::Vacant(v) = map.entry("poneyland") {
    ///     v.into_key();
    /// }
    /// ```
    #[inline]
    pub fn into_key(self) -> K {
        self.key
    }

    /// Sets the value of the entry with the VacantEntry's key,
    /// and returns a mutable reference to it.
    ///
    /// # Examples
    ///
    /// ```
    /// use ordnung::Map;
    /// use ordnung::Entry;
    ///
    /// let mut map: Map<&str, u32> = Map::new();
    ///
    /// if let Entry::Vacant(o) = map.entry("poneyland") {
    ///     o.insert(37);
    /// }
    /// assert_eq!(map["poneyland"], 37);
    /// ```
    #[inline]
    pub fn insert(self, value: V) -> &'a mut V
    where
        K: Eq + Hash,
    {
        let i = self.map.store.len();
        self.map.insert(self.key, value);
        if let Node { value: Some(v), .. } = unsafe { self.map.store.get_unchecked_mut(i) } {
            v
        } else {
            unreachable!()
        }
    }
}
