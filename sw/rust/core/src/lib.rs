#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod coord;
pub mod path;
pub mod set;

// CoordMap1: no_alloc, single-syllable, inline array (22 KB).
// Always available — no heap allocator required.
pub mod flat;

// CoordMap6, CoordMap12, CoordMap19: multi-syllable, heap-backed tree.
// Requires alloc feature (default: on).
#[cfg(feature = "alloc")]
pub mod map;

pub use coord::Coord;
pub use path::CoordPath;
pub use set::CoordSet;

// ── CoordMap series — unified naming ──

/// 1-syllable: 11,172 identifiers. No allocator required.
pub use flat::CoordMap;
pub use flat::CoordMap1;

#[cfg(feature = "alloc")]
pub use map::CoordMap12;
#[cfg(feature = "alloc")]
pub use map::CoordMap19;
#[cfg(feature = "alloc")]
pub use map::CoordMap6;

// Internal types (used by CoordMap1):
pub use flat::FlatDrain;
pub use flat::FlatEntry;
pub use flat::FlatIter;
