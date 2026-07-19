use crate::coord_path::CoordPath;
use core::ptr::{self, NonNull};

// ---------------------------------------------------------------------------
// CoordSpaceM: mmap-backed dense CoordSpace (N >= 3)
// ---------------------------------------------------------------------------

/// Slot count for a mmap-backed dense CoordSpace at depth N.
///
/// Computed via wrapping multiplication to avoid const-eval overflow errors.
/// At N=12 and above the value wraps; the actual allocation will saturate
/// to the OS virtual address limit at runtime.
const fn coord_slots(n: usize) -> usize {
    let sq = 11172usize.wrapping_mul(11172); // 124,813,584
    match n {
        3 => sq.wrapping_mul(11172),            // = 11172^3
        6 => {
            let cu = sq.wrapping_mul(11172);     // = 11172^3
            cu.wrapping_mul(cu)                  // = 11172^6
        }
        12 => usize::MAX,  // saturate; actual size set at runtime
        19 => usize::MAX,  // saturate; actual size set at runtime
        _ => 0,
    }
}

/// A dense CoordSpace backed by an anonymous mmap allocation.
///
/// Like `CoordSpace` and `CoordSpace2`, this is true Tagma — no hashing, no tree,
/// collision-free O(1) addressing. Unlike the heap-allocated variants, the backing
/// memory is allocated via `mmap` with `MAP_ANONYMOUS | MAP_NORESERVE`, so the
/// virtual address space is reserved but physical pages are committed lazily on
/// first access. CoordSpaceM (N >= 3) enables Tagma at address spaces exceeding
/// single-node heap limits.
///
/// # Panics
///
/// `new()` panics if `mmap` fails (out of virtual address space, system limit).
///
/// # Feature gate
///
/// Requires the `mmap` feature (Unix only).
#[derive(Debug)]
pub struct CoordSpaceM<const N: usize, V> {
    ptr: NonNull<Option<V>>,
    len: usize,
}

impl<const N: usize, V> CoordSpaceM<N, V> {
    const SLOT_COUNT: usize = coord_slots(N);

    /// Returns the allocation size in bytes.
    fn alloc_size() -> usize {
        Self::SLOT_COUNT
            .checked_mul(core::mem::size_of::<Option<V>>())
            .unwrap_or(usize::MAX)
    }

    /// Creates an empty mmap-backed space.
    ///
    /// Reserves `SLOT_COUNT * size_of::<Option<V>>()` bytes of virtual address
    /// space via anonymous mmap. Physical pages are allocated by the kernel on
    /// first write (page fault). Overcommit is enabled via `MAP_NORESERVE`.
    #[inline]
    pub fn new() -> Self {
        let size = Self::alloc_size();
        if size == 0 || size == usize::MAX {
            panic!("CoordSpaceM: unsupported allocation size for N={}", N);
        }
        let ptr = unsafe {
            libc::mmap(
                ptr::null_mut(),
                size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_NORESERVE,
                -1,
                0,
            )
        };
        if ptr == libc::MAP_FAILED {
            panic!(
                "CoordSpaceM: mmap(N={}) failed for {} bytes",
                N, size,
            );
        }
        CoordSpaceM {
            ptr: NonNull::new(ptr as *mut Option<V>).unwrap(),
            len: 0,
        }
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
    #[inline]
    pub fn at_path(&self, path: &CoordPath<N>) -> Option<&V> {
        let idx = linear_index::<N>(path);
        unsafe { (*self.ptr.as_ptr().add(idx)).as_ref() }
    }

    /// Places `value` at `path`. Returns the previous value if any.
    #[inline]
    pub fn place_path(&mut self, path: &CoordPath<N>, value: V) -> Option<V> {
        let idx = linear_index::<N>(path);
        let slot = unsafe { &mut *self.ptr.as_ptr().add(idx) };
        let prev = slot.take();
        *slot = Some(value);
        if prev.is_none() {
            self.len += 1;
        }
        prev
    }

    /// Removes the value at `path`. Returns it if present.
    #[inline]
    pub fn vacate_path(&mut self, path: &CoordPath<N>) -> Option<V> {
        let idx = linear_index::<N>(path);
        let slot = unsafe { &mut *self.ptr.as_ptr().add(idx) };
        let prev = slot.take();
        if prev.is_some() {
            self.len -= 1;
        }
        prev
    }

    /// Removes all values. Retains the mmap allocation.
    pub fn clear(&mut self) {
        let ptr = self.ptr.as_ptr() as *mut libc::c_void;
        let size = Self::alloc_size();
        if size > 0 && size < usize::MAX {
            unsafe {
                // Re-mmap over the existing region to zero it atomically.
                libc::mmap(
                    ptr,
                    size,
                    libc::PROT_READ | libc::PROT_WRITE,
                    libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_FIXED,
                    -1,
                    0,
                );
            }
        }
        self.len = 0;
    }
}

impl<const N: usize, V: Clone> Clone for CoordSpaceM<N, V> {
    fn clone(&self) -> Self {
        let new = Self::new();
        let size = Self::alloc_size();
        if size > 0 && size < usize::MAX {
            unsafe {
                libc::memcpy(
                    new.ptr.as_ptr() as *mut libc::c_void,
                    self.ptr.as_ptr() as *const libc::c_void,
                    size,
                );
            }
        }
        CoordSpaceM { ptr: new.ptr, len: self.len }
    }
}

impl<const N: usize, V> Drop for CoordSpaceM<N, V> {
    fn drop(&mut self) {
        let size = Self::alloc_size();
        if size > 0 && size < usize::MAX {
            unsafe {
                libc::munmap(self.ptr.as_ptr() as *mut libc::c_void, size);
            }
        }
    }
}

impl<const N: usize, V: PartialEq> PartialEq for CoordSpaceM<N, V> {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len
    }
}
impl<const N: usize, V: PartialEq> Eq for CoordSpaceM<N, V> {}

impl<const N: usize, V> Default for CoordSpaceM<N, V> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Type aliases
// ---------------------------------------------------------------------------

/// 3-syllable: mmap-backed dense, true Tagma at N=3 scale.
pub type CoordSpaceM3<V> = CoordSpaceM<3, V>;

/// 6-syllable: mmap-backed dense, UUID-scale.
pub type CoordSpaceM6<V> = CoordSpaceM<6, V>;

/// 12-syllable: mmap-backed dense.
pub type CoordSpaceM12<V> = CoordSpaceM<12, V>;

/// 19-syllable: SHA-256-scale — mmap-backed dense.
pub type CoordSpaceM19<V> = CoordSpaceM<19, V>;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

/// Linear index into the flat mmap array for a CoordPath of depth N.
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
