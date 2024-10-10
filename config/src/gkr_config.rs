mod bn254_keccak;
mod bn254_sha2;
mod gf2_ext_keccak;
mod gf2_ext_sha2;
mod m31_ext_keccak;
mod m31_ext_sha2;

use arith::{ExtensionField, Field, FieldForECC, FieldSerde, SimdField};
use ark_std::{end_timer, start_timer};
use transcript::FiatShamirHash;

pub use bn254_keccak::BN254ConfigKeccak;
pub use bn254_sha2::BN254ConfigSha2;
pub use gf2_ext_keccak::GF2ExtConfigKeccak;
pub use gf2_ext_sha2::GF2ExtConfigSha2;
pub use m31_ext_keccak::M31ExtConfigKeccak;
pub use m31_ext_sha2::M31ExtConfigSha2;

#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    M31,
    BN254,
    GF2,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum FiatShamirHashType {
    #[default]
    SHA256,
    Keccak256,
    Poseidon,
    Animoe,
    MIMC7,
}

pub trait GKRConfig: Default + Clone + Send + Sync + 'static {
    /// Field type for the circuit, e.g., M31
    type CircuitField: Field + FieldSerde + FieldForECC + Send;

    /// Field type for the challenge, e.g., M31Ext3
    type ChallengeField: ExtensionField<BaseField = Self::CircuitField> + Send;

    /// Main field type for the scheme, e.g., M31Ext3x16
    type Field: ExtensionField<BaseField = Self::SimdCircuitField>
        + SimdField<Scalar = Self::ChallengeField>
        + Send;

    /// Simd field for circuit, e.g., M31x16
    type SimdCircuitField: SimdField<Scalar = Self::CircuitField> + FieldSerde + Send;

    /// Fiat Shamir hash type
    type FiatShamirHashType: FiatShamirHash;

    /// Enum type for Self::Field
    const FIELD_TYPE: FieldType;

    /// API to allow for multiplications between the challenge and the circuit field
    fn challenge_mul_circuit_field(
        a: &Self::ChallengeField,
        b: &Self::CircuitField,
    ) -> Self::ChallengeField;

    /// API to allow for multiplications between the main field and the circuit field
    fn field_mul_circuit_field(a: &Self::Field, b: &Self::CircuitField) -> Self::Field;

    /// API to allow for addition between the main field and the circuit field
    fn field_add_circuit_field(a: &Self::Field, b: &Self::CircuitField) -> Self::Field;

    /// API to allow multiplications between the main field and the simd circuit field
    fn field_add_simd_circuit_field(a: &Self::Field, b: &Self::SimdCircuitField) -> Self::Field;

    /// API to allow multiplications between the main field and the simd circuit field
    fn field_mul_simd_circuit_field(a: &Self::Field, b: &Self::SimdCircuitField) -> Self::Field;

    /// API to allow for multiplications between the challenge and the main field
    fn challenge_mul_field(a: &Self::ChallengeField, b: &Self::Field) -> Self::Field;

    /// API to allow for multiplications between the challenge and the simd circuit field
    fn circuit_field_into_field(a: &Self::SimdCircuitField) -> Self::Field;

    /// API to allow for multiplications between the simd circuit field and the challenge
    fn circuit_field_mul_simd_circuit_field(
        a: &Self::CircuitField,
        b: &Self::SimdCircuitField,
    ) -> Self::SimdCircuitField;

    /// Convert a circuit field to a simd circuit field
    fn circuit_field_to_simd_circuit_field(a: &Self::CircuitField) -> Self::SimdCircuitField;

    /// Convert a simd circuit field to a circuit field
    fn simd_circuit_field_into_field(a: &Self::SimdCircuitField) -> Self::Field;

    /// API to allow for multiplications between the simd circuit field and the challenge
    fn simd_circuit_field_mul_challenge_field(
        a: &Self::SimdCircuitField,
        b: &Self::ChallengeField,
    ) -> Self::Field;

    /// The pack size for the simd circuit field, e.g., 16 for M31x16
    fn get_field_pack_size() -> usize {
        Self::SimdCircuitField::pack_size()
    }

    /// Evaluate the circuit values at the challenge
    #[inline]
    fn eval_circuit_vals_at_challenge(
        evals: &[Self::SimdCircuitField],
        x: &[Self::ChallengeField],
        scratch: &mut [Self::Field],
    ) -> Self::Field {
        let timer = start_timer!(|| format!("eval mle with {} vars", x.len()));
        assert_eq!(1 << x.len(), evals.len());

        let ret = if x.is_empty() {
            Self::simd_circuit_field_into_field(&evals[0])
        } else {
            for i in 0..(evals.len() >> 1) {
                scratch[i] = Self::field_add_simd_circuit_field(
                    &Self::simd_circuit_field_mul_challenge_field(
                        &(evals[i * 2 + 1] - evals[i * 2]),
                        &x[0],
                    ),
                    &evals[i * 2],
                );
            }

            let mut cur_eval_size = evals.len() >> 2;
            for r in x.iter().skip(1) {
                for i in 0..cur_eval_size {
                    scratch[i] = scratch[i * 2] + (scratch[i * 2 + 1] - scratch[i * 2]).scale(r);
                }
                cur_eval_size >>= 1;
            }
            scratch[0]
        };
        end_timer!(timer);

        ret
    }
}
