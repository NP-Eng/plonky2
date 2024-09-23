use common::{estimate_commitment_time, estimate_commitment_time_mixed_capped};

mod common;

fn main() {
    estimate_commitment_time_mixed_capped(28, 27, 4, "Poseidon", "Poseidon");
    estimate_commitment_time_mixed_capped(28, 27, 4, "Keccak", "Poseidon");
    estimate_commitment_time_mixed_capped(28, 27, 4, "Blake3", "Poseidon");

    estimate_commitment_time(16, "Poseidon");
}
