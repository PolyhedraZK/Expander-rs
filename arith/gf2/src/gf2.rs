// Galios field over 2^128
// credit to intel for the original implementation
// https://www.intel.com/content/dam/develop/external/us/en/documents/clmul-wp-rev-2-02-2014-04-20.pdf

use std::io::Read;
use std::iter::{Product, Sum};
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use arith::{field_common, FieldSerde, FieldSerdeResult};
use arith::{Field, FieldForECC};
use ark_std::iterable::Iterable;

pub const MOD: u32 = 2;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GF2 {
    pub v: u8,
}

field_common!(GF2);

impl FieldSerde for GF2 {
    const SERIALIZED_SIZE: usize = 1;

    #[inline(always)]
    fn serialize_into<W: std::io::Write>(&self, mut writer: W) -> FieldSerdeResult<()> {
        writer.write_all(self.v.to_le_bytes().as_ref())?;
        Ok(())
    }

    #[inline(always)]
    fn deserialize_from<R: std::io::Read>(mut reader: R) -> FieldSerdeResult<Self> {
        let mut u = [0u8; Self::SERIALIZED_SIZE];
        reader.read_exact(&mut u)?;
        Ok(GF2 { v: u[0] % 2 })
    }

    #[inline(always)]
    fn try_deserialize_from_ecc_format<R: Read>(mut reader: R) -> FieldSerdeResult<Self> {
        let mut u = [0u8; 32];
        reader.read_exact(&mut u)?;

        // FIXME:
        // assert!(u.iter().skip(1).all(|x| x == 0u8));
        assert!(u.iter().skip(4).all(|x| x == 0u8));

        Ok(GF2 { v: u[0] % 2 })
    }
}

impl Field for GF2 {
    // still will pack 8 bits into a u8

    const NAME: &'static str = "Galios Field 2";

    const SIZE: usize = 1;

    const FIELD_SIZE: usize = 1; // in bits

    const ZERO: Self = GF2 { v: 0 };

    const ONE: Self = GF2 { v: 1 };

    const INV_2: Self = GF2 { v: 0 }; // should not be used

    #[inline(always)]
    fn zero() -> Self {
        GF2 { v: 0 }
    }

    #[inline(always)]
    fn one() -> Self {
        GF2 { v: 1 }
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        self.v == 0
    }

    #[inline(always)]
    fn random_unsafe(mut rng: impl rand::RngCore) -> Self {
        GF2 {
            v: (rng.next_u32() % 2) as u8,
        }
    }

    #[inline(always)]
    fn random_bool(mut rng: impl rand::RngCore) -> Self {
        GF2 {
            v: (rng.next_u32() % 2) as u8,
        }
    }

    #[inline(always)]
    fn exp(&self, exponent: u128) -> Self {
        if exponent % 2 == 0 {
            Self::one()
        } else {
            *self
        }
    }

    #[inline(always)]
    fn inv(&self) -> Option<Self> {
        if self.v == 0 {
            None
        } else {
            Some(Self::one())
        }
    }

    #[inline(always)]
    fn as_u32_unchecked(&self) -> u32 {
        self.v as u32 % 2
    }

    #[inline(always)]
    fn from_uniform_bytes(bytes: &[u8; 32]) -> Self {
        GF2 { v: bytes[0] % 2 }
    }

    #[inline(always)]
    fn mul_by_5(&self) -> Self {
        *self
    }

    #[inline(always)]
    fn mul_by_6(&self) -> Self {
        Self::ZERO
    }
}

impl FieldForECC for GF2 {
    fn modulus() -> ethnum::U256 {
        ethnum::U256::from(MOD)
    }
    fn from_u256(x: ethnum::U256) -> Self {
        GF2 {
            v: (x.as_u32() & 1) as u8,
        }
    }
    fn to_u256(&self) -> ethnum::U256 {
        ethnum::U256::from(self.v as u32)
    }
}

impl Neg for GF2 {
    type Output = GF2;

    #[inline(always)]
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn neg(self) -> GF2 {
        GF2 { v: self.v }
    }
}

impl From<u32> for GF2 {
    #[inline(always)]
    fn from(v: u32) -> Self {
        GF2 { v: (v % 2) as u8 }
    }
}

impl From<bool> for GF2 {
    #[inline(always)]
    fn from(value: bool) -> Self {
        GF2 { v: value.into() }
    }
}

#[inline(always)]
fn add_internal(a: &GF2, b: &GF2) -> GF2 {
    GF2 { v: a.v ^ b.v }
}

#[inline(always)]
fn sub_internal(a: &GF2, b: &GF2) -> GF2 {
    GF2 { v: a.v ^ b.v }
}

#[inline(always)]
fn mul_internal(a: &GF2, b: &GF2) -> GF2 {
    GF2 { v: a.v & b.v }
}
