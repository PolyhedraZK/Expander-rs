use super::{
    field::{random_field_tests, random_inversion_tests},
    simd_field::random_simd_field_tests,
};
use crate::{BabyBear, BabyBearx16};
use ark_std::test_rng;

// CMD: RUSTFLAGS="-C target-feature=+avx512f" cargo test --package arith --lib -- tests::baby_bear::test_field --exact --show-output
#[test]
fn test_field() {
    random_field_tests::<BabyBear>("BabyBear".to_string());

    let mut rng = test_rng();
    random_inversion_tests::<BabyBear, _>(&mut rng, "BabyBear".to_string());
    random_inversion_tests::<BabyBearx16, _>(&mut rng, "Vectorized BabyBear".to_string());

    random_simd_field_tests::<BabyBearx16>("Vectorized BabyBear".to_string());
}
