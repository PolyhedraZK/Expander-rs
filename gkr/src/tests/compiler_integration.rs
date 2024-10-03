// use std::fs;

// use arith::{Field, M31x16};
// use expander_rs::utils::*;
// use expander_rs::{Circuit, Config, GKRScheme, M31ExtConfigSha2, Prover, Verifier};
// use rand::Rng;

// const FILENAME_PROOF: &str = "data/proof.bin";

// #[test]
// fn test_compiler_format_integration() {
//     let config = Config::<M31ExtConfigSha2>::new(GKRScheme::Vanilla);

//     let mut circuit = Circuit::<M31ExtConfigSha2>::load_circuit(KECCAK_M31_CIRCUIT);
//     println!("Circuit loaded.");
//     circuit.load_witness_file(KECCAK_WITNESS);
//     println!("Witness loaded.");
//     circuit.evaluate();
//     println!("Circuit evaluated.");

//     // check last layer first output
//     let last_layer = circuit.layers.last().unwrap();
//     let last_layer_first_output = last_layer.output_vals[0];
//     assert_eq!(last_layer_first_output, M31x16::zero());

//     let mut prover = Prover::new(&config);
//     prover.prepare_mem(&circuit);
//     let (claimed_v, proof) = prover.prove(&mut circuit);
//     println!("Proof generated. Size: {} bytes", proof.bytes.len());
//     // write proof to file
//     fs::write(FILENAME_PROOF, &proof.bytes).expect("Unable to write proof to file.");

//     let verifier = Verifier::new(&config);
//     println!("Verifier created.");
//     assert!(verifier.verify(&mut circuit, &claimed_v, &proof));
//     println!("Correct proof verified.");
//     let mut bad_proof = proof.clone();
//     let rng = &mut rand::thread_rng();
//     let random_idx = rng.gen_range(0..bad_proof.bytes.len());
//     let random_change = rng.gen_range(1..256) as u8;
//     bad_proof.bytes[random_idx] ^= random_change;
//     assert!(!verifier.verify(&mut circuit, &claimed_v, &bad_proof));
//     println!("Bad proof rejected.");
// }

// #[test]
// fn test_compiler_format_integration_no_prove() {
//     println!("Config created.");
//     let mut circuit = Circuit::<M31ExtConfigSha2>::load_circuit(KECCAK_M31_CIRCUIT);
//     println!("Circuit loaded.");
//     circuit.load_witness_file(KECCAK_WITNESS);
//     println!("Witness loaded.");
//     circuit.evaluate();
//     println!("Circuit evaluated.");
//     // check last layer first output
//     let last_layer = circuit.layers.last().unwrap();
//     let last_layer_first_output = last_layer.output_vals[0];
//     assert_eq!(last_layer_first_output, M31x16::zero());
// }
