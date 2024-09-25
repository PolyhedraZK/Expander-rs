use rand::RngCore;
use std::{
    io::{Read, Write},
    iter::{Product, Sum},
    mem::transmute,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use arith::ExtensionField;
use arith::{field_common, Field, FieldSerde, FieldSerdeResult};

use crate::m31::{mod_reduce_u32, M31};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct M31Ext3 {
    pub v: [M31; 3],
}

field_common!(M31Ext3);

impl FieldSerde for M31Ext3 {
    const SERIALIZED_SIZE: usize = (32 / 8) * 3;

    #[inline(always)]
    fn serialize_into<W: Write>(&self, mut writer: W) -> FieldSerdeResult<()> {
        self.v[0].serialize_into(&mut writer)?;
        self.v[1].serialize_into(&mut writer)?;
        self.v[2].serialize_into(&mut writer)
    }

    // FIXME: this deserialization function auto corrects invalid inputs.
    // We should use separate APIs for this and for the actual deserialization.
    #[inline(always)]
    fn deserialize_from<R: Read>(mut reader: R) -> FieldSerdeResult<Self> {
        Ok(M31Ext3 {
            v: [
                M31::deserialize_from(&mut reader)?,
                M31::deserialize_from(&mut reader)?,
                M31::deserialize_from(&mut reader)?,
            ],
        })
    }

    #[inline]
    fn try_deserialize_from_ecc_format<R: Read>(mut reader: R) -> FieldSerdeResult<Self> {
        let mut buf = [0u8; 32];
        reader.read_exact(&mut buf)?;
        assert!(
            buf.iter().skip(4).all(|&x| x == 0),
            "non-zero byte found in witness byte"
        );
        Ok(Self::from(u32::from_le_bytes(buf[..4].try_into().unwrap())))
    }
}

impl Field for M31Ext3 {
    const NAME: &'static str = "Mersenne 31 Extension 3";

    const SIZE: usize = 32 / 8 * 3;

    const FIELD_SIZE: usize = 32 * 3;

    const ZERO: Self = M31Ext3 {
        v: [M31::ZERO, M31::ZERO, M31::ZERO],
    };

    const ONE: Self = M31Ext3 {
        v: [M31::ONE, M31::ZERO, M31::ZERO],
    };

    const INV_2: M31Ext3 = M31Ext3 {
        v: [M31::INV_2, M31 { v: 0 }, M31 { v: 0 }],
    };

    #[inline(always)]
    fn zero() -> Self {
        M31Ext3 {
            v: [M31 { v: 0 }; 3],
        }
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        self.v[0].is_zero() && self.v[1].is_zero() && self.v[2].is_zero()
    }

    #[inline(always)]
    fn one() -> Self {
        M31Ext3 {
            v: [M31 { v: 1 }, M31 { v: 0 }, M31 { v: 0 }],
        }
    }

    fn random_unsafe(mut rng: impl RngCore) -> Self {
        M31Ext3 {
            v: [
                M31::random_unsafe(&mut rng),
                M31::random_unsafe(&mut rng),
                M31::random_unsafe(&mut rng),
            ],
        }
    }

    fn random_bool(mut rng: impl RngCore) -> Self {
        M31Ext3 {
            v: [M31::random_bool(&mut rng), M31::zero(), M31::zero()],
        }
    }

    fn exp(&self, exponent: u128) -> Self {
        // raise to the exp only when exponent is a base field element

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
            let base_field_size = (1u128 << 31) - 1;
            Some(self.exp(base_field_size * base_field_size * base_field_size - 2))
        }
    }

    /// Squaring
    #[inline(always)]
    fn square(&self) -> Self {
        Self {
            v: square_internal(&self.v),
        }
    }

    #[inline(always)]
    fn as_u32_unchecked(&self) -> u32 {
        self.v[0].as_u32_unchecked()
    }

    #[inline(always)]
    fn from_uniform_bytes(bytes: &[u8; 32]) -> Self {
        let v1 = mod_reduce_u32(u32::from_be_bytes(bytes[0..4].try_into().unwrap()));
        let v2 = mod_reduce_u32(u32::from_be_bytes(bytes[4..8].try_into().unwrap()));
        let v3 = mod_reduce_u32(u32::from_be_bytes(bytes[8..12].try_into().unwrap()));
        Self {
            v: [
                M31 {
                    v: mod_reduce_u32(v1),
                },
                M31 {
                    v: mod_reduce_u32(v2),
                },
                M31 {
                    v: mod_reduce_u32(v3),
                },
            ],
        }
    }
}

impl ExtensionField for M31Ext3 {
    const DEGREE: usize = 3;

    /// Extension Field
    const W: u32 = 5;

    const X: Self = M31Ext3 {
        v: [M31::ZERO, M31::ONE, M31::ZERO],
    };

    /// Base field for the extension
    type BaseField = M31;

    #[inline(always)]
    /// Multiply the extension field with the base field
    fn mul_by_base_field(&self, base: &Self::BaseField) -> Self {
        let mut res = self.v;
        res[0] *= base;
        res[1] *= base;
        res[2] *= base;
        Self { v: res }
    }

    #[inline(always)]
    /// Add the extension field with the base field
    fn add_by_base_field(&self, base: &Self::BaseField) -> Self {
        let mut res = self.v;
        res[0] += base;
        Self { v: res }
    }

    /// Multiply the extension field by x, i.e, 0 + x + 0 x^2 + 0 x^3 + ...
    #[inline(always)]
    fn mul_by_x(&self) -> Self {
        Self {
            v: [self.v[2].mul_by_5(), self.v[0], self.v[1]],
        }
    }
}

impl Add<M31> for M31Ext3 {
    type Output = M31Ext3;

    #[inline(always)]
    fn add(self, rhs: M31) -> Self::Output {
        self + M31Ext3::from(rhs)
    }
}

impl Neg for M31Ext3 {
    type Output = M31Ext3;
    #[inline(always)]
    fn neg(self) -> Self::Output {
        M31Ext3 {
            v: [-self.v[0], -self.v[1], -self.v[2]],
        }
    }
}

impl From<u32> for M31Ext3 {
    #[inline(always)]
    fn from(x: u32) -> Self {
        M31Ext3 {
            v: [M31::from(x), M31::zero(), M31::zero()],
        }
    }
}

impl M31Ext3 {
    #[inline(always)]
    pub fn to_base_field(&self) -> M31 {
        assert!(
            self.v[1].is_zero() && self.v[2].is_zero(),
            "M31Ext3 cannot be converted to base field"
        );

        self.to_base_field_unsafe()
    }

    #[inline(always)]
    pub fn to_base_field_unsafe(&self) -> M31 {
        self.v[0]
    }

    #[inline(always)]
    pub fn as_u32_array(&self) -> [u32; 3] {
        unsafe { transmute(self.v) }
    }
}

impl From<M31> for M31Ext3 {
    #[inline(always)]
    fn from(x: M31) -> Self {
        M31Ext3 {
            v: [x, M31::zero(), M31::zero()],
        }
    }
}

impl From<&M31> for M31Ext3 {
    #[inline(always)]
    fn from(x: &M31) -> Self {
        M31Ext3 {
            v: [*x, M31::zero(), M31::zero()],
        }
    }
}

impl From<M31Ext3> for M31 {
    #[inline(always)]
    fn from(x: M31Ext3) -> Self {
        x.to_base_field()
    }
}

impl From<&M31Ext3> for M31 {
    #[inline(always)]
    fn from(x: &M31Ext3) -> Self {
        x.to_base_field()
    }
}

#[inline(always)]
fn add_internal(a: &M31Ext3, b: &M31Ext3) -> M31Ext3 {
    let mut vv = a.v;
    vv[0] += b.v[0];
    vv[1] += b.v[1];
    vv[2] += b.v[2];

    M31Ext3 { v: vv }
}

#[inline(always)]
fn sub_internal(a: &M31Ext3, b: &M31Ext3) -> M31Ext3 {
    let mut vv = a.v;
    vv[0] -= b.v[0];
    vv[1] -= b.v[1];
    vv[2] -= b.v[2];

    M31Ext3 { v: vv }
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
fn mul_internal(a: &M31Ext3, b: &M31Ext3) -> M31Ext3 {
    let a = &a.v;
    let b = &b.v;
    let mut res = [M31::default(); 3];
    res[0] = a[0] * b[0] + M31 { v: 5 } * (a[1] * b[2] + a[2] * b[1]);
    res[1] = a[0] * b[1] + a[1] * b[0] + M31 { v: 5 } * a[2] * b[2];
    res[2] = a[0] * b[2] + a[1] * b[1] + a[2] * b[0];
    M31Ext3 { v: res }
}

#[inline(always)]
fn square_internal(a: &[M31; 3]) -> [M31; 3] {
    let mut res = [M31::default(); 3];
    res[0] = a[0].square() + M31 { v: 10 } * (a[1] * a[2]);
    res[1] = a[0] * a[1].double() + M31 { v: 5 } * a[2].square();
    res[2] = a[0] * a[2].double() + a[1].square();
    res
}
