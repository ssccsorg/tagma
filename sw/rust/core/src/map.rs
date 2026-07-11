use crate::coord::TagmaCoord;

/// A collision-free, fixed-size, hash-less associative array indexed by
/// [`TagmaCoord`].
///
/// `TagmaMap<V>` is a direct-address table: each valid coordinate maps to
/// exactly one slot.  There is no hashing, no collision resolution, and no
/// dynamic resizing.
///
/// # Memory
///
/// An empty `TagmaMap<()>` occupies `core::mem::size_of::<Option<()>>() × 11172`
/// bytes (~11 KB for ZST).  For a `V` of pointer size the map is ~89 KB
/// (empty) and grows only through the stored values, not the slots themselves.
///
/// # Guarantees
///
/// - `get`/`insert`/`remove` are **O(1)** in the worst case (not just average).
/// - No reallocation ever occurs.
/// - `no_std` compatible (no allocator required).
#[derive(Clone, Debug)]
pub struct TagmaMap<V> {
    slots: [Option<V>; TagmaCoord::N_VALID],
    len: usize,
}

impl<V> TagmaMap<V> {
    /// Returns the number of elements in the map.
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the map contains no elements.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the capacity of the map (always `TagmaCoord::N_VALID`).
    #[inline]
    pub const fn capacity() -> usize {
        TagmaCoord::N_VALID
    }

    /// Returns a reference to the value stored at `coord`, or `None`.
    #[inline]
    pub fn get(&self, coord: TagmaCoord) -> Option<&V> {
        // Safety: coord.index() is guaranteed < N_VALID.
        unsafe { self.slots.get_unchecked(coord.index() as usize).as_ref() }
    }

    /// Returns a mutable reference to the value stored at `coord`, or `None`.
    #[inline]
    pub fn get_mut(&mut self, coord: TagmaCoord) -> Option<&mut V> {
        unsafe { self.slots.get_unchecked_mut(coord.index() as usize).as_mut() }
    }

    /// Inserts a value at `coord`.
    ///
    /// If the map already had a value at this coordinate, the old value is
    /// returned; otherwise `None` is returned.
    pub fn insert(&mut self, coord: TagmaCoord, value: V) -> Option<V> {
        let slot = unsafe { self.slots.get_unchecked_mut(coord.index() as usize) };
        let old = slot.take();
        *slot = Some(value);
        if old.is_none() {
            self.len += 1;
        }
        old
    }

    /// Removes the value at `coord`, returning it if present.
    pub fn remove(&mut self, coord: TagmaCoord) -> Option<V> {
        let slot = unsafe { self.slots.get_unchecked_mut(coord.index() as usize) };
        let old = slot.take();
        if old.is_some() {
            self.len -= 1;
        }
        old
    }

    /// Returns `true` if the map contains a value at `coord`.
    #[inline]
    pub fn contains_key(&self, coord: TagmaCoord) -> bool {
        unsafe { self.slots.get_unchecked(coord.index() as usize).is_some() }
    }

    /// Clears the map, removing all key-value pairs.
    pub fn clear(&mut self) {
        for slot in self.slots.iter_mut() {
            *slot = None;
        }
        self.len = 0;
    }

    /// An iterator visiting all key-value pairs in index order.
    ///
    /// The iterator yields `(TagmaCoord, &V)` for every occupied slot.
    #[inline]
    pub fn iter(&self) -> Iter<'_, V> {
        Iter {
            inner: self.slots.iter(),
            idx: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Iteration
// ---------------------------------------------------------------------------

/// An iterator over the entries of a `TagmaMap`.
pub struct Iter<'a, V> {
    inner: core::slice::Iter<'a, Option<V>>,
    idx: u16,
}

impl<'a, V> Iterator for Iter<'a, V> {
    type Item = (TagmaCoord, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(slot) = self.inner.next() {
            let coord = TagmaCoord::new(self.idx).unwrap();
            self.idx += 1;
            if let Some(value) = slot.as_ref() {
                return Some((coord, value));
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.inner.len()))
    }
}

// ---------------------------------------------------------------------------
// FromIterator
// ---------------------------------------------------------------------------

impl<V> FromIterator<(TagmaCoord, V)> for TagmaMap<V> {
    fn from_iter<I: IntoIterator<Item = (TagmaCoord, V)>>(iter: I) -> Self {
        let mut map = Self::new();
        for (coord, value) in iter {
            map.insert(coord, value);
        }
        map
    }
}

// ---------------------------------------------------------------------------
// Default
// ---------------------------------------------------------------------------

impl<V> TagmaMap<V> {
    /// Creates an empty `TagmaMap`.
    ///
    /// Every slot is initialised to `None`.
    pub fn new() -> Self {
        let slots = core::array::from_fn(|_| None::<V>);
        TagmaMap { slots, len: 0 }
    }
}

impl<V> Default for TagmaMap<V> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// PartialEq
// ---------------------------------------------------------------------------

impl<V: PartialEq> PartialEq for TagmaMap<V> {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.slots == other.slots
    }
}

impl<V: PartialEq> Eq for TagmaMap<V> {}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_map_is_empty() {
        let map: TagmaMap<u32> = TagmaMap::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn insert_and_get() {
        let mut map = TagmaMap::new();
        let coord = TagmaCoord::new(0).unwrap();
        assert_eq!(map.insert(coord, 42), None);
        assert_eq!(map.get(coord), Some(&42));
        assert!(!map.is_empty());
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn insert_overwrite() {
        let mut map = TagmaMap::new();
        let coord = TagmaCoord::new(0).unwrap();
        map.insert(coord, 1);
        assert_eq!(map.insert(coord, 2), Some(1));
        assert_eq!(map.get(coord), Some(&2));
    }

    #[test]
    fn remove() {
        let mut map = TagmaMap::new();
        let coord = TagmaCoord::new(0).unwrap();
        map.insert(coord, 42);
        assert_eq!(map.remove(coord), Some(42));
        assert_eq!(map.get(coord), None);
        assert!(map.is_empty());
    }

    #[test]
    fn contains_key() {
        let mut map = TagmaMap::new();
        let coord = TagmaCoord::new(0).unwrap();
        assert!(!map.contains_key(coord));
        map.insert(coord, ());
        assert!(map.contains_key(coord));
    }

    #[test]
    fn clear() {
        let mut map = TagmaMap::new();
        map.insert(TagmaCoord::new(0).unwrap(), 1);
        map.insert(TagmaCoord::new(100).unwrap(), 2);
        map.clear();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn iter_empty() {
        let map: TagmaMap<u32> = TagmaMap::new();
        assert_eq!(map.iter().count(), 0);
    }

    #[test]
    fn iter_non_empty() {
        let mut map = TagmaMap::new();
        let c1 = TagmaCoord::new(0).unwrap();
        let c2 = TagmaCoord::new(9999).unwrap();
        map.insert(c1, 10);
        map.insert(c2, 20);
        let entries: Vec<_> = map.iter().collect();
        assert_eq!(entries.len(), 2);
        assert!(entries.contains(&(c1, &10)));
        assert!(entries.contains(&(c2, &20)));
    }

    #[test]
    fn from_iterator() {
        let coords: Vec<_> = (0..5u16)
            .map(|i| (TagmaCoord::new(i).unwrap(), i * 10))
            .collect();
        let map: TagmaMap<u16> = coords.into_iter().collect();
        assert_eq!(map.len(), 5);
        assert_eq!(map.get(TagmaCoord::new(3).unwrap()), Some(&30));
    }

    #[test]
    fn slot_independent() {
        let mut map = TagmaMap::new();
        let a = TagmaCoord::new(0).unwrap();
        let b = TagmaCoord::new(11171).unwrap();
        map.insert(a, "first");
        map.insert(b, "last");
        assert_eq!(map.get(a), Some(&"first"));
        assert_eq!(map.get(b), Some(&"last"));
        assert_eq!(map.len(), 2);
    }
}
