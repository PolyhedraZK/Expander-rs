use circuit::Circuit;
use config::{BN254ConfigSha2, Config, GKRConfig, GKRScheme, M31ExtConfigSha2, MPIConfig};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use gkr::{
    utils::{
        KECCAK_BN254_CIRCUIT, KECCAK_BN254_WITNESS, KECCAK_M31_CIRCUIT, KECCAK_M31_WITNESS,
        POSEIDON_BN254_CIRCUIT, POSEIDON_M31_CIRCUIT,
    },
    Prover,
};
use std::hint::black_box;

fn prover_run<C: GKRConfig>(config: &Config<C>, circuit: &mut Circuit<C>) {
    let mut prover = Prover::new(config);
    prover.prepare_mem(circuit);
    prover.prove(circuit);
}

fn benchmark_setup<C: GKRConfig>(
    scheme: GKRScheme,
    circuit_file: &str,
    witness_file: Option<&str>,
) -> (Config<C>, Circuit<C>) {
    let config = Config::<C>::new(scheme, MPIConfig::new());
    let mut circuit = Circuit::<C>::load_circuit(circuit_file);
    if witness_file.is_some() {
        circuit.load_witness_file(witness_file.unwrap());
    } else {
        circuit.set_random_input_for_test();
    }
    (config, circuit)
}

fn criterion_gkr_keccak(c: &mut Criterion) {
    let (m31_config, mut m31_circuit) = benchmark_setup::<M31ExtConfigSha2>(
        GKRScheme::Vanilla,
        KECCAK_M31_CIRCUIT,
        Some(KECCAK_M31_WITNESS),
    );
    let (bn254_config, mut bn254_circuit) = benchmark_setup::<BN254ConfigSha2>(
        GKRScheme::Vanilla,
        KECCAK_BN254_CIRCUIT,
        Some(KECCAK_BN254_WITNESS),
    );

    let num_keccak_m31 = 2 * M31ExtConfigSha2::get_field_pack_size();
    let num_keccak_bn254 = 2 * BN254ConfigSha2::get_field_pack_size();

    let mut group = c.benchmark_group("single thread proving keccak by GKR vanilla");
    group.bench_function(
        BenchmarkId::new(
            format!(
                "Over M31, with {} keccak instances per proof",
                num_keccak_m31
            ),
            0,
        ),
        |b| {
            b.iter(|| {
                {
                    prover_run(&m31_config, &mut m31_circuit);
                    black_box(())
                };
            })
        },
    );

    group.bench_function(
        BenchmarkId::new(
            format!(
                "Over BN254, with {} keccak instances per proof",
                num_keccak_bn254
            ),
            0,
        ),
        |b| {
            b.iter(|| {
                {
                    prover_run(&bn254_config, &mut bn254_circuit);
                    black_box(())
                };
            })
        },
    );
}

fn criterion_gkr_poseidon(c: &mut Criterion) {
    let (m31_config, mut m31_circuit) =
        benchmark_setup::<M31ExtConfigSha2>(GKRScheme::GkrSquare, POSEIDON_M31_CIRCUIT, None);
    let (bn254_config, mut bn254_circuit) =
        benchmark_setup::<BN254ConfigSha2>(GKRScheme::GkrSquare, POSEIDON_BN254_CIRCUIT, None);

    let mut group = c.benchmark_group("single thread proving poseidon by GKR^2");
    let num_poseidon_m31 = 120 * M31ExtConfigSha2::get_field_pack_size();
    let num_poseidon_bn254 = 120 * BN254ConfigSha2::get_field_pack_size();
    group.bench_function(
        BenchmarkId::new(
            format!(
                "Over M31, with {} poseidon instances per proof",
                num_poseidon_m31
            ),
            0,
        ),
        |b| {
            b.iter(|| {
                {
                    prover_run(&m31_config, &mut m31_circuit);
                    black_box(())
                };
            })
        },
    );

    group.bench_function(
        BenchmarkId::new(
            format!(
                "Over BN254, with {} poseidon instances per proof",
                num_poseidon_bn254
            ),
            0,
        ),
        |b| {
            b.iter(|| {
                {
                    prover_run(&bn254_config, &mut bn254_circuit);
                    black_box(())
                };
            })
        },
    );
}

criterion_group!(benches, criterion_gkr_keccak, criterion_gkr_poseidon);
criterion_main!(benches);
