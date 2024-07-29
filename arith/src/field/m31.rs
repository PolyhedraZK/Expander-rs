mod simd_m31;
pub use simd_m31::SimdM31;

#[cfg(target_arch = "x86_64")]
pub(crate) mod m31_avx;

#[cfg(target_arch = "aarch64")]
pub mod m31_neon;

use rand::RngCore;

use crate::{Field, FieldSerde};
use std::{
    io::{Read, Write},
    iter::{Product, Sum},
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use ark_std::Zero;

pub const M31_MOD: u32 = 2147483647;

#[inline]
pub(crate) fn mod_reduce_u32(x: u32) -> u32 {
    (x & M31_MOD) + (x >> 31)
}

#[inline]
fn mod_reduce_i64(x: i64) -> i64 {
    (x & M31_MOD as i64) + (x >> 31)
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct M31 {
    pub v: u32,
}

impl FieldSerde for M31 {
    #[inline(always)]
    fn serialize_into<W: Write>(&self, mut writer: W) {
        writer.write_all(self.v.to_le_bytes().as_ref()).unwrap(); // todo: error propagation
    }

    #[inline(always)]
    fn serialized_size() -> usize {
        32 / 8
    }

    // FIXME: this deserialization function auto corrects invalid inputs.
    // We should use separate APIs for this and for the actual deserialization.
    #[inline(always)]
    fn deserialize_from<R: Read>(mut reader: R) -> Self {
        let mut u = [0u8; 4];
        reader.read_exact(&mut u).unwrap(); // todo: error propagation
        let mut v = u32::from_le_bytes(u);
        v = mod_reduce_u32(v);
        M31 { v }
    }

    #[inline(always)]
    fn deserialize_from_ecc_format<R: Read>(mut reader: R) -> Self {
        let mut buf = [0u8; 32];
        reader.read_exact(&mut buf).unwrap();
        assert!(
            buf.iter().skip(4).all(|&x| x == 0),
            "non-zero byte found in witness byte"
        );
        Self::from(u32::from_le_bytes(buf[..4].try_into().unwrap()))
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
        Ok(Self::from(u32::from_le_bytes(buf[..4].try_into().unwrap())))
    }
}

impl M31 {
    // Add two M31 without mod reduction
    #[inline(always)]
    pub fn unsafe_add(&self, rhs: &Self) -> Self {
        Self { v: self.v + rhs.v }
    }

    // Double an M31 without mod reduction
    #[inline(always)]
    pub fn unsafe_double(&self) -> Self {
        Self { v: self.v << 1 }
    }
}

impl Field for M31 {
    const NAME: &'static str = "Mersenne 31";

    const SIZE: usize = 32 / 8;

    const ZERO: Self = M31 { v: 0 };

    const INV_2: M31 = M31 { v: 1 << 30 };

    #[inline(always)]
    fn zero() -> Self {
        M31 { v: 0 }
    }

    #[inline(always)]
    fn one() -> Self {
        M31 { v: 1 }
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        self.v == 0 || self.v == M31_MOD
    }

    fn random_unsafe(mut rng: impl RngCore) -> Self {
        rng.next_u32().into()
    }

    fn random_bool(mut rng: impl RngCore) -> Self {
        (rng.next_u32() & 1).into()
    }

    fn exp(&self, exponent: &Self) -> Self {
        let mut e = exponent.v;
        let mut res = Self::one();
        let mut t = *self;
        while !e.is_zero() {
            let b = e & 1;
            if b == 1 {
                res *= self;
            }
            t = t * t;
            e >>= 1;
        }
        res
    }

    fn inv(&self) -> Option<Self> {
        self.try_inverse()
    }

    #[inline(always)]
    fn as_u32_unchecked(&self) -> u32 {
        self.v
    }

    #[inline(always)]
    fn from_uniform_bytes(bytes: &[u8; 32]) -> Self {
        let mut v = u32::from_le_bytes(bytes[..4].try_into().unwrap());
        v = mod_reduce_u32(v);
        M31 { v }
    }
}

// ====================================
// Arithmetics for M31
// ====================================

impl Mul<&M31> for M31 {
    type Output = M31;
    #[inline(always)]
    fn mul(self, rhs: &M31) -> Self::Output {
        let mut vv = self.v as i64 * rhs.v as i64;
        vv = mod_reduce_i64(vv);

        if vv >= M31_MOD as i64 {
            vv -= M31_MOD as i64;
        }
        M31 { v: vv as u32 }
    }
}

impl Mul for M31 {
    type Output = M31;
    #[inline(always)]
    #[allow(clippy::op_ref)]
    fn mul(self, rhs: M31) -> Self::Output {
        self * &rhs
    }
}

impl MulAssign<&M31> for M31 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: &M31) {
        *self = *self * rhs;
    }
}

impl MulAssign for M31 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self *= &rhs;
    }
}

impl<T: ::core::borrow::Borrow<M31>> Product<T> for M31 {
    fn product<I: Iterator<Item = T>>(iter: I) -> Self {
        iter.fold(Self::one(), |acc, item| acc * item.borrow())
    }
}

impl Add<&M31> for M31 {
    type Output = M31;
    #[inline(always)]
    fn add(self, rhs: &M31) -> Self::Output {
        let mut vv = self.v + rhs.v;
        if vv >= M31_MOD {
            vv -= M31_MOD;
        }
        M31 { v: vv }
    }
}

impl Add for M31 {
    type Output = M31;
    #[inline(always)]
    #[allow(clippy::op_ref)]
    fn add(self, rhs: M31) -> Self::Output {
        self + &rhs
    }
}

impl AddAssign<&M31> for M31 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: &M31) {
        *self = *self + rhs;
    }
}

impl AddAssign for M31 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self += &rhs;
    }
}

impl<T: ::core::borrow::Borrow<M31>> Sum<T> for M31 {
    fn sum<I: Iterator<Item = T>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, item| acc + item.borrow())
    }
}

impl Neg for M31 {
    type Output = M31;
    #[inline(always)]
    fn neg(self) -> Self::Output {
        M31 {
            v: if self.v == 0 { 0 } else { M31_MOD - self.v },
        }
    }
}

impl Sub<&M31> for M31 {
    type Output = M31;
    #[inline(always)]
    #[allow(clippy::op_ref)]
    fn sub(self, rhs: &M31) -> Self::Output {
        self + &(-*rhs)
    }
}

impl Sub for M31 {
    type Output = M31;
    #[inline(always)]
    #[allow(clippy::op_ref)]
    fn sub(self, rhs: M31) -> Self::Output {
        self - &rhs
    }
}

impl SubAssign<&M31> for M31 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: &M31) {
        *self = *self - rhs;
    }
}

impl SubAssign for M31 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self -= &rhs;
    }
}

impl From<u32> for M31 {
    #[inline(always)]
    fn from(x: u32) -> Self {
        M31 {
            v: if x < M31_MOD { x } else { x % M31_MOD },
        }
    }
}

impl M31 {
    #[inline(always)]
    fn exp_power_of_2(&self, power_log: usize) -> Self {
        let mut res = *self;
        for _ in 0..power_log {
            res = res.square();
        }
        res
    }

    /// credit: https://github.com/Plonky3/Plonky3/blob/ed21a5e11cb20effadaab606598ccad4e70e1a3e/mersenne-31/src/mersenne_31.rs#L235
    #[inline(always)]
    fn try_inverse(&self) -> Option<Self> {
        if self.is_zero() {
            return None;
        }

        // From Fermat's little theorem, in a prime field `F_p`, the inverse of `a` is `a^(p-2)`.
        // Here p-2 = 2147483646 = 1111111111111111111111111111101_2.
        // Uses 30 Squares + 7 Multiplications => 37 Operations total.

        let p1 = *self;
        let p101 = p1.exp_power_of_2(2) * p1;
        let p1111 = p101.square() * p101;
        let p11111111 = p1111.exp_power_of_2(4) * p1111;
        let p111111110000 = p11111111.exp_power_of_2(4);
        let p111111111111 = p111111110000 * p1111;
        let p1111111111111111 = p111111110000.exp_power_of_2(4) * p11111111;
        let p1111111111111111111111111111 = p1111111111111111.exp_power_of_2(12) * p111111111111;
        let p1111111111111111111111111111101 =
            p1111111111111111111111111111.exp_power_of_2(3) * p101;
        Some(p1111111111111111111111111111101)
    }
}
