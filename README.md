# synTagma

<<<<<<< Updated upstream
**Content-addressable structural primitive defined by the Unicode Hangul syllable block.**

Tagma replaces hash-based identity generation with direct structural addressing over a fixed 16-bit coordinate space. Every valid 16-bit value in the Hangul syllable block (U+AC00--U+D7AF) decomposes into three independent axes (initial, medial, final), serving simultaneously as a 1-D address and a 3-D coordinate. The reference implementation is a `#![no_std]` Rust library with optional `alloc` support.

## Core types
=======
**Spatial coordinate space computing system based on Tagma**

synTagma is a spatial coordinate space computing system built on Tagma, a 16-bit coordinate primitive embedded in the Unicode Hangul syllable block (U+AC00--U+D7AF). Every valid 16-bit value decomposes into three independent axes (initial, medial, final), serving simultaneously as a 1-D address and a 3-D coordinate. The reference implementation is a `#![no_std]` Rust library.

## Layers

```
synTagma (system)
  └─ Coordination layer (protocol, topology, distributed resolver)
  └─ Tagma core primitive (Coord, CoordPath, CoordSet, CoordSpace)
```

- **Tagma** — the core primitive: a 16-bit structural coordinate with closed-form composition, zero collisions, and single-cycle combinational decoding. The atomic identity primitive.
- **synTagma coordination layer** — recursive coordinate space expansion, physical topology mapping, distributed routing, and consistency protocol. Defined in the [synTagma](https://docs.ssccs.org/projects/syntagma/tagma/syn.html).

## Tagma primitive: Feature levels

Tagma provides a single feature gate: `alloc` (default: on). Without it (`--no-default-features`), all Tagma types are `no_std` + `no_alloc`.

| Level | Feature flags | Heap | Available types |
|-------|---------------|------|-----------------|
| **no_alloc** | (none) | Never | Coord, CoordPath, CoordSet, CoordSpace |
| **alloc** | `alloc` (default) | Optional | + CoordSpaceN\<N\>, DynCoordSpace |

## Tagma type reference

### Always available (no_std, no allocator)
>>>>>>> Stashed changes

| Type | Description | File |
|------|-------------|------|
| **Coord** | 16-bit atomic coordinate (0..11172), 3-axis composition/decomposition, Hamming distance, Hangul display | `core/src/coord.rs` |
<<<<<<< Updated upstream
| **CoordMap\<V\>** | Single-syllable direct-address table, inline `[Option<V>; 11172]`, no allocator. O(1), no hashing | `core/src/flat.rs` |
| **CoordMap6\<V\>** | 6-syllable (UUID-scale, $1.94 \times 10^{24}$ space), heap-backed lazy tree | `core/src/map.rs` |
| **CoordMap12\<V\>** | 12-syllable ($2.41 \times 10^{67}$ space) | `core/src/map.rs` |
| **CoordMap19\<V\>** | 19-syllable ($\approx 2^{256}$ space, SHA-256-scale) | `core/src/map.rs` |
| **DynCoordMap\<V\>** | Variable-depth trie, `&[Coord]` path, runtime depth | `core/src/dyn_coord.rs` |
| **CoordPath\<N\>** | Index path (not a hash key), compile-time N-element Coord array | `core/src/path.rs` |
| **CoordSet** | Bit array over 11,172 slots (1.4 KB, no allocator). Union, intersection, difference, subset tests | `core/src/set.rs` |
| **base11172** | Self-validating serialization: arbitrary bytes to Hangul syllable strings | `base11172/src/lib.rs` |

Test coverage: 163 unit/integration tests + 15 doc-tests, all passing. Zero clippy warnings. CI runs `cargo fmt --check`, `cargo clippy`, `cargo build --release`, `cargo test --release`, `cargo build --no-default-features` (no_alloc verification).
=======
| **CoordPath\<N\>** | Index path (not a hash key), compile-time N-element Coord array | `core/src/coord_path.rs` |
| **CoordSet** | Bit array over 11,172 slots (1.4 KB). Union, intersection, difference, subset tests, `Copy` | `core/src/coord_set.rs` |
| **CoordSpace\<V\>** | Single-syllable direct-address table. Inline `[Option<V>; 11172]` — zero heap, no hashing, no collisions | `core/src/coord_space.rs` |

### Requires alloc (default feature)

| Type | Description | File |
|------|-------------|------|
| **CoordSpaceN\<N, V\>** | N-level direct-address tree. Lazy heap allocation. `N` dereferences per lookup | `core/src/coord_space_n.rs` |
| **CoordSpace2\<V\>** | 2-syllable ($1.25 \times 10^8$). Alias for `CoordSpaceN<2, V>` | `core/src/coord_space_n.rs` |
| **CoordSpace6\<V\>** | 6-syllable UUID-scale ($1.94 \times 10^{24}$). Alias for `CoordSpaceN<6, V>` | `core/src/coord_space_n.rs` |
| **CoordSpace12\<V\>** | 12-syllable ($2.41 \times 10^{67}$) | `core/src/coord_space_n.rs` |
| **CoordSpace19\<V\>** | 19-syllable ($\approx 2^{256}$, SHA-256-scale) | `core/src/coord_space_n.rs` |
| **DynCoordSpace\<V\>** | Variable-depth trie, `&[Coord]` runtime path | `core/src/dyn_coord_space.rs` |
>>>>>>> Stashed changes

## Quick start

```sh
git clone https://github.com/ssccsorg/syntagma
cd syntagma
./run.sh                # fmt → clippy → build → test → no_alloc check
```

Or directly:

```sh
cd sw/rust
cargo test --release    # Run all 163 tests
cargo bench -p tagma-benchmarks -- stress  # 500k mixed-operation stress benchmark
```

## Usage

```rust
use tagma_core::{Coord, CoordMap, CoordMap6, CoordPath, CoordSet};

// Compose a coordinate from three axes
let c = Coord::from_axes(5, 10, 15).unwrap();
assert_eq!(c.to_axes(), (5, 10, 15));
assert_eq!(c.to_char(), '걐');  // Hangul syllable display
assert_eq!(c.hamming_distance(c), (0, 0, 0));

// Single-syllable map (no allocator required)
let mut map = CoordMap::new();
map.insert(c, "tagma");
assert_eq!(map.get(&c), Some(&"tagma"));
*map.entry(c).or_insert("default") = "updated";

// Multi-syllable map (UUID-scale, heap allocated)
let mut map6 = CoordMap6::<u32>::new();
let path = CoordPath::new([
    Coord::new(1).unwrap(),
    Coord::new(2).unwrap(),
    Coord::new(3).unwrap(),
    Coord::new(4).unwrap(),
    Coord::new(5).unwrap(),
    Coord::new(6).unwrap(),
]);
map6.insert_path(&path, 42);
assert_eq!(map6.get_path(&path), Some(&42));

// Bit-array set
let mut set = CoordSet::new();
set.insert(c);
assert!(set.contains(c));
```

<<<<<<< Updated upstream
## How it works
=======
## Feature matrix

| Feature | `no_alloc` | `alloc` (default) |
|---------|-----------|-------------------|
| Coord | ✅ | ✅ |
| CoordPath\<N\> | ✅ | ✅ |
| CoordSet | ✅ | ✅ |
| CoordSpace (inline array) | ✅ | ✅ |
| CoordSpaceN (heap tree, N>1) | ❌ | ✅ |
| DynCoordSpace (runtime trie) | ❌ | ✅ |

## How Tagma works
>>>>>>> Stashed changes

A Tagma coordinate is computed from three structural axes via the Hangul composition formula defined in ISO/IEC 10646:

$$C(i,m,f) = \text{U+AC00} + 588i + 28m + f, \quad 0 \leq i < 19,\; 0 \leq m < 21,\; 0 \leq f < 28$$

Of 65,536 representable 16-bit states, only 11,172 satisfy this formula. The remaining 54,364 are structurally invalid and hardware-detectable. Each valid value is:

- A 1-D address (Unicode code point) for flat array indexing.
- A 3-D coordinate (initial, medial, final) for structural queries.
- A Hangul syllable for human-readable display.

N-syllable sequences (CoordPath) extend the address space to $11172^N$ identifiers via direct-index tree traversal. A 6-syllable identifier covers UUID-scale space; 19 syllables match SHA-256's $2^{256}$ identifier space.

<<<<<<< Updated upstream
The three-axis composition formula admits unbounded recursive embedding: each axis
of a SynTagma can itself be a full CoordPath, producing $(11,172^{19})^3
\approx 7.30 \times 10^{231}$ addresses at the first recursion level. The SynTagma
specification defines how this recursive structure is mapped onto physical topologies.

## Benchmark

On Apple M1 (development platform):
=======
The three-axis composition formula admits unbounded recursive embedding: each axis of a synTagma coordinate can itself be a full CoordPath, enabling physical topology mapping across distributed nodes without modifying the core arithmetic.

## Benchmark: Tagma identity generation (Apple M1)
>>>>>>> Stashed changes

```
Tagma (1-syll):         2 ns/op    (space: 1.1e4)
Tagma (6-syll):        11 ns/op    (space: 1.9e24, UUID-scale)
Tagma (19-syll):       35 ns/op    (space: 2^256)
SHA-256:              227 ns/op

<<<<<<< Updated upstream
Speedup vs SHA-256:
  1-syll:   115x
  6-syll:    21x
  19-syll:    6.5x
```

On GitHub CI (x86_64, ubuntu-latest) for reproducibility:

```
Tagma (1-syll):        20 ns/op
Tagma (6-syll):       153 ns/op
Tagma (19-syll):      456 ns/op
SHA-256:             4437 ns/op
```
=======
## Benchmark: Spatial query vs HashMap (Apple M1)

Same algorithm (iterate + decompose + filter on axis), different memory layout. CoordSpace stores values in contiguous `[Option<V>; 11172]` — no hash, no collision, no fragmentation. HashMap scatters across buckets.

| Operation | CoordSpace | HashMap | Ratio |
|-----------|-----------|---------|-------|
| **Insert** 11,172 | 26.5 µs | 377 µs | **14x** |
| **Get** 11,172 | 6.49 µs | 101 µs | **16x** |
| **Remove** 11,172 | 15.0 µs | 268 µs | **18x** |
| **Axis filter** (medial=10) | 58.2 Melem/s | 24.2 Melem/s | **2.4x** |
| **Range filter** (initial 3--7) | 312 Melem/s | 139 Melem/s | **2.3x** |
| **CoordSet compound** (initial=3 AND medial=5) | 94.4 ns | 13.0 µs | **138x** |
| **Get single** (random coord) | 0.81 ns | 8.9 ns | **11x** |
>>>>>>> Stashed changes

## Documentation

- **[synTagma project page](https://docs.ssccs.org/projects/syntagma/)** — Project overview, paradigm shift, papers
- **[White Paper](https://docs.ssccs.org/projects/syntagma/tagma/wp.html)** — Tagma coordinate space, decoder, hardware implementation, benchmarks
- **[synTagma coordination layer](https://docs.ssccs.org/projects/syntagma/tagma/syn.html)** — Recursive topology mapping, transport, distributed resolver, consistency
- **[Tagma-ID](https://docs.ssccs.org/projects/syntagma/tagma/id.html)** — Content-addressable identity without hash functions
- **[Specification](spec/coord-space.md)** — Language-independent Tagma coordinate space definition
- **Rustdoc** — `cargo doc --no-deps -p tagma-core` for API reference

## License

Apache 2.0 — see [LICENSE](LICENSE).
