use tagma_core::{Coord, CoordPath, FlatMap, TreeMap, TreeMap19, TreeMap6};

// ── FlatMap — no_alloc single-syllable ──

#[test]
fn flat_map_insert_11172_values() {
    let mut map = FlatMap::new();
    for i in 0u16..11172 {
        assert_eq!(map.insert(Coord::new(i).unwrap(), i as u32), None);
    }
    assert_eq!(map.len(), 11172);
}

#[test]
fn flat_map_all_11172_accessible() {
    let mut map = FlatMap::new();
    for i in 0u16..11172 {
        map.insert(Coord::new(i).unwrap(), i);
    }
    for i in 0u16..11172 {
        assert_eq!(map.get(Coord::new(i).unwrap()), Some(&i));
    }
}

#[test]
fn flat_map_path_api() {
    let mut map = FlatMap::new();
    let c = Coord::new(5555).unwrap();
    map.insert(c, 100);
    assert_eq!(map.get(c), Some(&100));
    assert_eq!(map.get_path(&CoordPath::new([c])), Some(&100));
}

#[test]
fn flat_map_remove_all() {
    let mut map = FlatMap::new();
    for i in 0u16..11172 {
        map.insert(Coord::new(i).unwrap(), i as u32);
    }
    for i in 0u16..11172 {
        map.remove(Coord::new(i).unwrap());
    }
    assert!(map.is_empty());
}

#[test]
fn flat_map_clear() {
    let mut map = FlatMap::new();
    for i in 0u16..100 {
        map.insert(Coord::new(i).unwrap(), i);
    }
    map.clear();
    assert!(map.is_empty());
}

// ── TreeMap<2> — multi-syllable tree ──

#[test]
fn tree2_basic() {
    let mut map = TreeMap::<2, u32>::new();
    let path = CoordPath::new([Coord::new(0).unwrap(), Coord::new(1).unwrap()]);
    assert_eq!(map.insert_path(&path, 42), None);
    assert_eq!(map.get_path(&path), Some(&42));
    assert_eq!(map.len(), 1);
}

#[test]
fn tree2_multiple_branches() {
    let mut map = TreeMap::<2, u32>::new();
    let mut count = 0u32;
    for i in 0u16..5 {
        for m in 0u16..5 {
            let path = CoordPath::new([Coord::new(i).unwrap(), Coord::new(m).unwrap()]);
            map.insert_path(&path, count);
            count += 1;
        }
    }
    assert_eq!(map.len(), 25);
    for i in 0u16..5 {
        for m in 0u16..5 {
            let path = CoordPath::new([Coord::new(i).unwrap(), Coord::new(m).unwrap()]);
            assert_eq!(map.get_path(&path), Some(&((i * 5 + m) as u32)));
        }
    }
}

#[test]
fn tree2_overwrite_preserves_others() {
    let mut map = TreeMap::<2, u32>::new();
    let path_a = CoordPath::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()]);
    let path_b = CoordPath::new([Coord::new(5).unwrap(), Coord::new(3).unwrap()]);
    map.insert_path(&path_a, 1);
    map.insert_path(&path_b, 2);
    map.insert_path(&path_a, 10);
    assert_eq!(map.len(), 2);
    assert_eq!(map.get_path(&path_a), Some(&10));
    assert_eq!(map.get_path(&path_b), Some(&2));
}

// ── TreeMap6 — UUID-scale ──

#[test]
fn tree6_basic() {
    let mut map = TreeMap6::new();
    let coords: [Coord; 6] = core::array::from_fn(|i| Coord::new(i as u16).unwrap());
    let path = CoordPath::new(coords);
    map.insert_path(&path, 42);
    assert_eq!(map.get_path(&path), Some(&42));
    assert_eq!(map.len(), 1);
}

// ── TreeMap19 — SHA-256-scale ──

#[test]
fn tree19_insert_and_get() {
    let mut map = TreeMap19::new();
    let coords: [Coord; 19] = core::array::from_fn(|i| Coord::new(i as u16).unwrap());
    let path = CoordPath::new(coords);
    map.insert_path(&path, 42);
    assert_eq!(map.get_path(&path), Some(&42));
    assert_eq!(map.len(), 1);
}

#[test]
fn tree19_multiple_paths() {
    let mut map = TreeMap19::new();
    let make_path = |offset: u16| -> CoordPath<19> {
        let mut coords = [Coord::new(0).unwrap(); 19];
        for i in 0..19u16 {
            coords[i as usize] = Coord::new((i * 587 + offset) % 11172).unwrap();
        }
        CoordPath::new(coords)
    };
    let path_a = make_path(0);
    let path_b = make_path(7);
    map.insert_path(&path_a, "first");
    map.insert_path(&path_b, "second");
    assert_eq!(map.len(), 2);
    assert_eq!(map.get_path(&path_a), Some(&"first"));
    assert_eq!(map.get_path(&path_b), Some(&"second"));
}

// ── Boundary cases ──

#[test]
fn tree2_boundary_coords() {
    let mut map = TreeMap::<2, u32>::new();
    let c0 = Coord::from_axes(18, 20, 27).unwrap();
    let c1 = Coord::from_axes(18, 20, 27).unwrap();
    let path = CoordPath::new([c0, c1]);
    map.insert_path(&path, 42);
    assert_eq!(map.get_path(&path), Some(&42));
}

#[test]
fn all_depths_use_same_pattern() {
    let _f: FlatMap<u32> = FlatMap::new();
    let _t2: TreeMap<2, u32> = TreeMap::new();
    let _t6: TreeMap<6, u32> = TreeMap::new();
    let _t19: TreeMap<19, u32> = TreeMap::new();
}
