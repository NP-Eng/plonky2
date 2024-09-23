use plonky2::plonk::config::{GenericConfig, Hasher, PoseidonGoldilocksConfig};
use plonky2_field::goldilocks_field::GoldilocksField;
use plonky2_field::types::Sample;

type GF = GoldilocksField;
type H = <PoseidonGoldilocksConfig as GenericConfig<2>>::Hasher;

const N_ITER: usize = 2 << 16;

fn main() {
    let inputs = vec![GF::rand(); N_ITER];

    let start = std::time::Instant::now();

    for pair in inputs.windows(2) {
        H::hash_no_pad(pair);
    }

    let elapsed = start.elapsed();

    println!("Two-to-one Poseidon Hash");
    println!("Goldilocks field");
    println!("{N_ITER} iterations");
    println!("Time: {:?}", elapsed);
    println!("Time per iteration: {:?}", elapsed / N_ITER as u32);
}
