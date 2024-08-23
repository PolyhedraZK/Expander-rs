use std::iter::{Product, Sum};
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use std::{arch::aarch64::*, mem::transmute};

use crate::{field_common, BinomialExtensionField, Field, FieldSerde, GF2};

#[derive(Clone, Copy, Debug)]
pub struct NeonGF2_128 {
    pub(crate) v: uint32x4_t,
}

field_common!(NeonGF2_128);

#[inline(always)]
fn add_internal(a: &NeonGF2_128, b: &NeonGF2_128) -> NeonGF2_128 {
    NeonGF2_128 {
        v: unsafe { gfadd(a.v, b.v) },
    }
}

#[inline(always)]
fn mul_internal(a: &NeonGF2_128, b: &NeonGF2_128) -> NeonGF2_128 {
    NeonGF2_128 {
        v: unsafe { gfmul(a.v, b.v) },
    }
}

#[inline(always)]
fn sub_internal(a: &NeonGF2_128, b: &NeonGF2_128) -> NeonGF2_128 {
    add_internal(a, b)
}

impl FieldSerde for NeonGF2_128 {
    #[inline(always)]
    fn serialize_into<W: std::io::Write>(&self, mut writer: W) {
        unsafe {
            writer
                .write_all(transmute::<uint32x4_t, [u8; 16]>(self.v).as_ref())
                .unwrap(); // todo: error propagation
        }
    }

    #[inline(always)]
    fn serialized_size() -> usize {
        16
    }

    #[inline(always)]
    fn deserialize_from<R: std::io::Read>(mut reader: R) -> Self {
        let mut u = [0u8; 16];
        reader.read_exact(&mut u).unwrap(); // todo: error propagation
        unsafe {
            NeonGF2_128 {
                v: transmute::<[u8; 16], uint32x4_t>(u),
            }
        }
    }

    #[inline]
    fn try_deserialize_from_ecc_format<R: std::io::Read>(
        mut reader: R,
    ) -> std::result::Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

impl Field for NeonGF2_128 {
    const NAME: &'static str = "Galios Field 2^128";
    const SIZE: usize = 128 / 8;
    const FIELD_SIZE: usize = 128; // in bits

    const ZERO: Self = NeonGF2_128 {
        v: unsafe { std::mem::zeroed() },
    };

    const INV_2: Self = NeonGF2_128 {
        v: unsafe { std::mem::zeroed() },
    }; // should not be used

    #[inline(always)]
    fn zero() -> Self {
        NeonGF2_128 {
            v: unsafe { std::mem::zeroed() },
        }
    }

    #[inline(always)]
    fn one() -> Self {
        NeonGF2_128 {
            // 1 in the first bit
            v: unsafe { transmute::<[u32; 4], uint32x4_t>([1, 0, 0, 0]) }, // TODO check bit order
        }
    }

    #[inline(always)]
    fn random_unsafe(mut rng: impl rand::RngCore) -> Self {
        let mut u = [0u8; 16];
        rng.fill_bytes(&mut u);
        unsafe {
            NeonGF2_128 {
                v: transmute::<[u8; 16], uint32x4_t>(u),
            }
        }
    }

    #[inline(always)]
    fn random_bool(mut rng: impl rand::RngCore) -> Self {
        NeonGF2_128 {
            v: unsafe { transmute::<[u32; 4], uint32x4_t>([rng.next_u32() % 2, 0, 0, 0]) },
        }
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        unsafe { transmute::<uint32x4_t, [u8; 16]>(self.v) == [0; 16] }
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
        if self.is_zero() {
            return None;
        }
        let p_m2 = !(0u128) - 1;
        Some(Self::exp(self, p_m2))
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
    fn from_uniform_bytes(bytes: &[u8; 32]) -> Self {
        unsafe {
            NeonGF2_128 {
                v: transmute::<[u8; 16], uint32x4_t>(bytes[..16].try_into().unwrap()),
            }
        }
    }
}

impl BinomialExtensionField for NeonGF2_128 {
    const DEGREE: usize = 128;
    const W: u32 = 0x87;

    type BaseField = GF2;

    #[inline(always)]
    fn mul_by_base_field(&self, base: &Self::BaseField) -> Self {
        if base.v == 0 {
            Self::zero()
        } else {
            *self
        }
    }

    #[inline(always)]
    fn add_by_base_field(&self, _base: &Self::BaseField) -> Self {
        todo!()
        // let mut res = *self;
        // res.v = unsafe { _mm_xor_si128(res.v, _mm_set_epi64x(0, base.v as i64)) };
        // res
    }

    #[inline(always)]
    fn first_base_field(&self) -> Self::BaseField {
        todo!()
        // // but this doesn't make sense for NeonGF2_128
        // let v = unsafe { _mm_extract_epi64(self.v, 0) };
        // GF2 { v: v as u8 }
    }
}

impl From<GF2> for NeonGF2_128 {
    #[inline(always)]
    fn from(v: GF2) -> Self {
        match v.v {
            0 => Self::zero(),
            1 => Self::one(),
            _ => panic!("Invalid value for GF2"),
        }
    }
}

impl Default for NeonGF2_128 {
    #[inline(always)]
    fn default() -> Self {
        Self::zero()
    }
}

impl PartialEq for NeonGF2_128 {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            transmute::<uint32x4_t, [u8; 16]>(self.v) == transmute::<uint32x4_t, [u8; 16]>(other.v)
        }
    }
}

impl Neg for NeonGF2_128 {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self {
        self
    }
}

impl From<u32> for NeonGF2_128 {
    #[inline(always)]
    fn from(v: u32) -> Self {
        NeonGF2_128 {
            v: unsafe { transmute::<[u32; 4], uint32x4_t>([v, 0, 0, 0]) },
        }
    }
}

// multiply the polynomial by x^32, without reducing the irreducible polynomial
// equivalent to _mm_shuffle_epi32(a, 147)
// TODO: Is there an instruction for this?
#[inline(always)]
unsafe fn cyclic_rotate_1(input: uint32x4_t) -> uint32x4_t {
    let [a, b, c, d] = transmute::<uint32x4_t, [u32; 4]>(input);
    transmute([d, a, b, c])
}

// multiply the polynomial by x^64, without reducing the irreducible polynomial
// equivalent to _mm_shuffle_epi32(a, 78)
// TODO: Is there an instruction for this?
#[inline(always)]
unsafe fn cyclic_rotate_2(input: uint32x4_t) -> uint32x4_t {
    let [a, b, c, d] = transmute::<uint32x4_t, [u32; 4]>(input);
    transmute([c, d, a, b])
}

#[inline(always)]
pub(crate) unsafe fn gfadd(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    veorq_u32(a, b)
}

#[inline]
pub(crate) unsafe fn gfmul(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    let xmm_mask = transmute::<[u32; 4], uint32x4_t>([0xffffffffu32, 0, 0, 0]);

    // case a and b as u64 vectors
    // a = a0|a1, b = b0|b1
    let a64 = vreinterpretq_u64_u32(a);
    let b64 = vreinterpretq_u64_u32(b);

    // =========================================
    // step 1: compute a0 * b0, a1 * b1, and (a0 * b1 + a1 * b0)
    // =========================================

    // tmp3 = a0 * b0
    let tmp3 = transmute::<u128, uint64x2_t>(vmull_p64(
        transmute::<uint64x1_t, u64>(vget_low_u64(a64)),
        transmute::<uint64x1_t, u64>(vget_low_u64(b64)),
    ));
    // tmp6 = a1 * b1
    let tmp6 = transmute::<u128, uint64x2_t>(vmull_p64(
        transmute::<uint64x1_t, u64>(vget_high_u64(a64)),
        transmute::<uint64x1_t, u64>(vget_high_u64(b64)),
    ));

    // shuffle the lanes, i.e., multiply by x^64
    let tmp4 = cyclic_rotate_2(a);
    let tmp5 = cyclic_rotate_2(b);

    // tmp4 = (a0 + a1) | (a0 + a1)
    let tmp4 = veorq_u32(tmp4, a);
    // tmp5 = (b0 + b1) | (b0 + b1)
    let tmp5 = veorq_u32(tmp5, b);

    // tmp4 = (a0 + a1) * (b0 + b1)
    let tmp4_64 = transmute::<u128, uint64x2_t>(vmull_p64(
        transmute::<uint32x2_t, u64>(vget_low_u32(tmp4)),
        transmute::<uint32x2_t, u64>(vget_low_u32(tmp5)),
    ));

    // tmp4 = (a0 + a1) * (b0 + b1) - a0 * b0
    let tmp4_64 = veorq_u64(tmp4_64, tmp3);

    // tmp4 = (a0 + a1) * (b0 + b1) - a0 * b0 - a1 * b1 = a0 * b1 + a1 * b0
    let tmp4_64 = veorq_u64(tmp4_64, tmp6);

    // =========================================
    // step 2: mod reductions
    // =========================================

    // tmp5_shifted_left = (a0 * b1) << 64
    // TODO: is there a better way to do this?
    let tmp5_shifted_left =
        transmute::<u128, uint64x2_t>(transmute::<uint64x2_t, u128>(tmp4_64) << 64);
    // tmp4_64 = (a0 * b1) >> 64
    // TODO: is there a better way to do this?
    let tmp4_64 = transmute::<u128, uint64x2_t>(transmute::<uint64x2_t, u128>(tmp4_64) >> 64);
    // tmp3 = a0 * b0 xor ((a0 * b1) << 64), i.e., low 128 coeff of the poly
    let tmp3 = veorq_u64(tmp3, tmp5_shifted_left);
    // tmp6 = a1 * b1 xor ((a0 * b1) >> 64), i.e., high 128 coeff of the poly
    let tmp6 = veorq_u64(tmp6, tmp4_64);

    // performs necessary shifts as per the avx code
    // 31, 30, 25 as reflecting the non-zero entries of the irreducible polynomial
    let tmp7 = vshrq_n_u32(vreinterpretq_u32_u64(tmp6), 31);
    let tmp8 = vshrq_n_u32(vreinterpretq_u32_u64(tmp6), 30);
    let tmp9 = vshrq_n_u32(vreinterpretq_u32_u64(tmp6), 25);

    // xor all the shifted values
    let tmp7 = veorq_u32(tmp7, tmp8);
    let tmp7 = veorq_u32(tmp7, tmp9);

    // shuffle the lanes, i.e., multiply by x^32
    let tmp8 = cyclic_rotate_1(tmp7);

    let tmp7 = vandq_u32(tmp8, xmm_mask);
    let tmp8 = vbicq_u32(tmp8, xmm_mask);

    // tmp3 has the low 128 bits of the polynomial
    // tmp6 has the high 128 bits of the polynomial
    // now we perform the mod reduction and put the result back to tmp3
    let tmp3 = veorq_u64(tmp3, vreinterpretq_u64_u32(tmp8));
    let tmp6 = veorq_u64(tmp6, vreinterpretq_u64_u32(tmp7));

    let tmp10 = vshlq_n_u32(vreinterpretq_u32_u64(tmp6), 1);
    let tmp3 = veorq_u64(tmp3, vreinterpretq_u64_u32(tmp10));

    let tmp11 = vshlq_n_u32(vreinterpretq_u32_u64(tmp6), 2);
    let tmp3 = veorq_u64(tmp3, vreinterpretq_u64_u32(tmp11));

    let tmp12 = vshlq_n_u32(vreinterpretq_u32_u64(tmp6), 7);
    let tmp3 = veorq_u64(tmp3, vreinterpretq_u64_u32(tmp12));

    vreinterpretq_u32_u64(veorq_u64(tmp3, tmp6))
}
