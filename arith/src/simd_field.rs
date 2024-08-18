use crate::{Field, FieldSerde};

/// Configurations for the SimdField.
pub trait SimdField: From<Self::Scalar> + Field + FieldSerde {
    /// Field for the challenge. Can be self.
    type Scalar: Field + FieldSerde + Send;

    /// The number of scalars in a single SimdField
    const SIMD_SIZE: usize;

    /// Pack an array of scalars into the field
    fn from_scalar_array(scalars: &[Self::Scalar]) -> Self;

    /// scale self with the challenge
    fn scale(&self, challenge: &Self::Scalar) -> Self;
}
