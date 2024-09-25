use super::{
    extension_field::random_extension_field_tests, field::random_field_tests,
    simd_field::random_simd_field_tests,
};
use crate::{BabyBearExt3, BabyBearExt3x16, BabyBearExt4, BabyBearExt4x16};

// CMD: RUSTFLAGS="-C target-feature=+avx512f" cargo test --package arith --lib -- tests::baby_bear_ext::test_field --exact --show-output
#[test]
fn test_field() {
    // Deg 3
    random_field_tests::<BabyBearExt3>("Baby Bear Ext3".to_string());
    random_extension_field_tests::<BabyBearExt3>("Baby Bear Ext3".to_string());

    random_field_tests::<BabyBearExt3x16>("Simd Baby Bear Ext3".to_string());
    random_extension_field_tests::<BabyBearExt3x16>("Simd Baby Bear Ext3".to_string());
    random_simd_field_tests::<BabyBearExt3x16>("Simd Baby Bear Ext3".to_string());

    // Deg 4
    random_field_tests::<BabyBearExt4>("Baby Bear Ext4".to_string());
    random_extension_field_tests::<BabyBearExt4>("Baby Bear Ext4".to_string());

    random_field_tests::<BabyBearExt4x16>("Simd Baby Bear Ext4".to_string());
    random_extension_field_tests::<BabyBearExt4x16>("Simd Baby Bear Ext4".to_string());
    random_simd_field_tests::<BabyBearExt4x16>("Simd Baby Bear Ext4".to_string());
}
