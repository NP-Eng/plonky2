use core::hash;
use std::collections::HashMap;

const HASHING_TIMES: [(&str, usize); 3] = [("Poseidon", 3), ("Keccak", 10), ("Blake3", 20)];

fn get_hashing_time(hash: &str) -> usize {
    HASHING_TIMES.iter().find(|(h, _)| *h == hash).unwrap().1
}

// n: level of the leaves (i.e. the number of leaves is 2^n)
// m: level where the switch to h1 happens (already computed with h1)
// k: level of the caps
// Returns the estimated running time in ns
fn estimate_running_time_mixed_capped(n: usize, m: usize, k: usize, h1: &str, h2: &str) -> usize {
    let time_h1 = get_hashing_time(h1);
    let time_h2 = get_hashing_time(h2);
    ((2 << n) - (2 << m)) * time_h1 + ((2 << m) - (2 << k)) * time_h2
}

fn estimate_running_time_mixed(n: usize, m: usize, h1: &str, h2: &str) -> usize {
    estimate_running_time_mixed_capped(n, m, 0, h1, h2)
}

fn estimate_running_time(n: usize, h: &str) -> usize {
    estimate_running_time_mixed_capped(n, 0, 0, h, h)
}

// Sanity checks:
// - passing the same hash twice yields the same as the single-hash version
// - it matches the formulae we wrote
