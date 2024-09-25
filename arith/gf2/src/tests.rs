use ark_std::test_rng;
use std::io::Cursor;

use arith::{random_field_tests, random_inversion_tests, random_simd_field_tests, FieldSerde};

use crate::{GF2x8, GF2};

#[test]
fn test_field() {
    random_field_tests::<GF2>("GF2".to_string());

    let mut rng = test_rng();
    random_inversion_tests::<GF2, _>(&mut rng, "GF2".to_string());
}

#[test]
fn test_simd_field() {
    random_field_tests::<GF2x8>("Vectorized GF2".to_string());
    random_simd_field_tests::<GF2x8>("Vectorized GF2".to_string());
}

#[test]
fn test_custom_serde_vectorize_gf2() {
    let a = GF2x8::from(0);
    let mut buffer = vec![];
    assert!(a.serialize_into(&mut buffer).is_ok());
    let mut cursor = Cursor::new(buffer);
    let b = GF2x8::deserialize_from(&mut cursor);
    assert!(b.is_ok());
    let b = b.unwrap();
    assert_eq!(a, b);
}
