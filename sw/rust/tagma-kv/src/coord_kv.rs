use crate::coord_gen::CoordKey;

/// Core operations for any [`CoordKV`] implementation.
///
/// The primary API uses `&str` keys.  Fixed-key variants additionally
/// implement [`CoordKVKey`] for explicit [`CoordKey`]-based access.
///
/// # Trait lego
///
/// | Trait | Provided methods | Scope |
/// |-------|------------------|-------|
/// | [`CoordKV`] | `insert`, `get`, `remove` via `&str` | all KV types |
/// | [`CoordKVKey<N>`] | `_by_coordkey` suffixed methods | fixed-key types only |
pub trait CoordKV {
    /// Returns the number of stored entries.
    fn len(&self) -> usize;

    /// Returns `true` if the store is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Removes all entries.
    fn clear(&mut self);

    /// Inserts a key-value pair.
    fn insert(&mut self, key: &str, value: Vec<u8>);

    /// Retrieves a value by key.
    fn get(&self, key: &str) -> Option<Vec<u8>>;

    /// Removes a key-value pair.  Returns the value if present.
    fn remove(&mut self, key: &str) -> Option<Vec<u8>>;
}

/// [`CoordKey`]-based access for fixed-size-key KV stores.
///
/// Requires [`CoordKV`] and adds `_by_coordkey` methods.
pub trait CoordKVKey<const N: usize>: CoordKV {
    /// Inserts a key-value pair using a [`CoordKey<N>`].
    fn insert_by_coordkey(&mut self, key: &CoordKey<N>, value: Vec<u8>);

    /// Retrieves a value by [`CoordKey<N>`].
    fn get_by_coordkey(&self, key: &CoordKey<N>) -> Option<Vec<u8>>;

    /// Removes a key-value pair by [`CoordKey<N>`].
    fn remove_by_coordkey(&mut self, key: &CoordKey<N>) -> Option<Vec<u8>>;
}
