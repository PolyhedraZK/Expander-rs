use std::{
    iter::{Product, Sum},
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use arith::{field_common, ExtensionField, Field, FieldSerde, FieldSerdeResult};

use crate::{BabyBear, BABYBEAR_MODULUS};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct BabyBearExt3 {
    pub v: [BabyBear; 3],
}

field_common!(BabyBearExt3);

impl FieldSerde for BabyBearExt3 {
    const SERIALIZED_SIZE: usize = 32 / 8 * 3;

    #[inline(always)]
    fn serialize_into<W: std::io::Write>(&self, mut writer: W) -> FieldSerdeResult<()> {
        self.v[0].serialize_into(&mut writer)?;
        self.v[1].serialize_into(&mut writer)?;
        self.v[2].serialize_into(&mut writer)?;
        Ok(())
    }

    #[inline(always)]
    fn deserialize_from<R: std::io::Read>(mut reader: R) -> FieldSerdeResult<Self> {
        Ok(Self {
            v: [
                BabyBear::deserialize_from(&mut reader)?,
                BabyBear::deserialize_from(&mut reader)?,
                BabyBear::deserialize_from(&mut reader)?,
            ],
        })
    }

    #[inline]
    fn try_deserialize_from_ecc_format<R: std::io::Read>(mut reader: R) -> FieldSerdeResult<Self> {
        let mut buf = [0u8; 32];
        reader.read_exact(&mut buf)?;
        assert!(
            buf.iter().skip(4).all(|&x| x == 0),
            "non-zero byte found in witness byte"
        );
        Ok(Self::from(u32::from_le_bytes(buf[..4].try_into().unwrap())))
    }
}

impl Field for BabyBearExt3 {
    const NAME: &'static str = "Baby Bear Extension 4";

    const SIZE: usize = 32 / 8 * 3;

    const FIELD_SIZE: usize = 32 * 3;

    const ZERO: Self = Self {
        v: [BabyBear::ZERO; 3],
    };

    const ONE: Self = Self {
        v: [BabyBear::ONE, BabyBear::ZERO, BabyBear::ZERO],
    };

    const INV_2: Self = Self {
        v: [BabyBear::INV_2, BabyBear::ZERO, BabyBear::ZERO],
    };

    fn zero() -> Self {
        Self::ZERO
    }

    fn is_zero(&self) -> bool {
        *self == Self::ZERO
    }

    fn one() -> Self {
        Self::ONE
    }

    fn random_unsafe(mut rng: impl rand::RngCore) -> Self {
        Self {
            v: [
                BabyBear::random_unsafe(&mut rng),
                BabyBear::random_unsafe(&mut rng),
                BabyBear::random_unsafe(&mut rng),
            ],
        }
    }

    fn random_bool(rng: impl rand::RngCore) -> Self {
        Self {
            v: [BabyBear::random_bool(rng), BabyBear::ZERO, BabyBear::ZERO],
        }
    }

    fn exp(&self, exponent: u128) -> Self {
        let mut e = exponent;
        let mut res = Self::one();
        let mut t = *self;
        while e != 0 {
            let b = e & 1;
            if b == 1 {
                res *= t;
            }
            t = t * t;
            e >>= 1;
        }
        res
    }

    fn inv(&self) -> Option<Self> {
        if self.is_zero() {
            None
        } else {
            // TODO: Implement a more efficient inversion
            let e = (BABYBEAR_MODULUS as u128).pow(3) - 2;
            Some(self.exp(e))
        }
    }

    #[inline(always)]
    fn square(&self) -> Self {
        Self {
            v: square_internal(&self.v),
        }
    }

    fn as_u32_unchecked(&self) -> u32 {
        self.v[0].as_u32_unchecked()
    }

    fn from_uniform_bytes(bytes: &[u8; 32]) -> Self {
        let v1 = BabyBear::from(u32::from_be_bytes(bytes[0..4].try_into().unwrap()));
        let v2 = BabyBear::from(u32::from_be_bytes(bytes[4..8].try_into().unwrap()));
        let v3 = BabyBear::from(u32::from_be_bytes(bytes[8..12].try_into().unwrap()));
        Self { v: [v1, v2, v3] }
    }
}

impl ExtensionField for BabyBearExt3 {
    const DEGREE: usize = 3;

    const W: u32 = 2;

    const X: Self = Self {
        v: [BabyBear::ZERO, BabyBear::ONE, BabyBear::ZERO],
    };

    type BaseField = BabyBear;

    #[inline(always)]
    fn mul_by_base_field(&self, base: &Self::BaseField) -> Self {
        let mut res = self.v;
        res[0] *= base;
        res[1] *= base;
        res[2] *= base;
        Self { v: res }
    }

    #[inline(always)]
    fn add_by_base_field(&self, base: &Self::BaseField) -> Self {
        let mut res = self.v;
        res[0] += base;
        Self { v: res }
    }

    #[inline(always)]
    fn mul_by_x(&self) -> Self {
        // Note: W = 2
        Self {
            v: [self.v[2].double(), self.v[0], self.v[1]],
        }
    }
}

impl Add<BabyBear> for BabyBearExt3 {
    type Output = Self;

    fn add(self, rhs: BabyBear) -> Self::Output {
        self + BabyBearExt3::from(rhs)
    }
}

impl Neg for BabyBearExt3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let mut v = self.v;
        v[0] = -v[0];
        v[1] = -v[1];
        v[2] = -v[2];
        Self { v }
    }
}

impl From<u32> for BabyBearExt3 {
    fn from(val: u32) -> Self {
        Self {
            v: [BabyBear::new(val), BabyBear::ZERO, BabyBear::ZERO],
        }
    }
}

impl BabyBearExt3 {
    #[inline(always)]
    pub fn to_base_field(&self) -> BabyBear {
        assert!(
            self.v[1].is_zero() && self.v[2].is_zero(),
            "BabyBearExt3 cannot be converted to base field"
        );

        self.to_base_field_unsafe()
    }

    #[inline(always)]
    pub fn to_base_field_unsafe(&self) -> BabyBear {
        self.v[0]
    }

    #[inline(always)]
    pub fn as_u32_array(&self) -> [u32; 3] {
        // Note: as_u32_unchecked converts to canonical form
        [
            self.v[0].as_u32_unchecked(),
            self.v[1].as_u32_unchecked(),
            self.v[2].as_u32_unchecked(),
        ]
    }
}

impl From<BabyBear> for BabyBearExt3 {
    #[inline(always)]
    fn from(val: BabyBear) -> Self {
        Self {
            v: [val, BabyBear::ZERO, BabyBear::ZERO],
        }
    }
}

impl From<&BabyBear> for BabyBearExt3 {
    #[inline(always)]
    fn from(val: &BabyBear) -> Self {
        (*val).into()
    }
}

impl From<BabyBearExt3> for BabyBear {
    #[inline(always)]
    fn from(x: BabyBearExt3) -> Self {
        x.to_base_field()
    }
}

impl From<&BabyBearExt3> for BabyBear {
    #[inline(always)]
    fn from(x: &BabyBearExt3) -> Self {
        x.to_base_field()
    }
}

#[inline(always)]
fn add_internal(a: &BabyBearExt3, b: &BabyBearExt3) -> BabyBearExt3 {
    let mut vv = a.v;
    vv[0] += b.v[0];
    vv[1] += b.v[1];
    vv[2] += b.v[2];
    BabyBearExt3 { v: vv }
}

#[inline(always)]
fn sub_internal(a: &BabyBearExt3, b: &BabyBearExt3) -> BabyBearExt3 {
    let mut vv = a.v;
    vv[0] -= b.v[0];
    vv[1] -= b.v[1];
    vv[2] -= b.v[2];
    BabyBearExt3 { v: vv }
}

// polynomial mod x^3 - w
//
// (a0 + a1 x + a2 x^2) * (b0 + b1 x + b2 x^2)
// = a0 b0 + (a0 b1 + a1 b0) x + (a0 b2 + a1 b1 + a2 b0) x^2 + (a1 b2 + a2 b1) x^3 + a2 b2 x^4
// = a0 b0 + w * (a1 b2 + a2 b1)
//   + {(a0 b1 + a1 b0) + w * a2 b2} x
//   + {(a0 b2 + a1 b1 + a2 b0)} x^2
#[inline(always)]
fn mul_internal(a: &BabyBearExt3, b: &BabyBearExt3) -> BabyBearExt3 {
    // Note: W = 2
    let a = a.v;
    let b = b.v;
    let mut res = [BabyBear::default(); 3];
    res[0] = a[0] * b[0] + (a[1] * b[2] + a[2] * b[1]).double();
    res[1] = (a[0] * b[1] + a[1] * b[0]) + a[2] * b[2].double();
    res[2] = a[0] * b[2] + a[1] * b[1] + a[2] * b[0];
    BabyBearExt3 { v: res }
}

#[inline(always)]
fn square_internal(a: &[BabyBear; 3]) -> [BabyBear; 3] {
    // Note: W = 2
    let mut res = [BabyBear::default(); 3];
    res[0] = a[0].square() + (a[1] * a[2]).double().double();
    res[1] = (a[0] * a[1]).double() + a[2].square().double();
    res[2] = a[0] * a[2].double() + a[1].square();
    res
}
