use tagma_core::{Coord, CoordCube, CoordPath};

// ---------------------------------------------------------------------------
// Bounding Box Iterator
// ---------------------------------------------------------------------------

/// An iterator that yields all `CoordPath<N>` within a multi-dimensional
/// bounding box, where each syllable position has a `(min, max)` range.
///
/// Uses a mixed-radix counter: O(N) per yield.
pub struct BoundingBoxIter<const N: usize> {
    ranges: [(u16, u16); N],
    current: [u16; N],
    finished: bool,
}

impl<const N: usize> BoundingBoxIter<N> {
    /// Creates a new bounding box iterator over the given per-syllable
    /// `(min, max)` ranges.
    ///
    /// # Panics
    ///
    /// Panics if any range is inverted (`min > max`) or out of bounds
    /// (`max >= 11172`).
    pub fn new(ranges: [(u16, u16); N]) -> Self {
        for (i, &(min, max)) in ranges.iter().enumerate() {
            assert!(
                min <= max,
                "BoundingBoxIter: range {} has min {} > max {}",
                i, min, max
            );
            assert!(
                max < 11172,
                "BoundingBoxIter: range {} has max {} >= 11172",
                i, max
            );
        }
        let mut current = [0u16; N];
        let mut i = 0;
        while i < N {
            current[i] = ranges[i].0;
            i += 1;
        }
        BoundingBoxIter {
            ranges,
            current,
            finished: N == 0,
        }
    }

    /// Returns `true` if the bounding box is empty.
    pub fn is_empty(&self) -> bool {
        self.finished
    }

    /// Returns the total count of paths (product of all range widths).
    pub fn count_paths(&self) -> usize {
        let mut total = 1usize;
        for &(min, max) in &self.ranges {
            let width = (max - min + 1) as usize;
            total = total.saturating_mul(width);
        }
        total
    }
}

impl<const N: usize> Iterator for BoundingBoxIter<N> {
    type Item = CoordPath<N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let mut coords = [Coord::new(0).unwrap(); N];
        for i in 0..N {
            // SAFETY: current[i] is always < N_VALID because ranges are
            // validated to have max < 11172.
            coords[i] = unsafe { Coord::new_unchecked(self.current[i]) };
        }
        let result = CoordPath::new(coords);

        // Increment the mixed-radix counter.
        let mut pos = N;
        while pos > 0 {
            pos -= 1;
            if self.current[pos] < self.ranges[pos].1 {
                self.current[pos] += 1;
                let mut reset = pos + 1;
                while reset < N {
                    self.current[reset] = self.ranges[reset].0;
                    reset += 1;
                }
                return Some(result);
            }
            self.current[pos] = self.ranges[pos].0;
        }
        self.finished = true;
        Some(result)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.finished {
            return (0, Some(0));
        }
        (0, Some(self.count_paths()))
    }
}

// ---------------------------------------------------------------------------
// Hamming-filtered iterator
// ---------------------------------------------------------------------------

/// An iterator that yields only paths within a Hamming radius of a center.
pub struct HammingFilter<const N: usize> {
    inner: BoundingBoxIter<N>,
    center: CoordPath<N>,
    max_distance: usize,
}

impl<const N: usize> Iterator for HammingFilter<N> {
    type Item = CoordPath<N>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(candidate) = self.inner.next() {
            let distance = candidate
                .coords()
                .iter()
                .zip(self.center.coords().iter())
                .filter(|(a, b)| a != b)
                .count();
            if distance <= self.max_distance {
                return Some(candidate);
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.inner.size_hint().1)
    }
}

// ---------------------------------------------------------------------------
// SpatialOps trait — spatial region generation
// ---------------------------------------------------------------------------

/// Extension trait providing spatial region generation on [`CoordCube`].
///
/// These methods generate `CoordPath` values within a spatial region;
/// they do **not** perform storage lookups.
pub trait SpatialOps<const N: usize> {
    /// Generates all `CoordPath<N>` within a bounding box defined by
    /// per-syllable `(min, max)` ranges.
    fn bounding_box(&self, ranges: &[(u16, u16); N]) -> BoundingBoxIter<N>;

    /// Generates all `CoordPath<N>` within an L∞ (Chebyshev) proximity
    /// radius of the cube's center.
    fn proximity(&self, radius: usize) -> BoundingBoxIter<N>;

    /// Generates all `CoordPath<N>` within a Hamming distance `radius`
    /// of the cube's center.
    fn proximity_hamming(&self, radius: usize) -> HammingFilter<N>;
}

impl<const N: usize, const D: usize, const R: usize> SpatialOps<N> for CoordCube<N, D, R> {
    fn bounding_box(&self, ranges: &[(u16, u16); N]) -> BoundingBoxIter<N> {
        BoundingBoxIter::new(*ranges)
    }

    fn proximity(&self, radius: usize) -> BoundingBoxIter<N> {
        let mut ranges = [(0u16, 0u16); N];
        for i in 0..N {
            let idx = self.coords()[i].index() as usize;
            let min = idx.saturating_sub(radius);
            let max = (idx + radius).min(11171);
            ranges[i] = (min as u16, max as u16);
        }
        BoundingBoxIter::new(ranges)
    }

    fn proximity_hamming(&self, radius: usize) -> HammingFilter<N> {
        let bb = self.proximity(radius.max(1));
        let center = *self.as_path();
        HammingFilter {
            inner: bb,
            center,
            max_distance: radius,
        }
    }
}

// ---------------------------------------------------------------------------
// DistanceMetrics trait — measurement between two CoordCubes
// ---------------------------------------------------------------------------

/// Extension trait providing distance metrics for [`CoordCube`].
///
/// All methods compare `self` to `other` and return a scalar distance.
///
/// # Example
///
/// ```rust
/// use tagma_core::{Coord, CoordPath, CoordCube};
/// use tagma_geo::spatial::DistanceMetrics;
///
/// let a = CoordCube::<2, 2, 1>::from_path(
///     CoordPath::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()])
/// );
/// let b = CoordCube::<2, 2, 1>::from_path(
///     CoordPath::new([Coord::new(0).unwrap(), Coord::new(5).unwrap()])
/// );
/// assert_eq!(a.hamming_distance(&b), 1);
/// ```
pub trait DistanceMetrics<const N: usize, const D: usize, const R: usize> {
    /// Hamming distance: count of syllable positions that differ.
    fn hamming_distance(&self, other: &CoordCube<N, D, R>) -> usize;

    /// Axis-wise Hamming distance: writes per-dimension differences into `out`.
    /// `out` must have length at least `D`.
    fn hamming_distance_axes(
        &self,
        other: &CoordCube<N, D, R>,
        out: &mut [usize],
    );

    /// Normalised Euclidean distance approximation.
    ///
    /// Each dimension's R-syllable value is normalised to `[0, 1]`, then
    /// Euclidean distance is computed in D-dimensional normalised space.
    /// Result is in `[0, sqrt(D)]`.
    fn euclidean_distance_approx(&self, other: &CoordCube<N, D, R>) -> f64;

    /// Manhattan (L1) distance: sum of absolute differences across all
    /// syllable positions.
    fn manhattan_distance(&self, other: &CoordCube<N, D, R>) -> u64;
}

impl<const N: usize, const D: usize, const R: usize> DistanceMetrics<N, D, R>
    for CoordCube<N, D, R>
{
    fn hamming_distance(&self, other: &CoordCube<N, D, R>) -> usize {
        self.coords()
            .iter()
            .zip(other.coords().iter())
            .filter(|(a, b)| a != b)
            .count()
    }

    fn hamming_distance_axes(
        &self,
        other: &CoordCube<N, D, R>,
        out: &mut [usize],
    ) {
        for dim in 0..D {
            let start = dim * R;
            let mut syllable_diff = 0;
            for i in 0..R {
                if self.coords()[start + i] != other.coords()[start + i] {
                    syllable_diff += 1;
                }
            }
            out[dim] = syllable_diff;
        }
    }

    fn euclidean_distance_approx(&self, other: &CoordCube<N, D, R>) -> f64 {
        let mut sum_sq = 0.0f64;
        let max_val = dimension_max_value::<R>() as f64;
        for dim in 0..D {
            let v1 = dimension_value::<N, D, R>(self, dim);
            let v2 = dimension_value::<N, D, R>(other, dim);
            let diff = (v1 as f64 - v2 as f64) / max_val;
            sum_sq += diff * diff;
        }
        sqrt_approx(sum_sq)
    }

    fn manhattan_distance(&self, other: &CoordCube<N, D, R>) -> u64 {
        let mut sum = 0u64;
        for dim in 0..D {
            let v1 = dimension_value::<N, D, R>(self, dim);
            let v2 = dimension_value::<N, D, R>(other, dim);
            sum += v1.abs_diff(v2);
        }
        sum
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Interprets the R syllables of dimension `dim` of `cube` as a little-endian
/// base-11172 integer in `[0, 11172^R)`.
fn dimension_value<const N: usize, const D: usize, const R: usize>(
    cube: &CoordCube<N, D, R>,
    dim: usize,
) -> u64 {
    let start = dim * R;
    let mut val = 0u64;
    let mut mul = 1u64;
    for i in 0..R {
        let idx = cube.coords()[start + i].index() as u64;
        val = val.wrapping_add(idx.wrapping_mul(mul));
        mul = mul.wrapping_mul(11172);
    }
    val
}

/// Maximum possible value for a single dimension (`11172^R - 1`).
fn dimension_max_value<const R: usize>() -> u64 {
    let mut max = 0u64;
    let mut mul = 1u64;
    for _ in 0..R {
        max = max.wrapping_add(11171u64.wrapping_mul(mul));
        mul = mul.wrapping_mul(11172);
    }
    max
}

/// Newton-Raphson square root approximation (no_std compatible).
fn sqrt_approx(x: f64) -> f64 {
    if x <= 0.0f64 {
        return 0.0f64;
    }
    let mut guess = x;
    for _ in 0..12 {
        guess = (guess + x / guess) * 0.5;
    }
    guess
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tagma_core::{Coord, CoordCube, CoordPath};

    // ── BoundingBoxIter ────────────────────────────────────────────

    #[test]
    fn bb_iter_single_syllable() {
        let iter = BoundingBoxIter::<1>::new([(5, 7)]);
        let paths: Vec<_> = iter.collect();
        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0].coords()[0].index(), 5);
        assert_eq!(paths[2].coords()[0].index(), 7);
    }

    #[test]
    fn bb_iter_two_syllables() {
        let iter = BoundingBoxIter::<2>::new([(1, 2), (3, 4)]);
        let paths: Vec<_> = iter.collect();
        assert_eq!(paths.len(), 4);
        assert_eq!(paths[0].coords()[0].index(), 1);
        assert_eq!(paths[0].coords()[1].index(), 3);
        assert_eq!(paths[3].coords()[0].index(), 2);
        assert_eq!(paths[3].coords()[1].index(), 4);
    }

    #[test]
    fn bb_iter_empty_n0() {
        let iter = BoundingBoxIter::<0>::new([]);
        assert!(iter.collect::<Vec<_>>().is_empty());
    }

    #[test]
    #[should_panic(expected = "min 5 > max 3")]
    fn bb_iter_inverted_range_panics() {
        let _ = BoundingBoxIter::<1>::new([(5, 3)]);
    }

    // ── SpatialOps ─────────────────────────────────────────────────

    #[test]
    fn cube_bounding_box_basic() {
        let path =
            CoordPath::<2>::new([Coord::new(5).unwrap(), Coord::new(5).unwrap()]);
        let cube = CoordCube::<2, 2, 1>::from_path(path);
        let ranges = [(3u16, 6u16), (4u16, 5u16)];
        let paths: Vec<_> = cube.bounding_box(&ranges).collect();
        assert_eq!(paths.len(), 8);
    }

    #[test]
    fn cube_proximity_radius_zero() {
        let path =
            CoordPath::<2>::new([Coord::new(5).unwrap(), Coord::new(5).unwrap()]);
        let cube = CoordCube::<2, 2, 1>::from_path(path);
        let paths: Vec<_> = cube.proximity(0).collect();
        assert_eq!(paths.len(), 1);
    }

    #[test]
    fn cube_proximity_radius_one() {
        let path =
            CoordPath::<2>::new([Coord::new(5).unwrap(), Coord::new(5).unwrap()]);
        let cube = CoordCube::<2, 2, 1>::from_path(path);
        let paths: Vec<_> = cube.proximity(1).collect();
        assert_eq!(paths.len(), 9);
    }

    #[test]
    fn cube_proximity_clamp_to_bounds() {
        let path = CoordPath::<1>::new([Coord::new(1).unwrap()]);
        let cube = CoordCube::<1, 1, 1>::from_path(path);
        let paths: Vec<_> = cube.proximity(3).collect();
        assert_eq!(paths.len(), 5);
        assert_eq!(paths[0].coords()[0].index(), 0);
        assert_eq!(paths[4].coords()[0].index(), 4);
    }

    // ── DistanceMetrics ────────────────────────────────────────────

    #[test]
    fn hamming_identical() {
        let path = CoordPath::<3>::new([
            Coord::new(100).unwrap(),
            Coord::new(200).unwrap(),
            Coord::new(300).unwrap(),
        ]);
        let a = CoordCube::<3, 3, 1>::from_path(path);
        let b = CoordCube::<3, 3, 1>::from_path(path);
        assert_eq!(a.hamming_distance(&b), 0);
    }

    #[test]
    fn hamming_all_differ() {
        let a = CoordCube::<3, 3, 1>::from_path(CoordPath::new([
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
        ]));
        let b = CoordCube::<3, 3, 1>::from_path(CoordPath::new([
            Coord::new(1).unwrap(),
            Coord::new(2).unwrap(),
            Coord::new(3).unwrap(),
        ]));
        assert_eq!(a.hamming_distance(&b), 3);
    }

    #[test]
    fn hamming_axes_works() {
        let a = CoordCube::<4, 2, 2>::from_path(CoordPath::new([
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
        ]));
        let b = CoordCube::<4, 2, 2>::from_path(CoordPath::new([
            Coord::new(0).unwrap(),
            Coord::new(1).unwrap(),
            Coord::new(2).unwrap(),
            Coord::new(0).unwrap(),
        ]));
        let mut out = [0usize; 2];
        a.hamming_distance_axes(&b, &mut out);
        assert_eq!(out, [1, 1]);
    }

    #[test]
    fn euclidean_identical() {
        let path =
            CoordPath::<2>::new([Coord::new(5000).unwrap(), Coord::new(5000).unwrap()]);
        let a = CoordCube::<2, 2, 1>::from_path(path);
        let b = CoordCube::<2, 2, 1>::from_path(path);
        assert!((a.euclidean_distance_approx(&b)).abs() < 1e-10);
    }

    #[test]
    fn euclidean_max_in_one_dim() {
        let a = CoordCube::<2, 2, 1>::from_path(CoordPath::new([
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
        ]));
        let b = CoordCube::<2, 2, 1>::from_path(CoordPath::new([
            Coord::new(11171).unwrap(),
            Coord::new(0).unwrap(),
        ]));
        let d = a.euclidean_distance_approx(&b);
        assert!((d - 1.0).abs() < 0.001, "got {}", d);
    }

    #[test]
    fn manhattan_identical() {
        let path =
            CoordPath::<2>::new([Coord::new(100).unwrap(), Coord::new(200).unwrap()]);
        let a = CoordCube::<2, 2, 1>::from_path(path);
        let b = CoordCube::<2, 2, 1>::from_path(path);
        assert_eq!(a.manhattan_distance(&b), 0);
    }

    #[test]
    fn manhattan_different() {
        let a = CoordCube::<2, 2, 1>::from_path(CoordPath::new([
            Coord::new(5).unwrap(),
            Coord::new(10).unwrap(),
        ]));
        let b = CoordCube::<2, 2, 1>::from_path(CoordPath::new([
            Coord::new(5).unwrap(),
            Coord::new(20).unwrap(),
        ]));
        assert_eq!(a.manhattan_distance(&b), 10);
    }

    // ── Hamming proximity ──────────────────────────────────────────

    #[test]
    fn proximity_hamming_radius_zero() {
        let path = CoordPath::<2>::new([Coord::new(5).unwrap(), Coord::new(5).unwrap()]);
        let cube = CoordCube::<2, 2, 1>::from_path(path);
        let paths: Vec<_> = cube.proximity_hamming(0).collect();
        assert_eq!(paths.len(), 1);
    }

    #[test]
    fn proximity_hamming_radius_one() {
        let path = CoordPath::<2>::new([Coord::new(5).unwrap(), Coord::new(5).unwrap()]);
        let cube = CoordCube::<2, 2, 1>::from_path(path);
        let paths: Vec<_> = cube.proximity_hamming(1).collect();
        assert_eq!(paths.len(), 5);
    }

    // ── count_paths ────────────────────────────────────────────────

    #[test]
    fn bb_count_paths() {
        let iter = BoundingBoxIter::<3>::new([(0, 1), (0, 2), (0, 3)]);
        assert_eq!(iter.count_paths(), 2 * 3 * 4);
    }

    #[test]
    fn bb_count_paths_large() {
        let iter = BoundingBoxIter::<2>::new([(0, 11171), (0, 11171)]);
        assert_eq!(iter.count_paths(), 124_813_584);
    }
}
