use std::io::{Read, Write};

use halo2curves::ff::{Field as Halo2Field, FromUniformBytes};
use halo2curves::{bn256::Fr, ff::PrimeField};
use rand::RngCore;

use crate::{Field, FieldSerde, SimdField};

impl Field for Fr {
    /// name
    const NAME: &'static str = "bn254 scalar field";

    /// size required to store the data
    const SIZE: usize = 32;

    /// zero
    const ZERO: Self = Fr::zero();

    /// Inverse of 2
    const INV_2: Self = Fr::TWO_INV;

    // ====================================
    // constants
    // ====================================
    /// Zero element
    #[inline(always)]
    fn zero() -> Self {
        Fr::zero()
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        *self == Fr::zero()
    }

    /// Identity element
    #[inline(always)]
    fn one() -> Self {
        Fr::one()
    }

    // ====================================
    // generators
    // ====================================
    /// create a random element from rng.
    /// test only -- the output may not be uniformly random.
    #[inline(always)]
    fn random_unsafe(rng: impl RngCore) -> Self {
        Fr::random(rng)
    }

    /// create a random boolean element from rng
    #[inline(always)]
    fn random_bool(mut rng: impl RngCore) -> Self {
        Self::from((rng.next_u32() & 1) as u64)
    }

    // ====================================
    // arithmetics
    // ====================================
    /// Squaring
    #[inline(always)]
    fn square(&self) -> Self {
        *self * *self
    }

    /// Doubling
    #[inline(always)]
    fn double(&self) -> Self {
        *self + *self
    }

    /// Exp
    fn exp(&self, _exponent: &Self) -> Self {
        unimplemented!()
    }

    /// find the inverse of the element; return None if not exist
    #[inline(always)]
    fn inv(&self) -> Option<Self> {
        self.invert().into()
    }

    /// expose the element as u32.
    fn as_u32_unchecked(&self) -> u32 {
        todo!()
    }

    // TODO: better implementation
    fn from_uniform_bytes(bytes: &[u8; 32]) -> Self {
        <Fr as FromUniformBytes<64>>::from_uniform_bytes(
            &[bytes.as_slice(), [0u8; 32].as_slice()]
                .concat()
                .try_into()
                .unwrap(),
        )
    }
}

impl SimdField for Fr {
    type Scalar = Self;

    #[inline(always)]
    fn scale(&self, challenge: &Self::Scalar) -> Self {
        self * challenge
    }
}

impl FieldSerde for Fr {
    #[inline(always)]
    fn serialize_into<W: Write>(&self, mut writer: W) {
        writer.write_all(self.to_bytes().as_ref()).unwrap();
    }

    /// size of the serialized bytes
    #[inline(always)]
    fn serialized_size() -> usize {
        32
    }

    #[inline(always)]
    fn deserialize_from<R: Read>(mut reader: R) -> Self {
        let mut buffer = [0u8; 32];
        reader.read_exact(&mut buffer).unwrap();
        Fr::from_bytes(&buffer).unwrap()
    }

    #[inline(always)]
    fn deserialize_from_ecc_format<R: Read>(reader: R) -> Self {
        Fr::deserialize_from(reader) // same as deserialize_from
    }

    fn try_deserialize_from_ecc_format<R: Read>(
        mut reader: R,
    ) -> std::result::Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let mut buffer = [0u8; 32];
        reader.read_exact(&mut buffer)?;
        Ok(Fr::from_bytes(&buffer).unwrap())
    }
}
