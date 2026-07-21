use tagma_core::DynCoordSpace;

use crate::string_to_coord_path;
use crate::CoordKV;

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

// ── DynCoordKV ────────────────────────────────────────────────────────────

/// A hash-free, collision-free string KV store backed by [`DynCoordSpace`]
/// with [`ByteWise`](crate::ByteWise) coordinate generation.
///
/// Supports any non-empty string key.  Lookup cost is O(len(key)).
///
/// # Trait implementations
///
/// | Trait | Methods |
/// |-------|---------|
/// | [`CoordKV`] | `insert` / `get` / `remove` via `&str` |
///
/// # Example
///
/// ```
/// use tagma_kv::CoordKV;
/// use tagma_kv::dyn_coord_kv::DynCoordKV;
///
/// let mut kv = DynCoordKV::new();
/// kv.insert("hello", b"world".to_vec());
/// assert_eq!(kv.get("hello"), Some(b"world".to_vec()));
/// ```
pub struct DynCoordKV {
    space: DynCoordSpace<Box<[u8]>>,
    len: usize,
}

impl DynCoordKV {
    /// Creates an empty dynamic-mode store.
    pub fn new() -> Self {
        DynCoordKV {
            space: DynCoordSpace::new(),
            len: 0,
        }
    }
}

impl CoordKV for DynCoordKV {
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

    fn insert(&mut self, key: &str, value: Vec<u8>) -> Option<Vec<u8>> {
        if key.is_empty() {
            return None;
        }
        let path = string_to_coord_path(key).unwrap();
        let prev = self.space.place(&path, vec_to_box(value));
        if prev.is_none() {
            self.len += 1;
        }
        prev.map(box_to_vec_owned)
    }

    fn get(&self, key: &str) -> Option<Vec<u8>> {
        if key.is_empty() {
            return None;
        }
        string_to_coord_path(key)
            .and_then(|path| self.space.at(&path))
            .map(|v| box_to_vec(v.as_ref()))
    }

    fn remove(&mut self, key: &str) -> Option<Vec<u8>> {
        if key.is_empty() {
            return None;
        }
        let val = string_to_coord_path(key)
            .and_then(|path| self.space.vacate(&path))
            .map(box_to_vec_owned);
        if val.is_some() {
            self.len = self.len.saturating_sub(1);
        }
        val
    }
}

impl Default for DynCoordKV {
    fn default() -> Self {
        Self::new()
    }
}
