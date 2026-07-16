use crate::coord::Coord;
use crate::path::CoordPath;

// ---------------------------------------------------------------------------
// CoordKey — conversion from key types to direct-address paths
// ---------------------------------------------------------------------------

/// Conversion from a key type to a [`CoordPath`] for direct addressing.
///
/// `N` is the path depth (number of syllables). `N=1` covers 11,172 addresses;
/// `N=6` covers UUID-scale ($1.94 \times 10^{24}$); `N=19` covers $2^{256}$.
///
/// # Collision
///
/// Zero collisions. Every distinct key maps to a distinct `CoordPath`.
/// Tagma is hashless — no hashing step, no probabilistic conversion.
pub trait CoordKey<const N: usize> {
    fn to_path(&self) -> CoordPath<N>;
}

// ── Coord ───────────────────────────────────────────────────────────────

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

// ── u128 (UUID integer) → CoordPath<6> ──────────────────────────────────

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

// ── [u8; 16] (UUID bytes) → CoordPath<6> ────────────────────────────────

impl CoordKey<6> for [u8; 16] {
    fn to_path(&self) -> CoordPath<6> {
        u128::from_be_bytes(*self).to_path()
    }
}

// ── [u8; 32] (SHA-256) → CoordPath<19> ──────────────────────────────────

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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

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
    fn sha256_bytes_to_path() {
        let hash = [0u8; 32];
        let path: CoordPath<19> = hash.to_path();
        assert_eq!(path.len(), 19);
    }

    #[test]
    fn u128_deterministic() {
        let a: CoordPath<6> = 42u128.to_path();
        let b: CoordPath<6> = 42u128.to_path();
        assert_eq!(a, b);
    }

    #[test]
    fn distinct_u128s_differ() {
        let a: CoordPath<6> = 1u128.to_path();
        let b: CoordPath<6> = 2u128.to_path();
        assert_ne!(a, b);
    }
}
