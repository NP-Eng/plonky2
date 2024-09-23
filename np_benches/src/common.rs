use std::time::Duration;

use blake3::Hasher as Blake3Hasher;
use plonky2::hash::hash_types::{BytesHash, RichField};
use plonky2::hash::hashing::PlonkyPermutation;
use plonky2::hash::poseidon::PoseidonHash;
use plonky2::plonk::config::{GenericConfig, Hasher};
use plonky2::util::serialization::Write;
use plonky2_field::extension::quadratic::QuadraticExtension;
use plonky2_field::goldilocks_field::GoldilocksField;

pub const SPONGE_RATE: usize = 8;
pub const SPONGE_CAPACITY: usize = 4;
pub const SPONGE_WIDTH: usize = SPONGE_RATE + SPONGE_CAPACITY;

pub fn init_logger() {
    let _ = env_logger::builder().format_timestamp(None).try_init();
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Blake3Permutation<F: RichField> {
    state: [F; SPONGE_WIDTH],
}

impl<F: RichField> Eq for Blake3Permutation<F> {}

impl<F: RichField> AsRef<[F]> for Blake3Permutation<F> {
    fn as_ref(&self) -> &[F] {
        &self.state
    }
}

impl<F: RichField> PlonkyPermutation<F> for Blake3Permutation<F> {
    const RATE: usize = SPONGE_RATE;
    const WIDTH: usize = SPONGE_WIDTH;

    fn new<I: IntoIterator<Item = F>>(elts: I) -> Self {
        unimplemented!()
    }

    fn set_elt(&mut self, elt: F, idx: usize) {
        unimplemented!()
    }

    fn set_from_slice(&mut self, elts: &[F], start_idx: usize) {
        unimplemented!()
    }

    fn set_from_iter<I: IntoIterator<Item = F>>(&mut self, elts: I, start_idx: usize) {
        unimplemented!()
    }

    fn permute(&mut self) {
        unimplemented!()
    }

    fn squeeze(&self) -> &[F] {
        unimplemented!()
    }
}

/// Blake3 hash function.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Blake3Hash<const N: usize>;

impl<F: RichField, const N: usize> Hasher<F> for Blake3Hash<N> {
    const HASH_SIZE: usize = N;
    type Hash = BytesHash<N>;
    type Permutation = Blake3Permutation<F>;

    fn hash_no_pad(input: &[F]) -> Self::Hash {
        let mut buffer = Vec::with_capacity(input.len());
        buffer.write_field_vec(input).unwrap();
        let hash = Blake3Hasher::new().update(&buffer).finalize();
        let mut arr = [0; N];
        arr.copy_from_slice(&hash.as_bytes()[..N]);
        BytesHash(arr)
    }

    fn two_to_one(left: Self::Hash, right: Self::Hash) -> Self::Hash {
        let mut v = vec![0; N * 2];
        v[0..N].copy_from_slice(&left.0);
        v[N..].copy_from_slice(&right.0);
        let mut arr = [0; N];
        let hash = Blake3Hasher::new().update(&v).finalize();
        arr.copy_from_slice(&hash.as_bytes()[..N]);
        BytesHash(arr)
    }
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub struct Blake3GoldilocksConfig;
impl GenericConfig<2> for Blake3GoldilocksConfig {
    type F = GoldilocksField;
    type FE = QuadraticExtension<Self::F>;
    type Hasher = Blake3Hash<25>;
    type InnerHasher = PoseidonHash;
}

const VERBOSE: bool = true;

// Hashing times in nanoseconds
const HASHING_TIMES: [(&str, usize); 3] = [("Poseidon", 41000), ("Keccak", 19000), ("Blake3", 650)];

fn get_hashing_time(hash: &str) -> usize {
    HASHING_TIMES.iter().find(|(h, _)| *h == hash).unwrap().1
}

// n: level of the leaves (i.e. the number of leaves is 2^n)
// m: level where the switch to h1 happens (already computed with h1)
// k: level of the caps
// Returns the estimated running time in ns
pub fn estimate_commitment_time_mixed_capped(
    n: usize,
    m: usize,
    k: usize,
    h1: &str,
    h2: &str,
) -> usize {
    let time_h1 = get_hashing_time(h1);
    let time_h2 = get_hashing_time(h2);
    let time = ((2 << n) - (2 << m)) * time_h1 + ((2 << m) - (2 << k)) * time_h2;

    if VERBOSE {
        println!(
            "Estimating commitment time for n={}, m={}, k={}, h1={}, h2={}",
            n, m, k, h1, h2
        );

        let human_readable_time = Duration::from_nanos(time as u64);
        println!("Estimate: {:?}", human_readable_time);
    }

    time
}

pub fn estimate_commitment_time_mixed(n: usize, m: usize, h1: &str, h2: &str) -> usize {
    estimate_commitment_time_mixed_capped(n, m, 0, h1, h2)
}

pub fn estimate_commitment_time_capped(n: usize, k: usize, h: &str) -> usize {
    estimate_commitment_time_mixed_capped(n, k, k, h, h)
}

pub fn estimate_commitment_time(n: usize, h: &str) -> usize {
    estimate_commitment_time_mixed_capped(n, 0, 0, h, h)
}

pub fn estimate_verification_time_mixed_capped(
    n: usize,
    m: usize,
    k: usize,
    h1: &str,
    h2: &str,
) -> usize {
    let time_h1 = get_hashing_time(h1);
    let time_h2 = get_hashing_time(h2);
    (n - m) * time_h1 + (m - k) * time_h2
}

pub fn estimate_verification_time_mixed(n: usize, m: usize, h1: &str, h2: &str) -> usize {
    estimate_verification_time_mixed_capped(n, m, 0, h1, h2)
}

pub fn estimate_verification_time_capped(n: usize, k: usize, h: &str) -> usize {
    estimate_verification_time_mixed_capped(n, k, k, h, h)
}

pub fn estimate_verification_time(n: usize, h: &str) -> usize {
    estimate_verification_time_mixed_capped(n, 0, 0, h, h)
}

// Sanity checks:
// - passing the same hash twice yields the same as the single-hash version
// - it matches the formulae we wrote

#[cfg(test)]
#[test]
fn test_estimator() {
    assert_eq!(
        estimate_commitment_time_capped(28, 4, "Poseidon"),
        estimate_commitment_time_mixed_capped(28, 28, 4, "Poseidon", "Poseidon"),
    );
    // TODO add more
}
