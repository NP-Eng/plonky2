use std::time::Instant;

use common::{estimate_commitment_time_capped, init_logger, Blake3Hash};
use plonky2::hash::merkle_tree::MerkleTree;
use plonky2_field::goldilocks_field::GoldilocksField;
use plonky2_field::types::Sample;

mod common;

type F = GoldilocksField;
type H = Blake3Hash<25>;

// log2 of the number of leaves
const LEVEL_N: usize = 20;
// log2 of the number of caps
const LEVEL_K: usize = 0;

fn main() {
    init_logger();

    let leaves = vec![vec![F::rand()]; 2 << LEVEL_N];

    estimate_commitment_time_capped(LEVEL_N, LEVEL_K, "Blake3");

    let start = Instant::now();
    MerkleTree::<F, H>::new(leaves.clone(), LEVEL_K);
    let elapsed = start.elapsed();

    println!("Merkle tree built in {:?} ", elapsed);
}
