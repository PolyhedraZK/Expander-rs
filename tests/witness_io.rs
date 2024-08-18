use std::fs;

use expander_rs::{
    BN254Config, Circuit, CircuitLayer, Config, GKRConfig, GKRScheme, GateAdd, GateMul,
    M31ExtConfig, Prover, Verifier,
};

fn test_helper<C: GKRConfig>() {
    const HASH_INPUT_SIZE: usize = 512;
    const NB_HASHES_IN_CIRCUIT: usize = 8;
    const SIMD_SIZE: usize = 512 / 32;

    const INPUT_FILE: &str = "hash_input.bin";

    let file_bytes = fs::read(INPUT_FILE).unwrap();
    assert_eq!(HASH_INPUT_SIZE * NB_HASHES_IN_CIRCUIT * SIMD_SIZE, file_bytes.len() * 8);

    let input = file_bytes
                     .iter()
                     .map(|byte| {
                        (0..8).map(|i| (byte >> i) & 1).collect::<Vec<u8>>()
                     })
                     .flatten()
                     .collect::<Vec<u8>>();
}

#[test]
fn test() {
    test_helper::<M31ExtConfig>()
}