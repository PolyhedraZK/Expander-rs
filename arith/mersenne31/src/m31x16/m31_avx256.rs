use std::{
    arch::x86_64::*,
    fmt::Debug,
    io::{Read, Write},
    iter::{Product, Sum},
    mem::transmute,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use arith::{field_common, Field, FieldSerde, FieldSerdeResult, SimdField};
use rand::{Rng, RngCore};

use crate::m31::{M31, M31_MOD};

const M31_PACK_SIZE: usize = 16;
const PACKED_MOD: __m256i = unsafe { transmute([M31_MOD; M31_PACK_SIZE / 2]) };
const PACKED_0: __m256i = unsafe { transmute([0; M31_PACK_SIZE / 2]) };
const PACKED_INV_2: __m256i = unsafe { transmute([1 << 30; M31_PACK_SIZE / 2]) };

#[inline(always)]
unsafe fn mod_reduce_epi32(x: __m256i) -> __m256i {
    _mm256_add_epi32(_mm256_and_si256(x, PACKED_MOD), _mm256_srli_epi32(x, 31))
}

#[inline(always)]
unsafe fn mod_reduce_epi32_2(x: [__m256i; 2]) -> [__m256i; 2] {
    [mod_reduce_epi32(x[0]), mod_reduce_epi32(x[1])]
}

#[derive(Clone, Copy)]
pub struct AVXM31 {
    pub v: [__m256i; 2],
}

impl AVXM31 {
    #[inline(always)]
    pub(crate) fn pack_full(x: M31) -> AVXM31 {
        AVXM31 {
            v: unsafe { [_mm256_set1_epi32(x.v as i32), _mm256_set1_epi32(x.v as i32)] },
        }
    }
}

field_common!(AVXM31);

impl FieldSerde for AVXM31 {
    const SERIALIZED_SIZE: usize = 512 / 8;

    #[inline(always)]
    /// serialize self into bytes
    fn serialize_into<W: Write>(&self, mut writer: W) -> FieldSerdeResult<()> {
        let data = unsafe { transmute::<[__m256i; 2], [u8; 64]>(self.v) };
        writer.write_all(&data)?;
        Ok(())
    }

    /// deserialize bytes into field
    #[inline(always)]
    fn deserialize_from<R: Read>(mut reader: R) -> FieldSerdeResult<Self> {
        let mut data = [0; Self::SERIALIZED_SIZE];
        reader.read_exact(&mut data)?;
        unsafe {
            let mut value = transmute::<[u8; Self::SERIALIZED_SIZE], [__m256i; 2]>(data);
            value = mod_reduce_epi32_2(value);
            Ok(AVXM31 { v: value })
        }
    }

    #[inline(always)]
    fn try_deserialize_from_ecc_format<R: Read>(mut reader: R) -> FieldSerdeResult<Self> {
        let mut buf = [0u8; 32];
        reader.read_exact(&mut buf)?;
        assert!(
            buf.iter().skip(4).all(|&x| x == 0),
            "non-zero byte found in witness byte"
        );
        Ok(Self::pack_full(
            u32::from_le_bytes(buf[..4].try_into().unwrap()).into(),
        ))
    }
}

impl Field for AVXM31 {
    const NAME: &'static str = "AVX Packed Mersenne 31";

    // size in bytes
    const SIZE: usize = 512 / 8;

    const ZERO: Self = Self {
        v: [PACKED_0, PACKED_0],
    };

    const ONE: Self = Self {
        v: unsafe { transmute::<[u32; 16], [__m256i; 2]>([1; M31_PACK_SIZE]) },
    };

    const INV_2: Self = Self {
        v: [PACKED_INV_2, PACKED_INV_2],
    };

    const FIELD_SIZE: usize = 32;

    #[inline(always)]
    fn zero() -> Self {
        AVXM31 {
            v: unsafe { [_mm256_set1_epi32(0), _mm256_set1_epi32(0)] },
        }
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        // value is either zero or 0x7FFFFFFF
        unsafe {
            let cmp0 = _mm256_movemask_epi8(_mm256_cmpeq_epi32(self.v[0], PACKED_0));
            let cmp1 = _mm256_movemask_epi8(_mm256_cmpeq_epi32(self.v[1], PACKED_0));
            let cmp2 = _mm256_movemask_epi8(_mm256_cmpeq_epi32(self.v[0], PACKED_MOD));
            let cmp3 = _mm256_movemask_epi8(_mm256_cmpeq_epi32(self.v[1], PACKED_MOD));
            (cmp0 | cmp2) == !0i32 && (cmp1 | cmp3) == !0i32
        }
    }

    #[inline(always)]
    fn one() -> Self {
        AVXM31 {
            v: unsafe { [_mm256_set1_epi32(1), _mm256_set1_epi32(1)] },
        }
    }

    #[inline(always)]
    // this function is for internal testing only. it is not
    // a source for uniformly random field elements and
    // should not be used in production.
    fn random_unsafe(mut rng: impl RngCore) -> Self {
        // Caution: this may not produce uniformly random elements
        unsafe {
            let mut v = [
                _mm256_setr_epi32(
                    rng.gen::<i32>(),
                    rng.gen::<i32>(),
                    rng.gen::<i32>(),
                    rng.gen::<i32>(),
                    rng.gen::<i32>(),
                    rng.gen::<i32>(),
                    rng.gen::<i32>(),
                    rng.gen::<i32>(),
                ),
                _mm256_setr_epi32(
                    rng.gen::<i32>(),
                    rng.gen::<i32>(),
                    rng.gen::<i32>(),
                    rng.gen::<i32>(),
                    rng.gen::<i32>(),
                    rng.gen::<i32>(),
                    rng.gen::<i32>(),
                    rng.gen::<i32>(),
                ),
            ];
            v = mod_reduce_epi32_2(v);
            v = mod_reduce_epi32_2(v);
            AVXM31 { v }
        }
    }

    #[inline(always)]
    // modified
    fn random_bool(mut rng: impl RngCore) -> Self {
        // TODO: optimize this code
        AVXM31 {
            v: unsafe {
                [
                    _mm256_setr_epi32(
                        rng.gen::<bool>() as i32,
                        rng.gen::<bool>() as i32,
                        rng.gen::<bool>() as i32,
                        rng.gen::<bool>() as i32,
                        rng.gen::<bool>() as i32,
                        rng.gen::<bool>() as i32,
                        rng.gen::<bool>() as i32,
                        rng.gen::<bool>() as i32,
                    ),
                    _mm256_setr_epi32(
                        rng.gen::<bool>() as i32,
                        rng.gen::<bool>() as i32,
                        rng.gen::<bool>() as i32,
                        rng.gen::<bool>() as i32,
                        rng.gen::<bool>() as i32,
                        rng.gen::<bool>() as i32,
                        rng.gen::<bool>() as i32,
                        rng.gen::<bool>() as i32,
                    ),
                ]
            },
        }
    }

    fn exp(&self, _exponent: u128) -> Self {
        unimplemented!("exp not implemented for AVXM31")
    }

    #[inline(always)]
    fn double(&self) -> Self {
        self.mul_by_2()
    }

    #[inline(always)]
    // modified
    fn mul_by_5(&self) -> AVXM31 {
        let double = unsafe {
            mod_reduce_epi32_2([
                _mm256_slli_epi32::<1>(self.v[0]),
                _mm256_slli_epi32::<1>(self.v[1]),
            ])
        };
        let quad = unsafe {
            mod_reduce_epi32_2([
                _mm256_slli_epi32::<1>(double[0]),
                _mm256_slli_epi32::<1>(double[1]),
            ])
        };
        let res = unsafe {
            mod_reduce_epi32_2([
                _mm256_add_epi32(self.v[0], quad[0]),
                _mm256_add_epi32(self.v[1], quad[1]),
            ])
        };
        Self { v: res }
    }

    #[inline(always)]
    fn inv(&self) -> Option<Self> {
        // slow, should not be used in production
        let mut m31_vec = unsafe { transmute::<[__m256i; 2], [M31; 16]>(self.v) };
        let is_non_zero = m31_vec.iter().all(|x| !x.is_zero());
        if !is_non_zero {
            return None;
        }

        m31_vec.iter_mut().for_each(|x| *x = x.inv().unwrap()); // safe unwrap
        Some(Self {
            v: unsafe { transmute::<[M31; 16], [__m256i; 2]>(m31_vec) },
        })
    }

    fn as_u32_unchecked(&self) -> u32 {
        unimplemented!("self is a vector, cannot convert to u32")
    }

    #[inline]
    fn from_uniform_bytes(bytes: &[u8; 32]) -> Self {
        let m = M31::from_uniform_bytes(bytes);
        Self {
            v: unsafe { [_mm256_set1_epi32(m.v as i32), _mm256_set1_epi32(m.v as i32)] },
        }
    }

    #[inline(always)]
    fn mul_by_3(&self) -> AVXM31 {
        let double = unsafe {
            mod_reduce_epi32_2([
                _mm256_slli_epi32::<1>(self.v[0]),
                _mm256_slli_epi32::<1>(self.v[1]),
            ])
        };
        let res = unsafe {
            mod_reduce_epi32_2([
                _mm256_add_epi32(self.v[0], double[0]),
                _mm256_add_epi32(self.v[1], double[1]),
            ])
        };
        Self { v: res }
    }
}

impl SimdField for AVXM31 {
    type Scalar = M31;

    #[inline]
    fn scale(&self, challenge: &Self::Scalar) -> Self {
        *self * *challenge
    }

    #[inline(always)]
    fn pack_size() -> usize {
        M31_PACK_SIZE
    }

    fn pack(base_vec: &[Self::Scalar]) -> Self {
        assert_eq!(base_vec.len(), M31_PACK_SIZE);
        let ret: [Self::Scalar; M31_PACK_SIZE] = base_vec.try_into().unwrap();
        unsafe { transmute(ret) }
    }

    fn unpack(&self) -> Vec<Self::Scalar> {
        let ret = unsafe { transmute::<[__m256i; 2], [Self::Scalar; M31_PACK_SIZE]>(self.v) };
        ret.to_vec()
    }
}

impl From<M31> for AVXM31 {
    #[inline(always)]
    fn from(x: M31) -> Self {
        AVXM31::pack_full(x)
    }
}

impl Debug for AVXM31 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut data = [0; M31_PACK_SIZE];
        unsafe {
            _mm256_storeu_si256(data.as_mut_ptr() as *mut __m256i, self.v[0]);
            _mm256_storeu_si256(data.as_mut_ptr().add(8) as *mut __m256i, self.v[1]);
        }
        // if all data is the same, print only one
        if data.iter().all(|&x| x == data[0]) {
            write!(
                f,
                "mm256i<8 x {}>",
                if M31_MOD - data[0] > 1024 {
                    format!("{}", data[0])
                } else {
                    format!("-{}", M31_MOD - data[0])
                }
            )
        } else {
            write!(f, "mm256i<{:?}>", data)
        }
    }
}

impl Default for AVXM31 {
    fn default() -> Self {
        AVXM31::zero()
    }
}

impl PartialEq for AVXM31 {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            let cmp0 = _mm256_movemask_epi8(_mm256_cmpeq_epi8(self.v[0], other.v[0]));
            let cmp1 = _mm256_movemask_epi8(_mm256_cmpeq_epi8(self.v[1], other.v[1]));
            (cmp0 & cmp1) == !0i32
        }
    }
}

#[inline]
#[must_use]
fn movehdup_epi32(a: __m256i) -> __m256i {
    // The instruction is only available in the floating-point flavor; this distinction is only for
    // historical reasons and no longer matters. We cast to floats, do the thing, and cast back.
    unsafe {
        let a = _mm256_castsi256_ps(a);
        _mm256_castps_si256(_mm256_movehdup_ps(a))
    }
}

#[inline]
#[must_use]
fn moveldup_epi32(a: __m256i) -> __m256i {
    // The instruction is only available in the floating-point flavor; this distinction is only for
    // historical reasons and no longer matters. We cast to floats, do the thing, and cast back.
    unsafe {
        let a = _mm256_castsi256_ps(a);
        _mm256_castps_si256(_mm256_moveldup_ps(a))
    }
}

#[inline]
#[must_use]
fn add(lhs: __m256i, rhs: __m256i) -> __m256i {
    unsafe {
        let t = _mm256_add_epi32(lhs, rhs);
        let u = _mm256_sub_epi32(t, PACKED_MOD);
        _mm256_min_epu32(t, u)
    }
}

const EVENS: i32 = 0b01010101;
const ODDS: i32 = 0b10101010;

impl Mul<&M31> for AVXM31 {
    type Output = AVXM31;

    #[inline(always)]
    fn mul(self, rhs: &M31) -> Self::Output {
        let rhsv = AVXM31::pack_full(*rhs);
        unsafe {
            let mut res: [__m256i; 2] = [_mm256_setzero_si256(); 2];
            #[allow(clippy::needless_range_loop)]
            for i in 0..res.len() {
                let rhs_evn = rhsv.v[i];
                let lhs_odd_dbl = _mm256_srli_epi64(self.v[i], 31);
                let lhs_evn_dbl = _mm256_add_epi32(self.v[i], self.v[i]);
                let rhs_odd = movehdup_epi32(rhsv.v[i]);

                let prod_odd_dbl = _mm256_mul_epu32(lhs_odd_dbl, rhs_odd);
                let prod_evn_dbl = _mm256_mul_epu32(lhs_evn_dbl, rhs_evn);

                let prod_odd_dup = moveldup_epi32(prod_odd_dbl);
                let prod_evn_dup = movehdup_epi32(prod_evn_dbl);
                let prod_lo_dbl = _mm256_blend_epi32(prod_evn_dbl, prod_odd_dup, ODDS);
                let prod_hi = _mm256_blend_epi32(prod_odd_dbl, prod_evn_dup, EVENS);
                // Right shift to undo the doubling.
                let prod_lo = _mm256_srli_epi32::<1>(prod_lo_dbl);

                // Standard addition of two 31-bit values.
                res[i] = add(prod_lo, prod_hi);
            }
            AVXM31 { v: res }
        }
    }
}

impl Mul<M31> for AVXM31 {
    type Output = AVXM31;
    #[inline(always)]
    #[allow(clippy::op_ref)]
    fn mul(self, rhs: M31) -> Self::Output {
        self * &rhs
    }
}

impl Add<M31> for AVXM31 {
    type Output = AVXM31;
    #[inline(always)]
    #[allow(clippy::op_ref)]
    fn add(self, rhs: M31) -> Self::Output {
        self + AVXM31::pack_full(rhs)
    }
}

impl From<u32> for AVXM31 {
    #[inline(always)]
    fn from(x: u32) -> Self {
        AVXM31::pack_full(M31::from(x))
    }
}

impl Neg for AVXM31 {
    type Output = AVXM31;
    #[inline(always)]
    fn neg(self) -> Self::Output {
        AVXM31 {
            v: unsafe {
                [
                    _mm256_xor_epi32(self.v[0], PACKED_MOD),
                    _mm256_xor_epi32(self.v[1], PACKED_MOD),
                ]
            },
        }
    }
}

#[inline(always)]
fn add_internal(a: &AVXM31, b: &AVXM31) -> AVXM31 {
    unsafe {
        let mut result = [
            _mm256_add_epi32(a.v[0], b.v[0]),
            _mm256_add_epi32(a.v[1], b.v[1]),
        ];
        let subx = [
            _mm256_sub_epi32(result[0], PACKED_MOD),
            _mm256_sub_epi32(result[1], PACKED_MOD),
        ];
        result = [
            _mm256_min_epu32(result[0], subx[0]),
            _mm256_min_epu32(result[1], subx[1]),
        ];

        AVXM31 { v: result }
    }
}

#[inline(always)]
fn sub_internal(a: &AVXM31, b: &AVXM31) -> AVXM31 {
    AVXM31 {
        v: unsafe {
            let t = [
                _mm256_sub_epi32(a.v[0], b.v[0]),
                _mm256_sub_epi32(a.v[1], b.v[1]),
            ];
            let subx = [
                _mm256_add_epi32(t[0], PACKED_MOD),
                _mm256_add_epi32(t[1], PACKED_MOD),
            ];
            [
                _mm256_min_epu32(t[0], subx[0]),
                _mm256_min_epu32(t[1], subx[1]),
            ]
        },
    }
}

#[inline]
fn mul_internal(a: &AVXM31, b: &AVXM31) -> AVXM31 {
    // credit: https://github.com/Plonky3/Plonky3/blob/eeb4e37b20127c4daa871b2bad0df30a7c7380db/mersenne-31/src/x86_64_avx2/packing.rs#L154
    unsafe {
        let mut res: [__m256i; 2] = [_mm256_setzero_si256(); 2];
        #[allow(clippy::needless_range_loop)]
        for i in 0..res.len() {
            let rhs_evn = b.v[i];
            let lhs_odd_dbl = _mm256_srli_epi64(a.v[i], 31);
            let lhs_evn_dbl = _mm256_add_epi32(a.v[i], a.v[i]);
            let rhs_odd = movehdup_epi32(b.v[i]);

            let prod_odd_dbl = _mm256_mul_epu32(lhs_odd_dbl, rhs_odd);
            let prod_evn_dbl = _mm256_mul_epu32(lhs_evn_dbl, rhs_evn);

            let prod_odd_dup = moveldup_epi32(prod_odd_dbl);
            let prod_evn_dup = movehdup_epi32(prod_evn_dbl);
            let prod_lo_dbl = _mm256_blend_epi32(prod_evn_dbl, prod_odd_dup, ODDS);
            let prod_hi = _mm256_blend_epi32(prod_odd_dbl, prod_evn_dup, EVENS);
            // Right shift to undo the doubling.
            let prod_lo = _mm256_srli_epi32::<1>(prod_lo_dbl);

            // Standard addition of two 31-bit values.
            res[i] = add(prod_lo, prod_hi);
        }
        AVXM31 { v: res }
    }
}
