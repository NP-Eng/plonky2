use criterion::{criterion_group, criterion_main, Criterion};
use plonky2::field::types::Field;
use plonky2::fri::reduction_strategies::FriReductionStrategy;
use plonky2::fri::FriConfig;
use plonky2::plonk::config::{GenericConfig, KeccakGoldilocksConfig, PoseidonGoldilocksConfig};
use plonky2::util::timing::TimingTree;
use starky::config::StarkConfig;
use starky::fibonacci_stark::FibonacciStark;
use starky::prover::prove;
use starky::verifier::verify_stark_proof;

macro_rules! stark_config {
    ($arity:expr, $fri_reduction_arity:expr) => {
        (
            format!(
                "ConstantArityBits({}, {}){}",
                $arity,
                $fri_reduction_arity,
                if $arity == 4 && $fri_reduction_arity == 5 {
                    " aka Standard"
                } else {
                    ""
                }
            ),
            StarkConfig::new(
                100,
                2,
                FriConfig {
                    rate_bits: 1,
                    cap_height: 4,
                    proof_of_work_bits: 16,
                    reduction_strategy: FriReductionStrategy::ConstantArityBits(
                        $arity,
                        $fri_reduction_arity,
                    ),
                    num_query_rounds: 84,
                },
            ),
        )
    };
}

fn fibonacci<F: Field>(n: usize, x0: F, x1: F) -> F {
    (0..n).fold((x0, x1), |x, _| (x.1, x.0 + x.1)).1
}

fn generate_fri_reduction_strategy_configs() -> Vec<(String, StarkConfig)> {
    vec![
        stark_config!(2, 5),
        stark_config!(4, 5),
        stark_config!(6, 5),
        stark_config!(8, 5),
    ]
}

fn generate_pow_configs() -> Vec<(String, StarkConfig)> {
    vec![pow_config(0), pow_config(8), pow_config(16), pow_config(24)]
}

fn pow_config(proof_of_work_bits: usize) -> (String, StarkConfig) {
    const SECURITY_LEVEL: usize = 100;
    let num_query_rounds = SECURITY_LEVEL - proof_of_work_bits;
    (
        format!("PoW({} bits)", proof_of_work_bits),
        StarkConfig::new(
            SECURITY_LEVEL,
            2,
            FriConfig {
                rate_bits: 1,
                cap_height: 4,
                proof_of_work_bits: proof_of_work_bits as u32,
                reduction_strategy: FriReductionStrategy::ConstantArityBits(4, 5),
                num_query_rounds,
            },
        ),
    )
}

fn bench_fibonacci_stark(c: &mut Criterion, config: StarkConfig, config_name: &str) {
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type KC = KeccakGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;
    type S = FibonacciStark<F, D>;

    let row_sizes = [1 << 10, 1 << 20];

    for &num_rows in &row_sizes {
        let public_inputs = [F::ZERO, F::ONE, fibonacci(num_rows - 1, F::ZERO, F::ONE)];
        let stark = S::new(num_rows);
        let trace = stark.generate_trace(public_inputs[0], public_inputs[1]);

        // Benchmark Poseidon proving
        c.bench_function(
            &format!(
                "Fibonacci STARK Poseidon Prove {} - {}",
                num_rows, config_name
            ),
            |b| {
                b.iter(|| {
                    prove::<F, C, S, D>(
                        stark,
                        &config,
                        trace.clone(),
                        &public_inputs,
                        &mut TimingTree::default(),
                    )
                    .unwrap()
                })
            },
        );

        // Benchmark Poseidon verification
        let poseidon_proof = prove::<F, C, S, D>(
            stark,
            &config,
            trace.clone(),
            &public_inputs,
            &mut TimingTree::default(),
        )
        .unwrap();

        c.bench_function(
            &format!(
                "Fibonacci STARK Poseidon Verify {} - {}",
                num_rows, config_name
            ),
            |b| b.iter(|| verify_stark_proof(stark, poseidon_proof.clone(), &config).unwrap()),
        );

        // Benchmark Keccak proving
        c.bench_function(
            &format!(
                "Fibonacci STARK Keccak Prove {} - {}",
                num_rows, config_name
            ),
            |b| {
                b.iter(|| {
                    prove::<F, KC, S, D>(
                        stark,
                        &config,
                        trace.clone(),
                        &public_inputs,
                        &mut TimingTree::default(),
                    )
                    .unwrap()
                })
            },
        );

        // Benchmark Keccak verification
        let keccak_proof = prove::<F, KC, S, D>(
            stark,
            &config,
            trace.clone(),
            &public_inputs,
            &mut TimingTree::default(),
        )
        .unwrap();

        c.bench_function(
            &format!(
                "Fibonacci STARK Keccak Verify {} - {}",
                num_rows, config_name
            ),
            |b| b.iter(|| verify_stark_proof(stark, keccak_proof.clone(), &config).unwrap()),
        );
    }
}

fn bench_all_configs(c: &mut Criterion) {
    for (config_name, config) in generate_fri_reduction_strategy_configs() {
        bench_fibonacci_stark(c, config, &config_name);
    }

    for (config_name, config) in generate_pow_configs() {
        bench_fibonacci_stark(c, config, &config_name);
    }
}

criterion_group!(benches, bench_all_configs);
criterion_main!(benches);
