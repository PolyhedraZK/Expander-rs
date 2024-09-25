use core::{
    iter::{Product, Sum},
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use arith::{field_common, Field, FieldForECC, FieldSerde, FieldSerdeResult, SimdField};
use ark_std::Zero;
use p3_baby_bear::BabyBear as P3BabyBear;
use p3_field::{Field as P3Field, PrimeField32};
use rand::distributions::{Distribution, Standard};
use std::io::{Read, Write};

mod baby_bearx16;
pub use baby_bearx16::BabyBearx16;

#[cfg(target_arch = "x86_64")]
pub(crate) mod baby_bear_avx;
#[cfg(target_arch = "aarch64")]
pub(crate) mod baby_bear_neon;
#[cfg(target_arch = "x86_64")]
pub use baby_bear_avx::AVXBabyBear;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct BabyBear(P3BabyBear);

pub const BABYBEAR_MODULUS: u32 = 0x78000001;

field_common!(BabyBear);

impl BabyBear {
    /// Input is provided in canonical form and converted into Montgomery form.
    pub const fn new(value: u32) -> Self {
        Self(P3BabyBear::new(value))
    }
}

impl FieldSerde for BabyBear {
    const SERIALIZED_SIZE: usize = 32 / 8;

    #[inline(always)]
    fn serialize_into<W: Write>(&self, mut writer: W) -> FieldSerdeResult<()> {
        // Note: BabyBear's impl of as_u32_unchecked() converts to canonical form
        writer.write_all(self.as_u32_unchecked().to_le_bytes().as_ref())?;
        Ok(())
    }

    /// Note: This function performs modular reduction on inputs and
    /// converts from canonical to Montgomery form.
    #[inline(always)]
    fn deserialize_from<R: Read>(mut reader: R) -> FieldSerdeResult<Self> {
        let mut u = [0u8; Self::SERIALIZED_SIZE];
        reader.read_exact(&mut u)?;
        let v = u32::from_le_bytes(u);
        Ok(Self::from(v))
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

impl Field for BabyBear {
    const NAME: &'static str = "Baby Bear Field";

    const SIZE: usize = 32 / 8;

    const FIELD_SIZE: usize = 32;

    const ZERO: Self = BabyBear::new(0);

    const ONE: Self = BabyBear::new(1);

    // See test below
    const INV_2: Self = BabyBear::new(1006632961);

    #[inline(always)]
    fn zero() -> Self {
        Self::ZERO
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        *self == Self::ZERO
    }

    #[inline(always)]
    fn one() -> Self {
        Self::ONE
    }

    /// Uses rejection sampling to avoid bias.
    fn random_unsafe(mut rng: impl rand::RngCore) -> Self {
        let dist = Standard;
        Self(dist.sample(&mut rng))
    }

    fn random_bool(mut rng: impl rand::RngCore) -> Self {
        (rng.next_u32() & 1).into()
    }

    fn exp(&self, exponent: u128) -> Self {
        let mut e = exponent;
        let mut res = Self::one();
        let mut t = *self;
        while !e.is_zero() {
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
        self.0.try_inverse().map(Self)
    }

    /// Converts to canonical form.
    #[inline(always)]
    fn as_u32_unchecked(&self) -> u32 {
        self.0.as_canonical_u32()
    }

    #[inline(always)]
    fn from_uniform_bytes(bytes: &[u8; 32]) -> Self {
        // Note: From<u32> performs modular reduction
        u32::from_le_bytes(bytes[..4].try_into().unwrap()).into()
    }
}

impl FieldForECC for BabyBear {
    fn modulus() -> ethnum::U256 {
        ethnum::U256::from(<P3BabyBear as PrimeField32>::ORDER_U32)
    }

    fn from_u256(x: ethnum::U256) -> Self {
        Self::new((x % Self::modulus()).as_u32())
    }

    fn to_u256(&self) -> ethnum::U256 {
        // Converts to canonical form before casting to U256
        ethnum::U256::from(self.as_u32_unchecked())
    }
}

// TODO: Actual SIMD impl
// This is a dummy implementation to satisfy trait bounds
impl SimdField for BabyBear {
    type Scalar = Self;

    fn scale(&self, challenge: &Self::Scalar) -> Self {
        self * challenge
    }

    fn pack(base_vec: &[Self::Scalar]) -> Self {
        debug_assert!(base_vec.len() == 1);
        base_vec[0]
    }

    fn unpack(&self) -> Vec<Self::Scalar> {
        vec![*self]
    }

    fn pack_size() -> usize {
        1
    }
}

impl Neg for BabyBear {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self::Output {
        Self(self.0.neg())
    }
}

impl From<u32> for BabyBear {
    #[inline(always)]
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

#[inline(always)]
fn add_internal(a: &BabyBear, b: &BabyBear) -> BabyBear {
    BabyBear(a.0 + b.0)
}

#[inline(always)]
fn sub_internal(a: &BabyBear, b: &BabyBear) -> BabyBear {
    BabyBear(a.0 - b.0)
}

#[inline(always)]
fn mul_internal(a: &BabyBear, b: &BabyBear) -> BabyBear {
    BabyBear(a.0 * b.0)
}
