use tagma_core::CoordSpaceN;

use crate::coord_gen::CoordKey;
use crate::{CoordKV, CoordKVKey};

// ── Internal helpers ──────────────────────────────────────────────────────

fn vec_to_box(v: Vec<u8>) -> Box<[u8]> {
    v.into_boxed_slice()
}

fn box_to_vec(v: &[u8]) -> Vec<u8> {
    v.to_vec()
}

fn box_to_vec_owned(v: Box<[u8]>) -> Vec<u8> {
    v.into_vec()
}

// ── CoordKVN ──────────────────────────────────────────────────────────────

/// A fixed-N-byte-key KV store backed by [`CoordSpaceN`] — the sparse
/// tree for any depth `N`.
///
/// Keys must be exactly `N` bytes.  Lookup cost is O(N) via tree traversal.
///
/// # Trait implementations
///
/// | Trait | Methods |
/// |-------|---------|
/// | [`CoordKV`] | `insert` / `get` / `remove` via `&str` |
/// | [`CoordKVKey<N>`] | `insert_by_coordkey` / `get_by_coordkey` / `remove_by_coordkey` via `CoordKey<N>` |
///
/// # Example
///
/// ```
/// use tagma_kv::{CoordKV, CoordKVKey};
/// use tagma_kv::coord_kv_n::CoordKVN;
///
/// let mut kv = CoordKVN::<3>::new();
/// kv.insert("foo", b"bar".to_vec());
/// assert_eq!(kv.get("foo"), Some(b"bar".to_vec()));
/// ```
pub struct CoordKVN<const N: usize> {
    space: CoordSpaceN<N, Box<[u8]>>,
    len: usize,
}

impl<const N: usize> CoordKVN<N> {
    /// Creates an empty N-byte-key store.
    pub fn new() -> Self {
        CoordKVN {
            space: CoordSpaceN::new(),
            len: 0,
        }
    }
}

impl<const N: usize> CoordKV for CoordKVN<N> {
    fn len(&self) -> usize {
        self.len
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn clear(&mut self) {
        self.space.clear();
        self.len = 0;
    }

    fn insert(&mut self, key: &str, value: Vec<u8>) {
        // Panics if key.len() != N (via CoordKey::from)
        let ck: CoordKey<N> = key.into();
        self.insert_by_coordkey(&ck, value);
    }

    fn get(&self, key: &str) -> Option<Vec<u8>> {
        if key.len() != N {
            return None;
        }
        let ck: CoordKey<N> = key.into();
        self.get_by_coordkey(&ck)
    }

    fn remove(&mut self, key: &str) -> Option<Vec<u8>> {
        if key.len() != N {
            return None;
        }
        let ck: CoordKey<N> = key.into();
        self.remove_by_coordkey(&ck)
    }
}

impl<const N: usize> CoordKVKey<N> for CoordKVN<N> {
    fn insert_by_coordkey(&mut self, key: &CoordKey<N>, value: Vec<u8>) {
        let path = key.to_coord_path();
        if self.space.place_path(&path, vec_to_box(value)).is_none() {
            self.len += 1;
        }
    }

    fn get_by_coordkey(&self, key: &CoordKey<N>) -> Option<Vec<u8>> {
        let path = key.to_coord_path();
        self.space.at_path(&path).map(|v| box_to_vec(v.as_ref()))
    }

    fn remove_by_coordkey(&mut self, key: &CoordKey<N>) -> Option<Vec<u8>> {
        let path = key.to_coord_path();
        let val = self.space.vacate_path(&path).map(box_to_vec_owned);
        if val.is_some() {
            self.len = self.len.saturating_sub(1);
        }
        val
    }
}

impl<const N: usize> Default for CoordKVN<N> {
    fn default() -> Self {
        Self::new()
    }
}
