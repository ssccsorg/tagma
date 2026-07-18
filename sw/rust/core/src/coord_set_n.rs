use crate::coord_path::CoordPath;
use crate::coord_space_n::{CoordSpaceN, TreeIter};

// ---------------------------------------------------------------------------
// CoordSetN — sparse N-dimensional coordinate set
// ---------------------------------------------------------------------------

/// A sparse set of coordinates of depth N (>1), backed by a [`CoordSpaceN`]
/// tree. Memory is allocated lazily: only coordinates that are actually
/// inserted consume nodes.
///
/// For N=1, use [`CoordSet`](crate::coord_set::CoordSet) instead — it is a
/// dense bitset with zero allocation, bitwise operations, and approximately
/// 10x faster single-operation throughput.
///
/// # Set operations
///
/// `union`, `intersection`, `difference`, `is_subset`, and `is_disjoint`
/// are all implemented by walking the tree(s). Time is O(entries) per walk.
#[derive(Clone, Debug)]
pub struct CoordSetN<const N: usize>(CoordSpaceN<N, ()>);

impl<const N: usize> CoordSetN<N> {
    /// Creates an empty set.
    #[inline]
    pub fn new() -> Self {
        Self(CoordSpaceN::new())
    }

    /// Returns the number of coordinates in the set.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the set contains no coordinates.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns `true` if `path` is in the set.
    #[inline]
    pub fn contains(&self, path: &CoordPath<N>) -> bool {
        self.0.at_path(path).is_some()
    }

    /// Inserts `path` into the set.
    /// Returns `true` if `path` was newly inserted (was not already present).
    #[inline]
    pub fn insert(&mut self, path: CoordPath<N>) -> bool {
        self.0.place_path(&path, ()).is_none()
    }

    /// Removes all coordinates from the set.
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// An iterator over all `CoordPath` values in the set, in depth-first
    /// coordinate-ascending order.
    #[inline]
    pub fn iter(&self) -> TreeIter<'_, N, ()> {
        self.0.iter_tree()
    }

    // ── Set operations ──────────────────────────────────────────────

    /// Returns a new set containing all coordinates from `self` and `other`.
    pub fn union(&self, other: &Self) -> Self {
        let mut result = Self::new();
        for (path, _) in self.0.iter_tree() {
            result.0.place_path(&path, ());
        }
        for (path, _) in other.0.iter_tree() {
            result.0.place_path(&path, ());
        }
        result
    }

    /// Returns a new set containing only coordinates present in both sets.
    /// Iterates the smaller set for efficiency.
    pub fn intersection(&self, other: &Self) -> Self {
        let (iter, check) = if self.len() <= other.len() {
            (self, other)
        } else {
            (other, self)
        };
        let mut result = Self::new();
        for (path, _) in iter.0.iter_tree() {
            if check.contains(&path) {
                result.0.place_path(&path, ());
            }
        }
        result
    }

    /// Returns a new set containing coordinates in `self` but not in `other`.
    pub fn difference(&self, other: &Self) -> Self {
        let mut result = Self::new();
        for (path, _) in self.0.iter_tree() {
            if !other.contains(&path) {
                result.0.place_path(&path, ());
            }
        }
        result
    }

    /// Returns `true` if all coordinates in `self` are also in `other`.
    pub fn is_subset(&self, other: &Self) -> bool {
        for (path, _) in self.0.iter_tree() {
            if !other.contains(&path) {
                return false;
            }
        }
        true
    }

    /// Returns `true` if the two sets have no coordinates in common.
    pub fn is_disjoint(&self, other: &Self) -> bool {
        // Iterate the smaller set for efficiency.
        let (a, b) = if self.len() <= other.len() {
            (self, other)
        } else {
            (other, self)
        };
        for (path, _) in a.0.iter_tree() {
            if b.contains(&path) {
                return false;
            }
        }
        true
    }

    /// Returns a new set containing coordinates in `self` or `other` but not both.
    pub fn symmetric_difference(&self, other: &Self) -> Self {
        let mut result = Self::new();
        for (path, _) in self.0.iter_tree() {
            if !other.contains(&path) {
                result.0.place_path(&path, ());
            }
        }
        for (path, _) in other.0.iter_tree() {
            if !self.contains(&path) {
                result.0.place_path(&path, ());
            }
        }
        result
    }
}

impl<const N: usize> FromIterator<CoordPath<N>> for CoordSetN<N> {
    fn from_iter<I: IntoIterator<Item = CoordPath<N>>>(iter: I) -> Self {
        let mut set = Self::new();
        for path in iter {
            set.insert(path);
        }
        set
    }
}

impl<const N: usize> Default for CoordSetN<N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> PartialEq for CoordSetN<N> {
    fn eq(&self, other: &Self) -> bool {
        // Sets are equal if each is a subset of the other.
        if self.len() != other.len() {
            return false;
        }
        self.is_subset(other)
    }
}
impl<const N: usize> Eq for CoordSetN<N> {}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Coord;
    use alloc::vec;
    use alloc::vec::Vec;

    fn c(idx: u16) -> Coord {
        Coord::new(idx).unwrap()
    }
    fn path<const N: usize>(indices: &[u16]) -> CoordPath<N> {
        assert_eq!(indices.len(), N, "path length must match depth");
        let mut arr = [c(0); N];
        for (i, &idx) in indices.iter().enumerate() {
            arr[i] = c(idx);
        }
        CoordPath::new(arr)
    }

    #[test]
    fn new_set_is_empty() {
        let s: CoordSetN<2> = CoordSetN::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn insert_and_contains() {
        let mut s = CoordSetN::<2>::new();
        let p = path(&[1, 2]);
        assert!(s.insert(p));
        assert!(s.contains(&p));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn insert_duplicate_returns_false() {
        let mut s = CoordSetN::<2>::new();
        let p = path(&[1, 2]);
        assert!(s.insert(p));
        assert!(!s.insert(p));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn contains_nonexistent_returns_false() {
        let s: CoordSetN<2> = CoordSetN::new();
        assert!(!s.contains(&path(&[0, 0])));
    }

    #[test]
    fn clear_removes_all() {
        let mut s = CoordSetN::<2>::new();
        s.insert(path(&[1, 2]));
        s.insert(path(&[3, 4]));
        assert_eq!(s.len(), 2);
        s.clear();
        assert!(s.is_empty());
    }

    #[test]
    fn iter_yields_all_paths() {
        let mut s = CoordSetN::<2>::new();
        s.insert(path(&[1, 2]));
        s.insert(path(&[3, 4]));
        let paths: Vec<_> = s.iter().map(|(p, _)| p).collect();
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn union_combines_both() {
        let mut a = CoordSetN::<2>::new();
        a.insert(path(&[1, 2]));
        let mut b = CoordSetN::<2>::new();
        b.insert(path(&[3, 4]));
        let u = a.union(&b);
        assert!(u.contains(&path(&[1, 2])));
        assert!(u.contains(&path(&[3, 4])));
        assert_eq!(u.len(), 2);
    }

    #[test]
    fn union_deduplicates() {
        let mut a = CoordSetN::<2>::new();
        a.insert(path(&[1, 2]));
        let mut b = CoordSetN::<2>::new();
        b.insert(path(&[1, 2]));
        b.insert(path(&[3, 4]));
        let u = a.union(&b);
        assert_eq!(u.len(), 2);
    }

    #[test]
    fn intersection_common_only() {
        let mut a = CoordSetN::<2>::new();
        a.insert(path(&[1, 2]));
        a.insert(path(&[5, 6]));
        let mut b = CoordSetN::<2>::new();
        b.insert(path(&[1, 2]));
        b.insert(path(&[3, 4]));
        let i = a.intersection(&b);
        assert!(i.contains(&path(&[1, 2])));
        assert_eq!(i.len(), 1);
    }

    #[test]
    fn intersection_empty_when_disjoint() {
        let mut a = CoordSetN::<2>::new();
        a.insert(path(&[1, 2]));
        let mut b = CoordSetN::<2>::new();
        b.insert(path(&[3, 4]));
        let i = a.intersection(&b);
        assert!(i.is_empty());
    }

    #[test]
    fn difference_subtracts() {
        let mut a = CoordSetN::<2>::new();
        a.insert(path(&[1, 2]));
        a.insert(path(&[5, 6]));
        let mut b = CoordSetN::<2>::new();
        b.insert(path(&[1, 2]));
        let d = a.difference(&b);
        assert!(d.contains(&path(&[5, 6])));
        assert_eq!(d.len(), 1);
    }

    #[test]
    fn is_subset_true() {
        let mut a = CoordSetN::<2>::new();
        a.insert(path(&[1, 2]));
        let mut b = CoordSetN::<2>::new();
        b.insert(path(&[1, 2]));
        b.insert(path(&[3, 4]));
        assert!(a.is_subset(&b));
    }

    #[test]
    fn is_subset_false() {
        let mut a = CoordSetN::<2>::new();
        a.insert(path(&[1, 2]));
        a.insert(path(&[5, 6]));
        let mut b = CoordSetN::<2>::new();
        b.insert(path(&[1, 2]));
        assert!(!a.is_subset(&b));
    }

    #[test]
    fn is_disjoint_true() {
        let mut a = CoordSetN::<2>::new();
        a.insert(path(&[1, 2]));
        let mut b = CoordSetN::<2>::new();
        b.insert(path(&[3, 4]));
        assert!(a.is_disjoint(&b));
    }

    #[test]
    fn is_disjoint_false() {
        let mut a = CoordSetN::<2>::new();
        a.insert(path(&[1, 2]));
        let mut b = CoordSetN::<2>::new();
        b.insert(path(&[1, 2]));
        b.insert(path(&[3, 4]));
        assert!(!a.is_disjoint(&b));
    }

    #[test]
    fn depth_3_works() {
        let mut s = CoordSetN::<3>::new();
        let p = path(&[0, 1, 2]);
        s.insert(p);
        assert!(s.contains(&p));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn eq_same_content() {
        let mut a = CoordSetN::<2>::new();
        a.insert(path(&[1, 2]));
        a.insert(path(&[3, 4]));
        let mut b = CoordSetN::<2>::new();
        b.insert(path(&[3, 4]));
        b.insert(path(&[1, 2]));
        assert_eq!(a, b);
    }

    #[test]
    fn intersection_iterates_smaller_set() {
        let mut large = CoordSetN::<2>::new();
        for i in 0..100 {
            large.insert(path(&[i, 0]));
        }
        let mut small = CoordSetN::<2>::new();
        small.insert(path(&[1, 0]));
        let i = small.intersection(&large);
        assert_eq!(i.len(), 1);
    }

    #[test]
    fn symmetric_difference_exclusive_only() {
        let mut a = CoordSetN::<2>::new();
        a.insert(path(&[1, 2]));
        a.insert(path(&[3, 4]));
        let mut b = CoordSetN::<2>::new();
        b.insert(path(&[3, 4]));
        b.insert(path(&[5, 6]));
        let d = a.symmetric_difference(&b);
        assert!(d.contains(&path(&[1, 2])));
        assert!(!d.contains(&path(&[3, 4])));
        assert!(d.contains(&path(&[5, 6])));
        assert_eq!(d.len(), 2);
    }

    #[test]
    fn from_iterator_collects_all() {
        let paths = vec![path(&[1, 2]), path(&[3, 4])];
        let s: CoordSetN<2> = paths.into_iter().collect();
        assert_eq!(s.len(), 2);
        assert!(s.contains(&path(&[1, 2])));
        assert!(s.contains(&path(&[3, 4])));
    }

    #[test]
    fn from_iterator_deduplicates() {
        let paths = vec![path(&[1, 2]), path(&[1, 2]), path(&[3, 4])];
        let s: CoordSetN<2> = paths.into_iter().collect();
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn eq_different_content() {
        let mut a = CoordSetN::<2>::new();
        a.insert(path(&[1, 2]));
        let mut b = CoordSetN::<2>::new();
        b.insert(path(&[3, 4]));
        assert_ne!(a, b);
    }
}
