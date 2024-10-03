use arith::FFTField;

use super::TWO_ADIC_GENERATORS;

// A BabyBearx16 stores 512 bits of data.
// With AVX512 it stores one __m512i element.
// With AVX256 it stores two __m256i elements.
// With NEON it stores four uint32x4_t elements.
#[cfg(target_arch = "x86_64")]

pub type BabyBearx16 = super::baby_bear_avx::AVXBabyBear;

#[cfg(target_arch = "aarch64")]
pub type BabyBearx16 = super::baby_bear_neon::NeonBabyBear;

impl FFTField for BabyBearx16 {
    const TWO_ADICITY: u32 = 27;

    fn root_of_unity() -> Self {
        BabyBearx16::pack_full(super::BabyBear::new(0x1a427a41))
    }

    fn two_adic_generator(bits: usize) -> Self {
        BabyBearx16::pack_full(super::BabyBear::new(TWO_ADIC_GENERATORS[bits]))
    }
}
