#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod coord;
pub mod path;
pub mod set;

// FlatMap: no_alloc, single-syllable, inline array (22 KB).
// Always available — no heap allocator required.
pub mod flat;

// TreeMap and CoordMap<N>: multi-syllable, heap-backed tree.
// Requires alloc feature (default: on).
#[cfg(feature = "alloc")]
pub mod map;

pub use coord::Coord;
pub use path::CoordPath;
pub use set::CoordSet;

pub use flat::FlatDrain;
pub use flat::FlatEntry;
pub use flat::FlatIter;
pub use flat::FlatMap;
pub use flat::FlatOccupiedEntry;
pub use flat::FlatVacantEntry;

#[cfg(feature = "alloc")]
pub use map::TreeMap;
#[cfg(feature = "alloc")]
pub use map::TreeMap19;
#[cfg(feature = "alloc")]
pub use map::TreeMap6;
