use crate::{field_common, BabyBear, Field, FieldSerde, FieldSerdeResult, SimdField};
use ark_std::iterable::Iterable;
use p3_baby_bear::PackedBabyBearAVX512;
use rand::RngCore;
use std::{
    arch::x86_64::*,
    fmt::Debug,
    io::{Read, Write},
    iter::{Product, Sum},
    mem::transmute,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

const BABY_BEAR_PACK_SIZE: usize = 16;

#[derive(Clone, Copy)]
pub struct AVXBabyBear {
    pub v: __m512i,
}

impl AVXBabyBear {
    #[inline(always)]
    pub(crate) fn pack_full(x: BabyBear) -> Self {
        AVXBabyBear {
            v: unsafe { transmute([x; BABY_BEAR_PACK_SIZE]) },
        }
    }
}

field_common!(AVXBabyBear);

impl FieldSerde for AVXBabyBear {
    const SERIALIZED_SIZE: usize = 512 / 8;

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
        let data = unsafe { transmute::<_, [u8; Self::SERIALIZED_SIZE]>(canonical) };
        writer.write_all(&data)?;
        Ok(())
    }

    #[inline(always)]
    fn deserialize_from<R: Read>(mut reader: R) -> FieldSerdeResult<Self> {
        let mut data = [0u8; Self::SERIALIZED_SIZE];
        reader.read_exact(&mut data)?;
        // Transmute would fail to convert to Montgomery form
        let canonical = unsafe { transmute::<_, [u32; BABY_BEAR_PACK_SIZE]>(data) };
        let unpacked = canonical
            .iter()
            .map(|x| BabyBear::new(x))
            .collect::<Vec<_>>();
        Ok(Self::pack(&unpacked))
    }

    #[inline(always)]
    fn try_deserialize_from_ecc_format<R: Read>(mut reader: R) -> FieldSerdeResult<Self> {
        let mut buf = [0u8; 32];
        reader.read_exact(&mut buf)?;
        assert!(
            buf.iter().skip(4).all(|x| x == 0),
            "non-zero byte found in witness byte"
        );
        // BabyBear::from converts from canonical to Montgomery form
        Ok(Self::pack_full(BabyBear::from(u32::from_le_bytes(
            buf[..4].try_into().unwrap(),
        ))))
    }
}

impl Field for AVXBabyBear {
    const NAME: &'static str = "AVX Packed BabyBear";

    const SIZE: usize = 512 / 8;

    const FIELD_SIZE: usize = 32;

    const ZERO: Self = Self {
        v: unsafe { transmute([BabyBear::ZERO; BABY_BEAR_PACK_SIZE]) },
    };

    const ONE: Self = Self {
        v: unsafe { transmute([BabyBear::ONE; BABY_BEAR_PACK_SIZE]) },
    };

    const INV_2: Self = Self {
        v: unsafe { transmute([BabyBear::INV_2; BABY_BEAR_PACK_SIZE]) },
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
        for i in 0..BABY_BEAR_PACK_SIZE {
            sample[i] = BabyBear::random_unsafe(&mut rng);
        }
        Self::pack(&sample)
    }

    fn random_bool(mut rng: impl RngCore) -> Self {
        let sample = (0..BABY_BEAR_PACK_SIZE)
            .map(|_| BabyBear::random_bool(&mut rng))
            .collect::<Vec<_>>();
        Self::pack(&sample)
    }

    fn exp(&self, _: u128) -> Self {
        unimplemented!("exp not implemented for AVXBabyBear")
    }

    fn inv(&self) -> Option<Self> {
        // slow, should not be used in production
        let mut babybear_vec =
            unsafe { transmute::<__m512i, [BabyBear; BABY_BEAR_PACK_SIZE]>(self.v) };
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

impl SimdField for AVXBabyBear {
    // Note: Memory representation is in Montgomery form
    type Scalar = BabyBear;

    #[inline]
    fn scale(&self, challenge: &Self::Scalar) -> Self {
        *self * *challenge
    }

    #[inline(always)]
    fn pack(base_vec: &[Self::Scalar]) -> Self {
        debug_assert!(base_vec.len() == BABY_BEAR_PACK_SIZE);
        let ret: [Self::Scalar; BABY_BEAR_PACK_SIZE] = base_vec.try_into().unwrap();
        Self {
            // Transmute is reinterpreting an array of scalars in Montgomery form to an AVX register
            v: unsafe { transmute(ret) },
        }
    }

    #[inline(always)]
    fn unpack(&self) -> Vec<Self::Scalar> {
        // Transmute is reinterpreting an AVX register to an array of scalars in Montgomery form
        let ret = unsafe { transmute::<__m512i, [Self::Scalar; BABY_BEAR_PACK_SIZE]>(self.v) };
        ret.to_vec()
    }

    #[inline(always)]
    fn pack_size() -> usize {
        BABY_BEAR_PACK_SIZE
    }
}

impl From<BabyBear> for AVXBabyBear {
    #[inline(always)]
    fn from(value: BabyBear) -> Self {
        AVXBabyBear::pack_full(value)
    }
}

impl Debug for AVXBabyBear {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let unpacked = self.unpack();
        if unpacked.iter().all(|x| *x == unpacked[0]) {
            write!(f, "mm512i<16 x {:?}>", unpacked[0])
        } else {
            write!(f, "mm512i<{unpacked:?}>")
        }
    }
}

impl Default for AVXBabyBear {
    fn default() -> Self {
        Self::ZERO
    }
}

impl PartialEq for AVXBabyBear {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            let pcmp = _mm512_cmpeq_epi32_mask(self.v, other.v);
            pcmp == 0xFFFF
        }
    }
}

impl Mul<&BabyBear> for AVXBabyBear {
    type Output = AVXBabyBear;

    #[inline(always)]
    fn mul(self, rhs: &BabyBear) -> Self::Output {
        self * AVXBabyBear::pack_full(*rhs)
    }
}

impl Mul<BabyBear> for AVXBabyBear {
    type Output = AVXBabyBear;

    #[inline(always)]
    fn mul(self, rhs: BabyBear) -> Self::Output {
        self * &rhs
    }
}

impl Add<BabyBear> for AVXBabyBear {
    type Output = AVXBabyBear;

    #[inline(always)]
    fn add(self, rhs: BabyBear) -> Self::Output {
        self + AVXBabyBear::pack_full(rhs)
    }
}

impl From<u32> for AVXBabyBear {
    #[inline(always)]
    fn from(value: u32) -> Self {
        // BabyBear::new converts to Montgomery form
        AVXBabyBear::pack_full(BabyBear::new(value))
    }
}

impl Neg for AVXBabyBear {
    type Output = AVXBabyBear;

    #[inline(always)]
    fn neg(self) -> Self::Output {
        unsafe {
            let a: PackedBabyBearAVX512 = transmute(self);
            transmute(-a)
        }
    }
}

#[inline(always)]
fn add_internal(a: &AVXBabyBear, b: &AVXBabyBear) -> AVXBabyBear {
    unsafe {
        let a: PackedBabyBearAVX512 = transmute(*a);
        let b: PackedBabyBearAVX512 = transmute(*b);
        transmute(a + b)
    }
}

#[inline(always)]
fn sub_internal(a: &AVXBabyBear, b: &AVXBabyBear) -> AVXBabyBear {
    unsafe {
        let a: PackedBabyBearAVX512 = transmute(*a);
        let b: PackedBabyBearAVX512 = transmute(*b);
        transmute(a - b)
    }
}

#[inline(always)]
fn mul_internal(a: &AVXBabyBear, b: &AVXBabyBear) -> AVXBabyBear {
    unsafe {
        let a: PackedBabyBearAVX512 = transmute(*a);
        let b: PackedBabyBearAVX512 = transmute(*b);
        transmute(a * b)
    }
}
