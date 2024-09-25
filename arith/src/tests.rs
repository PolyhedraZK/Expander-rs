use ark_std::{end_timer, start_timer, test_rng};
use field::{
    associativity_tests, commutativity_tests, identity_tests, random_addition_tests,
    random_doubling_tests, random_expansion_tests, random_multiplication_tests,
    random_negation_tests, random_serde_tests, random_squaring_tests, random_subtraction_tests,
};
use rand::RngCore;

use crate::{ExtensionField, Field, FieldSerde, SimdField};

#[cfg(test)]
mod bn254;

mod field;

#[cfg(target_arch = "x86_64")]
#[test]
fn test_mm256_const_init() {
    use std::arch::x86_64::*;
    use std::mem::transmute;

    let all_equal = unsafe {
        let x = _mm256_set1_epi32(1);
        let y = transmute::<_, __m256i>([1, 1, 1, 1, 1, 1, 1, 1]);
        let cmp = _mm256_cmpeq_epi32(x, y);
        _mm256_testc_si256(cmp, _mm256_set1_epi32(-1))
    };

    assert!(all_equal != 0, "x and y are not equal");
}

#[cfg(target_arch = "aarch64")]
#[test]
fn test_uint32x4_const_init() {
    use std::arch::aarch64::*;
    use std::mem::transmute;

    let all_equal = unsafe {
        let x = vdupq_n_u32(1);
        let y = transmute::<_, uint32x4_t>([1, 1, 1, 1]);
        let cmp = vceqq_u32(x, y);
        vgetq_lane_u32(cmp, 0) == 0xffffffff
    };

    assert!(all_equal, "x and y are not equal");
}

pub fn random_extension_field_tests<F: ExtensionField>(_name: String) {
    let mut rng = test_rng();
    for _ in 0..1000 {
        {
            let a = F::random_unsafe(&mut rng);
            let s1 = F::BaseField::random_unsafe(&mut rng);
            let s2 = F::BaseField::random_unsafe(&mut rng);

            assert_eq!(
                a.mul_by_base_field(&s1).mul_by_base_field(&s2),
                a.mul_by_base_field(&s2).mul_by_base_field(&s1),
            );
            assert_eq!(
                a.mul_by_base_field(&s1).mul_by_base_field(&s2),
                a.mul_by_base_field(&(s1 * s2)),
            );

            assert_eq!(
                a.add_by_base_field(&s1).add_by_base_field(&s2),
                a.add_by_base_field(&s2).add_by_base_field(&s1),
            );
            assert_eq!(
                a.add_by_base_field(&s1).add_by_base_field(&s2),
                a.add_by_base_field(&(s1 + s2)),
            );
        }

        {
            let a = F::random_unsafe(&mut rng);
            let b = F::random_unsafe(&mut rng);
            let s = F::BaseField::random_unsafe(&mut rng);

            assert_eq!(a.mul_by_base_field(&s) * b, (a * b).mul_by_base_field(&s),);
            assert_eq!(b.mul_by_base_field(&s) * a, (a * b).mul_by_base_field(&s),);

            assert_eq!(a.add_by_base_field(&s) + b, (a + b).add_by_base_field(&s),);
            assert_eq!(b.add_by_base_field(&s) + a, (a + b).add_by_base_field(&s),);
        }

        {
            let a = F::random_unsafe(&mut rng);
            let b = F::X;
            let ax = a.mul_by_x();
            let ab = a * b;
            assert_eq!(ax, ab);
        }
    }
}

pub fn random_field_tests<F: Field + FieldSerde>(type_name: String) {
    let mut rng = test_rng();

    random_multiplication_tests::<F, _>(&mut rng, type_name.clone());
    random_addition_tests::<F, _>(&mut rng, type_name.clone());
    random_subtraction_tests::<F, _>(&mut rng, type_name.clone());
    random_negation_tests::<F, _>(&mut rng, type_name.clone());
    random_doubling_tests::<F, _>(&mut rng, type_name.clone());
    random_squaring_tests::<F, _>(&mut rng, type_name.clone());
    random_expansion_tests::<F, _>(&mut rng, type_name.clone()); // also serve as distributivity tests
    random_serde_tests::<F, _>(&mut rng, type_name.clone());
    associativity_tests::<F, _>(&mut rng, type_name.clone());
    commutativity_tests::<F, _>(&mut rng, type_name.clone());
    identity_tests::<F, _>(&mut rng, type_name.clone());
    //inverse_tests::<F, _>(&mut rng, type_name.clone());

    assert!(F::zero().is_zero());
    {
        let mut z = F::zero();
        z = z.neg();
        assert!(z.is_zero());
    }

    // Multiplication by zero
    {
        let mut a = F::random_unsafe(&mut rng);
        a.mul_assign(&F::zero());
        assert!(a.is_zero());
    }

    // Addition by zero
    {
        let mut a = F::random_unsafe(&mut rng);
        let copy = a;
        a.add_assign(&F::zero());
        assert_eq!(a, copy);
    }
}

pub fn random_inversion_tests<F: Field, R: RngCore>(mut rng: R, type_name: String) {
    assert!(F::zero().inv().is_none());

    let _message = format!("inversion {}", type_name);
    let start = start_timer!(|| _message);
    for _ in 0..1000 {
        let mut a = F::random_unsafe(&mut rng);
        if a.is_zero() {
            a = F::one();
        }
        let b = a.inv().unwrap(); // probabilistically nonzero
        a.mul_assign(&b);
        assert_eq!(a, F::one());
    }
    end_timer!(start);
}

pub fn random_simd_field_tests<F: SimdField>(_name: String) {
    let mut rng = test_rng();

    {
        let a = F::random_unsafe(&mut rng);
        let s1 = F::Scalar::random_unsafe(&mut rng);
        let s2 = F::Scalar::random_unsafe(&mut rng);

        assert_eq!(a.scale(&s1).scale(&s2), a.scale(&s2).scale(&s1),);
        assert_eq!(a.scale(&s1).scale(&s2), a.scale(&(s1 * s2)));
    }

    {
        let a = F::random_unsafe(&mut rng);
        let b = F::random_unsafe(&mut rng);
        let s = F::Scalar::random_unsafe(&mut rng);

        assert_eq!(a.scale(&s) * b, (a * b).scale(&s),);
        assert_eq!(b.scale(&s) * a, (a * b).scale(&s),);
    }

    {
        let x = F::random_unsafe(&mut rng);
        let scalars = x.unpack();
        let x_repacked = F::pack(&scalars);
        assert_eq!(x, x_repacked);
    }
}
