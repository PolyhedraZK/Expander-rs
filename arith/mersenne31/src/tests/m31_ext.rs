use crate::M31Ext3;
use crate::M31Ext3x16;

use arith::{random_extension_field_tests, random_field_tests, random_simd_field_tests};

#[test]
fn test_ext_field() {
    random_field_tests::<M31Ext3>("M31 Ext3".to_string());
    random_extension_field_tests::<M31Ext3>("M31 Ext3".to_string());
    random_field_tests::<M31Ext3x16>("Simd M31 Ext3".to_string());
    random_extension_field_tests::<M31Ext3x16>("Simd M31 Ext3".to_string());
    random_simd_field_tests::<M31Ext3x16>("Simd M31 Ext3".to_string());
}

/// Compare to test vectors generated in SageMath
#[test]
fn test_vectors() {
    use crate::{Field, M31};
    let a = M31Ext3 {
        v: [M31::from(1), M31::from(2), M31::from(3)],
    };
    let b = M31Ext3 {
        v: [M31::from(4), M31::from(5), M31::from(6)],
    };
    let expected_prod = M31Ext3 {
        v: [M31::from(139), M31::from(103), M31::from(28)],
    };
    assert_eq!(expected_prod, a * b);

    let expected_inv = M31Ext3 {
        v: [
            M31::from(1279570927),
            M31::from(2027416670),
            M31::from(696388467),
        ],
    };
    assert_eq!(expected_inv, a.inv().unwrap());
    let a_pow_11 = M31Ext3 {
        v: [
            M31::from(2145691179),
            M31::from(1848238717),
            M31::from(1954563431),
        ],
    };
    assert_eq!(a_pow_11, a.exp(11));
}
