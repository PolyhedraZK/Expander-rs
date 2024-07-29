use std::{
    arch::aarch64::*,
    fmt::Debug,
    io::{Read, Write},
    iter::{Product, Sum},
    mem::transmute,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use rand::{Rng, RngCore};

use crate::{Field, FieldSerde, SimdField, M31, M31_MOD};

const PACKED_MOD: uint32x4_t = unsafe { transmute([M31_MOD; 4]) };
const PACKED_0: uint32x4_t = unsafe { transmute([0; 4]) };
const PACKED_INV_2: uint32x4_t = unsafe { transmute([1 << 30; 4]) };

#[inline(always)]
fn reduce_sum(x: uint32x4_t) -> uint32x4_t {
    //aarch64 only
    unsafe { vminq_u32(x, vsubq_u32(x, PACKED_MOD)) }
}

/// NeonM31 packs 8 M31 elements and operates on them in parallel
#[derive(Clone, Copy)]
pub struct NeonM31 {
    pub v: [uint32x4_t; 2],
}

impl NeonM31 {
    #[inline(always)]
    pub fn pack_full(x: M31) -> NeonM31 {
        NeonM31 {
            v: unsafe { [vdupq_n_u32(x.v), vdupq_n_u32(x.v)] },
        }
    }

    #[inline(always)]
    pub(crate) fn mul_by_5(&self) -> NeonM31 {
        let mut res = NeonM31 {
            v: [PACKED_0, PACKED_0],
        };
        res.v[0] = unsafe {
            let double = reduce_sum(vshlq_n_u32(self.v[0], 1));
            let quad = reduce_sum(vshlq_n_u32(double, 1));
            reduce_sum(vaddq_u32(quad, self.v[0]))
        };
        res.v[1] = unsafe {
            let double = reduce_sum(vshlq_n_u32(self.v[1], 1));
            let quad = reduce_sum(vshlq_n_u32(double, 1));
            reduce_sum(vaddq_u32(quad, self.v[1]))
        };
        res
    }

    #[inline(always)]
    pub(crate) fn mul_by_10(&self) -> NeonM31 {
        let mut res = NeonM31 {
            v: [PACKED_0, PACKED_0],
        };
        res.v[0] = unsafe {
            let double = reduce_sum(vshlq_n_u32(self.v[0], 1));
            let quad = reduce_sum(vshlq_n_u32(double, 1));
            let oct = reduce_sum(vshlq_n_u32(quad, 1));
            reduce_sum(vaddq_u32(oct, double))
        };
        res.v[1] = unsafe {
            let double = reduce_sum(vshlq_n_u32(self.v[1], 1));
            let quad = reduce_sum(vshlq_n_u32(double, 1));
            let oct = reduce_sum(vshlq_n_u32(quad, 1));
            reduce_sum(vaddq_u32(oct, double))
        };
        res
    }
}

impl FieldSerde for NeonM31 {
    #[inline(always)]
    /// serialize self into bytes
    fn serialize_into<W: Write>(&self, mut writer: W) {
        let data = unsafe { transmute::<[uint32x4_t; 2], [u8; 32]>(self.v) };
        writer.write_all(&data).unwrap();
    }

    #[inline(always)]
    fn serialized_size() -> usize {
        128 / 8 * 2
    }

    /// deserialize bytes into field
    #[inline(always)]
    fn deserialize_from<R: Read>(mut reader: R) -> Self {
        let mut data = [0; 32];
        reader.read_exact(&mut data).unwrap();
        unsafe {
            NeonM31 {
                v: transmute::<[u8; 32], [uint32x4_t; 2]>(data),
            }
        }
    }

    #[inline(always)]
    fn deserialize_from_ecc_format<R: Read>(mut reader: R) -> Self {
        let mut buf = [0u8; 32];
        reader.read_exact(&mut buf).unwrap();
        assert!(
            buf.iter().skip(4).all(|&x| x == 0),
            "non-zero byte found in witness byte"
        );
        Self::pack_full(u32::from_le_bytes(buf[..4].try_into().unwrap()).into())
    }

    fn try_deserialize_from_ecc_format<R: Read>(
        mut reader: R,
    ) -> std::result::Result<Self, std::io::Error>
    where
        Self: Sized,
    {
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

impl Field for NeonM31 {
    const NAME: &'static str = "Neon Packed Mersenne 31";

    // size in bytes
    const SIZE: usize = 128 / 8 * 2;

    const ZERO: Self = Self {
        v: [PACKED_0, PACKED_0],
    };

    const INV_2: Self = Self {
        v: [PACKED_INV_2; 2],
    };

    #[inline(always)]
    fn zero() -> Self {
        Self { v: [PACKED_0; 2] }
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        unsafe {
            let comparison_0: uint32x4_t = vceqq_u32(self.v[0], PACKED_0);
            let comparison_1: uint32x4_t = vceqq_u32(self.v[1], PACKED_0);
            let result_0 = transmute::<uint32x4_t, [u32; 4]>(comparison_0);
            let result_1 = transmute::<uint32x4_t, [u32; 4]>(comparison_1);
            result_0.iter().all(|&x| x != 0) && result_1.iter().all(|&x| x != 0)
        }
    }

    #[inline(always)]
    fn one() -> Self {
        NeonM31 {
            v: unsafe { [vdupq_n_u32(1); 2] },
        }
    }

    #[inline(always)]
    fn random_unsafe(mut rng: impl RngCore) -> Self {
        // Caution: this may not produce uniformly random elements
        unsafe {
            NeonM31 {
                v: [
                    vld1q_u32(
                        [
                            rng.gen::<u32>() % M31_MOD,
                            rng.gen::<u32>() % M31_MOD,
                            rng.gen::<u32>() % M31_MOD,
                            rng.gen::<u32>() % M31_MOD,
                        ]
                        .as_ptr(),
                    ),
                    vld1q_u32(
                        [
                            rng.gen::<u32>() % M31_MOD,
                            rng.gen::<u32>() % M31_MOD,
                            rng.gen::<u32>() % M31_MOD,
                            rng.gen::<u32>() % M31_MOD,
                        ]
                        .as_ptr(),
                    ),
                ],
            }
        }
    }

    #[inline(always)]
    fn random_bool(mut rng: impl RngCore) -> Self {
        unsafe {
            NeonM31 {
                v: [
                    vld1q_u32(
                        [
                            rng.gen::<bool>() as u32,
                            rng.gen::<bool>() as u32,
                            rng.gen::<bool>() as u32,
                            rng.gen::<bool>() as u32,
                        ]
                        .as_ptr(),
                    ),
                    vld1q_u32(
                        [
                            rng.gen::<bool>() as u32,
                            rng.gen::<bool>() as u32,
                            rng.gen::<bool>() as u32,
                            rng.gen::<bool>() as u32,
                        ]
                        .as_ptr(),
                    ),
                ],
            }
        }
    }

    #[inline]
    fn double(&self) -> Self {
        self.mul_by_2()
    }

    fn exp(&self, _exponent: &Self) -> Self {
        todo!()
    }

    #[inline(always)]
    fn inv(&self) -> Option<Self> {
        // slow, should not be used in production
        let mut m31_vec = unsafe { transmute::<[uint32x4_t; 2], [M31; 8]>(self.v) };
        let is_non_zero = m31_vec.iter().all(|x| !x.is_zero());
        if !is_non_zero {
            return None;
        }

        m31_vec.iter_mut().for_each(|x| *x = x.inv().unwrap()); // safe unwrap
        Some(Self {
            v: unsafe { transmute::<[M31; 8], [uint32x4_t; 2]>(m31_vec) },
        })
    }

    fn as_u32_unchecked(&self) -> u32 {
        unimplemented!("self is a vector, cannot convert to u32")
    }

    #[inline]
    fn from_uniform_bytes(bytes: &[u8; 32]) -> Self {
        let m = M31::from_uniform_bytes(bytes);
        Self {
            v: unsafe { [vdupq_n_u32(m.v), vdupq_n_u32(m.v)] },
        }
    }

    #[inline(always)]
    fn mul_by_2(&self) -> NeonM31 {
        let mut res = NeonM31 {
            v: [PACKED_0, PACKED_0],
        };
        res.v[0] = unsafe {
            let double = vshlq_n_u32(self.v[0], 1);
            reduce_sum(double)
        };
        res.v[1] = unsafe {
            let double = vshlq_n_u32(self.v[1], 1);
            reduce_sum(double)
        };
        res
    }
}

impl SimdField for NeonM31 {
    type Scalar = M31;

    #[inline]
    fn scale(&self, challenge: &Self::Scalar) -> Self {
        let packed_challenge = NeonM31::pack_full(*challenge);
        *self * packed_challenge
    }
}

impl From<M31> for NeonM31 {
    #[inline(always)]
    fn from(x: M31) -> Self {
        NeonM31::pack_full(x)
    }
}

impl Debug for NeonM31 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for &v in self.v.iter() {
            unsafe {
                let data = [
                    vgetq_lane_u32(v, 0),
                    vgetq_lane_u32(v, 1),
                    vgetq_lane_u32(v, 2),
                    vgetq_lane_u32(v, 3),
                ];
                // if all data is the same, print only one
                if data.iter().all(|&x| x == data[0]) {
                    write!(
                        f,
                        "uint32x4_t<8 x {}>",
                        if M31_MOD - data[0] > 1024 {
                            format!("{}", data[0])
                        } else {
                            format!("-{}", M31_MOD - data[0])
                        }
                    )?;
                } else {
                    write!(f, "uint32x4_t<{:?}>", data)?;
                }
            }
        }
        Ok(())
    }
}

impl Default for NeonM31 {
    fn default() -> Self {
        NeonM31::zero()
    }
}

impl PartialEq for NeonM31 {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            let eq_v_0 = vceqq_u32(self.v[0], other.v[0]);
            let eq_v_1 = vceqq_u32(self.v[1], other.v[1]);
            vgetq_lane_u32(eq_v_0, 0) != 0
                && vgetq_lane_u32(eq_v_0, 1) != 0
                && vgetq_lane_u32(eq_v_0, 2) != 0
                && vgetq_lane_u32(eq_v_0, 3) != 0
                && vgetq_lane_u32(eq_v_1, 0) != 0
                && vgetq_lane_u32(eq_v_1, 1) != 0
                && vgetq_lane_u32(eq_v_1, 2) != 0
                && vgetq_lane_u32(eq_v_1, 3) != 0
        }
    }
}

impl Mul<&NeonM31> for NeonM31 {
    type Output = NeonM31;
    #[inline(always)]
    fn mul(self, rhs: &NeonM31) -> Self::Output {
        let mut res = NeonM31 {
            v: [PACKED_0, PACKED_0],
        };
        res.v[0] = unsafe {
            let prod_hi = vreinterpretq_u32_s32(vqdmulhq_s32(
                vreinterpretq_s32_u32(self.v[0]),
                vreinterpretq_s32_u32(rhs.v[0]),
            ));
            let prod_lo = vmulq_u32(self.v[0], rhs.v[0]);
            let t = vmlsq_u32(prod_lo, prod_hi, PACKED_MOD);
            reduce_sum(t)
        };
        res.v[1] = unsafe {
            let prod_hi = vreinterpretq_u32_s32(vqdmulhq_s32(
                vreinterpretq_s32_u32(self.v[1]),
                vreinterpretq_s32_u32(rhs.v[1]),
            ));
            let prod_lo = vmulq_u32(self.v[1], rhs.v[1]);
            let t = vmlsq_u32(prod_lo, prod_hi, PACKED_MOD);
            reduce_sum(t)
        };
        res
    }
}

impl Mul for NeonM31 {
    type Output = NeonM31;
    #[inline(always)]
    #[allow(clippy::op_ref)]
    fn mul(self, rhs: NeonM31) -> Self::Output {
        self * &rhs
    }
}

impl Mul<&M31> for NeonM31 {
    type Output = NeonM31;
    #[inline(always)]
    fn mul(self, rhs: &M31) -> Self::Output {
        let rhs_p = NeonM31::pack_full(*rhs);
        self * rhs_p
    }
}

impl Mul<M31> for NeonM31 {
    type Output = NeonM31;
    #[inline(always)]
    fn mul(self, rhs: M31) -> Self::Output {
        self * &rhs
    }
}

impl MulAssign<&NeonM31> for NeonM31 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: &NeonM31) {
        *self = *self * rhs;
    }
}

impl MulAssign for NeonM31 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self *= &rhs;
    }
}

impl<T: ::core::borrow::Borrow<NeonM31>> Product<T> for NeonM31 {
    fn product<I: Iterator<Item = T>>(iter: I) -> Self {
        iter.fold(Self::one(), |acc, item| acc * item.borrow())
    }
}

impl Add<&NeonM31> for NeonM31 {
    type Output = NeonM31;
    #[inline(always)]
    fn add(self, rhs: &NeonM31) -> Self::Output {
        unsafe {
            NeonM31 {
                v: [
                    reduce_sum(vaddq_u32(self.v[0], rhs.v[0])),
                    reduce_sum(vaddq_u32(self.v[1], rhs.v[1])),
                ],
            }
        }
    }
}

impl Add for NeonM31 {
    type Output = NeonM31;
    #[inline(always)]
    #[allow(clippy::op_ref)]
    fn add(self, rhs: NeonM31) -> Self::Output {
        self + &rhs
    }
}

impl AddAssign<&NeonM31> for NeonM31 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: &NeonM31) {
        *self = *self + rhs;
    }
}

impl AddAssign for NeonM31 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self += &rhs;
    }
}

impl Add<M31> for NeonM31 {
    type Output = NeonM31;
    #[inline(always)]
    #[allow(clippy::op_ref)]
    fn add(self, rhs: M31) -> Self::Output {
        self + NeonM31::pack_full(rhs)
    }
}

impl<T: ::core::borrow::Borrow<NeonM31>> Sum<T> for NeonM31 {
    fn sum<I: Iterator<Item = T>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, item| acc + item.borrow())
    }
}

impl From<u32> for NeonM31 {
    #[inline(always)]
    fn from(x: u32) -> Self {
        NeonM31::pack_full(M31::from(x))
    }
}

impl Neg for NeonM31 {
    type Output = NeonM31;
    #[inline(always)]
    fn neg(self) -> Self::Output {
        NeonM31::zero() - self
    }
}

impl Sub<&NeonM31> for NeonM31 {
    type Output = NeonM31;
    #[inline(always)]
    fn sub(self, rhs: &NeonM31) -> Self::Output {
        NeonM31 {
            v: [
                unsafe {
                    let diff = vsubq_u32(self.v[0], rhs.v[0]);
                    let u = vaddq_u32(diff, PACKED_MOD);
                    vminq_u32(diff, u)
                },
                unsafe {
                    let diff = vsubq_u32(self.v[1], rhs.v[1]);
                    let u = vaddq_u32(diff, PACKED_MOD);
                    vminq_u32(diff, u)
                },
            ],
        }
    }
}

impl Sub for NeonM31 {
    type Output = NeonM31;
    #[inline(always)]
    #[allow(clippy::op_ref)]
    fn sub(self, rhs: NeonM31) -> Self::Output {
        self - &rhs
    }
}

impl SubAssign<&NeonM31> for NeonM31 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: &NeonM31) {
        *self = *self - rhs;
    }
}

impl SubAssign for NeonM31 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self -= &rhs;
    }
}
