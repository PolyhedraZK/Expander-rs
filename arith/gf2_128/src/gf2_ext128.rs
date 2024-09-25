#[cfg(target_arch = "aarch64")]
pub(crate) mod neon;
#[cfg(target_arch = "aarch64")]
pub type GF2_128 = neon::NeonGF2_128;

#[cfg(all(target_arch = "x86_64", target_feature = "avx512f"))]
mod avx512;
#[cfg(all(target_arch = "x86_64", target_feature = "avx512f"))]
pub type GF2_128 = avx512::AVX512GF2_128;

// Fallback, use avx2
#[cfg(all(target_arch = "x86_64", not(target_feature = "avx512f")))]
mod avx256;
#[cfg(all(target_arch = "x86_64", not(target_feature = "avx512f")))]
pub type GF2_128 = avx256::AVX256GF2_128;
