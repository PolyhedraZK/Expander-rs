use std::fs;
use expander_rs::{Circuit, GKRConfig, M31ExtConfig};
use arith::{Field, SimdField, FieldSerde};
use halo2curves::pasta::pallas::Scalar;
use std::io::Cursor;

/// READ Sequential witness and pack them into simd
fn test_helper<C: GKRConfig>() {
    const CIRCUIT_NAME: &str = "data/circuit.txt";
    let mut circuit = Circuit::<C>::load_circuit(CIRCUIT_NAME);

    let input_size = 1usize << circuit.log_input_size();
    let simd_size = <M31ExtConfig as GKRConfig>::SimdCircuitField::SIMD_SIZE;
    let nb_field_elements = input_size * simd_size;

    const WITNESS_FILE: &str = "data/witness.txt";
    let witness_bytes = fs::read(WITNESS_FILE).unwrap(); // does this use buffer?

    assert!(witness_bytes.len() == nb_field_elements * 256 / 8); // we seem to be using 256 bits for a field anyway
    
    let mut cursor = Cursor::new(witness_bytes);
    let mut witness_field: Vec<<C::SimdCircuitField as SimdField>::Scalar> = vec![];
    for _ in 0..nb_field_elements {
        witness_field.push(<C::SimdCircuitField as SimdField>::Scalar::try_deserialize_from_ecc_format(&mut cursor).unwrap());
    }

    let mut input = &mut circuit.layers[0].input_vals;
    input.clear();

    for i in 0..input_size {
        let mut input_i: Vec<<C::SimdCircuitField as SimdField>::Scalar> = vec![];
        for j in 0..simd_size {
            input_i.push(witness_field[j * input_size + i])
        }
        input.push(C::SimdCircuitField::from_scalar_array(&input_i));
    }

}