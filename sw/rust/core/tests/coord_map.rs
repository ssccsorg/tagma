use tagma_core::{Coord, CoordMap, CoordMap1, CoordMap2, CoordMap19, CoordPath};

// ── CoordMap<1> — flat map integration tests ──

#[test]
fn flat_map_insert_11172_values() {
    let mut map = CoordMap::<1, u32>::new();
    for i in 0u16..11172 {
        let c = Coord::new(i).unwrap();
        assert_eq!(map.insert(c, i as u32), None);
    }
    assert_eq!(map.len(), 11172);
}

#[test]
fn flat_map_all_11172_accessible() {
    let mut map = CoordMap1::new();
    for i in 0u16..11172 {
        let c = Coord::new(i).unwrap();
        map.insert(c, i);
    }
    for i in 0u16..11172 {
        let c = Coord::new(i).unwrap();
        assert_eq!(map.get(c), Some(&i));
    }
}

#[test]
fn flat_map_path_api_equivalence() {
    let mut map = CoordMap::<1, u32>::new();
    let c = Coord::new(5555).unwrap();
    map.insert(c, 100);
    assert_eq!(map.get(c), Some(&100));
    assert_eq!(map.get_path(&CoordPath::new([c])), Some(&100));
}

#[test]
fn flat_map_remove_all() {
    let mut map = CoordMap::<1, u32>::new();
    for i in 0u16..11172 {
        map.insert(Coord::new(i).unwrap(), i as u32);
    }
    for i in 0u16..11172 {
        map.remove(Coord::new(i).unwrap());
    }
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
}

#[test]
fn flat_map_clear() {
    let mut map = CoordMap1::new();
    for i in 0u16..100 {
        map.insert(Coord::new(i).unwrap(), i);
    }
    map.clear();
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
}

// ── CoordMap<2> — tree integration tests ──

#[test]
fn tree2_multiple_branches() {
    let mut map = CoordMap2::new();
    let mut count = 0u32;
    for i in 0u16..5 {
        for m in 0u16..5 {
            let path = CoordPath::new([Coord::new(i).unwrap(), Coord::new(m).unwrap()]);
            map.insert_path(&path, count);
            count += 1;
        }
    }
    assert_eq!(map.len(), 25);

    // Verify
    for i in 0u16..5 {
        for m in 0u16..5 {
            let path = CoordPath::new([Coord::new(i).unwrap(), Coord::new(m).unwrap()]);
            assert_eq!(map.get_path(&path), Some(&((i * 5 + m) as u32)));
        }
    }
}

#[test]
fn tree2_missing_deep_path() {
    let map: CoordMap2<u32> = CoordMap2::new();
    let path = CoordPath::new([
        Coord::from_axes(18, 20, 27).unwrap(),
        Coord::from_axes(0, 0, 0).unwrap(),
    ]);
    assert_eq!(map.get_path(&path), None);
}

#[test]
fn tree2_overwrite_preserves_other_paths() {
    let mut map = CoordMap2::new();
    let path_a = CoordPath::new([
        Coord::from_axes(0, 0, 0).unwrap(),
        Coord::from_axes(0, 0, 0).unwrap(),
    ]);
    let path_b = CoordPath::new([
        Coord::from_axes(5, 5, 5).unwrap(),
        Coord::from_axes(3, 3, 3).unwrap(),
    ]);
    map.insert_path(&path_a, 1);
    map.insert_path(&path_b, 2);
    map.insert_path(&path_a, 10);
    assert_eq!(map.len(), 2);
    assert_eq!(map.get_path(&path_a), Some(&10));
    assert_eq!(map.get_path(&path_b), Some(&2));
}

// ── CoordMap<19> — SHA-256-scale tree ──

#[test]
fn tree19_insert_and_get() {
    let mut map = CoordMap19::new();
    let coords: [Coord; 19] = core::array::from_fn(|i| Coord::new(i as u16).unwrap());
    let path = CoordPath::new(coords);
    map.insert_path(&path, 42);
    assert_eq!(map.get_path(&path), Some(&42));
    assert_eq!(map.len(), 1);
}

#[test]
fn tree19_multiple_paths() {
    let mut map = CoordMap19::new();
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

// ── Cross-depth type consistency ──

#[test]
fn all_depths_share_same_construction_pattern() {
    let _m1: CoordMap<1, u32> = CoordMap::new();
    let _m2: CoordMap<2, u32> = CoordMap::new();
    let _m6: CoordMap<6, u32> = CoordMap::new();
    let _m12: CoordMap<12, u32> = CoordMap::new();
    let _m19: CoordMap<19, u32> = CoordMap::new();
}

#[test]
fn coord_path_is_not_a_key() {
    let path = CoordPath::<3>::new([
        Coord::new(0).unwrap(),
        Coord::new(1).unwrap(),
        Coord::new(2).unwrap(),
    ]);
    assert_eq!(path.len(), 3);
    assert!(!path.is_empty());
}

// ── Edge cases ──

#[test]
fn flat_map_coord_path_boundary() {
    let mut map = CoordMap::<1, u32>::new();
    let last = Coord::new(11171).unwrap();
    map.insert_path(&CoordPath::new([last]), 999);
    assert_eq!(map.get_path(&CoordPath::new([last])), Some(&999));
}

#[test]
fn tree2_boundary_coords() {
    let mut map = CoordMap2::new();
    let c0 = Coord::from_axes(18, 20, 27).unwrap();
    let c1 = Coord::from_axes(18, 20, 27).unwrap();
    let path = CoordPath::new([c0, c1]);
    map.insert_path(&path, 42);
    assert_eq!(map.get_path(&path), Some(&42));
}
