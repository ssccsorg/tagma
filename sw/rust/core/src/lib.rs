#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod coord;
pub mod path;
pub mod set;

#[cfg(feature = "alloc")]
pub mod map;

pub use coord::Coord;
pub use path::CoordPath;
pub use set::CoordSet;

#[cfg(feature = "alloc")]
pub use map::CoordMap;
#[cfg(feature = "alloc")]
pub use map::CoordMap1;
#[cfg(feature = "alloc")]
pub use map::CoordMap2;
#[cfg(feature = "alloc")]
pub use map::CoordMap6;
#[cfg(feature = "alloc")]
pub use map::CoordMap12;
#[cfg(feature = "alloc")]
pub use map::CoordMap19;
#[cfg(feature = "alloc")]
pub use map::Entry;
#[cfg(feature = "alloc")]
pub use map::OccupiedEntry;
#[cfg(feature = "alloc")]
pub use map::VacantEntry;
