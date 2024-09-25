// A BabyBearx16 stores 512 bits of data.
// With AVX512 it stores one __m512i element.
// With AVX256 it stores two __m256i elements.
// With NEON it stores four uint32x4_t elements.
#[cfg(target_arch = "x86_64")]
cfg_if::cfg_if! {
    if #[cfg(feature = "avx256")] {
        pub type BabyBearx16 = super::baby_bear_avx256::AVXBabyBear;
    } else {
        pub type BabyBearx16 = super::baby_bear_avx::AVXBabyBear;
    }
}

#[cfg(target_arch = "aarch64")]
pub type BabyBearx16 = super::baby_bear_neon::NeonBabyBear;
