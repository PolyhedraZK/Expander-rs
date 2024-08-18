use std::arch::aarch64::uint32x4_t;
use std::iter::{Product, Sum};
use std::mem::transmute;
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::SimdField;
use crate::{
    field_common,
    neon::{gfadd, gfmul, NeonGF2_128},
    Field, FieldSerde,
};

#[derive(Clone, Copy, Debug)]
pub struct NeonGF2_128x4 {
    v: [uint32x4_t; 4],
}

field_common!(NeonGF2_128x4);

impl Default for NeonGF2_128x4 {
    fn default() -> Self {
        Self::zero()
    }
}

impl PartialEq for NeonGF2_128x4 {
    fn eq(&self, other: &Self) -> bool {
        self.v.iter().zip(other.v.iter()).all(|(a, b)| unsafe {
            transmute::<uint32x4_t, [u8; 16]>(*a) == transmute::<uint32x4_t, [u8; 16]>(*b)
        })
    }
}

impl FieldSerde for NeonGF2_128x4 {
    #[inline(always)]
    fn serialize_into<W: std::io::Write>(&self, mut writer: W) {
        self.v.iter().for_each(|&vv| {
            writer
                .write_all(unsafe { transmute::<uint32x4_t, [u8; 16]>(vv) }.as_ref())
                .unwrap()
        })
    }

    #[inline(always)]
    fn serialized_size() -> usize {
        16 * 4
    }

    #[inline(always)]
    fn deserialize_from<R: std::io::Read>(mut reader: R) -> Self {
        let mut res = Self::zero();
        res.v.iter_mut().for_each(|vv| {
            let mut u = [0u8; 16];
            reader.read_exact(&mut u).unwrap();
            *vv = unsafe { transmute::<[u8; 16], uint32x4_t>(u) }
        });
        res
    }

    #[inline]
    fn try_deserialize_from_ecc_format<R: std::io::Read>(
        mut reader: R,
    ) -> std::result::Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let mut res = Self::zero();
        res.v.iter_mut().for_each(|vv| {
            let mut u = [0u8; 32];
            reader.read_exact(&mut u).unwrap();
            *vv = unsafe { transmute::<[u8; 16], uint32x4_t>(u[..16].try_into().unwrap()) };
        });
        Ok(res)
    }
}

impl Field for NeonGF2_128x4 {
    const NAME: &'static str = "Neon Galios Field 2 128x4";

    const SIZE: usize = 16 * 4;

    const FIELD_SIZE: usize = 128 * 4; // in bits

    const ZERO: Self = NeonGF2_128x4 {
        v: [unsafe { transmute::<[u32; 4], uint32x4_t>([0, 0, 0, 0]) }; 4],
    };

    const INV_2: Self = NeonGF2_128x4 {
        v: [unsafe { transmute::<[u32; 4], uint32x4_t>([0, 0, 0, 0]) }; 4],
    }; // should not be used

    #[inline(always)]
    fn zero() -> Self {
        NeonGF2_128x4 {
            v: [unsafe { transmute::<[u32; 4], uint32x4_t>([0, 0, 0, 0]) }; 4],
        }
    }

    #[inline(always)]
    fn one() -> Self {
        NeonGF2_128x4 {
            v: [unsafe { transmute::<[u32; 4], uint32x4_t>([1, 0, 0, 0]) }; 4],
        }
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        self.v
            .iter()
            .all(|vv| unsafe { transmute::<uint32x4_t, [u8; 16]>(*vv) == [0; 16] })
    }

    #[inline(always)]
    fn random_unsafe(mut rng: impl rand::RngCore) -> Self {
        NeonGF2_128x4 {
            v: [
                unsafe { transmute::<[u64; 2], uint32x4_t>([rng.next_u64(), rng.next_u64()]) },
                unsafe { transmute::<[u64; 2], uint32x4_t>([rng.next_u64(), rng.next_u64()]) },
                unsafe { transmute::<[u64; 2], uint32x4_t>([rng.next_u64(), rng.next_u64()]) },
                unsafe { transmute::<[u64; 2], uint32x4_t>([rng.next_u64(), rng.next_u64()]) },
            ],
        }
    }

    #[inline(always)]
    fn random_bool(mut rng: impl rand::RngCore) -> Self {
        NeonGF2_128x4 {
            v: [
                unsafe { transmute::<[u32; 4], uint32x4_t>([rng.next_u32() % 2, 0, 0, 0]) },
                unsafe { transmute::<[u32; 4], uint32x4_t>([rng.next_u32() % 2, 0, 0, 0]) },
                unsafe { transmute::<[u32; 4], uint32x4_t>([rng.next_u32() % 2, 0, 0, 0]) },
                unsafe { transmute::<[u32; 4], uint32x4_t>([rng.next_u32() % 2, 0, 0, 0]) },
            ],
        }
    }

    #[inline(always)]
    fn exp(&self, exponent: u128) -> Self {
        let mut e = exponent;
        let mut res = Self::one();
        let mut t = *self;
        while e > 0 {
            if e & 1 == 1 {
                res *= t;
            }
            t = t * t;
            e >>= 1;
        }
        res
    }

    #[inline(always)]
    fn inv(&self) -> Option<Self> {
        todo!()
    }

    #[inline(always)]
    fn square(&self) -> Self {
        self * self
    }

    #[inline(always)]
    fn as_u32_unchecked(&self) -> u32 {
        unimplemented!("u32 for GF128 doesn't make sense")
    }

    #[inline(always)]
    fn from_uniform_bytes(_bytes: &[u8; 32]) -> Self {
        unimplemented!("from_uniform_bytes for GF128 doesn't make sense")
    }
}

impl SimdField for NeonGF2_128x4 {
    type Scalar = NeonGF2_128;

    const SIMD_SIZE: usize = 4;

    fn from_scalar_array(_scalars: &[Self::Scalar]) -> Self {
        // Note: We don't necessarily need this function
        unimplemented!()
    }

    #[inline(always)]
    fn scale(&self, challenge: &Self::Scalar) -> Self {
        NeonGF2_128x4 {
            v: [
                unsafe { gfmul(self.v[0], challenge.v) },
                unsafe { gfmul(self.v[1], challenge.v) },
                unsafe { gfmul(self.v[2], challenge.v) },
                unsafe { gfmul(self.v[3], challenge.v) },
            ],
        }
    }
}

impl From<NeonGF2_128> for NeonGF2_128x4 {
    fn from(v: NeonGF2_128) -> Self {
        unsafe {
            NeonGF2_128x4 {
                v: [
                    v.v,
                    transmute::<u128, uint32x4_t>(0u128),
                    transmute::<u128, uint32x4_t>(0u128),
                    transmute::<u128, uint32x4_t>(0u128),
                ],
            }
        }
    }
}

impl Neg for NeonGF2_128x4 {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self::Output {
        self
    }
}

impl From<u32> for NeonGF2_128x4 {
    fn from(v: u32) -> Self {
        NeonGF2_128x4 {
            v: [
                unsafe { transmute::<[u32; 4], uint32x4_t>([v, 0, 0, 0]) },
                unsafe { transmute::<[u32; 4], uint32x4_t>([0, 0, 0, 0]) },
                unsafe { transmute::<[u32; 4], uint32x4_t>([0, 0, 0, 0]) },
                unsafe { transmute::<[u32; 4], uint32x4_t>([0, 0, 0, 0]) },
            ],
        }
    }
}

#[inline(always)]
fn add_internal(a: &NeonGF2_128x4, b: &NeonGF2_128x4) -> NeonGF2_128x4 {
    NeonGF2_128x4 {
        v: [
            unsafe { gfadd(a.v[0], b.v[0]) },
            unsafe { gfadd(a.v[1], b.v[1]) },
            unsafe { gfadd(a.v[2], b.v[2]) },
            unsafe { gfadd(a.v[3], b.v[3]) },
        ],
    }
}

#[inline(always)]
fn sub_internal(a: &NeonGF2_128x4, b: &NeonGF2_128x4) -> NeonGF2_128x4 {
    add_internal(a, b)
}

#[inline(always)]
fn mul_internal(a: &NeonGF2_128x4, b: &NeonGF2_128x4) -> NeonGF2_128x4 {
    NeonGF2_128x4 {
        v: [
            unsafe { gfmul(a.v[0], b.v[0]) },
            unsafe { gfmul(a.v[1], b.v[1]) },
            unsafe { gfmul(a.v[2], b.v[2]) },
            unsafe { gfmul(a.v[3], b.v[3]) },
        ],
    }
}
