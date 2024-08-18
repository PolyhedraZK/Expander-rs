use crate::field_common;

use crate::{Field, FieldSerde, SimdField, GF2_128};
use std::fmt::Debug;
use std::{
    arch::x86_64::*,
    iter::{Product, Sum},
    mem::transmute,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

#[derive(Clone, Copy)]
pub struct AVX512GF2_128x4 {
    data: __m512i,
}

field_common!(AVX512GF2_128x4);

impl AVX512GF2_128x4 {
    #[inline(always)]
    pub(crate) fn pack_full(data: __m128i) -> __m512i {
        unsafe { _mm512_broadcast_i32x4(data) }
    }
}

impl FieldSerde for AVX512GF2_128x4 {
    #[inline(always)]
    fn serialize_into<W: std::io::Write>(&self, mut writer: W) {
        unsafe {
            let mut data = [0u8; 64];
            _mm512_storeu_si512(data.as_mut_ptr() as *mut i32, self.data);
            writer.write_all(&data).unwrap();
        }
    }
    #[inline(always)]
    fn serialized_size() -> usize {
        512 / 8
    }
    #[inline(always)]
    fn deserialize_from<R: std::io::Read>(mut reader: R) -> Self {
        let mut data = [0u8; 64];
        reader.read_exact(&mut data).unwrap();
        unsafe {
            Self {
                data: _mm512_loadu_si512(data.as_ptr() as *const i32),
            }
        }
    }

    #[inline(always)]
    fn try_deserialize_from_ecc_format<R: std::io::Read>(
        mut reader: R,
    ) -> std::result::Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let mut buf = [0u8; 32];
        reader.read_exact(&mut buf)?;
        let data: __m128i = unsafe { _mm_loadu_si128(buf.as_ptr() as *const __m128i) };
        Ok(Self {
            data: Self::pack_full(data),
        })
    }
}
const PACKED_0: __m512i = unsafe { transmute([0; 16]) };
const PACKED_INV_2: __m512i = unsafe {
    transmute([
        67_u64,
        (1_u64) << 63,
        67_u64,
        (1_u64) << 63,
        67_u64,
        (1_u64) << 63,
        67_u64,
        (1_u64) << 63,
    ])
};

// p(x) = x^128 + x^7 + x^2 + x + 1
impl Field for AVX512GF2_128x4 {
    const NAME: &'static str = "AVX512 Galios Field 2^128";

    // size in bytes
    const SIZE: usize = 512 / 8;

    const ZERO: Self = Self { data: PACKED_0 };

    const INV_2: Self = Self { data: PACKED_INV_2 };

    const FIELD_SIZE: usize = 128;

    #[inline(always)]
    fn zero() -> Self {
        unsafe {
            let zero = _mm512_setzero_si512();
            Self { data: zero }
        }
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        unsafe {
            let zero = _mm512_setzero_si512();
            let cmp = _mm512_cmpeq_epi64_mask(self.data, zero);
            cmp == 0xFF // All 8 64-bit integers are equal (zero)
        }
    }

    #[inline(always)]
    fn one() -> Self {
        unsafe {
            let one = _mm512_set_epi64(0, 1, 0, 1, 0, 1, 0, 1);
            Self { data: one }
        }
    }

    #[inline(always)]
    fn random_unsafe(mut rng: impl rand::RngCore) -> Self {
        let data = unsafe {
            _mm512_set_epi64(
                rng.next_u64() as i64,
                rng.next_u64() as i64,
                rng.next_u64() as i64,
                rng.next_u64() as i64,
                rng.next_u64() as i64,
                rng.next_u64() as i64,
                rng.next_u64() as i64,
                rng.next_u64() as i64,
            )
        };
        Self { data }
    }

    #[inline(always)]
    fn random_bool(mut rng: impl rand::RngCore) -> Self {
        let data = unsafe {
            _mm512_set_epi64(
                0,
                (rng.next_u64() % 2) as i64,
                0,
                (rng.next_u64() % 2) as i64,
                0,
                (rng.next_u64() % 2) as i64,
                0,
                (rng.next_u64() % 2) as i64,
            )
        };
        Self { data }
    }

    #[inline(always)]
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

    #[inline(always)]
    fn inv(&self) -> Option<Self> {
        if self.is_zero() {
            return None;
        }
        let p_m2 = !(0u128) - 1;
        Some(Self::exp(self, p_m2))
    }

    #[inline(always)]
    fn as_u32_unchecked(&self) -> u32 {
        unimplemented!("self is a vector, cannot convert to u32")
    }

    #[inline(always)]
    fn from_uniform_bytes(_bytes: &[u8; 32]) -> Self {
        todo!()
    }

    #[inline(always)]
    fn square(&self) -> Self {
        *self * *self
    }

    #[inline(always)]
    fn double(&self) -> Self {
        Self::ZERO
    }

    #[inline(always)]
    fn mul_by_2(&self) -> Self {
        Self::ZERO
    }

    #[inline(always)]
    fn mul_by_3(&self) -> Self {
        *self
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
/*
credit to intel for the original implementation
void gfmul(__m128i a, __m128i b, __m128i *res) {
    __m128i tmp0, tmp1, tmp2, tmp3, tmp4, tmp5, tmp6;
    __m128i tmp7, tmp8, tmp9, tmp10, tmp11, tmp12;
    __m128i XMMMASK = _mm_setr_epi32(0xffffffff, 0x0, 0x0, 0x0);

    // a = a0|a1, b = b0|b1

    tmp3 = _mm_clmulepi64_si128(a, b, 0x00); // tmp3 = a0 * b0
    tmp6 = _mm_clmulepi64_si128(a, b, 0x11); // tmp6 = a1 * b1

    tmp4 = _mm_shuffle_epi32(a, 78); // tmp4 = a1|a0
    tmp5 = _mm_shuffle_epi32(b, 78); // tmp5 = b1|b0
    tmp4 = _mm_xor_si128(tmp4, a); // tmp4 = (a0 + a1) | (a0 + a1)
    tmp5 = _mm_xor_si128(tmp5, b); // tmp5 = (b0 + b1) | (b0 + b1)

    tmp4 = _mm_clmulepi64_si128(tmp4, tmp5, 0x00); // tmp4 = (a0 + a1) * (b0 + b1)
    tmp4 = _mm_xor_si128(tmp4, tmp3); // tmp4 = (a0 + a1) * (b0 + b1) - a0 * b0
    tmp4 = _mm_xor_si128(tmp4, tmp6); // tmp4 = (a0 + a1) * (b0 + b1) - a0 * b0 - a1 * b1 = a0 * b1 + a1 * b0

    tmp5 = _mm_slli_si128(tmp4, 8);
    tmp4 = _mm_srli_si128(tmp4, 8);
    tmp3 = _mm_xor_si128(tmp3, tmp5);
    tmp6 = _mm_xor_si128(tmp6, tmp4);

    tmp7 = _mm_srli_epi32(tmp6, 31);
    tmp8 = _mm_srli_epi32(tmp6, 30);
    tmp9 = _mm_srli_epi32(tmp6, 25);

    tmp7 = _mm_xor_si128(tmp7, tmp8);
    tmp7 = _mm_xor_si128(tmp7, tmp9);

    tmp8 = _mm_shuffle_epi32(tmp7, 147);
    tmp7 = _mm_and_si128(XMMMASK, tmp8);
    tmp8 = _mm_andnot_si128(XMMMASK, tmp8);

    tmp3 = _mm_xor_si128(tmp3, tmp8);
    tmp6 = _mm_xor_si128(tmp6, tmp7);

    tmp10 = _mm_slli_epi32(tmp6, 1);
    tmp3 = _mm_xor_si128(tmp3, tmp10);

    tmp11 = _mm_slli_epi32(tmp6, 2);
    tmp3 = _mm_xor_si128(tmp3, tmp11);

    tmp12 = _mm_slli_epi32(tmp6, 7);
    tmp3 = _mm_xor_si128(tmp3, tmp12);

    *res = _mm_xor_si128(tmp3, tmp6);
}

*/

/*
AVX 512 version
void gfmul_avx512(__m512i a, __m512i b, __m512i *res) {
    __m512i tmp0, tmp1, tmp2, tmp3, tmp4, tmp5, tmp6;
    __m512i tmp7, tmp8, tmp9, tmp10, tmp11, tmp12;
    __m512i XMMMASK = _mm512_set_epi32(
        0, 0, 0, 0xffffffff,
        0, 0, 0, 0xffffffff,
        0, 0, 0, 0xffffffff,
        0, 0, 0, 0xffffffff
    );

    tmp3 = _mm512_clmulepi64_epi128(a, b, 0x00);
    tmp6 = _mm512_clmulepi64_epi128(a, b, 0x11);

    tmp4 = _mm512_shuffle_epi32(a, _MM_PERM_BADC);
    tmp5 = _mm512_shuffle_epi32(b, _MM_PERM_BADC);
    tmp4 = _mm512_xor_si512(tmp4, a);
    tmp5 = _mm512_xor_si512(tmp5, b);

    tmp4 = _mm512_clmulepi64_epi128(tmp4, tmp5, 0x00);
    tmp4 = _mm512_xor_si512(tmp4, tmp3);
    tmp4 = _mm512_xor_si512(tmp4, tmp6);

    tmp5 = _mm512_bslli_epi128(tmp4, 8);
    tmp4 = _mm512_bsrli_epi128(tmp4, 8);
    tmp3 = _mm512_xor_si512(tmp3, tmp5);
    tmp6 = _mm512_xor_si512(tmp6, tmp4);

    tmp7 = _mm512_srli_epi32(tmp6, 31);
    tmp8 = _mm512_srli_epi32(tmp6, 30);
    tmp9 = _mm512_srli_epi32(tmp6, 25);

    tmp7 = _mm512_xor_si512(tmp7, tmp8);
    tmp7 = _mm512_xor_si512(tmp7, tmp9);

    tmp8 = _mm512_shuffle_epi32(tmp7, _MM_PERM_ABCD);
    tmp7 = _mm512_and_si512(XMMMASK, tmp8);
    tmp8 = _mm512_andnot_si512(XMMMASK, tmp8);

    tmp3 = _mm512_xor_si512(tmp3, tmp8);
    tmp6 = _mm512_xor_si512(tmp6, tmp7);

    tmp10 = _mm512_slli_epi32(tmp6, 1);
    tmp3 = _mm512_xor_si512(tmp3, tmp10);

    tmp11 = _mm512_slli_epi32(tmp6, 2);
    tmp3 = _mm512_xor_si512(tmp3, tmp11);

    tmp12 = _mm512_slli_epi32(tmp6, 7);
    tmp3 = _mm512_xor_si512(tmp3, tmp12);

    *res = _mm512_xor_si512(tmp3, tmp6);
}
 */

impl From<u32> for AVX512GF2_128x4 {
    #[inline(always)]
    fn from(v: u32) -> AVX512GF2_128x4 {
        assert!(v < 2); // only 0 and 1 are allowed
        let data = unsafe { _mm512_set_epi64(0, v as i64, 0, v as i64, 0, v as i64, 0, v as i64) };
        AVX512GF2_128x4 { data }
    }
}

impl Neg for AVX512GF2_128x4 {
    type Output = AVX512GF2_128x4;

    #[inline(always)]
    fn neg(self) -> AVX512GF2_128x4 {
        self
    }
}

impl Debug for AVX512GF2_128x4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut data = [0u8; 64];
        unsafe {
            _mm512_storeu_si512(data.as_mut_ptr() as *mut i32, self.data);
        }
        f.debug_struct("AVX512GF2_128x4")
            .field("data", &data)
            .finish()
    }
}

impl PartialEq for AVX512GF2_128x4 {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            let cmp = _mm512_cmpeq_epi64_mask(self.data, other.data);
            cmp == 0xFF // All 8 64-bit integers are equal
        }
    }
}

impl Default for AVX512GF2_128x4 {
    #[inline(always)]
    fn default() -> Self {
        Self::zero()
    }
}

impl From<GF2_128> for AVX512GF2_128x4 {
    #[inline(always)]
    fn from(v: GF2_128) -> AVX512GF2_128x4 {
        unsafe {
            let mut result = _mm512_setzero_si512(); // Initialize a zeroed _m512i
            result = _mm512_inserti32x4(result, v.v, 0); // Insert `a` at position 0
            result = _mm512_inserti32x4(result, v.v, 1); // Insert `b` at position 1
            result = _mm512_inserti32x4(result, v.v, 2); // Insert `c` at position 2
            result = _mm512_inserti32x4(result, v.v, 3); // Insert `d` at position 3
            AVX512GF2_128x4 { data: result }
        }
    }
}

impl SimdField for AVX512GF2_128x4 {
    type Scalar = GF2_128;

    fn from_scalar_array(_scalars: &[Self::Scalar]) -> Self {
        // Note: We don't necessarily need this function
        unimplemented!()
    }

    #[inline(always)]
    fn scale(&self, challenge: &Self::Scalar) -> Self {
        let simd_challenge = AVX512GF2_128x4::from(*challenge);
        *self * simd_challenge
    }
}

#[inline(always)]
fn add_internal(a: &AVX512GF2_128x4, b: &AVX512GF2_128x4) -> AVX512GF2_128x4 {
    unsafe {
        AVX512GF2_128x4 {
            data: _mm512_xor_si512(a.data, b.data),
        }
    }
}

#[inline(always)]
fn sub_internal(a: &AVX512GF2_128x4, b: &AVX512GF2_128x4) -> AVX512GF2_128x4 {
    unsafe {
        AVX512GF2_128x4 {
            data: _mm512_xor_si512(a.data, b.data),
        }
    }
}

#[inline(always)]
fn mul_internal(a: &AVX512GF2_128x4, b: &AVX512GF2_128x4) -> AVX512GF2_128x4 {
    unsafe {
        let xmmmask = _mm512_set_epi32(
            0,
            0,
            0,
            0xffffffffu32 as i32,
            0,
            0,
            0,
            0xffffffffu32 as i32,
            0,
            0,
            0,
            0xffffffffu32 as i32,
            0,
            0,
            0,
            0xffffffffu32 as i32,
        );

        let mut tmp3 = _mm512_clmulepi64_epi128(a.data, b.data, 0x00);
        let mut tmp6 = _mm512_clmulepi64_epi128(a.data, b.data, 0x11);

        let mut tmp4 = _mm512_shuffle_epi32(a.data, _MM_PERM_BADC);
        let mut tmp5 = _mm512_shuffle_epi32(b.data, _MM_PERM_BADC);
        tmp4 = _mm512_xor_si512(tmp4, a.data);
        tmp5 = _mm512_xor_si512(tmp5, b.data);

        tmp4 = _mm512_clmulepi64_epi128(tmp4, tmp5, 0x00);
        tmp4 = _mm512_xor_si512(tmp4, tmp3);
        tmp4 = _mm512_xor_si512(tmp4, tmp6);

        tmp5 = _mm512_bslli_epi128(tmp4, 8);
        tmp4 = _mm512_bsrli_epi128(tmp4, 8);
        tmp3 = _mm512_xor_si512(tmp3, tmp5);
        tmp6 = _mm512_xor_si512(tmp6, tmp4);

        let tmp7 = _mm512_srli_epi32(tmp6, 31);
        let tmp8 = _mm512_srli_epi32(tmp6, 30);
        let tmp9 = _mm512_srli_epi32(tmp6, 25);

        let mut tmp7 = _mm512_xor_si512(tmp7, tmp8);
        tmp7 = _mm512_xor_si512(tmp7, tmp9);

        let mut tmp8 = _mm512_shuffle_epi32(tmp7, _MM_PERM_CBAD);
        tmp7 = _mm512_and_si512(xmmmask, tmp8);
        tmp8 = _mm512_andnot_si512(xmmmask, tmp8);

        tmp3 = _mm512_xor_si512(tmp3, tmp8);
        tmp6 = _mm512_xor_si512(tmp6, tmp7);

        let tmp10 = _mm512_slli_epi32(tmp6, 1);
        tmp3 = _mm512_xor_si512(tmp3, tmp10);

        let tmp11 = _mm512_slli_epi32(tmp6, 2);
        tmp3 = _mm512_xor_si512(tmp3, tmp11);

        let tmp12 = _mm512_slli_epi32(tmp6, 7);
        tmp3 = _mm512_xor_si512(tmp3, tmp12);

        let result = _mm512_xor_si512(tmp3, tmp6);

        AVX512GF2_128x4 { data: result }
    }
}
