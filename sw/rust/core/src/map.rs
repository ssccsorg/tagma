use crate::coord::TagmaCoord;
use alloc::boxed::Box;

/// A collision-free, fixed-size, hash-less associative array indexed by
/// [`TagmaCoord`].
///
/// See the [module-level documentation](crate) for design rationale.
#[derive(Clone, Debug)]
pub struct TagmaMap<V> {
    slots: Box<[Option<V>]>,
    len: usize,
}

// ---------------------------------------------------------------------------
// Core read / write
// ---------------------------------------------------------------------------

impl<V> TagmaMap<V> {
    const N: usize = TagmaCoord::N_VALID;

    #[inline]
    fn idx(coord: TagmaCoord) -> usize {
        coord.index() as usize
    }

    #[inline]
    fn slot(&self, coord: TagmaCoord) -> &Option<V> {
        unsafe { self.slots.get_unchecked(Self::idx(coord)) }
    }

    #[inline]
    fn slot_mut(&mut self, coord: TagmaCoord) -> &mut Option<V> {
        unsafe { self.slots.get_unchecked_mut(Self::idx(coord)) }
    }

    // -- query ------------------------------------------------------------

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    #[inline]
    pub fn capacity() -> usize {
        Self::N
    }

    // -- lookup -----------------------------------------------------------

    #[inline]
    pub fn get(&self, coord: TagmaCoord) -> Option<&V> {
        self.slot(coord).as_ref()
    }
    #[inline]
    pub fn get_mut(&mut self, coord: TagmaCoord) -> Option<&mut V> {
        self.slot_mut(coord).as_mut()
    }
    #[inline]
    pub fn contains_key(&self, coord: TagmaCoord) -> bool {
        self.slot(coord).is_some()
    }

    // -- mutation ---------------------------------------------------------

    pub fn insert(&mut self, coord: TagmaCoord, value: V) -> Option<V> {
        let slot = self.slot_mut(coord);
        let old = slot.take();
        *slot = Some(value);
        if old.is_none() {
            self.len += 1;
        }
        old
    }

    pub fn remove(&mut self, coord: TagmaCoord) -> Option<V> {
        let slot = self.slot_mut(coord);
        let old = slot.take();
        if old.is_some() {
            self.len -= 1;
        }
        old
    }

    pub fn clear(&mut self) {
        for slot in self.slots.iter_mut() {
            *slot = None;
        }
        self.len = 0;
    }
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

impl<V> TagmaMap<V> {
    #[inline]
    pub fn new() -> Self {
        let slots = (0..Self::N).map(|_| None).collect::<Box<[_]>>();
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
// Entry API
// ---------------------------------------------------------------------------

impl<V> TagmaMap<V> {
    pub fn entry(&mut self, coord: TagmaCoord) -> Entry<'_, V> {
        if self.contains_key(coord) {
            Entry::Occupied(OccupiedEntry { map: self, coord })
        } else {
            Entry::Vacant(VacantEntry { map: self, coord })
        }
    }
}

pub enum Entry<'a, V> {
    Occupied(OccupiedEntry<'a, V>),
    Vacant(VacantEntry<'a, V>),
}

pub struct OccupiedEntry<'a, V> {
    pub(super) map: &'a mut TagmaMap<V>,
    pub(super) coord: TagmaCoord,
}

impl<'a, V> OccupiedEntry<'a, V> {
    pub fn key(&self) -> TagmaCoord {
        self.coord
    }

    /// Returns a reference to the stored value.
    ///
    /// # Safety
    ///
    /// `OccupiedEntry` guarantees the slot is occupied: the entry was created
    /// only after `contains_key` returned `true`, and the mutable borrow on
    /// the map prevents any concurrent removal.
    pub fn get(&self) -> &V {
        // SAFETY: the slot is verified occupied at entry creation.
        unsafe { self.map.get(self.coord).unwrap_unchecked() }
    }

    /// Returns a mutable reference to the stored value.
    pub fn get_mut(&mut self) -> &mut V {
        // SAFETY: same occupancy invariant as `get`.
        unsafe { self.map.get_mut(self.coord).unwrap_unchecked() }
    }

    /// Inserts a new value, returning the old one.
    pub fn insert(&mut self, value: V) -> V {
        // SAFETY: same occupancy invariant.
        unsafe { self.map.insert(self.coord, value).unwrap_unchecked() }
    }

    /// Removes and returns the value.
    pub fn remove_entry(self) -> V {
        // SAFETY: same occupancy invariant.
        unsafe { self.map.remove(self.coord).unwrap_unchecked() }
    }
}

pub struct VacantEntry<'a, V> {
    pub(super) map: &'a mut TagmaMap<V>,
    pub(super) coord: TagmaCoord,
}

impl<'a, V> VacantEntry<'a, V> {
    pub fn key(&self) -> TagmaCoord {
        self.coord
    }

    pub fn into_key(self) -> TagmaCoord {
        self.coord
    }

    /// Inserts a value and returns a mutable reference to it.
    pub fn insert(self, value: V) -> &'a mut V {
        // Vacant → guaranteed no old value to discard.
        let _ = self.map.insert(self.coord, value);
        // SAFETY: we just inserted the value above.
        unsafe { self.map.get_mut(self.coord).unwrap_unchecked() }
    }
}

impl<'a, V> Entry<'a, V> {
    pub fn key(&self) -> TagmaCoord {
        match self {
            Entry::Occupied(e) => e.key(),
            Entry::Vacant(e) => e.key(),
        }
    }

    pub fn or_insert(self, default: V) -> &'a mut V {
        self.or_insert_with(|| default)
    }

    pub fn or_insert_with<F: FnOnce() -> V>(self, f: F) -> &'a mut V {
        match self {
            Entry::Occupied(e) => unsafe { e.map.get_mut(e.coord).unwrap_unchecked() },
            Entry::Vacant(e) => e.insert(f()),
        }
    }

    pub fn or_insert_with_key<F: FnOnce(TagmaCoord) -> V>(self, f: F) -> &'a mut V {
        match self {
            Entry::Occupied(e) => unsafe { e.map.get_mut(e.coord).unwrap_unchecked() },
            Entry::Vacant(e) => {
                let v = f(e.coord);
                e.insert(v)
            }
        }
    }

    pub fn and_modify<F: FnOnce(&mut V)>(mut self, f: F) -> Self {
        if let Entry::Occupied(ref mut e) = self {
            f(e.get_mut());
        }
        self
    }
}

// ---------------------------------------------------------------------------
// Iteration
// ---------------------------------------------------------------------------

pub struct Iter<'a, V> {
    slots: core::slice::Iter<'a, Option<V>>,
    idx: u16,
}

impl<'a, V> Iterator for Iter<'a, V> {
    type Item = (TagmaCoord, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        for slot in self.slots.by_ref() {
            let coord = TagmaCoord::new(self.idx).unwrap();
            self.idx += 1;
            if let Some(val) = slot.as_ref() {
                return Some((coord, val));
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.slots.len()))
    }
}

pub struct IterMut<'a, V> {
    slots: core::slice::IterMut<'a, Option<V>>,
    idx: u16,
}

impl<'a, V> Iterator for IterMut<'a, V> {
    type Item = (TagmaCoord, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        for slot in self.slots.by_ref() {
            let coord = TagmaCoord::new(self.idx).unwrap();
            self.idx += 1;
            if let Some(val) = slot.as_mut() {
                return Some((coord, val));
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.slots.len()))
    }
}

pub struct IntoIter<V> {
    slots: alloc::vec::IntoIter<Option<V>>,
    idx: u16,
}

impl<V> Iterator for IntoIter<V> {
    type Item = (TagmaCoord, V);

    fn next(&mut self) -> Option<Self::Item> {
        for slot in self.slots.by_ref() {
            let coord = TagmaCoord::new(self.idx).unwrap();
            self.idx += 1;
            if let Some(val) = slot {
                return Some((coord, val));
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.slots.len()))
    }
}

pub struct Drain<'a, V> {
    slots: core::slice::IterMut<'a, Option<V>>,
    map_len: &'a mut usize,
    idx: u16,
}

impl<'a, V> Iterator for Drain<'a, V> {
    type Item = (TagmaCoord, V);

    fn next(&mut self) -> Option<Self::Item> {
        for slot in self.slots.by_ref() {
            let coord = TagmaCoord::new(self.idx).unwrap();
            self.idx += 1;
            if let Some(val) = slot.take() {
                *self.map_len -= 1;
                return Some((coord, val));
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.slots.len()))
    }
}

impl<'a, V> Drop for Drain<'a, V> {
    fn drop(&mut self) {
        for slot in self.slots.by_ref() {
            if slot.take().is_some() {
                *self.map_len -= 1;
            }
        }
        // After draining all remaining entries, the map must be empty.
        debug_assert_eq!(*self.map_len, 0);
    }
}

// ---------------------------------------------------------------------------
// Iterator constructors
// ---------------------------------------------------------------------------

impl<V> TagmaMap<V> {
    pub fn iter(&self) -> Iter<'_, V> {
        Iter {
            slots: self.slots.iter(),
            idx: 0,
        }
    }
    pub fn iter_mut(&mut self) -> IterMut<'_, V> {
        IterMut {
            slots: self.slots.iter_mut(),
            idx: 0,
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = TagmaCoord> + '_ {
        self.iter().map(|(k, _)| k)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> + '_ {
        self.iter().map(|(_, v)| v)
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> + '_ {
        self.iter_mut().map(|(_, v)| v)
    }

    pub fn drain(&mut self) -> Drain<'_, V> {
        Drain {
            slots: self.slots.iter_mut(),
            map_len: &mut self.len,
            idx: 0,
        }
    }

    pub fn retain<F: FnMut(TagmaCoord, &mut V) -> bool>(&mut self, mut f: F) {
        let mut idx = 0u16;
        self.slots.iter_mut().for_each(|slot| {
            let coord = TagmaCoord::new(idx).unwrap();
            idx += 1;
            if let Some(val) = slot.as_mut() {
                if !f(coord, val) {
                    *slot = None;
                    self.len -= 1;
                }
            }
        });
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
// IntoIterator
// ---------------------------------------------------------------------------

impl<V> IntoIterator for TagmaMap<V> {
    type Item = (TagmaCoord, V);
    type IntoIter = IntoIter<V>;

    fn into_iter(self) -> IntoIter<V> {
        let vec: alloc::vec::Vec<Option<V>> = self.slots.into_vec();
        IntoIter {
            slots: vec.into_iter(),
            idx: 0,
        }
    }
}

impl<'a, V> IntoIterator for &'a TagmaMap<V> {
    type Item = (TagmaCoord, &'a V);
    type IntoIter = Iter<'a, V>;

    fn into_iter(self) -> Iter<'a, V> {
        self.iter()
    }
}

impl<'a, V> IntoIterator for &'a mut TagmaMap<V> {
    type Item = (TagmaCoord, &'a mut V);
    type IntoIter = IterMut<'a, V>;

    fn into_iter(self) -> IterMut<'a, V> {
        self.iter_mut()
    }
}

// ---------------------------------------------------------------------------
// Index
// ---------------------------------------------------------------------------

impl<V> core::ops::Index<TagmaCoord> for TagmaMap<V> {
    type Output = V;

    fn index(&self, coord: TagmaCoord) -> &V {
        self.get(coord).expect("TagmaMap::index: key not present")
    }
}

impl<V> core::ops::IndexMut<TagmaCoord> for TagmaMap<V> {
    fn index_mut(&mut self, coord: TagmaCoord) -> &mut V {
        self.get_mut(coord)
            .expect("TagmaMap::index_mut: key not present")
    }
}

// ---------------------------------------------------------------------------
// Equality
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
        assert_eq!(TagmaMap::<u32>::capacity(), 11172);
    }

    #[test]
    fn insert_and_get() {
        let mut map = TagmaMap::new();
        let c = TagmaCoord::new(0).unwrap();
        assert_eq!(map.insert(c, 42), None);
        assert_eq!(map.get(c), Some(&42));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn insert_overwrite() {
        let mut map = TagmaMap::new();
        let c = TagmaCoord::new(0).unwrap();
        map.insert(c, 1);
        assert_eq!(map.insert(c, 2), Some(1));
        assert_eq!(map.get(c), Some(&2));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn remove() {
        let mut map = TagmaMap::new();
        let c = TagmaCoord::new(0).unwrap();
        map.insert(c, 42);
        assert_eq!(map.remove(c), Some(42));
        assert_eq!(map.get(c), None);
        assert!(map.is_empty());
    }

    #[test]
    fn contains_key() {
        let mut map = TagmaMap::new();
        let c = TagmaCoord::new(0).unwrap();
        assert!(!map.contains_key(c));
        map.insert(c, ());
        assert!(map.contains_key(c));
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
    fn iter_mut() {
        let mut map = TagmaMap::new();
        let c = TagmaCoord::new(5).unwrap();
        map.insert(c, 1);
        for (_, v) in map.iter_mut() {
            *v += 1;
        }
        assert_eq!(map.get(c), Some(&2));
    }

    #[test]
    fn into_iter_consuming() {
        let mut map = TagmaMap::new();
        let c = TagmaCoord::new(42).unwrap();
        map.insert(c, "hello");
        let collected: Vec<_> = map.into_iter().collect();
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].0, c);
        assert_eq!(collected[0].1, "hello");
    }

    #[test]
    fn into_iter_ref() {
        let mut map = TagmaMap::new();
        let c = TagmaCoord::new(7).unwrap();
        map.insert(c, 99);
        let collected: Vec<_> = (&map).into_iter().collect();
        assert_eq!(collected[0], (c, &99));
    }

    #[test]
    fn into_iter_mut_ref() {
        let mut map = TagmaMap::new();
        let c = TagmaCoord::new(7).unwrap();
        map.insert(c, 99);
        for (_, v) in &mut map {
            *v = 0;
        }
        assert_eq!(map.get(c), Some(&0));
    }

    #[test]
    fn keys_values() {
        let mut map = TagmaMap::new();
        map.insert(TagmaCoord::new(0).unwrap(), "a");
        map.insert(TagmaCoord::new(1).unwrap(), "b");
        assert_eq!(map.keys().count(), 2);
        assert_eq!(map.values().cloned().collect::<Vec<_>>(), vec!["a", "b"]);
    }

    #[test]
    fn drain_empties_map() {
        let mut map = TagmaMap::new();
        map.insert(TagmaCoord::new(0).unwrap(), 1);
        map.insert(TagmaCoord::new(1).unwrap(), 2);
        let drained: Vec<_> = map.drain().collect();
        assert_eq!(drained.len(), 2);
        assert!(map.is_empty());
    }

    #[test]
    fn retain() {
        let mut map = TagmaMap::new();
        for i in 0..10u16 {
            map.insert(TagmaCoord::new(i).unwrap(), i);
        }
        map.retain(|_, v| *v % 2 == 0);
        assert_eq!(map.len(), 5);
        for i in 0..10u16 {
            let c = TagmaCoord::new(i).unwrap();
            assert_eq!(map.get(c).is_some(), i % 2 == 0);
        }
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
    fn entry_or_insert() {
        let mut map = TagmaMap::new();
        let c = TagmaCoord::new(0).unwrap();
        map.entry(c).or_insert(42);
        assert_eq!(map.get(c), Some(&42));
        map.entry(c).or_insert(99);
        assert_eq!(map.get(c), Some(&42));
    }

    #[test]
    fn entry_or_insert_with_key() {
        let mut map = TagmaMap::new();
        let c = TagmaCoord::new(5).unwrap();
        map.entry(c).or_insert_with_key(|k| k.index() as u32);
        assert_eq!(map.get(c), Some(&5));
    }

    #[test]
    fn entry_and_modify() {
        let mut map = TagmaMap::new();
        let c = TagmaCoord::new(0).unwrap();
        map.insert(c, 1);
        map.entry(c).and_modify(|v| *v += 1).or_insert(0);
        assert_eq!(map.get(c), Some(&2));
    }

    #[test]
    fn entry_vacant_insert() {
        let mut map = TagmaMap::new();
        let c = TagmaCoord::new(42).unwrap();
        if let Entry::Vacant(e) = map.entry(c) {
            e.insert("hello");
        } else {
            panic!("should be vacant");
        }
        assert_eq!(map.get(c), Some(&"hello"));
    }

    #[test]
    fn entry_occupied_remove() {
        let mut map = TagmaMap::new();
        let c = TagmaCoord::new(7).unwrap();
        map.insert(c, "x");
        if let Entry::Occupied(e) = map.entry(c) {
            assert_eq!(e.remove_entry(), "x");
        } else {
            panic!("should be occupied");
        }
        assert!(!map.contains_key(c));
    }

    #[test]
    fn index_trait() {
        let mut map = TagmaMap::new();
        let c = TagmaCoord::new(3).unwrap();
        map.insert(c, 10);
        assert_eq!(map[c], 10);
        map[c] = 20;
        assert_eq!(map[c], 20);
    }

    #[test]
    #[should_panic(expected = "key not present")]
    fn index_panics_on_missing() {
        let map: TagmaMap<u32> = TagmaMap::new();
        let _ = &map[TagmaCoord::new(0).unwrap()];
    }

    #[test]
    fn eq() {
        let mut a = TagmaMap::new();
        let mut b = TagmaMap::new();
        a.insert(TagmaCoord::new(0).unwrap(), 1);
        b.insert(TagmaCoord::new(0).unwrap(), 1);
        assert_eq!(a, b);
        b.insert(TagmaCoord::new(1).unwrap(), 2);
        assert_ne!(a, b);
    }
}
