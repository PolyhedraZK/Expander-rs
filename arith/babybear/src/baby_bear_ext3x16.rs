use std::{
    io::{Read, Write},
    iter::{Product, Sum},
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use arith::{field_common, ExtensionField, Field, FieldSerde, FieldSerdeResult, SimdField};

use crate::{baby_bear::BabyBearx16, BabyBear, BabyBearExt3};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BabyBearExt3x16 {
    pub v: [BabyBearx16; 3],
}

field_common!(BabyBearExt3x16);

impl FieldSerde for BabyBearExt3x16 {
    const SERIALIZED_SIZE: usize = 512 / 8 * 3;

    #[inline(always)]
    fn serialize_into<W: Write>(&self, mut writer: W) -> FieldSerdeResult<()> {
        self.v[0].serialize_into(&mut writer)?;
        self.v[1].serialize_into(&mut writer)?;
        self.v[2].serialize_into(&mut writer)
    }

    #[inline(always)]
    fn deserialize_from<R: Read>(mut reader: R) -> FieldSerdeResult<Self> {
        Ok(Self {
            v: [
                BabyBearx16::deserialize_from(&mut reader)?,
                BabyBearx16::deserialize_from(&mut reader)?,
                BabyBearx16::deserialize_from(&mut reader)?,
            ],
        })
    }

    #[inline(always)]
    fn try_deserialize_from_ecc_format<R: Read>(mut reader: R) -> FieldSerdeResult<Self> {
        Ok(Self {
            v: [
                BabyBearx16::try_deserialize_from_ecc_format(&mut reader)?,
                BabyBearx16::zero(),
                BabyBearx16::zero(),
            ],
        })
    }
}

impl SimdField for BabyBearExt3x16 {
    type Scalar = BabyBearExt3;

    #[inline]
    fn scale(&self, challenge: &Self::Scalar) -> Self {
        *self * *challenge
    }

    #[inline(always)]
    fn pack(base_vec: &[Self::Scalar]) -> Self {
        assert!(base_vec.len() == Self::pack_size());
        let mut v0s = vec![];
        let mut v1s = vec![];
        let mut v2s = vec![];

        for scalar in base_vec {
            v0s.push(scalar.v[0]);
            v1s.push(scalar.v[1]);
            v2s.push(scalar.v[2]);
        }

        Self {
            v: [
                BabyBearx16::pack(&v0s),
                BabyBearx16::pack(&v1s),
                BabyBearx16::pack(&v2s),
            ],
        }
    }

    #[inline(always)]
    fn unpack(&self) -> Vec<Self::Scalar> {
        let v0s = self.v[0].unpack();
        let v1s = self.v[1].unpack();
        let v2s = self.v[2].unpack();

        v0s.into_iter()
            .zip(v1s)
            .zip(v2s)
            .map(|((v0, v1), v2)| BabyBearExt3 { v: [v0, v1, v2] })
            .collect()
    }

    #[inline(always)]
    fn pack_size() -> usize {
        BabyBearx16::pack_size()
    }
}

impl From<BabyBearx16> for BabyBearExt3x16 {
    #[inline(always)]
    fn from(x: BabyBearx16) -> Self {
        Self {
            v: [x, BabyBearx16::ZERO, BabyBearx16::ZERO],
        }
    }
}

impl ExtensionField for BabyBearExt3x16 {
    const DEGREE: usize = 3;

    const W: u32 = 2;

    const X: Self = Self {
        v: [BabyBearx16::ZERO, BabyBearx16::ONE, BabyBearx16::ZERO],
    };

    type BaseField = BabyBearx16;

    #[inline(always)]
    fn mul_by_base_field(&self, base: &Self::BaseField) -> Self {
        Self {
            v: [self.v[0] * base, self.v[1] * base, self.v[2] * base],
        }
    }

    #[inline(always)]
    fn add_by_base_field(&self, base: &Self::BaseField) -> Self {
        Self {
            v: [self.v[0] + base, self.v[1], self.v[2]],
        }
    }

    #[inline(always)]
    fn mul_by_x(&self) -> Self {
        Self {
            // Note: W = 2
            v: [self.v[2].double(), self.v[0], self.v[1]],
        }
    }
}

impl From<BabyBearExt3> for BabyBearExt3x16 {
    #[inline(always)]
    fn from(x: BabyBearExt3) -> Self {
        Self {
            v: [
                BabyBearx16::pack_full(x.v[0]),
                BabyBearx16::pack_full(x.v[1]),
                BabyBearx16::pack_full(x.v[2]),
            ],
        }
    }
}

impl Field for BabyBearExt3x16 {
    #[cfg(target_arch = "x86_64")]
    const NAME: &'static str = "AVX Vectorized BabyBear Extension 3";

    #[cfg(target_arch = "aarch64")]
    const NAME: &'static str = "NEON Vectorized BabyBear Extension 3";

    const SIZE: usize = 512 / 8 * 3;

    const FIELD_SIZE: usize = 32 * 3;

    const ZERO: Self = Self {
        v: [BabyBearx16::ZERO; 3],
    };

    const ONE: Self = Self {
        v: [BabyBearx16::ONE, BabyBearx16::ZERO, BabyBearx16::ZERO],
    };

    const INV_2: Self = Self {
        v: [BabyBearx16::INV_2, BabyBearx16::ZERO, BabyBearx16::ZERO],
    };

    #[inline(always)]
    fn zero() -> Self {
        Self::ZERO
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        *self == Self::ZERO
    }

    #[inline(always)]
    fn one() -> Self {
        Self::ONE
    }

    #[inline(always)]
    fn random_unsafe(mut rng: impl rand::RngCore) -> Self {
        Self {
            v: [
                BabyBearx16::random_unsafe(&mut rng),
                BabyBearx16::random_unsafe(&mut rng),
                BabyBearx16::random_unsafe(&mut rng),
            ],
        }
    }

    #[inline(always)]
    fn random_bool(mut rng: impl rand::RngCore) -> Self {
        Self {
            v: [
                BabyBearx16::random_bool(&mut rng),
                BabyBearx16::random_bool(&mut rng),
                BabyBearx16::random_bool(&mut rng),
            ],
        }
    }

    #[inline(always)]
    fn square(&self) -> Self {
        Self {
            v: square_internal(&self.v),
        }
    }

    fn exp(&self, _: u128) -> Self {
        unimplemented!()
    }

    fn inv(&self) -> Option<Self> {
        unimplemented!()
    }

    fn as_u32_unchecked(&self) -> u32 {
        unimplemented!("self is a vector, cannot convert to u32")
    }

    fn from_uniform_bytes(_: &[u8; 32]) -> Self {
        unimplemented!("vec babybear: cannot convert from 32 bytes")
    }
}

impl Mul<BabyBearExt3> for BabyBearExt3x16 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: BabyBearExt3) -> Self {
        // polynomial mod x^3 - w
        //
        // (a0 + a1 x + a2 x^2) * (b0 + b1 x + b2 x^2)
        // = a0 b0 + (a0 b1 + a1 b0) x + (a0 b2 + a1 b1 + a2 b0) x^2 + (a1 b2 + a2 b1) x^3 + a2 b2 x^4
        // = a0 b0 + w * (a1 b2 + a2 b1)
        //   + {(a0 b1 + a1 b0) + w * a2 b2} x
        //   + {(a0 b2 + a1 b1 + a2 b0)} x^2

        // Note: W = 2
        let mut res = [BabyBearx16::ZERO; 3];
        res[0] = self.v[0] * rhs.v[0] + (self.v[1] * rhs.v[2] + self.v[2] * rhs.v[1]).double();
        res[1] = self.v[0] * rhs.v[1] + self.v[1] * rhs.v[0] + self.v[2] * rhs.v[2].double();
        res[2] = self.v[0] * rhs.v[2] + self.v[1] * rhs.v[1] + self.v[2] * rhs.v[0];
        Self { v: res }
    }
}

impl Mul<BabyBear> for BabyBearExt3x16 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: BabyBear) -> Self {
        Self {
            v: [self.v[0] * rhs, self.v[1] * rhs, self.v[2] * rhs],
        }
    }
}

impl Add<BabyBear> for BabyBearExt3x16 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: BabyBear) -> Self {
        Self {
            v: [self.v[0] + rhs, self.v[1], self.v[2]],
        }
    }
}

impl Neg for BabyBearExt3x16 {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self {
        Self {
            v: [-self.v[0], -self.v[1], -self.v[2]],
        }
    }
}

impl From<u32> for BabyBearExt3x16 {
    #[inline(always)]
    fn from(value: u32) -> Self {
        Self {
            v: [
                BabyBearx16::from(value),
                BabyBearx16::ZERO,
                BabyBearx16::ZERO,
            ],
        }
    }
}

#[inline(always)]
fn add_internal(a: &BabyBearExt3x16, b: &BabyBearExt3x16) -> BabyBearExt3x16 {
    let mut vv = a.v;
    vv[0] += b.v[0];
    vv[1] += b.v[1];
    vv[2] += b.v[2];

    BabyBearExt3x16 { v: vv }
}

#[inline(always)]
fn sub_internal(a: &BabyBearExt3x16, b: &BabyBearExt3x16) -> BabyBearExt3x16 {
    let mut vv = a.v;
    vv[0] -= b.v[0];
    vv[1] -= b.v[1];
    vv[2] -= b.v[2];

    BabyBearExt3x16 { v: vv }
}

#[inline(always)]
fn mul_internal(a: &BabyBearExt3x16, b: &BabyBearExt3x16) -> BabyBearExt3x16 {
    // polynomial mod x^3 - w
    //
    // (a0 + a1 x + a2 x^2) * (b0 + b1 x + b2 x^2)
    // = a0 b0 + (a0 b1 + a1 b0) x + (a0 b2 + a1 b1 + a2 b0) x^2 + (a1 b2 + a2 b1) x^3 + a2 b2 x^4
    // = a0 b0 + w * (a1 b2 + a2 b1)
    //   + {(a0 b1 + a1 b0) + w * a2 b2} x
    //   + {(a0 b2 + a1 b1 + a2 b0)} x^2
    let a = &a.v;
    let b = &b.v;
    let mut res = [BabyBearx16::default(); 3];
    // Note: W = 2
    res[0] = a[0] * b[0] + (a[1] * b[2] + a[2] * b[1]).double();
    res[1] = (a[0] * b[1] + a[1] * b[0]) + a[2] * b[2].double();
    res[2] = a[0] * b[2] + a[1] * b[1] + a[2] * b[0];

    BabyBearExt3x16 { v: res }
}

#[inline(always)]
fn square_internal(a: &[BabyBearx16; 3]) -> [BabyBearx16; 3] {
    let mut res = [BabyBearx16::default(); 3];
    // Note: W = 2
    res[0] = a[0].square() + (a[1] * a[2]).double().double();
    res[1] = (a[0] * a[1]).double() + a[2].square().double();
    res[2] = a[0] * a[2].double() + a[1].square();

    res
}
