use std::collections::HashMap;
use tagma_core::DynCoordMap;

fn hashmap_fixture() -> HashMap<String, u32> {
    let mut m = HashMap::new();
    m.insert("alpha".into(), 1);
    m.insert("beta".into(), 2);
    m.insert("gamma".into(), 3);
    m
}

fn dynmap_fixture() -> DynCoordMap<u32> {
    let mut m = DynCoordMap::new();
    m.insert_str("alpha", 1);
    m.insert_str("beta", 2);
    m.insert_str("gamma", 3);
    m
}

#[test]
fn insert_and_get() {
    let mut hm: HashMap<String, u32> = HashMap::new();
    let mut dm: DynCoordMap<u32> = DynCoordMap::new();
    for k in ["a", "b", "c"] {
        assert_eq!(hm.insert(k.to_string(), 42), dm.insert_str(k, 42));
    }
    for k in ["a", "b", "c", "d"] {
        assert_eq!(hm.get(k), dm.get_str(k));
    }
}

#[test]
fn insert_overwrite_returns_old() {
    let mut hm: HashMap<String, u32> = HashMap::new();
    let mut dm: DynCoordMap<u32> = DynCoordMap::new();
    assert_eq!(hm.insert("k".into(), 1), dm.insert_str("k", 1));
    assert_eq!(hm.insert("k".into(), 2), dm.insert_str("k", 2));
    assert_eq!(hm.get("k"), dm.get_str("k"));
}

#[test]
fn remove_returns_value() {
    let mut hm = hashmap_fixture();
    let mut dm = dynmap_fixture();
    assert_eq!(hm.remove("alpha"), dm.remove_str("alpha"));
    assert_eq!(hm.remove("alpha"), dm.remove_str("alpha"));
    assert_eq!(hm.get("beta"), dm.get_str("beta"));
    assert_eq!(hm.is_empty(), dm.entry_count() == 0);
}

#[test]
fn remove_nonexistent() {
    let mut hm = hashmap_fixture();
    let mut dm = dynmap_fixture();
    assert_eq!(hm.remove("nonexistent"), dm.remove_str("nonexistent"));
}

#[test]
fn contains_key() {
    let hm = hashmap_fixture();
    let dm = dynmap_fixture();
    assert_eq!(hm.contains_key("alpha"), dm.get_str("alpha").is_some());
    assert_eq!(hm.contains_key("nonexistent"), dm.get_str("nonexistent").is_some());
}

#[test]
fn len_and_is_empty() {
    let mut hm: HashMap<String, u32> = HashMap::new();
    let mut dm: DynCoordMap<u32> = DynCoordMap::new();
    assert_eq!(hm.len(), dm.entry_count());
    assert_eq!(hm.is_empty(), dm.entry_count() == 0);
    hm.insert("x".into(), 1);
    dm.insert_str("x", 1);
    assert_eq!(hm.len(), dm.entry_count());
    hm.remove("x");
    dm.remove_str("x");
    assert_eq!(hm.len(), dm.entry_count());
}

#[test]
fn clear() {
    let mut hm = hashmap_fixture();
    let mut dm = dynmap_fixture();
    hm.clear();
    dm.clear();
    assert_eq!(hm.len(), dm.entry_count());
    assert_eq!(hm.is_empty(), dm.entry_count() == 0);
    assert_eq!(hm.get("alpha"), dm.get_str("alpha"));
}

#[test]
fn bulk_insert_1000() {
    let mut hm: HashMap<String, u32> = HashMap::new();
    let mut dm: DynCoordMap<u32> = DynCoordMap::new();
    for i in 0..1000u32 {
        let k = format!("key-{i}");
        assert_eq!(hm.insert(k.clone(), i), dm.insert_str(&k, i));
    }
    assert_eq!(hm.len(), dm.entry_count());
    for i in (0..1000u32).step_by(100) {
        let k = format!("key-{i}");
        assert_eq!(hm.get(&k), dm.get_str(&k), "mismatch at key-{i}");
    }
}

#[test]
fn bulk_insert_then_remove_all() {
    let mut hm: HashMap<String, u32> = HashMap::new();
    let mut dm: DynCoordMap<u32> = DynCoordMap::new();
    for i in 0..500u32 {
        let k = format!("k{i}");
        hm.insert(k.clone(), i);
        dm.insert_str(&k, i);
    }
    assert_eq!(hm.len(), dm.entry_count());
    for i in 0..500u32 {
        let k = format!("k{i}");
        assert_eq!(hm.remove(&k), dm.remove_str(&k));
    }
    assert_eq!(hm.is_empty(), dm.entry_count() == 0);
}

#[test]
fn long_string_equivalent() {
    let mut hm: HashMap<String, u32> = HashMap::new();
    let mut dm: DynCoordMap<u32> = DynCoordMap::new();
    let long_key = "a".repeat(1000);
    hm.insert(long_key.clone(), 999);
    dm.insert_str(&long_key, 999);
    assert_eq!(hm.get(&long_key), dm.get_str(&long_key));
    hm.remove(&long_key);
    dm.remove_str(&long_key);
    assert_eq!(hm.get(&long_key), dm.get_str(&long_key));
}

#[test]
fn unicode_keys_equivalent() {
    let mut hm: HashMap<String, u32> = HashMap::new();
    let mut dm: DynCoordMap<u32> = DynCoordMap::new();
    let keys = [
        "Hello, 世界!",
        "한글/조합/형",
        " \t\n\r ",
    ];
    for (i, &k) in keys.iter().enumerate() {
        hm.insert(k.to_string(), i as u32);
        dm.insert_str(k, i as u32);
    }
    for (i, &k) in keys.iter().enumerate() {
        assert_eq!(hm.get(k), dm.get_str(k), "mismatch for key {i}: {k:?}");
    }
    assert_eq!(hm.len(), dm.entry_count());
}

#[test]
fn determinism() {
    let mut dm = DynCoordMap::new();
    dm.insert_str("deterministic", 1);
    assert_eq!(dm.get_str("deterministic"), Some(&1));
    assert_eq!(dm.insert_str("deterministic", 2), Some(1));
    assert_eq!(dm.get_str("deterministic"), Some(&2));
}

#[test]
fn no_false_collisions() {
    let mut dm: DynCoordMap<u32> = DynCoordMap::new();
    for i in 0..500u32 {
        dm.insert_str(&format!("user-{i}"), i);
    }
    for i in 0..500u32 {
        let k = format!("user-{i}");
        assert_eq!(dm.get_str(&k), Some(&i), "missing or wrong value for {k}");
    }
    assert_eq!(dm.entry_count(), 500);
}

#[test]
fn empty_map_behavior() {
    let hm: HashMap<String, u32> = HashMap::new();
    let dm: DynCoordMap<u32> = DynCoordMap::new();
    assert_eq!(hm.len(), dm.entry_count());
    assert_eq!(hm.is_empty(), dm.entry_count() == 0);
    assert_eq!(hm.get("anything"), dm.get_str("anything"));
}
