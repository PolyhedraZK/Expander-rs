use arith::ExtensionField;
use mersenne31::{M31Ext3, M31Ext3x16, M31x16, M31};

use crate::Keccak256hasher;

use super::{FieldType, GKRConfig};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct M31ExtConfigKeccak;

impl GKRConfig for M31ExtConfigKeccak {
    type CircuitField = M31;

    type SimdCircuitField = M31x16;

    type ChallengeField = M31Ext3;

    type Field = M31Ext3x16;

    type FiatShamirHashType = Keccak256hasher;

    const FIELD_TYPE: FieldType = FieldType::M31;

    #[inline(always)]
    fn challenge_mul_circuit_field(
        a: &Self::ChallengeField,
        b: &Self::CircuitField,
    ) -> Self::ChallengeField {
        a.mul_by_base_field(b)
    }

    #[inline(always)]
    fn field_mul_circuit_field(a: &Self::Field, b: &Self::CircuitField) -> Self::Field {
        // directly multiply M31Ext3 with M31
        // skipping the conversion M31 -> M31Ext3
        *a * *b
    }

    #[inline(always)]
    fn field_add_circuit_field(a: &Self::Field, b: &Self::CircuitField) -> Self::Field {
        // directly add M31Ext3 with M31
        // skipping the conversion M31 -> M31Ext3
        *a + *b
    }

    #[inline(always)]
    fn field_add_simd_circuit_field(a: &Self::Field, b: &Self::SimdCircuitField) -> Self::Field {
        a.add_by_base_field(b)
    }

    #[inline(always)]
    fn field_mul_simd_circuit_field(a: &Self::Field, b: &Self::SimdCircuitField) -> Self::Field {
        a.mul_by_base_field(b)
    }

    #[inline(always)]
    fn challenge_mul_field(a: &Self::ChallengeField, b: &Self::Field) -> Self::Field {
        let a_simd = Self::Field::from(*a);
        a_simd * b
    }

    #[inline(always)]
    fn circuit_field_into_field(a: &Self::SimdCircuitField) -> Self::Field {
        Self::Field::from(*a)
    }

    #[inline(always)]
    fn circuit_field_mul_simd_circuit_field(
        a: &Self::CircuitField,
        b: &Self::SimdCircuitField,
    ) -> Self::SimdCircuitField {
        Self::SimdCircuitField::from(*a) * *b
    }
    #[inline(always)]
    fn circuit_field_to_simd_circuit_field(a: &Self::CircuitField) -> Self::SimdCircuitField {
        Self::SimdCircuitField::from(*a)
    }

    #[inline(always)]
    fn simd_circuit_field_into_field(a: &Self::SimdCircuitField) -> Self::Field {
        Self::Field::from(*a)
    }

    #[inline(always)]
    fn simd_circuit_field_mul_challenge_field(
        a: &Self::SimdCircuitField,
        b: &Self::ChallengeField,
    ) -> Self::Field {
        let b_simd_ext = Self::Field::from(*b);
        Self::Field {
            v: [
                b_simd_ext.v[0] * a,
                b_simd_ext.v[1] * a,
                b_simd_ext.v[2] * a,
            ],
        }
    }
}
