use crate::coord::Coord;
use alloc::boxed::Box;
use alloc::vec::Vec;

// ---------------------------------------------------------------------------
// DynCoordMap — dynamic-depth Coord-addressed map
// ---------------------------------------------------------------------------

/// A collision-free map indexed by a slice of [`Coord`], with dynamic depth.
///
/// Each level is a fixed 11,172-slot array indexed directly by `Coord` —
/// no hashing, no collisions, regardless of depth.
///
/// Unlike [`CoordMap`] (no_alloc, N=1) and [`CoordMap6`] (compile-time N=6),
/// the depth is determined at runtime by the length of the path slice.
/// Memory is allocated lazily: only paths that are actually written to
/// consume nodes.
#[derive(Debug)]
pub struct DynCoordMap<V> {
    slots: Box<[Option<Slot<V>>]>,
}

#[derive(Debug)]
enum Slot<V> {
    Leaf(V),
    Node(Box<DynCoordMap<V>>),
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

impl<V> DynCoordMap<V> {
    /// Creates an empty `DynCoordMap`.
    #[inline]
    pub fn new() -> Self {
        DynCoordMap {
            slots: (0..11172)
                .map(|_| None)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }
}

impl<V> Default for DynCoordMap<V> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Core: get / insert / remove
// ---------------------------------------------------------------------------

impl<V> DynCoordMap<V> {
    /// Returns a reference to the value at `path`.
    ///
    /// Time: O(path.len()) — one array access per coord.
    pub fn get(&self, path: &[Coord]) -> Option<&V> {
        let mut node = self;
        for (i, &coord) in path.iter().enumerate() {
            let idx = coord.index() as usize;
            match node.slots[idx].as_ref()? {
                Slot::Leaf(v) if i == path.len() - 1 => return Some(v),
                Slot::Node(child) => node = child,
                _ => return None,
            }
        }
        None
    }

    /// Inserts a value at `path`. Creates intermediate nodes as needed.
    /// Returns the previous value if the exact path already existed.
    pub fn insert(&mut self, path: &[Coord], value: V) -> Option<V> {
        self.insert_rec(path, 0, value)
    }

    fn insert_rec(&mut self, path: &[Coord], depth: usize, value: V) -> Option<V> {
        let idx = path[depth].index() as usize;
        if depth == path.len() - 1 {
            let slot = &mut self.slots[idx];
            match slot {
                Some(Slot::Leaf(old)) => Some(core::mem::replace(old, value)),
                Some(Slot::Node(_)) => {
                    *slot = Some(Slot::Leaf(value));
                    None
                }
                None => {
                    *slot = Some(Slot::Leaf(value));
                    None
                }
            }
        } else {
            let child = &mut self.slots[idx];
            if child.is_none() {
                *child = Some(Slot::Node(Box::default()));
            }
            match child.as_mut().unwrap() {
                Slot::Node(sub) => sub.insert_rec(path, depth + 1, value),
                Slot::Leaf(_) => {
                    *child = Some(Slot::Node(Box::default()));
                    match child.as_mut().unwrap() {
                        Slot::Node(sub) => sub.insert_rec(path, depth + 1, value),
                        _ => unreachable!(),
                    }
                }
            }
        }
    }

    /// Removes the value at `path`, returning it if present.
    pub fn remove(&mut self, path: &[Coord]) -> Option<V> {
        self.remove_rec(path, 0)
    }

    fn remove_rec(&mut self, path: &[Coord], depth: usize) -> Option<V> {
        let idx = path[depth].index() as usize;
        if depth == path.len() - 1 {
            match self.slots[idx].take() {
                Some(Slot::Leaf(v)) => Some(v),
                _ => None,
            }
        } else {
            match &mut self.slots[idx] {
                Some(Slot::Node(sub)) => sub.remove_rec(path, depth + 1),
                _ => None,
            }
        }
    }

    /// Clears all entries.
    pub fn clear(&mut self) {
        for slot in self.slots.iter_mut() {
            *slot = None;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let m: DynCoordMap<u32> = DynCoordMap::new();
        assert_eq!(m.get(&[Coord::new(0).unwrap()]), None);
    }

    #[test]
    fn depth_1() {
        let mut m = DynCoordMap::new();
        let c = Coord::new(42).unwrap();
        assert_eq!(m.insert(&[c], 7), None);
        assert_eq!(m.get(&[c]), Some(&7));
    }

    #[test]
    fn depth_2() {
        let mut m = DynCoordMap::new();
        let path = [Coord::new(0).unwrap(), Coord::new(1).unwrap()];
        m.insert(&path, 42);
        assert_eq!(m.get(&path), Some(&42));
    }

    #[test]
    fn depth_3() {
        let mut m = DynCoordMap::new();
        let path = [
            Coord::new(0).unwrap(),
            Coord::new(1).unwrap(),
            Coord::new(2).unwrap(),
        ];
        m.insert(&path, 99);
        assert_eq!(m.get(&path), Some(&99));
    }

    #[test]
    fn independent_paths() {
        let mut m = DynCoordMap::new();
        let a = [Coord::new(0).unwrap(), Coord::new(0).unwrap()];
        let b = [Coord::new(0).unwrap(), Coord::new(1).unwrap()];
        m.insert(&a, 10);
        m.insert(&b, 20);
        assert_eq!(m.get(&a), Some(&10));
        assert_eq!(m.get(&b), Some(&20));
    }

    #[test]
    fn overwrite() {
        let mut m = DynCoordMap::new();
        let path = [Coord::new(5).unwrap()];
        m.insert(&path, 1);
        assert_eq!(m.insert(&path, 2), Some(1));
        assert_eq!(m.get(&path), Some(&2));
    }

    #[test]
    fn remove() {
        let mut m = DynCoordMap::new();
        let path = [Coord::new(0).unwrap(), Coord::new(1).unwrap()];
        m.insert(&path, 42);
        assert_eq!(m.remove(&path), Some(42));
        assert_eq!(m.get(&path), None);
    }

    #[test]
    fn mixed_depths() {
        let mut m = DynCoordMap::new();
        let d1 = [Coord::new(1).unwrap()];
        let d3 = [
            Coord::new(1).unwrap(),
            Coord::new(2).unwrap(),
            Coord::new(3).unwrap(),
        ];
        m.insert(&d1, 10);
        // Inserting deeper at the same prefix overwrites the shallow value
        m.insert(&d3, 30);
        assert_eq!(m.get(&d3), Some(&30));
        // d1 was destroyed because its coord slot was converted to a Node
        assert_eq!(m.get(&d1), None);
    }

    #[test]
    fn clear() {
        let mut m = DynCoordMap::new();
        m.insert(&[Coord::new(0).unwrap()], 1);
        m.insert(&[Coord::new(1).unwrap(), Coord::new(0).unwrap()], 2);
        m.clear();
        // After clear, both paths should return None
        assert_eq!(m.get(&[Coord::new(0).unwrap()]), None);
        assert_eq!(
            m.get(&[Coord::new(1).unwrap(), Coord::new(0).unwrap()]),
            None
        );
    }

    #[test]
    fn boundary() {
        let mut m = DynCoordMap::new();
        let first = Coord::new(0).unwrap();
        let last = Coord::new(11171).unwrap();
        m.insert(&[first, last], 42);
        assert_eq!(m.get(&[first, last]), Some(&42));
    }

    #[test]
    fn missing_path() {
        let m: DynCoordMap<u32> = DynCoordMap::new();
        assert_eq!(
            m.get(&[Coord::new(0).unwrap(), Coord::new(0).unwrap()]),
            None
        );
    }
}
