use std::{
    io::{Read, Write},
    iter::{Product, Sum},
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use crate::{
    field_common, BinomialExtensionField, Field, FieldSerde, M31Ext3, M31x16, SimdField, M31,
};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct M31Ext3x16 {
    pub v: [M31x16; 3],
}

field_common!(M31Ext3x16);

impl FieldSerde for M31Ext3x16 {
    #[inline(always)]
    fn serialize_into<W: Write>(&self, mut writer: W) {
        self.v[0].serialize_into(&mut writer);
        self.v[1].serialize_into(&mut writer);
        self.v[2].serialize_into(&mut writer);
    }

    #[inline(always)]
    fn serialized_size() -> usize {
        512 / 8 * 3
    }

    // FIXME: this deserialization function auto corrects invalid inputs.
    // We should use separate APIs for this and for the actual deserialization.
    #[inline(always)]
    fn deserialize_from<R: Read>(mut reader: R) -> Self {
        Self {
            v: [
                M31x16::deserialize_from(&mut reader),
                M31x16::deserialize_from(&mut reader),
                M31x16::deserialize_from(&mut reader),
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
                M31x16::try_deserialize_from_ecc_format(&mut reader)?,
                M31x16::zero(),
                M31x16::zero(),
            ],
        })
    }
}

impl SimdField for M31Ext3x16 {
    type Scalar = M31Ext3;

    fn from_scalar_array(scalars: &[Self::Scalar]) -> Self {
        // We don't necessarily need this function
        let v: Vec<M31x16> = (0..3)
            .map(|i| {
                M31x16::from_scalar_array(
                    &scalars
                        .iter()
                        .map(|scalar| scalar.v[i])
                        .collect::<Vec<M31>>()
                )
            })
            .collect();

        Self {
            v: v.try_into().unwrap(),
        }
    }

    #[inline]
    fn scale(&self, challenge: &Self::Scalar) -> Self {
        *self * *challenge
    }
}

impl From<M31x16> for M31Ext3x16 {
    #[inline(always)]
    fn from(x: M31x16) -> Self {
        Self {
            v: [x, M31x16::zero(), M31x16::zero()],
        }
    }
}

impl BinomialExtensionField for M31Ext3x16 {
    const DEGREE: usize = 3;

    const W: u32 = 5;

    type BaseField = M31x16;

    #[inline(always)]
    fn mul_by_base_field(&self, base: &Self::BaseField) -> Self {
        M31Ext3x16 {
            v: [self.v[0] * base, self.v[1] * base, self.v[2] * base],
        }
    }

    #[inline(always)]
    fn add_by_base_field(&self, base: &Self::BaseField) -> Self {
        M31Ext3x16 {
            v: [self.v[0] + base, self.v[1], self.v[2]],
        }
    }

    #[inline(always)]
    fn first_base_field(&self) -> Self::BaseField {
        self.v[0]
    }
}

impl From<M31Ext3> for M31Ext3x16 {
    #[inline(always)]
    fn from(x: M31Ext3) -> Self {
        Self {
            v: [
                M31x16::pack_full(x.v[0]),
                M31x16::pack_full(x.v[1]),
                M31x16::pack_full(x.v[2]),
            ],
        }
    }
}

impl Field for M31Ext3x16 {
    #[cfg(target_arch = "x86_64")]
    const NAME: &'static str = "AVX Vectorized Mersenne 31 Extension 3";

    #[cfg(target_arch = "aarch64")]
    const NAME: &'static str = "Neon Vectorized Mersenne 31 Extension 3";

    const SIZE: usize = 512 / 8 * 3;

    const FIELD_SIZE: usize = 32 * 3;

    const ZERO: Self = Self {
        v: [M31x16::ZERO; 3],
    };

    const INV_2: Self = Self {
        v: [M31x16::INV_2, M31x16::ZERO, M31x16::ZERO],
    };

    #[inline(always)]
    fn zero() -> Self {
        M31Ext3x16 {
            v: [M31x16::zero(); 3],
        }
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        self.v[0].is_zero() && self.v[1].is_zero() && self.v[2].is_zero()
    }

    #[inline(always)]
    fn one() -> Self {
        M31Ext3x16 {
            v: [M31x16::one(), M31x16::zero(), M31x16::zero()],
        }
    }

    #[inline(always)]
    fn random_unsafe(mut rng: impl rand::RngCore) -> Self {
        M31Ext3x16 {
            v: [
                M31x16::random_unsafe(&mut rng),
                M31x16::random_unsafe(&mut rng),
                M31x16::random_unsafe(&mut rng),
            ],
        }
    }

    #[inline(always)]
    fn random_bool(mut rng: impl rand::RngCore) -> Self {
        M31Ext3x16 {
            v: [
                M31x16::random_bool(&mut rng),
                M31x16::zero(),
                M31x16::zero(),
            ],
        }
    }

    #[inline(always)]
    fn square(&self) -> Self {
        Self {
            v: square_internal(&self.v),
        }
    }

    fn exp(&self, _exponent: u128) -> Self {
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

impl Mul<M31Ext3> for M31Ext3x16 {
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
        let mut res = [M31x16::default(); 3];
        res[0] =
            self.v[0] * rhs.v[0] + self.v[1] * (rhs.v[2] * five) + self.v[2] * (rhs.v[1] * five);
        res[1] = self.v[0] * rhs.v[1] + self.v[1] * rhs.v[0] + self.v[2] * (rhs.v[2] * five);
        res[2] = self.v[0] * rhs.v[2] + self.v[1] * rhs.v[1] + self.v[2] * rhs.v[0];
        Self { v: res }
    }
}

impl Mul<M31> for M31Ext3x16 {
    type Output = M31Ext3x16;
    #[inline(always)]
    fn mul(self, rhs: M31) -> Self::Output {
        M31Ext3x16 {
            // M31x16 * M31
            v: [self.v[0] * rhs, self.v[1] * rhs, self.v[2] * rhs],
        }
    }
}

impl Add<M31> for M31Ext3x16 {
    type Output = M31Ext3x16;
    #[inline(always)]
    fn add(self, rhs: M31) -> Self::Output {
        M31Ext3x16 {
            // M31x16 + M31
            v: [self.v[0] + rhs, self.v[1], self.v[2]],
        }
    }
}

impl Neg for M31Ext3x16 {
    type Output = M31Ext3x16;
    #[inline(always)]
    fn neg(self) -> Self::Output {
        M31Ext3x16 {
            v: [-self.v[0], -self.v[1], -self.v[2]],
        }
    }
}

impl From<u32> for M31Ext3x16 {
    #[inline(always)]
    fn from(x: u32) -> Self {
        M31Ext3x16 {
            v: [M31x16::from(x), M31x16::zero(), M31x16::zero()],
        }
    }
}

fn add_internal(a: &M31Ext3x16, b: &M31Ext3x16) -> M31Ext3x16 {
    let mut vv = a.v;
    vv[0] += b.v[0];
    vv[1] += b.v[1];
    vv[2] += b.v[2];

    M31Ext3x16 { v: vv }
}

fn sub_internal(a: &M31Ext3x16, b: &M31Ext3x16) -> M31Ext3x16 {
    let mut vv = a.v;
    vv[0] -= b.v[0];
    vv[1] -= b.v[1];
    vv[2] -= b.v[2];

    M31Ext3x16 { v: vv }
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
fn mul_internal(a: &M31Ext3x16, b: &M31Ext3x16) -> M31Ext3x16 {
    let a = &a.v;
    let b = &b.v;
    let mut res = [M31x16::default(); 3];
    res[0] = a[0] * b[0] + (a[1] * b[2] + a[2] * b[1]).mul_by_5();
    res[1] = a[0] * b[1] + a[1] * b[0] + a[2] * b[2].mul_by_5();
    res[2] = a[0] * b[2] + a[1] * b[1] + a[2] * b[0];
    M31Ext3x16 { v: res }
}

// same as mul; merge identical terms
#[inline(always)]
fn square_internal(a: &[M31x16; 3]) -> [M31x16; 3] {
    let mut res = [M31x16::default(); 3];
    let a2_w = a[2].mul_by_5();
    res[0] = a[0].square() + a[1] * a2_w.double();
    res[1] = a[0] * a[1].double() + a[2] * a2_w;
    res[2] = a[0] * a[2].double() + a[1] * a[1];
    res
}
