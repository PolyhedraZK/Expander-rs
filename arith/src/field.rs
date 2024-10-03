use rand::RngCore;

use std::{
    fmt::Debug,
    hash::Hash,
    iter::{Product, Sum},
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

/// Field definitions.
pub trait Field:
    Copy
    + Clone
    + Debug
    + Default
    + PartialEq
    + From<u32>
    + Neg<Output = Self>
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Sum
    + Product
    + for<'a> Add<&'a Self, Output = Self>
    + for<'a> Sub<&'a Self, Output = Self>
    + for<'a> Mul<&'a Self, Output = Self>
    + for<'a> Sum<&'a Self>
    + for<'a> Product<&'a Self>
    + AddAssign
    + SubAssign
    + MulAssign
    + for<'a> AddAssign<&'a Self>
    + for<'a> SubAssign<&'a Self>
    + for<'a> MulAssign<&'a Self>
{
    /// name
    const NAME: &'static str;

    /// size required to store the data
    const SIZE: usize;

    /// Field element size in bits, e.g., log_2(modulus), rounded up to the next power of 2.
    const FIELD_SIZE: usize;

    /// zero
    const ZERO: Self;

    /// One
    const ONE: Self;

    /// Inverse of 2
    const INV_2: Self;

    // ====================================
    // constants
    // ====================================
    /// Zero element
    fn zero() -> Self;

    /// Is zero
    fn is_zero(&self) -> bool;

    /// Identity element
    fn one() -> Self;

    // ====================================
    // generators
    // ====================================
    /// create a random element from rng.
    /// test only -- the output may not be uniformly random.
    fn random_unsafe(rng: impl RngCore) -> Self;

    /// create a random boolean element from rng
    fn random_bool(rng: impl RngCore) -> Self;

    // ====================================
    #[inline(always)]
    fn square(&self) -> Self {
        *self * *self
    }

    /// Doubling
    #[inline(always)]
    fn double(&self) -> Self {
        *self + *self
    }

    /// Exp
    fn exp(&self, exponent: u128) -> Self;

    /// find the inverse of the element; return None if not exist
    fn inv(&self) -> Option<Self>;

    /// expose the element as u32.
    fn as_u32_unchecked(&self) -> u32;

    /// sample from a 32 bytes
    fn from_uniform_bytes(bytes: &[u8; 32]) -> Self;

    /// multiply by 2
    #[inline(always)]
    fn mul_by_2(&self) -> Self {
        *self + *self
    }

    #[inline(always)]
    /// multiply by 3
    fn mul_by_3(&self) -> Self {
        *self + *self + *self
    }

    #[inline(always)]
    /// multiply by 5
    fn mul_by_5(&self) -> Self {
        let double = self.mul_by_2();
        let quad = double.mul_by_2();
        *self + quad
    }

    #[inline(always)]
    /// multiply by 6
    fn mul_by_6(&self) -> Self {
        let t = self.mul_by_3();
        t + t
    }
}

pub trait FieldForECC: Field + Hash + Eq + PartialOrd + Ord {
    /// Modulus
    fn modulus() -> ethnum::U256;

    fn from_u256(x: ethnum::U256) -> Self;

    fn to_u256(&self) -> ethnum::U256;
}
