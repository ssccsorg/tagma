use tagma_core::{Coord, CoordMap, CoordMap19, CoordMap6, CoordPath};

// ── CoordMap — no_alloc single-syllable ──

#[test]
fn cm_insert_11172_values() {
    let mut map = CoordMap::new();
    for i in 0u16..11172 {
        assert_eq!(map.insert(Coord::new(i).unwrap(), i as u32), None);
    }
    assert_eq!(map.len(), 11172);
}

#[test]
fn cm_all_11172_accessible() {
    let mut map = CoordMap::new();
    for i in 0u16..11172 {
        map.insert(Coord::new(i).unwrap(), i);
    }
    for i in 0u16..11172 {
        assert_eq!(map.get(&Coord::new(i).unwrap()), Some(&i));
    }
}

#[test]
fn cm_path_api() {
    let mut map = CoordMap::new();
    let c = Coord::new(5555).unwrap();
    map.insert(c, 100);
    assert_eq!(map.get(&c), Some(&100));
    assert_eq!(map.get_path(&CoordPath::new([c])), Some(&100));
}

#[test]
fn cm_remove_all() {
    let mut map = CoordMap::new();
    for i in 0u16..11172 {
        map.insert(Coord::new(i).unwrap(), i as u32);
    }
    for i in 0u16..11172 {
        map.remove(&Coord::new(i).unwrap());
    }
    assert!(map.is_empty());
}

#[test]
fn cm_clear() {
    let mut map = CoordMap::new();
    for i in 0u16..100 {
        map.insert(Coord::new(i).unwrap(), i);
    }
    map.clear();
    assert!(map.is_empty());
}

// ── CoordMap6 — UUID-scale tree ──

#[test]
fn cm6_basic() {
    let mut map = CoordMap6::new();
    let coords: [Coord; 6] = core::array::from_fn(|i| Coord::new(i as u16).unwrap());
    let path = CoordPath::new(coords);
    map.insert_path(&path, 42);
    assert_eq!(map.get_path(&path), Some(&42));
    assert_eq!(map.len(), 1);
}

#[test]
fn cm6_missing_path() {
    let map: CoordMap6<u32> = CoordMap6::new();
    let path = CoordPath::new(core::array::from_fn(|_| Coord::new(0).unwrap()));
    assert_eq!(map.get_path(&path), None);
}

// ── CoordMap19 — SHA-256-scale tree ──

#[test]
fn cm19_insert_and_get() {
    let mut map = CoordMap19::new();
    let coords: [Coord; 19] = core::array::from_fn(|i| Coord::new(i as u16).unwrap());
    let path = CoordPath::new(coords);
    map.insert_path(&path, 42);
    assert_eq!(map.get_path(&path), Some(&42));
    assert_eq!(map.len(), 1);
}

#[test]
fn cm19_multiple_paths() {
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

// ── Consistency ──

#[test]
fn all_series_use_same_pattern() {
    let _m: CoordMap<u32> = CoordMap::new();
    let _m6: CoordMap6<u32> = CoordMap6::new();
    let _m19: CoordMap19<u32> = CoordMap19::new();
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
