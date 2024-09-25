use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use arith::{Field, FieldSerde, FieldSerdeResult, SimdField};

use super::GF2;

/// A GF2x8 stores 8 bits of data.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct GF2x8 {
    pub v: u8,
}

impl FieldSerde for GF2x8 {
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
        Ok(GF2x8 { v: u[0] })
    }

    #[inline]
    fn try_deserialize_from_ecc_format<R: std::io::Read>(mut _reader: R) -> FieldSerdeResult<Self> {
        unimplemented!("We don't have serialization in ecc for gf2x8")
        // let mut u = [0u8; 32];
        // reader.read_exact(&mut u)?;
        // Ok(GF2x8 { v: u[0] })
    }
}

impl Field for GF2x8 {
    // still will pack 8 bits into a u8

    const NAME: &'static str = "Galios Field 2 SIMD";

    const SIZE: usize = 1;

    const FIELD_SIZE: usize = 1; // in bits

    const ZERO: Self = GF2x8 { v: 0 };

    const ONE: Self = GF2x8 { v: 255 };

    const INV_2: Self = GF2x8 { v: 0 }; // should not be used

    #[inline(always)]
    fn zero() -> Self {
        GF2x8 { v: 0 }
    }

    #[inline(always)]
    fn one() -> Self {
        GF2x8 { v: 255 }
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        self.v == 0
    }

    #[inline(always)]
    fn random_unsafe(mut rng: impl rand::RngCore) -> Self {
        GF2x8 {
            v: (rng.next_u32() % 256) as u8,
        }
    }

    #[inline(always)]
    fn random_bool(mut rng: impl rand::RngCore) -> Self {
        GF2x8 {
            v: (rng.next_u32() % 256) as u8,
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
        unimplemented!()
    }

    #[inline(always)]
    fn as_u32_unchecked(&self) -> u32 {
        self.v as u32 % 256
    }

    #[inline(always)]
    fn from_uniform_bytes(bytes: &[u8; 32]) -> Self {
        GF2x8 { v: bytes[0] }
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

impl Mul<&GF2x8> for GF2x8 {
    type Output = GF2x8;

    #[inline(always)]
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, rhs: &GF2x8) -> GF2x8 {
        GF2x8 { v: self.v & rhs.v }
    }
}

impl Mul<GF2x8> for GF2x8 {
    type Output = GF2x8;

    #[inline(always)]
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, rhs: GF2x8) -> GF2x8 {
        GF2x8 { v: self.v & rhs.v }
    }
}

impl MulAssign<&GF2x8> for GF2x8 {
    #[inline(always)]
    #[allow(clippy::suspicious_op_assign_impl)]
    fn mul_assign(&mut self, rhs: &GF2x8) {
        self.v &= rhs.v;
    }
}

impl MulAssign<GF2x8> for GF2x8 {
    #[inline(always)]
    #[allow(clippy::suspicious_op_assign_impl)]
    fn mul_assign(&mut self, rhs: GF2x8) {
        self.v &= rhs.v;
    }
}

impl Sub for GF2x8 {
    type Output = GF2x8;

    #[inline(always)]
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn sub(self, rhs: GF2x8) -> GF2x8 {
        GF2x8 { v: self.v ^ rhs.v }
    }
}

impl SubAssign for GF2x8 {
    #[inline(always)]
    #[allow(clippy::suspicious_op_assign_impl)]
    fn sub_assign(&mut self, rhs: GF2x8) {
        self.v ^= rhs.v;
    }
}

impl Add for GF2x8 {
    type Output = GF2x8;

    #[inline(always)]
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: GF2x8) -> GF2x8 {
        GF2x8 { v: self.v ^ rhs.v }
    }
}

impl AddAssign for GF2x8 {
    #[inline(always)]
    #[allow(clippy::suspicious_op_assign_impl)]
    fn add_assign(&mut self, rhs: GF2x8) {
        self.v ^= rhs.v;
    }
}

impl Add<&GF2x8> for GF2x8 {
    type Output = GF2x8;

    #[inline(always)]
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: &GF2x8) -> GF2x8 {
        GF2x8 { v: self.v ^ rhs.v }
    }
}

impl Sub<&GF2x8> for GF2x8 {
    type Output = GF2x8;

    #[inline(always)]
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn sub(self, rhs: &GF2x8) -> GF2x8 {
        GF2x8 { v: self.v ^ rhs.v }
    }
}

impl<T: std::borrow::Borrow<GF2x8>> std::iter::Sum<T> for GF2x8 {
    fn sum<I: Iterator<Item = T>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, item| acc + item.borrow())
    }
}

impl<T: std::borrow::Borrow<GF2x8>> std::iter::Product<T> for GF2x8 {
    fn product<I: Iterator<Item = T>>(iter: I) -> Self {
        iter.fold(Self::one(), |acc, item| acc * item.borrow())
    }
}

impl Neg for GF2x8 {
    type Output = GF2x8;

    #[inline(always)]
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn neg(self) -> GF2x8 {
        GF2x8 { v: self.v }
    }
}

impl AddAssign<&GF2x8> for GF2x8 {
    #[inline(always)]
    #[allow(clippy::suspicious_op_assign_impl)]
    fn add_assign(&mut self, rhs: &GF2x8) {
        self.v ^= rhs.v;
    }
}

impl SubAssign<&GF2x8> for GF2x8 {
    #[inline(always)]
    #[allow(clippy::suspicious_op_assign_impl)]
    fn sub_assign(&mut self, rhs: &GF2x8) {
        self.v ^= rhs.v;
    }
}

impl From<u32> for GF2x8 {
    #[inline(always)]
    fn from(v: u32) -> Self {
        assert!(v < 2);
        if v == 0 {
            GF2x8 { v: 0 }
        } else {
            GF2x8 { v: 0xFF }
        }
    }
}

impl From<GF2> for GF2x8 {
    #[inline(always)]
    fn from(v: GF2) -> Self {
        assert!(v.v < 2);
        if v.v == 0 {
            GF2x8 { v: 0 }
        } else {
            GF2x8 { v: 0xFF }
        }
    }
}

impl SimdField for GF2x8 {
    #[inline(always)]
    fn scale(&self, challenge: &Self::Scalar) -> Self {
        if challenge.v == 0 {
            Self::zero()
        } else {
            *self
        }
    }

    #[inline(always)]
    fn pack_size() -> usize {
        8
    }

    #[inline(always)]
    fn pack(base_vec: &[Self::Scalar]) -> Self {
        assert!(base_vec.len() == Self::pack_size());
        let mut ret = 0u8;
        for (i, scalar) in base_vec.iter().enumerate() {
            ret |= scalar.v << (7 - i);
        }
        Self { v: ret }
    }

    #[inline(always)]
    fn unpack(&self) -> Vec<Self::Scalar> {
        let mut ret = vec![];
        for i in 0..Self::pack_size() {
            ret.push(Self::Scalar {
                v: (self.v >> (7 - i)) & 1u8,
            });
        }
        ret
    }

    type Scalar = crate::GF2;
}
