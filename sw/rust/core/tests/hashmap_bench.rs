use std::collections::HashMap;
use std::time::Instant;
use tagma_core::DynCoordMap;

const N: u32 = 10_000;

fn bench_insert() {
    let mut hm: HashMap<String, u32> = HashMap::new();
    let mut dm: DynCoordMap<u32> = DynCoordMap::new();

    // HashMap
    let start = Instant::now();
    for i in 0..N {
        hm.insert(format!("key-{i}"), i);
    }
    let t_hm = start.elapsed();

    // DynCoordMap
    let start = Instant::now();
    for i in 0..N {
        dm.insert_str(&format!("key-{i}"), i);
    }
    let t_dm = start.elapsed();

    println!("── insert {N} ──");
    println!("  HashMap:     {:>8?}  {:>6.0} ns/op", t_hm, t_hm.as_nanos() as f64 / N as f64);
    println!("  DynCoordMap: {:>8?}  {:>6.0} ns/op", t_dm, t_dm.as_nanos() as f64 / N as f64);
    let ratio = t_dm.as_nanos() as f64 / t_hm.as_nanos() as f64;
    println!("  ratio: {ratio:.2}x");
}

fn bench_get() {
    let mut hm: HashMap<String, u32> = HashMap::new();
    let mut dm: DynCoordMap<u32> = DynCoordMap::new();

    for i in 0..N {
        let k = format!("key-{i}");
        hm.insert(k.clone(), i);
        dm.insert_str(&k, i);
    }

    let keys: Vec<String> = (0..N).map(|i| format!("key-{i}")).collect();

    // HashMap
    let start = Instant::now();
    for k in &keys {
        std::hint::black_box(hm.get(k));
    }
    let t_hm = start.elapsed();

    // DynCoordMap
    let start = Instant::now();
    for k in &keys {
        std::hint::black_box(dm.get_str(k));
    }
    let t_dm = start.elapsed();

    println!("── get {N} ──");
    println!("  HashMap:     {:>8?}  {:>6.0} ns/op", t_hm, t_hm.as_nanos() as f64 / N as f64);
    println!("  DynCoordMap: {:>8?}  {:>6.0} ns/op", t_dm, t_dm.as_nanos() as f64 / N as f64);
    let ratio = t_dm.as_nanos() as f64 / t_hm.as_nanos() as f64;
    println!("  ratio: {ratio:.2}x");
}

fn bench_remove() {
    let mut hm: HashMap<String, u32> = HashMap::new();
    let mut dm: DynCoordMap<u32> = DynCoordMap::new();

    for i in 0..N {
        let k = format!("key-{i}");
        hm.insert(k.clone(), i);
        dm.insert_str(&k, i);
    }

    let keys: Vec<String> = (0..N).map(|i| format!("key-{i}")).collect();

    // HashMap
    let start = Instant::now();
    for k in &keys {
        std::hint::black_box(hm.remove(k));
    }
    let t_hm = start.elapsed();

    // DynCoordMap
    let start = Instant::now();
    for k in &keys {
        std::hint::black_box(dm.remove_str(k));
    }
    let t_dm = start.elapsed();

    println!("── remove {N} ──");
    println!("  HashMap:     {:>8?}  {:>6.0} ns/op", t_hm, t_hm.as_nanos() as f64 / N as f64);
    println!("  DynCoordMap: {:>8?}  {:>6.0} ns/op", t_dm, t_dm.as_nanos() as f64 / N as f64);
    let ratio = t_dm.as_nanos() as f64 / t_hm.as_nanos() as f64;
    println!("  ratio: {ratio:.2}x");
}

fn bench_long_key_get() {
    let long_key = "k".repeat(500);
    let mut hm: HashMap<String, u32> = HashMap::new();
    let mut dm: DynCoordMap<u32> = DynCoordMap::new();
    hm.insert(long_key.clone(), 42);
    dm.insert_str(&long_key, 42);

    // HashMap
    let start = Instant::now();
    for _ in 0..N {
        std::hint::black_box(hm.get(&long_key));
    }
    let t_hm = start.elapsed();

    // DynCoordMap
    let start = Instant::now();
    for _ in 0..N {
        std::hint::black_box(dm.get_str(&long_key));
    }
    let t_dm = start.elapsed();

    println!("── get 500-char key {N} ──");
    println!("  HashMap:     {:>8?}  {:>6.0} ns/op", t_hm, t_hm.as_nanos() as f64 / N as f64);
    println!("  DynCoordMap: {:>8?}  {:>6.0} ns/op", t_dm, t_dm.as_nanos() as f64 / N as f64);
    let ratio = t_dm.as_nanos() as f64 / t_hm.as_nanos() as f64;
    println!("  ratio: {ratio:.2}x");
}

#[test]
fn bench_report() {
    bench_insert();
    bench_get();
    bench_remove();
    bench_long_key_get();
}
