use arith::ExtensionField;
use babybear::{
    BabyBear, BabyBearExt3, BabyBearExt3x16, BabyBearExt4, BabyBearExt4x16, BabyBearx16,
};

use super::{FieldType, GKRConfig};
use crate::SHA256hasher;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BabyBearExt4ConfigSha2;

impl GKRConfig for BabyBearExt4ConfigSha2 {
    type CircuitField = BabyBear;

    type ChallengeField = BabyBearExt4;

    type Field = BabyBearExt4x16;

    type SimdCircuitField = BabyBearx16;

    type FiatShamirHashType = SHA256hasher;

    const FIELD_TYPE: FieldType = FieldType::BabyBear;

    fn challenge_mul_circuit_field(
        a: &Self::ChallengeField,
        b: &Self::CircuitField,
    ) -> Self::ChallengeField {
        a.mul_by_base_field(b)
    }

    fn field_mul_circuit_field(a: &Self::Field, b: &Self::CircuitField) -> Self::Field {
        *a * *b
    }

    fn field_add_circuit_field(a: &Self::Field, b: &Self::CircuitField) -> Self::Field {
        *a + *b
    }

    fn field_add_simd_circuit_field(a: &Self::Field, b: &Self::SimdCircuitField) -> Self::Field {
        a.add_by_base_field(b)
    }

    fn field_mul_simd_circuit_field(a: &Self::Field, b: &Self::SimdCircuitField) -> Self::Field {
        a.mul_by_base_field(b)
    }

    fn challenge_mul_field(a: &Self::ChallengeField, b: &Self::Field) -> Self::Field {
        let a_simd = Self::Field::from(*a);
        a_simd * b
    }

    fn circuit_field_into_field(a: &Self::SimdCircuitField) -> Self::Field {
        Self::Field::from(*a)
    }

    fn circuit_field_mul_simd_circuit_field(
        a: &Self::CircuitField,
        b: &Self::SimdCircuitField,
    ) -> Self::SimdCircuitField {
        Self::SimdCircuitField::from(*a) * *b
    }

    fn circuit_field_to_simd_circuit_field(a: &Self::CircuitField) -> Self::SimdCircuitField {
        Self::SimdCircuitField::from(*a)
    }

    fn simd_circuit_field_into_field(a: &Self::SimdCircuitField) -> Self::Field {
        Self::Field::from(*a)
    }

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
                b_simd_ext.v[3] * a,
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BabyBearExt3ConfigSha2;

impl GKRConfig for BabyBearExt3ConfigSha2 {
    type CircuitField = BabyBear;

    type ChallengeField = BabyBearExt3;

    type Field = BabyBearExt3x16;

    type SimdCircuitField = BabyBearx16;

    type FiatShamirHashType = SHA256hasher;

    const FIELD_TYPE: FieldType = FieldType::BabyBear;

    fn challenge_mul_circuit_field(
        a: &Self::ChallengeField,
        b: &Self::CircuitField,
    ) -> Self::ChallengeField {
        a.mul_by_base_field(b)
    }

    fn field_mul_circuit_field(a: &Self::Field, b: &Self::CircuitField) -> Self::Field {
        *a * *b
    }

    fn field_add_circuit_field(a: &Self::Field, b: &Self::CircuitField) -> Self::Field {
        *a + *b
    }

    fn field_add_simd_circuit_field(a: &Self::Field, b: &Self::SimdCircuitField) -> Self::Field {
        a.add_by_base_field(b)
    }

    fn field_mul_simd_circuit_field(a: &Self::Field, b: &Self::SimdCircuitField) -> Self::Field {
        a.mul_by_base_field(b)
    }

    fn challenge_mul_field(a: &Self::ChallengeField, b: &Self::Field) -> Self::Field {
        let a_simd = Self::Field::from(*a);
        a_simd * b
    }

    fn circuit_field_into_field(a: &Self::SimdCircuitField) -> Self::Field {
        Self::Field::from(*a)
    }

    fn circuit_field_mul_simd_circuit_field(
        a: &Self::CircuitField,
        b: &Self::SimdCircuitField,
    ) -> Self::SimdCircuitField {
        Self::SimdCircuitField::from(*a) * *b
    }

    fn circuit_field_to_simd_circuit_field(a: &Self::CircuitField) -> Self::SimdCircuitField {
        Self::SimdCircuitField::from(*a)
    }

    fn simd_circuit_field_into_field(a: &Self::SimdCircuitField) -> Self::Field {
        Self::Field::from(*a)
    }

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
