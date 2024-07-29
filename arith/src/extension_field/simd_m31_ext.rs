use std::{
    io::{Read, Write},
    iter::{Product, Sum},
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use crate::{BinomialExtensionField, Field, FieldSerde, M31Ext3, SimdField, SimdM31, M31};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SimdM31Ext3 {
    pub v: [SimdM31; 3],
}

impl FieldSerde for SimdM31Ext3 {
    #[inline(always)]
    fn serialize_into<W: Write>(&self, mut writer: W) {
        self.v[0].serialize_into(&mut writer);
        self.v[1].serialize_into(&mut writer);
        self.v[2].serialize_into(&mut writer);
    }

    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    fn serialized_size() -> usize {
        512 / 8 * 3
    }

    #[cfg(target_arch = "aarch64")]
    #[inline(always)]
    fn serialized_size() -> usize {
        256 / 8 * 3
    }

    // FIXME: this deserialization function auto corrects invalid inputs.
    // We should use separate APIs for this and for the actual deserialization.
    #[inline(always)]
    fn deserialize_from<R: Read>(mut reader: R) -> Self {
        Self {
            v: [
                SimdM31::deserialize_from(&mut reader),
                SimdM31::deserialize_from(&mut reader),
                SimdM31::deserialize_from(&mut reader),
            ],
        }
    }

    #[inline(always)]
    fn deserialize_from_ecc_format<R: Read>(mut reader: R) -> Self {
        Self {
            v: [
                SimdM31::deserialize_from_ecc_format(&mut reader),
                SimdM31::zero(),
                SimdM31::zero(),
            ],
        }
    }

    fn try_deserialize_from_ecc_format<R: Read>(
        mut reader: R,
    ) -> std::result::Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        Ok(Self {
            v: [
                SimdM31::try_deserialize_from_ecc_format(&mut reader)?,
                SimdM31::zero(),
                SimdM31::zero(),
            ],
        })
    }
}

impl SimdField for SimdM31Ext3 {
    type Scalar = M31Ext3;

    #[inline]
    fn scale(&self, challenge: &Self::Scalar) -> Self {
        *self * *challenge
    }
}

impl From<SimdM31> for SimdM31Ext3 {
    #[inline(always)]
    fn from(x: SimdM31) -> Self {
        Self {
            v: [x, SimdM31::zero(), SimdM31::zero()],
        }
    }
}

impl BinomialExtensionField for SimdM31Ext3 {
    const DEGREE: usize = 3;

    const W: u32 = 5;

    type BaseField = SimdM31;

    #[inline(always)]
    fn mul_by_base_field(&self, base: &Self::BaseField) -> Self {
        SimdM31Ext3 {
            v: [self.v[0] * base, self.v[1] * base, self.v[2] * base],
        }
    }

    #[inline(always)]
    fn add_by_base_field(&self, base: &Self::BaseField) -> Self {
        SimdM31Ext3 {
            v: [self.v[0] + base, self.v[1], self.v[2]],
        }
    }

    #[inline(always)]
    fn first_base_field(&self) -> Self::BaseField {
        self.v[0]
    }
}

impl From<M31Ext3> for SimdM31Ext3 {
    #[inline(always)]
    fn from(x: M31Ext3) -> Self {
        Self {
            v: [
                SimdM31::pack_full(x.v[0]),
                SimdM31::pack_full(x.v[1]),
                SimdM31::pack_full(x.v[2]),
            ],
        }
    }
}

impl Field for SimdM31Ext3 {
    #[cfg(target_arch = "x86_64")]
    const NAME: &'static str = "AVX Vectorized Mersenne 31 Extension 3";
    #[cfg(target_arch = "x86_64")]
    const SIZE: usize = 512 / 8 * 3;

    #[cfg(target_arch = "aarch64")]
    const NAME: &'static str = "Neon Vectorized Mersenne 31 Extension 3";
    #[cfg(target_arch = "aarch64")]
    const SIZE: usize = 256 / 8 * 3;

    const ZERO: Self = Self {
        v: [SimdM31::ZERO; 3],
    };

    const INV_2: Self = Self {
        v: [SimdM31::INV_2, SimdM31::ZERO, SimdM31::ZERO],
    };

    #[inline(always)]
    fn zero() -> Self {
        SimdM31Ext3 {
            v: [SimdM31::zero(); 3],
        }
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        self.v[0].is_zero() && self.v[1].is_zero() && self.v[2].is_zero()
    }

    #[inline(always)]
    fn one() -> Self {
        SimdM31Ext3 {
            v: [SimdM31::one(), SimdM31::zero(), SimdM31::zero()],
        }
    }

    #[inline(always)]
    fn random_unsafe(mut rng: impl rand::RngCore) -> Self {
        SimdM31Ext3 {
            v: [
                SimdM31::random_unsafe(&mut rng),
                SimdM31::random_unsafe(&mut rng),
                SimdM31::random_unsafe(&mut rng),
            ],
        }
    }

    #[inline(always)]
    fn random_bool(mut rng: impl rand::RngCore) -> Self {
        SimdM31Ext3 {
            v: [
                SimdM31::random_bool(&mut rng),
                SimdM31::zero(),
                SimdM31::zero(),
            ],
        }
    }

    #[inline(always)]
    fn square(&self) -> Self {
        Self {
            v: square_internal(&self.v),
        }
    }

    fn exp(&self, _exponent: &Self) -> Self {
        unimplemented!()
    }

    fn inv(&self) -> Option<Self> {
        unimplemented!()
    }

    fn as_u32_unchecked(&self) -> u32 {
        unimplemented!("self is a vector, cannot convert to u32")
    }

    fn from_uniform_bytes(_bytes: &[u8; 32]) -> Self {
        unimplemented!("vec m31: cannot convert from 32 bytes")
    }
}

// ====================================
// Arithmetics for M31Ext
// ====================================

impl Mul<&SimdM31Ext3> for SimdM31Ext3 {
    type Output = SimdM31Ext3;
    #[inline(always)]
    fn mul(self, rhs: &SimdM31Ext3) -> Self::Output {
        SimdM31Ext3 {
            v: mul_internal(&self.v, &rhs.v),
        }
    }
}

impl Mul for SimdM31Ext3 {
    type Output = SimdM31Ext3;
    #[inline(always)]
    #[allow(clippy::op_ref)]
    fn mul(self, rhs: SimdM31Ext3) -> Self::Output {
        self * &rhs
    }
}

impl MulAssign<&SimdM31Ext3> for SimdM31Ext3 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: &SimdM31Ext3) {
        *self = *self * rhs;
    }
}

impl MulAssign for SimdM31Ext3 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self *= &rhs;
    }
}

impl Mul<M31Ext3> for SimdM31Ext3 {
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: M31Ext3) -> Self::Output {
        // polynomial mod (x^3 - 5)
        //
        //   (a0 + a1*x + a2*x^2) * (b0 + b1*x + b2*x^2) mod (x^3 - 5)
        // = a0*b0 + (a0*b1 + a1*b0)*x + (a0*b2 + a1*b1 + a2*b0)*x^2
        // + (a1*b2 + a2*b1)*x^3 + a2*b2*x^4 mod (x^3 - 5)
        // = a0*b0 + 5*(a1*b2 + a2*b1)
        // + (a0*b1 + a1*b0)*x + 5* a2*b2
        // + (a0*b2 + a1*b1 + a2*b0)*x^2

        let five = M31::from(5);
        let mut res = [SimdM31::default(); 3];
        res[0] =
            self.v[0] * rhs.v[0] + self.v[1] * (rhs.v[2] * five) + self.v[2] * (rhs.v[1] * five);
        res[1] = self.v[0] * rhs.v[1] + self.v[1] * rhs.v[0] + self.v[2] * (rhs.v[2] * five);
        res[2] = self.v[0] * rhs.v[2] + self.v[1] * rhs.v[1] + self.v[2] * rhs.v[0];
        Self { v: res }
    }
}

impl Mul<M31> for SimdM31Ext3 {
    type Output = SimdM31Ext3;
    #[inline(always)]
    fn mul(self, rhs: M31) -> Self::Output {
        SimdM31Ext3 {
            // SimdM31 * M31
            v: [self.v[0] * rhs, self.v[1] * rhs, self.v[2] * rhs],
        }
    }
}

impl Add<M31> for SimdM31Ext3 {
    type Output = SimdM31Ext3;
    #[inline(always)]
    fn add(self, rhs: M31) -> Self::Output {
        SimdM31Ext3 {
            // SimdM31 + M31
            v: [self.v[0] + rhs, self.v[1], self.v[2]],
        }
    }
}

impl<T: ::core::borrow::Borrow<SimdM31Ext3>> Product<T> for SimdM31Ext3 {
    fn product<I: Iterator<Item = T>>(iter: I) -> Self {
        iter.fold(Self::one(), |acc, item| acc * item.borrow())
    }
}

impl Add<&SimdM31Ext3> for SimdM31Ext3 {
    type Output = SimdM31Ext3;
    #[inline(always)]
    fn add(self, rhs: &SimdM31Ext3) -> Self::Output {
        SimdM31Ext3 {
            v: [
                self.v[0] + rhs.v[0],
                self.v[1] + rhs.v[1],
                self.v[2] + rhs.v[2],
            ],
        }
    }
}

impl Add for SimdM31Ext3 {
    type Output = SimdM31Ext3;
    #[inline(always)]
    #[allow(clippy::op_ref)]
    fn add(self, rhs: SimdM31Ext3) -> Self::Output {
        self + &rhs
    }
}

impl AddAssign<&SimdM31Ext3> for SimdM31Ext3 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: &SimdM31Ext3) {
        *self = *self + rhs;
    }
}

impl AddAssign for SimdM31Ext3 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self += &rhs;
    }
}

impl<T: ::core::borrow::Borrow<SimdM31Ext3>> Sum<T> for SimdM31Ext3 {
    fn sum<I: Iterator<Item = T>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, item| acc + item.borrow())
    }
}

impl Neg for SimdM31Ext3 {
    type Output = SimdM31Ext3;
    #[inline(always)]
    fn neg(self) -> Self::Output {
        SimdM31Ext3 {
            v: [-self.v[0], -self.v[1], -self.v[2]],
        }
    }
}

impl Sub<&SimdM31Ext3> for SimdM31Ext3 {
    type Output = SimdM31Ext3;
    #[inline(always)]
    #[allow(clippy::op_ref)]
    fn sub(self, rhs: &SimdM31Ext3) -> Self::Output {
        self + &(-*rhs)
    }
}

impl Sub for SimdM31Ext3 {
    type Output = SimdM31Ext3;
    #[inline(always)]
    #[allow(clippy::op_ref)]
    fn sub(self, rhs: SimdM31Ext3) -> Self::Output {
        self - &rhs
    }
}

impl SubAssign<&SimdM31Ext3> for SimdM31Ext3 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: &SimdM31Ext3) {
        *self = *self - rhs;
    }
}

impl SubAssign for SimdM31Ext3 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self -= &rhs;
    }
}

impl From<u32> for SimdM31Ext3 {
    #[inline(always)]
    fn from(x: u32) -> Self {
        SimdM31Ext3 {
            v: [SimdM31::from(x), SimdM31::zero(), SimdM31::zero()],
        }
    }
}

// polynomial mod (x^3 - 5)
//
//   (a0 + a1*x + a2*x^2) * (b0 + b1*x + b2*x^2) mod (x^3 - 5)
// = a0*b0 + (a0*b1 + a1*b0)*x + (a0*b2 + a1*b1 + a2*b0)*x^2
// + (a1*b2 + a2*b1)*x^3 + a2*b2*x^4 mod (x^3 - 5)
// = a0*b0 + 5*(a1*b2 + a2*b1)
// + (a0*b1 + a1*b0)*x + 5* a2*b2
// + (a0*b2 + a1*b1 + a2*b0)*x^2
#[inline(always)]
fn mul_internal(a: &[SimdM31; 3], b: &[SimdM31; 3]) -> [SimdM31; 3] {
    let mut res = [SimdM31::default(); 3];
    res[0] = a[0] * b[0] + (a[1] * b[2] + a[2] * b[1]).mul_by_5();
    res[1] = a[0] * b[1] + a[1] * b[0] + a[2] * b[2].mul_by_5();
    res[2] = a[0] * b[2] + a[1] * b[1] + a[2] * b[0];
    res
}

// same as mul; merge identical terms
#[inline(always)]
fn square_internal(a: &[SimdM31; 3]) -> [SimdM31; 3] {
    let mut res = [SimdM31::default(); 3];
    res[0] = a[0].square() + a[1] * a[2].mul_by_10();
    let t = a[0] * a[1];
    res[1] = t.double() + a[2].square().mul_by_5();
    let t = a[0] * a[2];
    res[2] = t.double() + a[1] * a[1];
    res
}
