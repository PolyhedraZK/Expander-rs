use std::{
    iter::{Product, Sum},
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use arith::{field_common, ExtensionField, Field, FieldSerde, FieldSerdeResult};
use p3_field::{AbstractExtensionField, Field as P3Field, PrimeField32};

use crate::BabyBear;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct BabyBearExt4 {
    pub v: [BabyBear; 4],
}

field_common!(BabyBearExt4);

impl FieldSerde for BabyBearExt4 {
    const SERIALIZED_SIZE: usize = 32 / 8 * 4;

    #[inline(always)]
    fn serialize_into<W: std::io::Write>(&self, mut writer: W) -> FieldSerdeResult<()> {
        self.v[0].serialize_into(&mut writer)?;
        self.v[1].serialize_into(&mut writer)?;
        self.v[2].serialize_into(&mut writer)?;
        self.v[3].serialize_into(&mut writer)?;
        Ok(())
    }

    #[inline(always)]
    fn deserialize_from<R: std::io::Read>(mut reader: R) -> FieldSerdeResult<Self> {
        Ok(Self {
            v: [
                BabyBear::deserialize_from(&mut reader)?,
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
        // ? this can only read in a base field element, do we ever need to read in an ext'n field?
        Ok(Self::from(u32::from_le_bytes(buf[..4].try_into().unwrap())))
    }
}

impl Field for BabyBearExt4 {
    const NAME: &'static str = "Baby Bear Extension 4";

    const SIZE: usize = 32 / 8 * 4;

    const FIELD_SIZE: usize = 32 * 4;

    const ZERO: Self = Self {
        v: [BabyBear::ZERO; 4],
    };

    const ONE: Self = Self {
        v: [
            BabyBear::ONE,
            BabyBear::ZERO,
            BabyBear::ZERO,
            BabyBear::ZERO,
        ],
    };

    const INV_2: Self = Self {
        v: [
            BabyBear::INV_2,
            BabyBear::ZERO,
            BabyBear::ZERO,
            BabyBear::ZERO,
        ],
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
                BabyBear::random_unsafe(&mut rng),
            ],
        }
    }

    fn random_bool(rng: impl rand::RngCore) -> Self {
        Self {
            v: [
                BabyBear::random_bool(rng),
                BabyBear::ZERO,
                BabyBear::ZERO,
                BabyBear::ZERO,
            ],
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
        // Cast to P3 type, invert, and cast back
        let p3_self: P3BabyBearExt4 = self.into();
        let p3_inv = p3_self.try_inverse()?;
        Some((&p3_inv).into())
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
        let v4 = BabyBear::from(u32::from_be_bytes(bytes[12..16].try_into().unwrap()));
        Self {
            v: [v1, v2, v3, v4],
        }
    }
}

impl ExtensionField for BabyBearExt4 {
    const DEGREE: usize = 4;

    const W: u32 = 11;

    const X: Self = Self {
        v: [
            BabyBear::ZERO,
            BabyBear::ONE,
            BabyBear::ZERO,
            BabyBear::ZERO,
        ],
    };

    type BaseField = BabyBear;

    #[inline(always)]
    fn mul_by_base_field(&self, base: &Self::BaseField) -> Self {
        let mut res = self.v;
        res[0] *= base;
        res[1] *= base;
        res[2] *= base;
        res[3] *= base;
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
        let w = BabyBear::from(Self::W);
        Self {
            v: [self.v[3] * w, self.v[0], self.v[1], self.v[2]],
        }
    }
}

impl Add<BabyBear> for BabyBearExt4 {
    type Output = Self;

    fn add(self, rhs: BabyBear) -> Self::Output {
        self + BabyBearExt4::from(rhs)
    }
}

impl Neg for BabyBearExt4 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let mut v = self.v;
        v[0] = -v[0];
        v[1] = -v[1];
        v[2] = -v[2];
        v[3] = -v[3];
        Self { v }
    }
}

impl From<u32> for BabyBearExt4 {
    fn from(val: u32) -> Self {
        Self {
            v: [
                BabyBear::new(val),
                BabyBear::ZERO,
                BabyBear::ZERO,
                BabyBear::ZERO,
            ],
        }
    }
}

impl BabyBearExt4 {
    #[inline(always)]
    pub fn to_base_field(&self) -> BabyBear {
        assert!(
            self.v[1].is_zero() && self.v[2].is_zero() && self.v[3].is_zero(),
            "BabyBearExt4 cannot be converted to base field"
        );

        self.to_base_field_unsafe()
    }

    #[inline(always)]
    pub fn to_base_field_unsafe(&self) -> BabyBear {
        self.v[0]
    }

    #[inline(always)]
    pub fn as_u32_array(&self) -> [u32; 4] {
        // Note: as_u32_unchecked converts to canonical form
        [
            self.v[0].as_u32_unchecked(),
            self.v[1].as_u32_unchecked(),
            self.v[2].as_u32_unchecked(),
            self.v[3].as_u32_unchecked(),
        ]
    }
}

impl From<BabyBear> for BabyBearExt4 {
    #[inline(always)]
    fn from(val: BabyBear) -> Self {
        Self {
            v: [val, BabyBear::ZERO, BabyBear::ZERO, BabyBear::ZERO],
        }
    }
}

impl From<&BabyBear> for BabyBearExt4 {
    #[inline(always)]
    fn from(val: &BabyBear) -> Self {
        (*val).into()
    }
}

impl From<BabyBearExt4> for BabyBear {
    #[inline(always)]
    fn from(x: BabyBearExt4) -> Self {
        x.to_base_field()
    }
}

impl From<&BabyBearExt4> for BabyBear {
    #[inline(always)]
    fn from(x: &BabyBearExt4) -> Self {
        x.to_base_field()
    }
}

#[inline(always)]
pub(crate) fn add_internal(a: &BabyBearExt4, b: &BabyBearExt4) -> BabyBearExt4 {
    let mut vv = a.v;
    vv[0] += b.v[0];
    vv[1] += b.v[1];
    vv[2] += b.v[2];
    vv[3] += b.v[3];
    BabyBearExt4 { v: vv }
}

#[inline(always)]
pub(crate) fn sub_internal(a: &BabyBearExt4, b: &BabyBearExt4) -> BabyBearExt4 {
    let mut vv = a.v;
    vv[0] -= b.v[0];
    vv[1] -= b.v[1];
    vv[2] -= b.v[2];
    vv[3] -= b.v[3];
    BabyBearExt4 { v: vv }
}

// polynomial mod x^4 - 11
//
// (a0 + a1 x + a2 x^2 + a3 x^3) * (b0 + b1 x + b2 x^2 + b3 x^3)
// = a0 b0 + (a0 b1 + a1 b0) x + (a0 b2 + a1 b1 + a2 b0) x^2 + (a0 b3 + a1 b2 + a2 b1 + a3 b0) x^3
// + (a1 b3 + a2 b2 + a3 b1) x^4 + (a2 b3 + a3 b2) x^5 + a3 b3 x^6 mod (x^4 - 11)
// = a0 b0 + 11 (a1 b3 + a2 b2 + a3 b1)
// + { (a0 b1 + a1 b0) + 11 (a2 b3 + a3 b2) } x
// + { (a0 b2 + a1 b1 + a2 b0) + 11 a3 b3 } x^2
// + { (a0 b3 + a1 b2 + a2 b1 + a3 b0) } x^3
#[inline(always)]
pub(crate) fn mul_internal(a: &BabyBearExt4, b: &BabyBearExt4) -> BabyBearExt4 {
    let w = BabyBear::new(BabyBearExt4::W);
    let a = a.v;
    let b = b.v;
    let mut res = [BabyBear::default(); 4];
    res[0] = a[0] * b[0] + w * (a[1] * b[3] + a[2] * b[2] + a[3] * b[1]);
    res[1] = (a[0] * b[1] + a[1] * b[0]) + w * (a[2] * b[3] + a[3] * b[2]);
    res[2] = (a[0] * b[2] + a[1] * b[1] + a[2] * b[0]) + w * a[3] * b[3];
    res[3] = a[0] * b[3] + a[1] * b[2] + a[2] * b[1] + a[3] * b[0];
    BabyBearExt4 { v: res }
}

#[inline(always)]
fn square_internal(a: &[BabyBear; 4]) -> [BabyBear; 4] {
    let w = BabyBear::new(BabyBearExt4::W);
    let mut res = [BabyBear::default(); 4];
    res[0] = a[0].square() + w * (a[1].double() * a[3] + a[2].square());
    res[1] = a[0] * a[1].double() + w * a[2] * a[3].double();
    res[2] = (a[0] * a[2].double() + a[1].square()) + w * a[3].square();
    res[3] = a[0] * a[3].double() + a[1] * a[2].double();
    res
}

// Useful for conversion to Plonky3
pub(crate) type P3BabyBearExt4 =
    p3_field::extension::BinomialExtensionField<p3_baby_bear::BabyBear, 4>;

impl From<&P3BabyBearExt4> for BabyBearExt4 {
    fn from(p3: &P3BabyBearExt4) -> Self {
        Self {
            v: p3
                .as_base_slice()
                .iter()
                .map(|x: &p3_baby_bear::BabyBear| x.as_canonical_u32().into())
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        }
    }
}

impl From<&BabyBearExt4> for P3BabyBearExt4 {
    fn from(b: &BabyBearExt4) -> Self {
        P3BabyBearExt4::from_base_slice(
            &b.v.iter()
                .map(|x| p3_baby_bear::BabyBear::new(x.as_u32_unchecked()))
                .collect::<Vec<_>>(),
        )
    }
}
