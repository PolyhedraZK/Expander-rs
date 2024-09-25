use std::io::Cursor;

use ark_std::test_rng;
use arith::{random_field_tests, random_inversion_tests, random_simd_field_tests, FieldSerde};

use crate::{M31x16, M31};

#[test]
fn test_base_field() {
    random_field_tests::<M31>("M31".to_string());

    let mut rng = test_rng();
    random_inversion_tests::<M31, _>(&mut rng, "M31".to_string());
}



#[test]
fn test_simd_field() {

    random_field_tests::<M31x16>("Vectorized M31".to_string());

    let mut rng = test_rng();
    random_inversion_tests::<M31x16, _>(&mut rng, "Vectorized M31".to_string());

    random_simd_field_tests::<M31x16>("Vectorized M31".to_string());

    let a = M31x16::from(256 + 2);
    let mut buffer = vec![];
    assert!(a.serialize_into(&mut buffer).is_ok());
    let mut cursor = Cursor::new(buffer);
    let b = M31x16::deserialize_from(&mut cursor);
    assert!(b.is_ok());
    let b = b.unwrap();
    assert_eq!(a, b);
}

/// Compare to test vectors generated in SageMath
#[test]
fn test_vectors() {
    // M31 inversion
    let a = M31::from(3);
    let a_inv = M31::from(1431655765);
    assert_eq!(a_inv, a.inv().unwrap());
    // M31 exponentiation
    let a_pow_11 = M31::from(177147);
    assert_eq!(a_pow_11, a.exp(11));
}
