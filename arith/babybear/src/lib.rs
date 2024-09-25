#![cfg_attr(target_arch = "x86_64", feature(stdarch_x86_avx512))]

mod baby_bear;
pub use baby_bear::*;

mod baby_bear_ext3;
pub use baby_bear_ext3::BabyBearExt3;

mod baby_bear_ext3x16;
pub use baby_bear_ext3x16::BabyBearExt3x16;

mod baby_bear_ext4;
pub use baby_bear_ext4::BabyBearExt4;

mod baby_bear_ext4x16;
pub use baby_bear_ext4x16::BabyBearExt4x16;

#[cfg(test)]
mod tests;
