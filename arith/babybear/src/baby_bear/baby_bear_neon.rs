use arith::{field_common, Field, FieldSerde, FieldSerdeResult, SimdField};
use p3_baby_bear::PackedBabyBearNeon;
use rand::RngCore;
use std::{
    arch::aarch64::*,
    fmt::Debug,
    io::{Read, Write},
    iter::{Product, Sum},
    mem::transmute,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use crate::BabyBear;

const BABY_BEAR_PACK_SIZE: usize = 16;

#[derive(Clone, Copy)]
pub struct NeonBabyBear {
    pub v: [uint32x4_t; 4],
}

field_common!(NeonBabyBear);

impl NeonBabyBear {
    #[inline(always)]
    pub fn pack_full(x: BabyBear) -> NeonBabyBear {
        NeonBabyBear {
            v: unsafe {
                // Safety: memory representation of [x; BABY_BEAR_PACK_SIZE]
                // is 16 u32s, which can be reinterpreted as 4 uint32x4_t.
                transmute::<[BabyBear; 16], [uint32x4_t; 4]>([x; BABY_BEAR_PACK_SIZE])
            },
        }
    }
}

impl FieldSerde for NeonBabyBear {
    const SERIALIZED_SIZE: usize = (128 / 8) * 4;

    #[inline(always)]
    fn serialize_into<W: Write>(&self, mut writer: W) -> FieldSerdeResult<()> {
        // Transmute would serialize the Montgomery form,
        // instead we convert to canonical form and serialize
        let unpacked = self.unpack();
        let canonical: [u32; BABY_BEAR_PACK_SIZE] = unpacked
            .iter()
            .map(|x| x.as_u32_unchecked())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let data = unsafe {
            transmute::<[u32; BABY_BEAR_PACK_SIZE], [u8; Self::SERIALIZED_SIZE]>(canonical)
        };
        writer.write_all(&data)?;
        Ok(())
    }

    #[inline(always)]
    fn deserialize_from<R: Read>(mut reader: R) -> FieldSerdeResult<Self> {
        let mut data = [0u8; Self::SERIALIZED_SIZE];
        reader.read_exact(&mut data)?;
        // Transmute would fail to convert to Montgomery form
        let canonical =
            unsafe { transmute::<[u8; Self::SERIALIZED_SIZE], [u32; BABY_BEAR_PACK_SIZE]>(data) };
        let unpacked = canonical
            .iter()
            .map(|x| BabyBear::new(*x))
            .collect::<Vec<_>>();
        Ok(Self::pack(&unpacked))
    }

    #[inline]
    fn try_deserialize_from_ecc_format<R: Read>(mut reader: R) -> FieldSerdeResult<Self> {
        let mut buf = [0u8; 32];
        reader.read_exact(&mut buf)?;
        assert!(
            buf.iter().skip(4).all(|x| *x == 0),
            "non-zero byte found in witness byte"
        );
        // BabyBear::from converts from canonical to Montgomery form
        Ok(Self::pack_full(BabyBear::from(u32::from_le_bytes(
            buf[..4].try_into().unwrap(),
        ))))
    }
}

impl Field for NeonBabyBear {
    const NAME: &'static str = "Neon Packed BabyBear";

    const SIZE: usize = 128 / 8 * 4;

    const FIELD_SIZE: usize = 32;

    const ZERO: Self = Self {
        v: unsafe {
            transmute::<[BabyBear; 16], [uint32x4_t; 4]>([BabyBear::ZERO; BABY_BEAR_PACK_SIZE])
        },
    };

    const ONE: Self = Self {
        v: unsafe {
            transmute::<[BabyBear; 16], [uint32x4_t; 4]>([BabyBear::ONE; BABY_BEAR_PACK_SIZE])
        },
    };

    const INV_2: Self = Self {
        v: unsafe {
            transmute::<[BabyBear; 16], [uint32x4_t; 4]>([BabyBear::INV_2; BABY_BEAR_PACK_SIZE])
        },
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

    fn random_unsafe(mut rng: impl RngCore) -> Self {
        // TODO: Is it safe to instead sample a u32, reduce mod p,
        // and treat this directly as the Montgomery form of an element?
        let mut sample = [BabyBear::ZERO; BABY_BEAR_PACK_SIZE];
        sample
            .iter_mut()
            .for_each(|s| *s = BabyBear::random_unsafe(&mut rng));
        Self::pack(&sample)
    }

    fn random_bool(mut rng: impl RngCore) -> Self {
        let sample = (0..BABY_BEAR_PACK_SIZE)
            .map(|_| BabyBear::random_bool(&mut rng))
            .collect::<Vec<_>>();
        Self::pack(&sample)
    }

    fn exp(&self, _: u128) -> Self {
        unimplemented!("exp not implemented for NeonBabyBear")
    }

    fn inv(&self) -> Option<Self> {
        // slow, should not be used in production
        let mut babybear_vec =
            unsafe { transmute::<[uint32x4_t; 4], [BabyBear; BABY_BEAR_PACK_SIZE]>(self.v) };
        let is_nonzero = babybear_vec.iter().all(|x| !x.is_zero());
        if !is_nonzero {
            return None;
        };
        babybear_vec.iter_mut().for_each(|x| *x = x.inv().unwrap());
        Some(Self::pack(&babybear_vec))
    }

    fn as_u32_unchecked(&self) -> u32 {
        unimplemented!("self is a vector, cannot convert to u32")
    }

    fn from_uniform_bytes(bytes: &[u8; 32]) -> Self {
        Self::pack_full(BabyBear::from_uniform_bytes(bytes))
    }
}

impl SimdField for NeonBabyBear {
    type Scalar = BabyBear;

    #[inline]
    fn scale(&self, challenge: &Self::Scalar) -> Self {
        *self * *challenge
    }

    #[inline(always)]
    fn pack(base_vec: &[Self::Scalar]) -> Self {
        assert!(base_vec.len() == BABY_BEAR_PACK_SIZE);
        let ret: [Self::Scalar; BABY_BEAR_PACK_SIZE] = base_vec.try_into().unwrap();
        Self {
            // Transmute is reinterpreting an array of scalars in Montgomery form to an AVX register
            v: unsafe { transmute::<[BabyBear; 16], [uint32x4_t; 4]>(ret) },
        }
    }

    #[inline(always)]
    fn unpack(&self) -> Vec<Self::Scalar> {
        // Transmute is reinterpreting an AVX register to an array of scalars in Montgomery form
        let ret =
            unsafe { transmute::<[uint32x4_t; 4], [Self::Scalar; BABY_BEAR_PACK_SIZE]>(self.v) };
        ret.to_vec()
    }

    #[inline(always)]
    fn pack_size() -> usize {
        BABY_BEAR_PACK_SIZE
    }
}

impl From<BabyBear> for NeonBabyBear {
    #[inline(always)]
    fn from(x: BabyBear) -> Self {
        NeonBabyBear::pack_full(x)
    }
}

impl Debug for NeonBabyBear {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let unpacked = self.unpack();
        if unpacked.iter().all(|x| *x == unpacked[0]) {
            write!(f, "uint32x4_t<16 x {:?}>", unpacked[0])
        } else {
            write!(f, "uint32x4_t<{unpacked:?}>")
        }
    }
}

impl Default for NeonBabyBear {
    fn default() -> Self {
        Self::ZERO
    }
}

impl PartialEq for NeonBabyBear {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            transmute::<[uint32x4_t; 4], [u32; 16]>(self.v)
                == transmute::<[uint32x4_t; 4], [u32; 16]>(other.v)
        }
    }
}

impl Mul<&BabyBear> for NeonBabyBear {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: &BabyBear) -> Self::Output {
        self * NeonBabyBear::pack_full(*rhs)
    }
}

impl Mul<BabyBear> for NeonBabyBear {
    type Output = NeonBabyBear;
    #[inline(always)]
    fn mul(self, rhs: BabyBear) -> Self::Output {
        self * &rhs
    }
}

impl Add<BabyBear> for NeonBabyBear {
    type Output = NeonBabyBear;
    #[inline(always)]
    fn add(self, rhs: BabyBear) -> Self::Output {
        self + NeonBabyBear::pack_full(rhs)
    }
}

impl From<u32> for NeonBabyBear {
    #[inline(always)]
    fn from(value: u32) -> Self {
        // BabyBear::new converts to Montgomery form
        NeonBabyBear::pack_full(BabyBear::new(value))
    }
}

impl Neg for NeonBabyBear {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self::Output {
        unsafe {
            let mut a: [PackedBabyBearNeon; 4] = transmute(self);
            a.iter_mut().for_each(|x| *x = x.neg());
            transmute(a)
        }
    }
}

#[inline(always)]
fn add_internal(a: &NeonBabyBear, b: &NeonBabyBear) -> NeonBabyBear {
    unsafe {
        let a: [PackedBabyBearNeon; 4] = transmute(*a);
        let b: [PackedBabyBearNeon; 4] = transmute(*b);
        let mut res = [PackedBabyBearNeon::default(); 4];
        for i in 0..4 {
            res[i] = a[i] + b[i];
        }
        transmute(res)
    }
}

#[inline(always)]
fn sub_internal(a: &NeonBabyBear, b: &NeonBabyBear) -> NeonBabyBear {
    unsafe {
        let a: [PackedBabyBearNeon; 4] = transmute(*a);
        let b: [PackedBabyBearNeon; 4] = transmute(*b);
        let mut res = [PackedBabyBearNeon::default(); 4];
        for i in 0..4 {
            res[i] = a[i] - b[i];
        }
        transmute(res)
    }
}

#[inline]
fn mul_internal(a: &NeonBabyBear, b: &NeonBabyBear) -> NeonBabyBear {
    unsafe {
        let a: [PackedBabyBearNeon; 4] = transmute(*a);
        let b: [PackedBabyBearNeon; 4] = transmute(*b);
        let mut res = [PackedBabyBearNeon::default(); 4];
        for i in 0..4 {
            res[i] = a[i] * b[i];
        }
        transmute(res)
    }
}
