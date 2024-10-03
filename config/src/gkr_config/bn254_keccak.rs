use halo2curves::bn256::Fr;
use transcript::Keccak256hasher;

use super::{FieldType, GKRConfig};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BN254ConfigKeccak;

impl GKRConfig for BN254ConfigKeccak {
    type CircuitField = Fr;

    type ChallengeField = Fr;

    type Field = Fr;

    type SimdCircuitField = Fr;

    type FiatShamirHashType = Keccak256hasher;

    const FIELD_TYPE: FieldType = FieldType::BN254;

    #[inline(always)]
    fn challenge_mul_circuit_field(
        a: &Self::ChallengeField,
        b: &Self::CircuitField,
    ) -> Self::ChallengeField {
        a * b
    }

    #[inline(always)]
    fn field_mul_circuit_field(a: &Self::Field, b: &Self::CircuitField) -> Self::Field {
        a * b
    }

    #[inline(always)]
    fn field_add_circuit_field(a: &Self::Field, b: &Self::CircuitField) -> Self::Field {
        *a + *b
    }

    #[inline(always)]
    fn field_add_simd_circuit_field(a: &Self::Field, b: &Self::SimdCircuitField) -> Self::Field {
        a + b
    }

    #[inline(always)]
    fn field_mul_simd_circuit_field(a: &Self::Field, b: &Self::SimdCircuitField) -> Self::Field {
        a * b
    }

    #[inline(always)]
    fn challenge_mul_field(a: &Self::ChallengeField, b: &Self::Field) -> Self::Field {
        a * b
    }

    #[inline(always)]
    fn circuit_field_into_field(a: &Self::CircuitField) -> Self::Field {
        *a
    }

    #[inline(always)]
    fn circuit_field_mul_simd_circuit_field(
        a: &Self::CircuitField,
        b: &Self::SimdCircuitField,
    ) -> Self::SimdCircuitField {
        *a * *b
    }

    #[inline(always)]
    fn circuit_field_to_simd_circuit_field(a: &Self::CircuitField) -> Self::SimdCircuitField {
        *a
    }
    #[inline(always)]
    fn simd_circuit_field_into_field(a: &Self::SimdCircuitField) -> Self::Field {
        *a
    }

    #[inline(always)]
    fn simd_circuit_field_mul_challenge_field(
        a: &Self::SimdCircuitField,
        b: &Self::ChallengeField,
    ) -> Self::Field {
        *a * b
    }
}
