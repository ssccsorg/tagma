use crate::coord_path::CoordPath;
use alloc::alloc::{alloc_zeroed, Layout};
use alloc::boxed::Box;
use core::ptr;
use core::slice;

// ---------------------------------------------------------------------------
// Dense CoordSpace family via macro
// ---------------------------------------------------------------------------

/// Slot count for a dense CoordSpace at depth N.
///
/// For N=1:  11,172
/// For N=2:  124,813,584
/// For N=3:  1,394,417,360,448  (mmap required; beyond single heap alloc)
/// For N=4+: overflow; not representable in usize on 64-bit.
/// Slot count for a dense CoordSpace at depth N.
const fn coord_slots(n: usize) -> usize {
    match n {
        1 => 11172,
        2 => 11172 * 11172,
        3 => 11172 * 11172 * 11172,
        _ => 0, // unreachable; guarded by call sites
    }
}

/// Defines a dense CoordSpace type with a fixed depth N.
///
/// Generated API: `new`, `at_path`, `place_path`, `vacate_path`,
/// `clear`, `len`, `is_empty`, `PartialEq`, `Clone`, `Debug`.
///
/// # Allocation
///
/// A single `alloc_zeroed` call on construction. For N=2 with V=()
/// this is 119 MB; for V=u32 it is 952 MB. The caller must ensure
/// the allocation size is acceptable for their deployment.
///
/// # Safety
///
/// Relies on `Option<V>` having an all-zero `None` bit pattern
/// (same invariant as `CoordSpace`).
macro_rules! define_dense_coord_space {
    ($name:ident, $n:expr) => {
        // const _: () = {
            const SLOT_COUNT: usize = coord_slots($n);

            #[doc = concat!(
                "Dense, zeroed, heap-allocated CoordSpace for N=", stringify!($n), ".\n\n",
                "All ", stringify!($n), " syllable(s): ", stringify!(SLOT_COUNT), " slots.\n",
                "Backed by a single `alloc_zeroed` call — true Tagma, no hashing, no tree.\n\n",
                "# Panics\n\n",
                "Panics if `SLOT_COUNT * size_of::<Option<V>>()` overflows `usize`."
            )]
            pub struct $name<V> {
                slots: Box<[Option<V>]>,
                len: usize,
            }

            impl<V> $name<V> {
                /// Creates an empty dense space with all slots zeroed (None).
                ///
                /// Allocation size: `SLOT_COUNT * size_of::<Option<V>>()` bytes.
                /// Initialized via `alloc_zeroed` — pages are lazily committed by the OS.
                #[inline]
                pub fn new() -> Self {
                    let slot_bytes = core::mem::size_of::<Option<V>>()
                        .checked_mul(SLOT_COUNT)
                        .expect("overflow in CoordSpace dense allocation size");
                    let layout = Layout::from_size_align(slot_bytes, core::mem::align_of::<Option<V>>())
                        .expect("invalid Layout for CoordSpace dense array");
                    let ptr = unsafe { alloc_zeroed(layout) as *mut Option<V> };
                    assert!(!ptr.is_null(), "CoordSpace dense allocation failed");
                    let slots = unsafe { Box::from_raw(slice::from_raw_parts_mut(ptr, SLOT_COUNT)) };
                    $name { slots, len: 0 }
                }

                /// Returns the number of occupied slots.
                #[inline]
                pub fn len(&self) -> usize {
                    self.len
                }

                /// Returns `true` if no slots are occupied.
                #[inline]
                pub fn is_empty(&self) -> bool {
                    self.len == 0
                }

                /// Returns a reference to the value at `path`, or `None`.
                pub fn at_path(&self, path: &CoordPath<$n>) -> Option<&V> {
                    let idx = linear_index::<$n>(path);
                    // SAFETY: linear_index returns a value < SLOT_COUNT for valid CoordPaths.
                    unsafe { (*self.slots.as_ptr().add(idx)).as_ref() }
                }

                /// Places `value` at `path`. Returns the previous value if any.
                pub fn place_path(&mut self, path: &CoordPath<$n>, value: V) -> Option<V> {
                    let idx = linear_index::<$n>(path);
                    let slot = unsafe { &mut *self.slots.as_mut_ptr().add(idx) };
                    let prev = slot.take();
                    *slot = Some(value);
                    if prev.is_none() {
                        self.len += 1;
                    }
                    prev
                }

                /// Removes the value at `path`. Returns it if present.
                pub fn vacate_path(&mut self, path: &CoordPath<$n>) -> Option<V> {
                    let idx = linear_index::<$n>(path);
                    let slot = unsafe { &mut *self.slots.as_mut_ptr().add(idx) };
                    let prev = slot.take();
                    if prev.is_some() {
                        self.len -= 1;
                    }
                    prev
                }

                /// Removes all values. Retains the allocation.
                pub fn clear(&mut self) {
                    // Zero the entire allocation (faster than iterating).
                    let ptr = self.slots.as_mut_ptr() as *mut u8;
                    let bytes = SLOT_COUNT * core::mem::size_of::<Option<V>>();
                    unsafe { ptr::write_bytes(ptr, 0, bytes) };
                    self.len = 0;
                }
            }

            impl<V> Default for $name<V> {
                #[inline]
                fn default() -> Self {
                    Self::new()
                }
            }

            impl<V: core::fmt::Debug> core::fmt::Debug for $name<V> {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.debug_struct(stringify!($name))
                        .field("N", &($n))
                        .field("len", &self.len)
                        .field("capacity", &SLOT_COUNT)
                        .finish()
                }
            }

            impl<V: PartialEq> PartialEq for $name<V> {
                fn eq(&self, other: &Self) -> bool {
                    self.slots == other.slots
                }
            }
            impl<V: PartialEq> Eq for $name<V> {}

            impl<V: Clone> Clone for $name<V> {
                fn clone(&self) -> Self {
                    $name {
                        slots: self.slots.clone(),
                        len: self.len,
                    }
                }
            }

            impl<V> FromIterator<(CoordPath<$n>, V)> for $name<V> {
                fn from_iter<I: IntoIterator<Item = (CoordPath<$n>, V)>>(iter: I) -> Self {
                    let mut space = Self::new();
                    for (path, value) in iter {
                        space.place_path(&path, value);
                    }
                    space
                }
            }
        // };
    };
}

// ---------------------------------------------------------------------------
// Concrete types
// ---------------------------------------------------------------------------

define_dense_coord_space!(CoordSpace2, 2);

// (N=3+ via mmap deferred — issue #28)

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Linear index into the flat dense array for a CoordPath of depth N.
///
/// For N=1: `c0`
/// For N=2: `c0 * 11172 + c1`
/// For N=3: `(c0 * 11172 + c1) * 11172 + c2`
#[inline]
fn linear_index<const N: usize>(path: &CoordPath<N>) -> usize {
    let mut idx = 0usize;
    let mut i = 0;
    while i < N {
        idx = idx.wrapping_mul(11172).wrapping_add(path.coords()[i].index() as usize);
        i += 1;
    }
    idx
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Coord, CoordPath};
    use alloc::vec::Vec;

    #[test]
    fn new_is_empty() {
        let s = CoordSpace2::<u32>::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn insert_and_get() {
        let mut s = CoordSpace2::<u32>::new();
        let p = CoordPath::new([Coord::new(0).unwrap(), Coord::new(1).unwrap()]);
        assert_eq!(s.place_path(&p, 42), None);
        assert_eq!(s.at_path(&p), Some(&42));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn insert_overwrite() {
        let mut s = CoordSpace2::<u32>::new();
        let p = CoordPath::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()]);
        s.place_path(&p, 1);
        assert_eq!(s.place_path(&p, 2), Some(1));
        assert_eq!(s.at_path(&p), Some(&2));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn vacate() {
        let mut s = CoordSpace2::<u32>::new();
        let p = CoordPath::new([Coord::new(5).unwrap(), Coord::new(5).unwrap()]);
        s.place_path(&p, 99);
        assert_eq!(s.vacate_path(&p), Some(99));
        assert!(s.is_empty());
    }

    #[test]
    fn clear() {
        let mut s = CoordSpace2::<u32>::new();
        s.place_path(&CoordPath::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()]), 1);
        s.place_path(&CoordPath::new([Coord::new(1).unwrap(), Coord::new(2).unwrap()]), 2);
        assert_eq!(s.len(), 2);
        s.clear();
        assert!(s.is_empty());
        assert_eq!(s.at_path(&CoordPath::new([
            Coord::new(0).unwrap(), Coord::new(0).unwrap()
        ])), None);
    }

    #[test]
    fn clone_eq() {
        let mut a = CoordSpace2::<u32>::new();
        let p = CoordPath::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()]);
        a.place_path(&p, 42);
        let b = a.clone();
        assert_eq!(a, b);
        assert_eq!(b.at_path(&p), Some(&42));
    }

    #[test]
    fn from_iterator() {
        let paths: Vec<_> = (0u16..10).map(|i| {
            let p = CoordPath::new([Coord::new(i).unwrap(), Coord::new(i + 1).unwrap()]);
            (p, i as u32)
        }).collect();
        let s: CoordSpace2<u32> = paths.into_iter().collect();
        assert_eq!(s.len(), 10);
    }

    #[test]
    fn third_slot_nonzero_stays_none() {
        let s = CoordSpace2::<u32>::new();
        // Slot that should never have been touched.
        let p = CoordPath::new([Coord::new(9999).unwrap(), Coord::new(8888).unwrap()]);
        assert_eq!(s.at_path(&p), None);
    }

    #[test]
    fn linear_index_correct() {
        let c0 = Coord::new(42).unwrap();
        let c1 = Coord::new(77).unwrap();
        let p = CoordPath::new([c0, c1]);
        let idx = linear_index::<2>(&p);
        assert_eq!(idx, 42usize * 11172 + 77);
    }

    #[test]
    fn linear_index_zero() {
        let c0 = Coord::new(0).unwrap();
        let c1 = Coord::new(0).unwrap();
        let p = CoordPath::new([c0, c1]);
        assert_eq!(linear_index::<2>(&p), 0);
    }

    #[test]
    fn linear_index_last() {
        let c0 = Coord::new(11171).unwrap();
        let c1 = Coord::new(11171).unwrap();
        let p = CoordPath::new([c0, c1]);
        assert_eq!(linear_index::<2>(&p), 11171 * 11172 + 11171);
    }
}
