use arith::{
    BinomialExtensionField, Field, FieldSerde, M31Ext3, M31Ext3x16, M31x16, SimdField, M31,
};
use halo2curves::bn256::Fr;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum PolynomialCommitmentType {
    #[default]
    Raw,
    KZG,
    Orion,
    FRI,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    M31,
    BabyBear,
    BN254,
}

pub const SENTINEL_M31: [u8; 32] = [
    255, 255, 255, 127, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0,
];

pub const SENTINEL_BN254: [u8; 32] = [
    1, 0, 0, 240, 147, 245, 225, 67, 145, 112, 185, 121, 72, 232, 51, 40, 93, 88, 129, 129, 182,
    69, 80, 184, 41, 160, 49, 225, 114, 78, 100, 48,
];

#[derive(Debug, Clone, PartialEq, Default)]
pub enum GKRScheme {
    #[default]
    Vanilla,
    GkrSquare,
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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Config<C: GKRConfig> {
    // Field size for the variables
    pub field_size: usize,
    // Targeted security level for the scheme
    pub security_bits: usize,
    #[cfg(feature = "grinding")]
    // Grinding bits to achieve the target security level
    pub grinding_bits: usize,
    // Polynomial commitment scheme
    pub polynomial_commitment_type: PolynomialCommitmentType,
    // Enum type for the field
    pub fs_hash: FiatShamirHashType,
    // Field configuration for GKR
    pub gkr_config: C,
    // Whether to use GKR^2
    pub gkr_scheme: GKRScheme,
}

impl Config<M31ExtConfig> {
    pub fn new(gkr_scheme: GKRScheme) -> Self {
        Config {
            field_size: 93,
            security_bits: 100,
            #[cfg(feature = "grinding")]
            grinding_bits: 10,
            polynomial_commitment_type: PolynomialCommitmentType::Raw,
            fs_hash: FiatShamirHashType::SHA256,
            gkr_config: M31ExtConfig,
            gkr_scheme,
        }
    }
}

impl Config<BN254Config> {
    pub fn new(gkr_scheme: GKRScheme) -> Self {
        Config {
            field_size: 93,
            security_bits: 100,
            #[cfg(feature = "grinding")]
            grinding_bits: 10,
            polynomial_commitment_type: PolynomialCommitmentType::Raw,
            fs_hash: FiatShamirHashType::SHA256,
            gkr_config: BN254Config,
            gkr_scheme,
        }
    }
}

pub trait GKRConfig: Default + Clone + Send + Sync + 'static {
    /// Field type for the circuit, e.g., M31
    type CircuitField: Field + FieldSerde + Send;

    /// Field type for the challenge, e.g., M31Ext3
    type ChallengeField: BinomialExtensionField<BaseField = Self::CircuitField> + Send;

    /// Main field type for the scheme, e.g., M31Ext3x16
    type Field: BinomialExtensionField + SimdField<Scalar = Self::ChallengeField> + Send;

    /// Simd field for circuit
    type SimdCircuitField: SimdField + FieldSerde + Send;

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

    fn circuit_field_into_field(a: &Self::SimdCircuitField) -> Self::Field;

    fn circuit_field_mul_simd_circuit_field(
        a: &Self::CircuitField,
        b: &Self::SimdCircuitField,
    ) -> Self::SimdCircuitField;

    fn circuit_field_to_simd_circuit_field(a: &Self::CircuitField) -> Self::SimdCircuitField;

    fn simd_circuit_field_from_circuit_field_array(a: &[Self::CircuitField]) -> Self::SimdCircuitField;

    fn simd_circuit_field_into_field(a: &Self::SimdCircuitField) -> Self::Field;

    fn simd_circuit_field_mul_challenge_field(
        a: &Self::SimdCircuitField,
        b: &Self::ChallengeField,
    ) -> Self::Field;
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct M31ExtConfig;

impl GKRConfig for M31ExtConfig {
    type CircuitField = M31;

    type SimdCircuitField = M31x16;

    type ChallengeField = M31Ext3;

    type Field = M31Ext3x16;

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
    fn simd_circuit_field_from_circuit_field_array(a: &[Self::CircuitField]) -> Self::SimdCircuitField {
        Self::SimdCircuitField::from_scalar_array(a)
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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BN254Config;

impl GKRConfig for BN254Config {
    type CircuitField = Fr;

    type ChallengeField = Fr;

    type Field = Fr;

    type SimdCircuitField = Fr;

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
    fn simd_circuit_field_from_circuit_field_array(a: &[Self::CircuitField]) -> Self::SimdCircuitField {
        a[0]
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
