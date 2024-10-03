use std::io::Cursor;

use arith::{
    random_extension_field_tests, random_fft_field_tests, random_field_tests,
    random_inversion_tests, random_simd_field_tests, Field, FieldSerde,
};
use ark_std::test_rng;
use p3_baby_bear::BabyBear as P3BabyBear;
use p3_field::{AbstractField, Field as P3Field};
use rand::{rngs::OsRng, Rng};

use crate::{
    baby_bear_ext4::P3BabyBearExt4, BabyBear, BabyBearExt3, BabyBearExt3x16, BabyBearExt4,
    BabyBearExt4x16, BabyBearx16,
};

#[test]
fn test_compare_plonky3() {
    for _ in 0..1000 {
        let mut rng = OsRng;
        let a = BabyBearExt4::random_unsafe(&mut rng);
        let b = BabyBearExt4::random_unsafe(&mut rng);

        // Test conversion
        let p3_a: P3BabyBearExt4 = (&a).into();
        let p3_b: P3BabyBearExt4 = (&b).into();
        assert_eq!(a, (&p3_a).into());
        assert_eq!(b, (&p3_b).into());

        // Test Add
        let a_plus_b = crate::baby_bear_ext4::add_internal(&a, &b);
        let p3_a_plus_b = p3_a + p3_b;
        assert_eq!(a_plus_b, (&p3_a_plus_b).into());

        // Test Sub
        let a_minus_b = crate::baby_bear_ext4::sub_internal(&a, &b);
        let p3_a_minus_b = p3_a - p3_b;
        assert_eq!(a_minus_b, (&p3_a_minus_b).into());

        // Test Mul
        let a_times_b = crate::baby_bear_ext4::mul_internal(&a, &b);
        let p3_a_times_b = p3_a * p3_b;
        assert_eq!(a_times_b, (&p3_a_times_b).into());

        // Test square
        let a_square = a.square();
        let p3_a_square = p3_a * p3_a;
        assert_eq!(a_square, (&p3_a_square).into());

        // Test exp
        let e = rng.gen_range(0..10);
        let a_exp_e = a.exp(e);
        let p3_a_exp_e = p3_a.exp_u64(e as u64);
        assert_eq!(a_exp_e, (&p3_a_exp_e).into());
    }
}

/// Compare to test vectors generated using SageMath
#[test]
fn test_ext4_compare_sage() {
    let a = BabyBearExt4 {
        v: [
            BabyBear::new(1),
            BabyBear::new(2),
            BabyBear::new(3),
            BabyBear::new(4),
        ],
    };
    let b = BabyBearExt4 {
        v: [
            BabyBear::new(5),
            BabyBear::new(6),
            BabyBear::new(7),
            BabyBear::new(8),
        ],
    };
    let expected_prod = BabyBearExt4 {
        v: [
            BabyBear::new(676),
            BabyBear::new(588),
            BabyBear::new(386),
            BabyBear::new(60),
        ],
    };
    assert_eq!(a * b, expected_prod);

    let a_inv = BabyBearExt4 {
        v: [
            BabyBear::new(1587469345),
            BabyBear::new(920666518),
            BabyBear::new(1160282443),
            BabyBear::new(647153706),
        ],
    };
    assert_eq!(a.inv().unwrap(), a_inv);

    let a_to_eleven = BabyBearExt4 {
        v: [
            BabyBear::new(374109212),
            BabyBear::new(621581642),
            BabyBear::new(269190551),
            BabyBear::new(1925703176),
        ],
    };
    assert_eq!(a.exp(11), a_to_eleven);
}

/// Compare to test vectors generated using SageMath
#[test]
fn test_ext3_compare_sage() {
    let a = BabyBearExt3 {
        v: [BabyBear::new(1), BabyBear::new(2), BabyBear::new(3)],
    };
    let b = BabyBearExt3 {
        v: [BabyBear::new(4), BabyBear::new(5), BabyBear::new(6)],
    };
    let expected_prod = BabyBearExt3 {
        v: [BabyBear::new(58), BabyBear::new(49), BabyBear::new(28)],
    };
    assert_eq!(a * b, expected_prod);

    let a_inv = BabyBearExt3 {
        v: [
            BabyBear::new(1628709509),
            BabyBear::new(1108427305),
            BabyBear::new(950080547),
        ],
    };
    assert_eq!(a.inv().unwrap(), a_inv);

    let a_to_eleven = BabyBearExt3 {
        v: [
            BabyBear::new(164947539),
            BabyBear::new(1313663563),
            BabyBear::new(627537568),
        ],
    };
    assert_eq!(a.exp(11), a_to_eleven);
}

#[test]
fn baby_bear_two_inverse() {
    let two = P3BabyBear::new(2);
    let two_inverse = <P3BabyBear as P3Field>::try_inverse(&two).unwrap();
    // Display impl converts to canonical form
    println!("2^-1 (canonical form): {two_inverse}");

    // Check correctness
    let two = BabyBear::new(2);
    let two_inverse_canonical: u32 = 1006632961;
    let two_inverse = BabyBear::new(two_inverse_canonical);
    let one = BabyBear::ONE;
    assert_eq!(one, two * two_inverse)
}

#[test]
fn test_exponentiation() {
    let mut rng = OsRng;

    for _ in 0..1000 {
        // Use a small base to avoid overflow
        let base_u32: u32 = rng.gen_range(0..=10);
        let base = BabyBear::new(base_u32);
        // Use a small exponent to avoid overflow
        let exponent: u32 = rng.gen_range(0..=5);
        let expected_result = BabyBear::new(base_u32.pow(exponent));
        assert_eq!(base.exp(exponent as u128), expected_result);
    }
}

#[test]
fn test_base_field() {
    random_field_tests::<BabyBear>("M31".to_string());

    let mut rng = test_rng();
    random_inversion_tests::<BabyBear, _>(&mut rng, "M31".to_string());
    random_fft_field_tests::<BabyBear>("bn254::Fr".to_string());
}

#[test]
fn test_simd_field() {
    random_field_tests::<BabyBearx16>("Vectorized M31".to_string());
    random_simd_field_tests::<BabyBearx16>("Vectorized M31".to_string());

    let mut rng = test_rng();
    random_inversion_tests::<BabyBearx16, _>(&mut rng, "Vectorized M31".to_string());
    random_fft_field_tests::<BabyBearx16>("bn254::Fr".to_string());

    let a = BabyBearx16::from(256 + 2);
    let mut buffer = vec![];
    assert!(a.serialize_into(&mut buffer).is_ok());
    let mut cursor = Cursor::new(buffer);
    let b = BabyBearx16::deserialize_from(&mut cursor);
    assert!(b.is_ok());
    let b = b.unwrap();
    assert_eq!(a, b);
}

#[test]
fn test_ext3_field() {
    random_field_tests::<BabyBearExt3>("M31 Ext3".to_string());
    random_extension_field_tests::<BabyBearExt3>("M31 Ext3".to_string());
    random_field_tests::<BabyBearExt3x16>("Simd M31 Ext3".to_string());
    random_extension_field_tests::<BabyBearExt3x16>("Simd M31 Ext3".to_string());
    random_simd_field_tests::<BabyBearExt3x16>("Simd M31 Ext3".to_string());
}

#[test]
fn test_ext4_field() {
    random_field_tests::<BabyBearExt4>("M31 Ext3".to_string());
    random_extension_field_tests::<BabyBearExt4>("M31 Ext3".to_string());
    random_field_tests::<BabyBearExt4x16>("Simd M31 Ext3".to_string());
    random_extension_field_tests::<BabyBearExt4x16>("Simd M31 Ext3".to_string());
    random_simd_field_tests::<BabyBearExt4x16>("Simd M31 Ext3".to_string());
}
