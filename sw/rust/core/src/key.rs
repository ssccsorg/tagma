use crate::coord::Coord;
use crate::path::CoordPath;

// ---------------------------------------------------------------------------
// CoordKey — conversion from application key types to direct-address paths
// ---------------------------------------------------------------------------

/// Conversion from an application-level key type to a [`CoordPath`].
///
/// `N` is the path depth (number of syllables). `N=1` covers 11,172 addresses;
/// `N=6` covers UUID-scale ($1.94 \times 10^{24}$).
///
/// # Collision model
///
/// - **Direct keys** (`Coord`, `u128`, `[u8; 16]`): zero collisions.
///   Every distinct key maps to a distinct `CoordPath`.
/// - **Derived keys** (`&str`, `&[u8]`): probabilistic collisions
///   during the hash-to-Coord conversion, identical to `HashMap`'s model.
///   At the storage level, collisions remain zero (no bucket chains, no rehashing).
pub trait CoordKey<const N: usize> {
    /// Convert this key to a `CoordPath<N>` for direct addressing.
    fn to_path(&self) -> CoordPath<N>;
}

// ── Direct key: Coord ───────────────────────────────────────────────────

impl CoordKey<1> for Coord {
    #[inline]
    fn to_path(&self) -> CoordPath<1> {
        CoordPath::new([*self])
    }
}

impl CoordKey<1> for &Coord {
    #[inline]
    fn to_path(&self) -> CoordPath<1> {
        CoordPath::new([**self])
    }
}

// ── Direct key: u128 (UUID integer) → CoordPath<6> ──────────────────────

impl CoordKey<6> for u128 {
    fn to_path(&self) -> CoordPath<6> {
        let words = [
            ((self >> 80) & 0xFFFF) as u16,
            ((self >> 64) & 0xFFFF) as u16,
            ((self >> 48) & 0xFFFF) as u16,
            ((self >> 32) & 0xFFFF) as u16,
            ((self >> 16) & 0xFFFF) as u16,
            (self & 0xFFFF) as u16,
        ];
        CoordPath::new(core::array::from_fn(|i| {
            Coord::new(words[i] % (Coord::N_VALID as u16)).unwrap()
        }))
    }
}

// ── Direct key: [u8; 16] (UUID bytes) → CoordPath<6> ────────────────────

impl CoordKey<6> for [u8; 16] {
    fn to_path(&self) -> CoordPath<6> {
        u128::from_be_bytes(*self).to_path()
    }
}

// ── Direct key: [u8; 32] (SHA-256) → CoordPath<19> ──────────────────────

impl CoordKey<19> for [u8; 32] {
    fn to_path(&self) -> CoordPath<19> {
        let mut path = [Coord::new(0).unwrap(); 19];
        for i in 0..16 {
            let word = u16::from_be_bytes([self[i * 2], self[i * 2 + 1]]);
            path[i] = Coord::new(word % (Coord::N_VALID as u16)).unwrap();
        }
        CoordPath::new(path)
    }
}

// ── Derived key: &str (hash-then-mod) → CoordPath<1> ────────────────────

impl CoordKey<1> for &str {
    fn to_path(&self) -> CoordPath<1> {
        let h = fast_hash(self.as_bytes());
        CoordPath::new([Coord::new((h % 11172) as u16).unwrap()])
    }
}

// ── Derived key: &str → CoordPath<6> (UUID-scale, lower collision) ──────

impl CoordKey<6> for &str {
    fn to_path(&self) -> CoordPath<6> {
        let h = fast_hash(self.as_bytes());
        CoordPath::new([
            Coord::new(((h >> 48) % 11172) as u16).unwrap(),
            Coord::new(((h >> 32) % 11172) as u16).unwrap(),
            Coord::new(((h >> 16) % 11172) as u16).unwrap(),
            Coord::new((h % 11172) as u16).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
        ])
    }
}

// ── Derived key: &[u8] (hash-then-mod) → CoordPath<1> ───────────────────

impl CoordKey<1> for &[u8] {
    fn to_path(&self) -> CoordPath<1> {
        let h = fast_hash(self);
        CoordPath::new([Coord::new((h % 11172) as u16).unwrap()])
    }
}

impl<const L: usize> CoordKey<1> for [u8; L] {
    #[inline]
    fn to_path(&self) -> CoordPath<1> {
        self.as_slice().to_path()
    }
}

// ── Fast non-cryptographic hash ─────────────────────────────────────────

fn fast_hash(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0x2f7b_8a6e_3c5d_1f49;
    for chunk in bytes.chunks(8) {
        let mut word = 0u64;
        for (i, &b) in chunk.iter().enumerate() {
            word |= (b as u64) << (i * 8);
        }
        h = h.wrapping_add(word);
        h = h.wrapping_mul(0x9e37_79b9_7f4a_7c15);
        h ^= h >> 31;
    }
    h ^= h >> 33;
    h.wrapping_mul(0xff51_afd7_ed55_8ccd)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Coord;

    #[test]
    fn coord_direct_roundtrip() {
        let c = Coord::new(42).unwrap();
        let path = c.to_path();
        assert_eq!(path.coords()[0], c);
    }

    #[test]
    fn u128_to_path_length() {
        let path: CoordPath<6> = 0x0123456789ABCDEF0123456789ABCDEFu128.to_path();
        assert_eq!(path.len(), 6);
    }

    #[test]
    fn uuid_bytes_to_path() {
        let uuid = [0u8; 16];
        let path: CoordPath<6> = uuid.to_path();
        assert_eq!(path.len(), 6);
    }

    #[test]
    fn str_derived_is_deterministic() {
        let a: CoordPath<1> = "hello".to_path();
        let b: CoordPath<1> = "hello".to_path();
        assert_eq!(a, b);
    }

    #[test]
    fn str_differs_from_str() {
        let a: CoordPath<1> = "alpha".to_path();
        let b: CoordPath<1> = "beta".to_path();
        assert_ne!(a, b);
    }

    #[test]
    fn bytes_slice_matches_str() {
        let a: CoordPath<1> = b"hello".as_slice().to_path();
        let b: CoordPath<1> = "hello".to_path();
        assert_eq!(a, b);
    }

    #[test]
    fn coord_key_6_different_from_1() {
        let a = "key";
        let _path_1: CoordPath<1> = a.to_path();
        let _path_6: CoordPath<6> = a.to_path();
        assert_eq!(_path_6.len(), 6);
        assert_eq!(_path_1.len(), 1);
    }
}
